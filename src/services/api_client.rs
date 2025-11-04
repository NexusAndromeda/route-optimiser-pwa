// ============================================================================
// API CLIENT - SOLO COMUNICACIÓN HTTP (Stateless)
// ============================================================================
// NO tiene lógica de negocio, solo hace requests HTTP
// ============================================================================

use gloo_net::http::Request;
use crate::models::session::DeliverySession;
use crate::models::sync::{Change, SyncRequest, SyncResponse};
use crate::models::company::Company;
use crate::utils::constants::BACKEND_URL;

/// Cliente API - SOLO comunicación HTTP (stateless)
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: BACKEND_URL.to_string(),
        }
    }
    
    /// Listar empresas
    pub async fn get_companies(&self) -> Result<Vec<Company>, String> {
        let url = format!("{}/v1/companies", self.base_url);
        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }
        response.json::<Vec<Company>>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
    
    /// Crear sesión (login)
    pub async fn create_session(
        &self,
        username: &str,
        password: &str,
        societe: &str,
    ) -> Result<CreateSessionResponse, String> {
        let url = format!("{}/v1/sessions", self.base_url);
        let request = CreateSessionRequest {
            username: username.to_string(),
            password: password.to_string(),
            societe: societe.to_string(),
        };
        
        log::info!("🔐 Creando sesión para usuario: {}", username);
        
        let response = Request::post(&url)
            .json(&request)
            .map_err(|e| format!("Serialization error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.ok() {
            response.json::<CreateSessionResponse>().await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            Err(format!("HTTP {}: {}", response.status(), response.status_text()))
        }
    }
    
    /// Buscar sesión en backend por driver_id + company_id (sin crear nueva)
    pub async fn find_session_by_driver(
        &self,
        driver_id: &str,
        company_id: &str,
    ) -> Result<Option<DeliverySession>, String> {
        let url = format!("{}/v1/sessions/by-driver/{}/{}", self.base_url, driver_id, company_id);
        
        log::info!("🔍 Buscando sesión en backend para driver: {} (company: {})", driver_id, company_id);
        
        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.status() == 404 {
            // No existe sesión en backend
            log::info!("⚠️ No hay sesión en backend para estos credenciales");
            return Ok(None);
        }
        
        if !response.ok() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("HTTP error {}: {}", status, error_text));
        }
        
        let get_response = response.json::<GetSessionResponse>().await
            .map_err(|e| format!("Parse error: {}", e))?;
        
        if get_response.success {
            log::info!("✅ Sesión encontrada en backend: {} ({} paquetes)", 
                get_response.session.session_id, get_response.session.stats.total_packages);
            Ok(Some(get_response.session))
        } else {
            Ok(None)
        }
    }
    
    /// Obtener sesión por ID
    pub async fn get_session(&self, session_id: &str) -> Result<DeliverySession, String> {
        let url = format!("{}/v1/sessions/{}", self.base_url, session_id);
        
        log::info!("📋 Obteniendo sesión: {}", session_id);
        
        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;
        
        if !response.ok() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("HTTP error {}: {}", status, error_text));
        }
        
        let session = response
            .json::<DeliverySession>()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        
        log::info!("✅ Sesión obtenida: {} paquetes, {} direcciones", 
                   session.packages.len(), session.addresses.len());
        
        Ok(session)
    }
    
    /// Obtener paquetes para una sesión
    pub async fn fetch_packages(
        &self,
        session_id: &str,
        username: &str,
        password: &str,
        societe: &str,
    ) -> Result<FetchPackagesResponse, String> {
        let url = format!("{}/v1/sessions/{}/fetch", self.base_url, session_id);
        let request = FetchPackagesRequest {
            username: username.to_string(),
            password: password.to_string(),
            societe: societe.to_string(),
        };
        
        log::info!("📦 Obteniendo paquetes para sesión: {}", session_id);
        
        let response = Request::post(&url)
            .json(&request)
            .map_err(|e| format!("Request build error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;
        
        if !response.ok() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("HTTP error {}: {}", status, error_text));
        }
        
        let response_data = response
            .json::<FetchPackagesResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        
        if response_data.success {
            log::info!("✅ Paquetes obtenidos: {} nuevos paquetes", 
                       response_data.new_packages_count.unwrap_or(0));
        } else {
            log::error!("❌ Error obteniendo paquetes: {:?}", response_data.error);
        }
        
        Ok(response_data)
    }
    
    /// Sincronizar sesión
    pub async fn sync_session(
        &self,
        session_id: &str,
        last_sync: i64,
        changes: Vec<Change>,
    ) -> Result<SyncResponse, String> {
        let url = format!("{}/v1/sessions/{}/sync", self.base_url, session_id);
        let request = SyncRequest {
            session_id: session_id.to_string(),
            last_sync,
            changes,
        };
        
        let response = Request::post(&url)
            .json(&request)
            .map_err(|e| format!("Serialization error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.ok() {
            response.json::<SyncResponse>().await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            Err(format!("HTTP {}: {}", response.status(), response.status_text()))
        }
    }
    
    /// Escanear paquete
    pub async fn scan_package(
        &self,
        session_id: &str,
        tracking: &str,
    ) -> Result<ScanResponse, String> {
        let url = format!("{}/v1/sessions/{}/scan", self.base_url, session_id);
        let request = ScanRequest {
            session_id: session_id.to_string(),
            tracking: tracking.to_string(),
        };
        
        log::info!("📱 Escaneando paquete: {} en sesión: {}", tracking, session_id);
        
        let response = Request::post(&url)
            .json(&request)
            .map_err(|e| format!("Request build error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;
        
        if !response.ok() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("HTTP error {}: {}", status, error_text));
        }
        
        let response_data = response
            .json::<ScanResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        
        if response_data.found {
            log::info!("✅ Paquete escaneado: {} (posición: {:?})", 
                       tracking, response_data.route_position);
        } else {
            log::warn!("⚠️ Paquete no encontrado: {}", tracking);
        }
        
        Ok(response_data)
    }
    
    /// Optimizar ruta
    pub async fn optimize_route(
        &self,
        session_id: &str,
        driver_latitude: f64,
        driver_longitude: f64,
    ) -> Result<OptimizeRouteResponse, String> {
        let url = format!("{}/v1/sessions/{}/optimize", self.base_url, session_id);
        let request = OptimizeRouteRequest {
            driver_latitude,
            driver_longitude,
        };
        
        log::info!("🗺️ Optimizando ruta para sesión: {} con ubicación: ({}, {})", 
                   session_id, driver_latitude, driver_longitude);
        
        let response = Request::post(&url)
            .json(&request)
            .map_err(|e| format!("Request build error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request error: {}", e))?;
        
        if !response.ok() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("HTTP error {}: {}", status, error_text));
        }
        
        let response_data = response
            .json::<OptimizeRouteResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        
        if response_data.success {
            log::info!("✅ Ruta optimizada: {} paradas, tiempo estimado: {} segundos", 
                       response_data.total_stops, response_data.estimated_time_seconds);
        } else {
            log::error!("❌ Error optimizando ruta");
        }
        
        Ok(response_data)
    }
    
    /// Actualizar solo campos específicos de dirección
    pub async fn update_address_fields(
        &self,
        session_id: &str,
        address_id: &str,
        door_code: Option<String>,
        has_mailbox_access: Option<bool>,
        driver_notes: Option<String>,
    ) -> Result<UpdateAddressFieldsResponse, String> {
        let url = format!("{}/v1/sessions/{}/address/{}/fields", self.base_url, session_id, address_id);
        let request = UpdateAddressFieldsRequest {
            door_code,
            has_mailbox_access,
            driver_notes,
        };
        
        log::info!("📝 Actualizando campos de dirección: {} en sesión: {}", address_id, session_id);
        
        let response = Request::put(&url)
            .json(&request)
            .map_err(|e| format!("Serialization error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.ok() {
            response.json::<UpdateAddressFieldsResponse>().await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            Err(format!("HTTP {}: {}", response.status(), response.status_text()))
        }
    }
    
    /// Actualizar dirección completa (para direcciones problemáticas)
    pub async fn update_address(
        &self,
        session_id: &str,
        address_id: &str,
        new_label: String,
        latitude: f64,
        longitude: f64,
    ) -> Result<UpdateAddressResponse, String> {
        let url = format!("{}/v1/sessions/{}/address/{}", self.base_url, session_id, address_id);
        let request = UpdateAddressRequest {
            new_label: Some(new_label.clone()),
            latitude: Some(latitude),
            longitude: Some(longitude),
        };
        
        log::info!("📍 Actualizando dirección: {} → {} ({}, {})", address_id, new_label, latitude, longitude);
        
        let response = Request::put(&url)
            .json(&request)
            .map_err(|e| format!("Serialization error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.ok() {
            response.json::<UpdateAddressResponse>().await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("HTTP error {}: {}", status, error_text))
        }
    }
    
    /// Sincronización incremental
    pub async fn sync_incremental(
        &self,
        session_id: &str,
        username: &str,
        societe: &str,
        date: Option<&str>,
    ) -> Result<SyncIncrementalResponse, String> {
        let url = format!("{}/v1/sessions/{}/sync-incremental", self.base_url, session_id);
        let request = SyncIncrementalRequest {
            username: username.to_string(),
            societe: societe.to_string(),
            date: date.map(|s| s.to_string()),
        };
        
        log::info!("🔄 Sincronización incremental para sesión: {} ({}:{})", session_id, societe, username);
        
        let response = Request::post(&url)
            .json(&request)
            .map_err(|e| format!("Serialization error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.ok() {
            response.json::<SyncIncrementalResponse>().await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("HTTP error {}: {}", status, error_text))
        }
    }
}

