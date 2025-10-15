use gloo_net::http::Request;
use crate::models::{OptimizationResponse, MapboxOptimizationRequest, OptimizationPackage, Package};
use crate::utils::BACKEND_URL;
use super::fetch_packages;

/// Optimize route using Mapbox Optimization API
pub async fn optimize_route(username: &str, societe: &str) -> Result<OptimizationResponse, String> {
    // Extract just the username part (without SOCIETE prefix)
    // username viene como "PCP0010699_C187518", extraemos "C187518"
    let matricule_only = if let Some(underscore_pos) = username.rfind('_') {
        &username[underscore_pos + 1..]
    } else {
        username
    };
    
    log::info!("🗺️ Optimizando ruta con Mapbox para: {} en societe: {}", matricule_only, societe);
    
    // Primero obtener los paquetes actuales
    let packages = fetch_packages(username, societe, false).await
        .map_err(|e| format!("Error obteniendo paquetes: {}", e))?;
    
    // Convertir paquetes al formato que espera Mapbox Optimization
    let mapbox_packages: Vec<OptimizationPackage> = packages.iter().map(|pkg| {
        // Extraer coordenadas del Package del frontend
        let (coord_x, coord_y) = if let Some(coords) = pkg.coords {
            (Some(coords[0]), Some(coords[1])) // [longitude, latitude]
        } else {
            (None, None)
        };
        
        OptimizationPackage {
            id: pkg.id.clone(),
            reference_colis: pkg.id.clone(), // Usar ID como reference_colis
            destinataire_nom: pkg.recipient.clone(),
            destinataire_adresse1: Some(pkg.address.clone()),
            destinataire_cp: None, // No disponible en Package del frontend
            destinataire_ville: None, // No disponible en Package del frontend
            coord_x_destinataire: coord_x,
            coord_y_destinataire: coord_y,
            statut: Some(pkg.status.clone()),
        }
    }).collect();
    
    // Usar el nuevo endpoint de Mapbox Optimization
    let url = format!("{}/api/mapbox-optimization/optimize", BACKEND_URL);
    let request_body = MapboxOptimizationRequest {
        matricule: matricule_only.to_string(),
        societe: societe.to_string(),
        packages: mapbox_packages,
    };
    
    let response = Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    response
        .json::<OptimizationResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Reorder packages based on optimization response
pub fn reorder_packages(current_packages: Vec<Package>, optimization_response: OptimizationResponse) -> Vec<Package> {
    if !optimization_response.success {
        log::warn!("⚠️ Optimización no exitosa");
        return current_packages;
    }
    
    let Some(data) = optimization_response.data else {
        log::warn!("⚠️ No hay datos de optimización");
        return current_packages;
    };
    
    let mut optimized_packages = Vec::new();
    
    // Mapear paquetes optimizados
    for opt_pkg in data.optimized_packages {
        // Buscar el paquete en la lista actual por referencia
        if let Some(ref_colis) = opt_pkg.reference_colis {
            if let Some(found) = current_packages.iter().find(|p| p.id == ref_colis) {
                optimized_packages.push(found.clone());
            } else {
                log::warn!("⚠️ No se encontró paquete con ID: {}", ref_colis);
            }
        }
    }
    
    if optimized_packages.is_empty() {
        log::warn!("⚠️ No se pudieron mapear los paquetes optimizados");
        return current_packages;
    }
    
    log::info!("📦 Paquetes reordenados según optimización: {} paquetes", optimized_packages.len());
    optimized_packages
}

