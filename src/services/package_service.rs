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
                        
                        // Cache valid for configured duration
                        if cache_age_minutes < CACHE_DURATION_MINUTES {
                            log::info!("📦 Usando paquetes del cache ({} min de antigüedad)", cache_age_minutes);
                            return Ok(cache.packages);
                        } else {
                            log::info!("📦 Cache expirado, obteniendo datos frescos...");
                        }
                    }
                }
            }
        }
    }
    
    // Fetch from API
    let url = format!("{}/api/colis-prive/packages", BACKEND_URL);
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
            
            // Parse packages from response
            if let Some(success) = packages_response.get("success").and_then(|s| s.as_bool()) {
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
                            };
                            if let Ok(cache_json) = serde_json::to_string(&cache) {
                                let _ = storage.set_item(&cache_key, &cache_json);
                                log::info!("💾 Paquetes guardados en cache");
                            }
                        }
                        
                        return Ok(packages);
                    }
                }
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

