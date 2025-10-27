use crate::models::{DeliverySession, package::Package, delivery_session::Address};

/// Convierte DeliverySession al formato Package original para mantener compatibilidad
pub struct DeliverySessionConverter;

impl DeliverySessionConverter {
    /// Convierte un DeliverySession a un vector de Package para el frontend
    pub fn convert_to_packages(session: &DeliverySession) -> Vec<Package> {
        let mut packages = Vec::new();
        
        // Iterar sobre todas las direcciones y sus paquetes
        for (address_id, address) in &session.addresses {
            for package_id in &address.package_ids {
                if let Some(pkg) = session.packages.get(package_id) {
                    let package = Self::convert_package(pkg, address);
                    packages.push(package);
                }
            }
        }
        
        packages
    }
    
    /// Convierte un paquete individual del DeliverySession al formato Package
    fn convert_package(pkg: &crate::models::delivery_session::Package, address: &Address) -> Package {
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
