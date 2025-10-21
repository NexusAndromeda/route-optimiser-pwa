use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::utils::BACKEND_URL;

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageLocation {
    pub id: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_livraison: Option<String>, // "RELAIS", "RCS", "DOMICILE"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepotLocation {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizeRouteRequest {
    pub locations: Vec<PackageLocation>,
    pub depot: DepotLocation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizeRouteResponse {
    pub success: bool,
    pub optimization_id: Option<String>,
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimized_order: Option<Vec<String>>, // IDs de paquetes en orden óptimo
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationStatus {
    pub optimization_id: String,
    pub status: String,
    pub solution: Option<OptimizationSolution>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptimizationSolution {
    pub dropped: DroppedItems,
    pub routes: Vec<Route>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DroppedItems {
    pub services: Vec<String>,
    pub shipments: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Route {
    pub vehicle: String,
    pub stops: Vec<Stop>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stop {
    pub location: String,
    pub eta: String,
    #[serde(rename = "type")]
    pub stop_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<String>>,
}

/// Start route optimization
pub async fn optimize_route(locations: Vec<PackageLocation>, depot: DepotLocation) -> Result<OptimizeRouteResponse, String> {
    let url = format!("{}/mapbox/optimize", BACKEND_URL);
    let request_body = OptimizeRouteRequest { locations, depot };
    
    log::info!("🎯 Enviando {} ubicaciones para optimizar", request_body.locations.len());
    
    let response = Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    response
        .json::<OptimizeRouteResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Get optimization status
pub async fn get_optimization_status(optimization_id: &str) -> Result<OptimizationStatus, String> {
    let url = format!("{}/mapbox/optimize/{}", BACKEND_URL, optimization_id);
    
    log::info!("🔍 Consultando status de optimización: {}", optimization_id);
    
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    response
        .json::<OptimizationStatus>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}
