use crate::models::{DeliverySession, package::{Package, GroupPackageInfo}, delivery_session::Address};

/// Convierte DeliverySession al formato Package original para mantener compatibilidad
pub struct DeliverySessionConverter;

impl DeliverySessionConverter {
    /// Convierte un DeliverySession a un vector de Package para el frontend
    /// AGRUPA paquetes por dirección - igual que en main
    pub fn convert_to_packages(session: &DeliverySession) -> Vec<Package> {
        let mut packages = Vec::new();
        
        // Iterar sobre todas las direcciones y sus paquetes
        for (address_id, address) in &session.addresses {
            let package_count = address.package_ids.len();
            
            // Si hay más de 1 paquete en esta dirección, crear un GRUPO
            if package_count > 1 {
                let mut group_packages_list = Vec::new();
                
                // Extraer todos los paquetes del grupo
                for package_id in &address.package_ids {
                    if let Some(pkg) = session.packages.get(package_id) {
                        group_packages_list.push(GroupPackageInfo {
                            id: pkg.internal_id.clone(),
                            tracking: pkg.tracking.clone(),
                            customer_name: pkg.customer_name.clone(),
                            phone_number: pkg.phone_number.clone(),
                            customer_indication: pkg.customer_indication.clone(),
                            code_statut_article: Some(pkg.status.clone()),
                            is_problematic: pkg.is_problematic,
                        });
                    }
                }
                
                // Crear un solo Package como grupo
                if !group_packages_list.is_empty() {
                    // Obtener el primer paquete para datos comunes
                    if let Some(first_pkg_id) = address.package_ids.first() {
                        if let Some(first_pkg) = session.packages.get(first_pkg_id) {
                            let group_package = Package {
                                id: address.address_id.clone(),
                                tracking: None,
                                recipient: format!("{} paquetes", package_count),
                                address: address.label.clone(),
                                status: first_pkg.status.clone(), // Heredar del primer paquete
                                code_statut_article: Some(first_pkg.status.clone()),
                                coords: Some([address.latitude, address.longitude]),
                                phone: None,
                                phone_fixed: None,
                                instructions: None,
                                door_code: address.door_code.clone(),
                                has_mailbox_access: address.mailbox_access,
                                driver_notes: Some(address.driver_notes.clone()),
                                is_group: true,
                                total_packages: Some(package_count),
                                group_packages: Some(group_packages_list),
                                is_problematic: false,
                                type_livraison: Some(match first_pkg.delivery_type {
                                    crate::models::DeliveryType::Home => "DOMICILE".to_string(),
                                    crate::models::DeliveryType::Rcs => "RCS".to_string(),
                                    crate::models::DeliveryType::PickupPoint => "RELAIS".to_string(),
                                }),
                            };
                            packages.push(group_package);
                        }
                    }
                }
            } else {
                // Solo 1 paquete en esta dirección - crear paquete individual
                if let Some(package_id) = address.package_ids.first() {
                    if let Some(pkg) = session.packages.get(package_id) {
                        let package = Self::convert_single_package(pkg, address);
                        packages.push(package);
                    }
                }
            }
        }
        
        packages
    }
    
    /// Convierte un paquete INDIVIDUAL (solo) del DeliverySession al formato Package
    fn convert_single_package(pkg: &crate::models::delivery_session::Package, address: &Address) -> Package {
        Package {
            id: pkg.internal_id.clone(),
            tracking: Some(pkg.tracking.clone()),
            recipient: pkg.customer_name.clone(),
            address: address.label.clone(),
            status: pkg.status.clone(),
            code_statut_article: Some(pkg.status.clone()),
            coords: {
                log::info!("📍 Coordenadas para {}: lat={}, lng={}", address.label, address.latitude, address.longitude);
                Some([address.latitude, address.longitude])
            },
            phone: pkg.phone_number.clone(),
            phone_fixed: None, // No disponible en DeliverySession
            instructions: pkg.customer_indication.clone(),
            door_code: address.door_code.clone(),
            has_mailbox_access: address.mailbox_access,
            driver_notes: Some(address.driver_notes.clone()),
            is_group: false, // Los paquetes individuales no son grupos
            total_packages: None,
            group_packages: None,
            is_problematic: pkg.is_problematic,
            type_livraison: Some(match pkg.delivery_type {
                crate::models::DeliveryType::Home => "DOMICILE".to_string(),
                crate::models::DeliveryType::Rcs => "RCS".to_string(),
                crate::models::DeliveryType::PickupPoint => "RELAIS".to_string(),
            }),
        }
    }
    
    /// Aplica filtros como en la implementación original
    pub fn apply_filters(packages: &[Package], filter_mode: bool) -> Vec<Package> {
        if filter_mode {
            packages.iter()
                .filter(|p| {
                    p.code_statut_article.as_ref()
                        .map(|code| code == "STATUT_CHARGER")
                        .unwrap_or(false)
                })
                .cloned()
                .collect()
        } else {
            packages.to_vec()
        }
    }
    
    /// Obtiene estadísticas como en la implementación original
    pub fn get_stats(packages: &[Package]) -> (usize, usize, usize) {
        let total = packages.len();
        let treated = packages.iter()
            .filter(|p| !p.status.starts_with("STATUT_CHARGER"))
            .count();
        let percentage = if total > 0 { (treated * 100) / total } else { 0 };
        
        (total, treated, percentage)
    }
}
