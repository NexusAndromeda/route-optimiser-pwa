// ============================================================================
// MAP VIEWMODEL - Lógica de negocio del mapa
// ============================================================================
// SOLO lógica de preparación de datos - Sin estado
// ============================================================================

use crate::models::{Package, session::DeliverySession};
use crate::views::package_list::PackageGroup;
use crate::utils::mapbox_ffi::*;
use gloo_timers::callback::Timeout;
use serde::Serialize;

/// Estructura para enviar al mapa (con coordenadas convertidas)
#[derive(Serialize, Clone)]
pub struct MapPackage {
    pub id: String,
    pub recipient: String,
    pub address: String,
    pub coords: [f64; 2], // [lat, lng]
    pub status: String,
    pub code_statut_article: String,
    pub type_livraison: String,
    pub is_problematic: bool,
    #[serde(rename = "group_idx")]
    pub group_idx: usize, // ⭐ Índice original del grupo en la lista completa (sin filtrar)
}

/// ViewModel del mapa - SOLO lógica de negocio
pub struct MapViewModel;

impl MapViewModel {
    /// Inicializar mapa (detecta dark mode)
    pub fn initialize_map() {
        let is_dark = web_sys::window()
            .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
            .flatten()
            .map(|mq| mq.matches())
            .unwrap_or(false);
        
        log::info!("🗺️ Inicializando mapa (dark mode: {})", is_dark);
        init_mapbox("map", is_dark);
    }
    
