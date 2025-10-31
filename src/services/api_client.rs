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
