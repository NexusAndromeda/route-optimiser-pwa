use gloo_net::http::Request;
use crate::models::{Package, PackageRequest, PackagesCache};
use crate::utils::{BACKEND_URL, CACHE_DURATION_MINUTES, get_local_storage, STORAGE_KEY_PACKAGES_PREFIX};

/// Fetch packages from API or cache
pub async fn fetch_packages(username: &str, societe: &str, force_refresh: bool) -> Result<Vec<Package>, String> {
    log::info!("📦 Obteniendo paquetes de Colis Privé...");
    
    // Extract matricule from username (format: "COMPANY_CODE_MATRICULE")
    let matricule = if let Some(underscore_pos) = username.rfind('_') {
        &username[underscore_pos + 1..]
    } else {
        username
    };
    
    let cache_key = format!("{}_{}", STORAGE_KEY_PACKAGES_PREFIX, format!("{}_{}", societe, username));
    
    // Check cache first if not forcing refresh
    if !force_refresh {
        if let Some(storage) = get_local_storage() {
            if let Ok(Some(cached_data)) = storage.get_item(&cache_key) {
                if let Ok(cache) = serde_json::from_str::<PackagesCache>(&cached_data) {
                    // Check cache age
                    if let Ok(cache_time) = chrono::DateTime::parse_from_rfc3339(&cache.timestamp) {
                        let now = chrono::Utc::now();
                        let cache_age = now.signed_duration_since(cache_time.with_timezone(&chrono::Utc));
                        let cache_age_minutes = cache_age.num_minutes();
                        
                        // Verificar versión del cache (v2 tiene code_statut_article)
                        let cache_version = cache.version;
                        let is_valid_version = cache_version >= 2;
                        
                        // Cache valid for configured duration AND correct version
                        if cache_age_minutes < CACHE_DURATION_MINUTES && is_valid_version {
                            log::info!("📦 Usando paquetes del cache v{} ({} min de antigüedad)", cache_version, cache_age_minutes);
                            return Ok(cache.packages);
                        } else {
                            if !is_valid_version {
                                log::info!("📦 Cache v{} obsoleto, obteniendo datos frescos...", cache_version);
                            } else {
                                log::info!("📦 Cache expirado, obteniendo datos frescos...");
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Fetch from API
    let url = format!("{}/colis-prive/packages", BACKEND_URL);
    let request_body = PackageRequest {
        matricule: matricule.to_string(),
        societe: societe.to_string(),
        date: None,
    };
    
    log::info!("📤 Request: matricule={}, societe={}", matricule, societe);
    
    match Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
    {
        Ok(response) => {
            if !response.ok() {
                // Try to use cache as fallback
                if let Some(storage) = get_local_storage() {
                    if let Ok(Some(cached_data)) = storage.get_item(&cache_key) {
                        if let Ok(cache) = serde_json::from_str::<PackagesCache>(&cached_data) {
                            log::info!("📦 Error en API, usando cache como fallback...");
                            return Ok(cache.packages);
                        }
                    }
                }
                return Err(format!("HTTP error: {}", response.status()));
            }
            
            let packages_response = response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| format!("Parse error: {}", e))?;
            
            // Detectar si es la nueva estructura agrupada o la vieja
            let mut all_packages = Vec::new();
            
            // Nueva estructura: {singles: [...], groups: [...]}
            if packages_response.get("singles").is_some() || packages_response.get("groups").is_some() {
                log::info!("📦 Detectada estructura agrupada (nueva)");
                
                // Parsear singles
                if let Some(singles_array) = packages_response.get("singles").and_then(|s| s.as_array()) {
                    for (index, single) in singles_array.iter().enumerate() {
                        if let Ok(pkg) = parse_single_package(single, index) {
                            all_packages.push(pkg);
                        }
                    }
                }
                
                // Parsear groups
                if let Some(groups_array) = packages_response.get("groups").and_then(|g| g.as_array()) {
                    for (index, group) in groups_array.iter().enumerate() {
                        if let Ok(pkg) = parse_group_package(group, all_packages.len() + index) {
                            all_packages.push(pkg);
                        }
                    }
                }
                
                log::info!("✅ {} paquetes parseados (singles + groups)", all_packages.len());
                
            // Estructura vieja: {success: true, packages: [...]}
            } else if let Some(success) = packages_response.get("success").and_then(|s| s.as_bool()) {
                log::info!("📦 Detectada estructura legacy (vieja)");
                if success {
                    if let Some(packages_array) = packages_response.get("packages").and_then(|p| p.as_array()) {
                        let packages: Result<Vec<Package>, String> = packages_array
                            .iter()
                            .enumerate()
                            .map(|(index, pkg)| {
                                Ok(Package {
                                    // Priorizar campos principales de Colis Privé
                                    id: pkg.get("reference_colis")
                                        .and_then(|r| r.as_str())
                                        .or_else(|| pkg.get("tracking_number").and_then(|t| t.as_str()))
                                        .unwrap_or(&format!("PKG-{}", index + 1))
                                        .to_string(),
                                    recipient: pkg.get("destinataire_nom")
                                        .and_then(|d| d.as_str())
                                        .or_else(|| pkg.get("recipient_name").and_then(|r| r.as_str()))
                                        .unwrap_or("Destinatario desconocido")
                                        .to_string(),
                                    address: {
                                        // Priorizar formatted_address del geocoding
                                        if let Some(addr) = pkg.get("formatted_address").and_then(|a| a.as_str()) {
                                            addr.to_string()
                                        } else if let Some(addr) = pkg.get("address").and_then(|a| a.as_str()) {
                                            addr.to_string()
                                        } else {
                                            // Construir dirección de campos Colis Privé
                                            let mut parts = Vec::new();
                                            if let Some(addr1) = pkg.get("destinataire_adresse1").and_then(|a| a.as_str()) {
                                                parts.push(addr1);
                                            }
                                            if let Some(cp) = pkg.get("destinataire_cp").and_then(|c| c.as_str()) {
                                                parts.push(cp);
                                            }
                                            if let Some(ville) = pkg.get("destinataire_ville").and_then(|v| v.as_str()) {
                                                parts.push(ville);
                                            }
                                            if !parts.is_empty() {
                                                parts.join(", ")
                                            } else {
                                                "Dirección no disponible".to_string()
                                            }
                                        }
                                    },
                                    status: pkg.get("statut")
                                        .and_then(|s| s.as_str())
                                        .or_else(|| pkg.get("status").and_then(|s| s.as_str()))
                                        .map(|s| if s.to_lowercase().contains("livr") { "delivered" } else { "pending" })
                                        .unwrap_or("pending")
                                        .to_string(),
                                    code_statut_article: pkg.get("code_statut_article")
                                        .and_then(|s| s.as_str())
                                        .map(|s| s.to_string()),
                                    coords: if let (Some(lat), Some(lng)) = (
                                        pkg.get("latitude").and_then(|l| l.as_f64()),
                                        pkg.get("longitude").and_then(|l| l.as_f64())
                                    ) {
                                        Some([lng, lat])
                                    } else {
                                        None
                                    },
                                    phone: pkg.get("phone").and_then(|p| p.as_str()).map(|s| s.to_string()),
                                    phone_fixed: pkg.get("phone_fixed").and_then(|p| p.as_str()).map(|s| s.to_string()),
                                    instructions: None,
                                    is_group: false,
                                    total_packages: None,
                                    group_packages: None,
                                    is_problematic: false,
                                })
                            })
                            .collect();
                        
                        let packages = packages?;
                        
                        log::info!("✅ Paquetes obtenidos: {} paquetes", packages.len());
                        let with_coords = packages.iter().filter(|p| p.coords.is_some()).count();
                        log::info!("📍 Paquetes con coordenadas: {} / {}", with_coords, packages.len());
                        
                        // Save to cache
                        if let Some(storage) = get_local_storage() {
                            let cache = PackagesCache {
                                packages: packages.clone(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                version: 2, // Version con code_statut_article
                            };
                            if let Ok(cache_json) = serde_json::to_string(&cache) {
                                let _ = storage.set_item(&cache_key, &cache_json);
                                log::info!("💾 Paquetes guardados en cache (v2)");
                            }
                        }
                        
                        all_packages = packages;
                    }
                }
            }
            
            // Si tenemos paquetes parseados, guardar en cache y retornar
            if !all_packages.is_empty() {
                // Save to cache
                if let Some(storage) = get_local_storage() {
                    let cache = PackagesCache {
                        packages: all_packages.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        version: 2, // Version con code_statut_article
                    };
                    if let Ok(cache_json) = serde_json::to_string(&cache) {
                        let _ = storage.set_item(&cache_key, &cache_json);
                        log::info!("💾 Paquetes guardados en cache (v2)");
                    }
                }
                
                return Ok(all_packages);
            }
            
            log::info!("⚠️ No hay paquetes disponibles");
            Ok(Vec::new())
        }
        Err(e) => {
            log::error!("❌ Error obteniendo paquetes: {}", e);
            
            // Try to use cache as fallback
            if let Some(storage) = get_local_storage() {
                if let Ok(Some(cached_data)) = storage.get_item(&cache_key) {
                    if let Ok(cache) = serde_json::from_str::<PackagesCache>(&cached_data) {
                        log::info!("📦 Error en API, usando cache como fallback...");
                        return Ok(cache.packages);
                    }
                }
            }
            
            Err(format!("Request error: {}", e))
        }
    }
}

use crate::models::GroupPackageInfo;

/// Parsea un single package de la nueva estructura
fn parse_single_package(single: &serde_json::Value, index: usize) -> Result<Package, String> {
    Ok(Package {
        id: single.get("tracking")
            .and_then(|t| t.as_str())
            .or_else(|| single.get("id").and_then(|i| i.as_str()))
            .unwrap_or(&format!("single-{}", index))
            .to_string(),
        recipient: single.get("customer_name")
            .and_then(|n| n.as_str())
            .unwrap_or("Destinatario desconocido")
            .to_string(),
        address: single.get("official_label")
            .and_then(|a| a.as_str())
            .unwrap_or("Dirección no disponible")
            .to_string(),
        status: "pending".to_string(),
        code_statut_article: {
            let code = single.get("code_statut_article")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            if let Some(ref c) = code {
                log::info!("📋 Paquete {}: code_statut_article = {}", index, c);
            } else {
                log::warn!("⚠️ Paquete {} sin code_statut_article", index);
            }
            code
        },
        coords: if let (Some(lat), Some(lng)) = (
            single.get("latitude").and_then(|l| l.as_f64()),
            single.get("longitude").and_then(|l| l.as_f64())
        ) {
            // No usar coordenadas inválidas (0,0) o problemáticos
            if lat != 0.0 && lng != 0.0 {
                Some([lng, lat])
            } else {
                None
            }
        } else {
            None
        },
        phone: single.get("phone_number").and_then(|p| p.as_str()).map(|s| s.to_string()),
        phone_fixed: None,
        instructions: single.get("customer_indication").and_then(|i| i.as_str()).map(|s| s.to_string()),
        is_group: false,
        total_packages: None,
        group_packages: None,
        is_problematic: single.get("is_problematic")
            .and_then(|p| p.as_bool())
            .unwrap_or(false),
    })
}

/// Parsea un delivery group de la nueva estructura
fn parse_group_package(group: &serde_json::Value, index: usize) -> Result<Package, String> {
    let total_packages = group.get("total_packages")
        .and_then(|t| t.as_u64())
        .unwrap_or(0) as usize;
    
    // Extraer todos los paquetes del grupo
    let mut group_packages_list = Vec::new();
    
    if let Some(customers) = group.get("customers").and_then(|c| c.as_array()) {
        for customer in customers {
            let customer_name = customer.get("customer_name")
                .and_then(|n| n.as_str())
                .unwrap_or("Cliente desconocido")
                .to_string();
            let phone_number = customer.get("phone_number")
                .and_then(|p| p.as_str())
                .map(|s| s.to_string());
            
            if let Some(packages) = customer.get("packages").and_then(|p| p.as_array()) {
                for pkg in packages {
                    group_packages_list.push(GroupPackageInfo {
                        id: pkg.get("id")
                            .and_then(|i| i.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        tracking: pkg.get("tracking")
                            .and_then(|t| t.as_str())
                            .unwrap_or("N/A")
                            .to_string(),
                        customer_name: customer_name.clone(),
                        phone_number: phone_number.clone(),
                        customer_indication: pkg.get("customer_indication")
                            .and_then(|i| i.as_str())
                            .map(|s| s.to_string()),
                    });
                }
            }
        }
    }
    
    Ok(Package {
        id: group.get("id")
            .and_then(|i| i.as_str())
            .unwrap_or(&format!("group-{}", index))
            .to_string(),
        recipient: format!("{} paquetes", total_packages),
        address: group.get("official_label")
            .and_then(|a| a.as_str())
            .unwrap_or("Dirección no disponible")
            .to_string(),
        status: "pending".to_string(),
        code_statut_article: None, // Los grupos no tienen status individual
        coords: if let (Some(lat), Some(lng)) = (
            group.get("latitude").and_then(|l| l.as_f64()),
            group.get("longitude").and_then(|l| l.as_f64())
        ) {
            Some([lng, lat])
        } else {
            None
        },
        phone: None,
        phone_fixed: None,
        instructions: group.get("driver_notes").and_then(|n| n.as_str()).map(|s| s.to_string()),
        is_group: true,
        total_packages: Some(total_packages),
        group_packages: Some(group_packages_list),
        is_problematic: false,
    })
}