    /// Convertir grupos a paquetes para el mapa
    /// LÓGICA DE AGRUPAMIENTO: Un grupo con múltiples paquetes = UN SOLO PUNTO en el mapa
    pub fn prepare_packages_for_map(
        groups: &[PackageGroup],
        session: &DeliverySession,
    ) -> Vec<MapPackage> {
        log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        log::info!("🗺️ PREPARANDO PAQUETES PARA MAPA");
        log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        log::info!("📊 Total grupos recibidos: {}", groups.len());
        
        let mut map_packages = Vec::new();
        let mut skipped_count = 0;
        
        // ⭐ Iterar con índice para mantener referencia al group_idx original
        for (group_idx, group) in groups.iter().enumerate() {
            // Si el grupo tiene más de 1 paquete = UN SOLO PUNTO en el mapa
            if group.count > 1 {
                // Usar el primer paquete para obtener la dirección y propiedades
                if let Some(first_pkg) = group.packages.first() {
                    if let Some(address) = session.addresses.get(&first_pkg.address_id) {
                        // Solo grupos con coordenadas válidas
                        if address.latitude == 0.0 && address.longitude == 0.0 {
                            log::warn!("⚠️ Grupo {} SKIPPEADO (sin coordenadas válidas): address_id={}, label={}", 
                                      group_idx, first_pkg.address_id, address.label);
                            skipped_count += 1;
                            continue; // ⚠️ Esto crea el desajuste - por eso guardamos group_idx
                        }
                        
                        let type_livraison = match first_pkg.delivery_type {
                            crate::models::package::DeliveryType::Home => "DOMICILE",
                            crate::models::package::DeliveryType::Rcs => "RCS",
                            crate::models::package::DeliveryType::PickupPoint => "RELAIS",
                        }.to_string();
                        
                        log::info!("✅ Grupo {} (AGRUPADO): {} paquetes → address_id={}, coords=[{}, {}], group_idx={}", 
                                  group_idx, group.count, first_pkg.address_id, 
                                  address.latitude, address.longitude, group_idx);
                        
                        map_packages.push(MapPackage {
                            id: first_pkg.address_id.clone(),
                            recipient: format!("{} paquets", group.count),
                            address: address.label.clone(),
                            coords: [address.latitude, address.longitude],
                            status: first_pkg.status.clone(),
                            code_statut_article: first_pkg.status.clone(),
                            type_livraison,
                            is_problematic: group.packages.iter().any(|p| p.is_problematic),
                            group_idx, // ⭐ Guardar índice original del grupo
                        });
                    } else {
                        log::warn!("⚠️ Grupo {} sin dirección encontrada: address_id={}", 
                                  group_idx, first_pkg.address_id);
                        skipped_count += 1;
                    }
                }
            } else {
                // Grupo con 1 solo paquete = UN PUNTO en el mapa
                if let Some(pkg) = group.packages.first() {
                    if let Some(address) = session.addresses.get(&pkg.address_id) {
                        // Solo paquetes con coordenadas válidas
                        if address.latitude == 0.0 && address.longitude == 0.0 {
                            log::warn!("⚠️ Grupo {} SKIPPEADO (sin coordenadas válidas): tracking={}, label={}", 
                                      group_idx, pkg.tracking, address.label);
                            skipped_count += 1;
                            continue; // ⚠️ Esto crea el desajuste - por eso guardamos group_idx
                        }
                        
                        let type_livraison = match pkg.delivery_type {
                            crate::models::package::DeliveryType::Home => "DOMICILE",
                            crate::models::package::DeliveryType::Rcs => "RCS",
                            crate::models::package::DeliveryType::PickupPoint => "RELAIS",
                        }.to_string();
                        
                        log::info!("✅ Grupo {} (INDIVIDUAL): tracking={}, recipient={}, coords=[{}, {}], group_idx={}", 
                                  group_idx, pkg.tracking, pkg.customer_name, 
                                  address.latitude, address.longitude, group_idx);
                        
                        map_packages.push(MapPackage {
                            id: pkg.tracking.clone(),
                            recipient: pkg.customer_name.clone(),
                            address: address.label.clone(),
                            coords: [address.latitude, address.longitude],
                            status: pkg.status.clone(),
                            code_statut_article: pkg.status.clone(),
                            type_livraison,
                            is_problematic: pkg.is_problematic,
                            group_idx, // ⭐ Guardar índice original del grupo
                        });
                    } else {
                        log::warn!("⚠️ Grupo {} sin dirección encontrada: tracking={}, address_id={}", 
                                  group_idx, pkg.tracking, pkg.address_id);
                        skipped_count += 1;
                    }
                }
            }
        }
        
        log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        log::info!("✅ RESUMEN PREPARACIÓN:");
        log::info!("   📦 Total grupos: {}", groups.len());
        log::info!("   📍 Puntos en mapa: {}", map_packages.len());
        log::info!("   ⚠️  Saltados (sin coordenadas/dirección): {}", skipped_count);
        log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Log detallado de los primeros 10 paquetes para debugging
        for (i, map_pkg) in map_packages.iter().take(10).enumerate() {
            log::info!("   [{i}] group_idx={}, id={}, address={}", 
                      map_pkg.group_idx, map_pkg.id, map_pkg.address);
        }
        if map_packages.len() > 10 {
            log::info!("   ... y {} paquetes más", map_packages.len() - 10);
        }
        
        map_packages
    }
    
    /// Enviar paquetes al mapa (con conversión de coordenadas)
    pub fn update_map_packages(packages: Vec<MapPackage>) {
        log::info!("🗺️ ViewModel: Enviando {} paquetes al mapa", packages.len());
        
        // LOG: Verificar que los paquetes tengan coordenadas
        for (i, pkg) in packages.iter().take(5).enumerate() {
            log::info!("📍 Paquete {}: {} - coords: [{}, {}]", 
                       i, pkg.address, pkg.coords[0], pkg.coords[1]);
        }
        if packages.len() > 5 {
            log::info!("📍 ... y {} paquetes más", packages.len() - 5);
        }
        
        // Serializar y enviar
        if let Ok(json) = serde_json::to_string(&packages) {
            log::info!("📤 JSON generado ({} bytes)", json.len());
            log::info!("📦 JSON preview: {}...", &json[..json.len().min(200)]);
            
            Timeout::new(100, move || {
                add_packages_to_map(&json);
                log::info!("✅ add_packages_to_map llamado");
            }).forget();
        } else {
            log::error!("❌ Error serializando paquetes para el mapa");
        }
    }
}