#[derive(serde::Serialize)]
struct CreateSessionRequest {
    username: String,
    password: String,
    societe: String,
}

#[derive(serde::Deserialize)]
pub struct CreateSessionResponse {
    pub success: bool,
    pub session: Option<DeliverySession>,
    pub session_id: Option<String>,
    pub error: Option<String>,
}

#[derive(serde::Serialize)]
struct FetchPackagesRequest {
    username: String,
    password: String,
    societe: String,
}

#[derive(serde::Deserialize)]
pub struct FetchPackagesResponse {
    pub success: bool,
    pub session: Option<DeliverySession>,
    pub new_packages_count: Option<usize>,
    pub error: Option<String>,
}

#[derive(serde::Serialize)]
struct ScanRequest {
    session_id: String,
    tracking: String,
}

#[derive(serde::Deserialize)]
pub struct ScanResponse {
    pub found: bool,
    pub route_position: Option<usize>,
    pub message: Option<String>,
}

#[derive(serde::Serialize)]
struct OptimizeRouteRequest {
    driver_latitude: f64,
    driver_longitude: f64,
}

#[derive(serde::Deserialize)]
pub struct OptimizeRouteResponse {
    pub success: bool,
    pub optimized_order: Vec<String>,
    pub session: DeliverySession,
    pub total_stops: usize,
    pub estimated_time_seconds: u32,
}

#[derive(serde::Serialize)]
struct UpdateAddressFieldsRequest {
    door_code: Option<String>,
    has_mailbox_access: Option<bool>,
    driver_notes: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateAddressFieldsResponse {
    pub success: bool,
    pub session: DeliverySession,
}

#[derive(serde::Serialize)]
struct UpdateAddressRequest {
    new_label: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(serde::Deserialize)]
pub struct UpdateAddressResponse {
    pub success: bool,
    pub session: DeliverySession,
}

#[derive(serde::Serialize)]
struct SyncIncrementalRequest {
    username: String,
    societe: String,
    date: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SyncIncrementalResponse {
    pub success: bool,
    pub delta: SyncDeltaResult,
    pub session: DeliverySession,
}

#[derive(serde::Deserialize)]
pub struct SyncDeltaResult {
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub removed: Vec<String>,
    pub unchanged: usize,
}

#[derive(serde::Deserialize)]
struct GetSessionResponse {
    pub success: bool,
    pub session: DeliverySession,
}
