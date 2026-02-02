// ============================================================================
// API CLIENT - SOLO COMUNICACIÓN HTTP (Stateless)
// ============================================================================
// NO tiene lógica de negocio, solo hace requests HTTP
// ============================================================================

use gloo_net::http::Request;
use crate::models::session::DeliverySession;
use crate::models::sync::{Change, SyncRequest, SyncResponse};
use crate::models::company::Company;
use crate::models::admin::{AdminDistrict, StatusChangeRequest, CloseDayResponse, PackageTraceabilityResponse, SearchTrackingRequest, SearchTrackingResponse};
use crate::models::session::DeliverySession as Session;
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
    
    /// Refrescar token SSO de una sesión existente (sin fetch de paquetes)
    pub async fn refresh_token(
        &self,
        session_id: &str,
        username: &str,
        password: &str,
        societe: &str,
    ) -> Result<GetSessionResponse, String> {
        let url = format!("{}/v1/sessions/{}/refresh-token", self.base_url, session_id);
        let request = CreateSessionRequest {
            username: username.to_string(),
            password: password.to_string(),
            societe: societe.to_string(),
        };
        
        log::info!("🔐 Refrescando token para sesión: {}", session_id);
        
        let response = Request::post(&url)
            .json(&request)
            .map_err(|e| format!("Serialization error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.ok() {
            response.json::<GetSessionResponse>().await
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
        
        log::info!("📝 [API] Actualizando campos de dirección: {} en sesión: {}", address_id, session_id);
        log::info!("📬 [API] Payload - door_code={:?}, has_mailbox_access={:?}, driver_notes={:?}",
                   request.door_code.is_some(), request.has_mailbox_access, request.driver_notes.is_some());
        
        let response = Request::put(&url)
            .json(&request)
            .map_err(|e| format!("Serialization error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if response.ok() {
            let parsed_response = response.json::<UpdateAddressFieldsResponse>().await
                .map_err(|e| format!("Parse error: {}", e))?;
            
            // Verificar que la dirección se actualizó correctamente
            if let Some(addr) = parsed_response.session.addresses.get(address_id) {
                log::info!("📬 [API] Respuesta recibida - mailbox_access={:?}, door_code={:?}, driver_notes={:?}",
                          addr.mailbox_access, addr.door_code.is_some(), addr.driver_notes.is_some());
            } else {
                log::warn!("⚠️ [API] Dirección no encontrada en respuesta del servidor: {}", address_id);
            }
            
            Ok(parsed_response)
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
pub struct GetSessionResponse {
    pub success: bool,
    pub session: DeliverySession,
}

// Admin DTOs
#[derive(serde::Serialize)]
struct AdminDashboardRequest {
    username: String,
    password: String,
    societe: String,
    date_debut: String,
}

#[derive(serde::Deserialize)]
pub struct AdminDashboardResponse {
    pub success: bool,
    pub districts: Vec<AdminDistrict>,
    pub total_packages: usize, // Total sin duplicados (calculado en backend)
    pub sso_token: String,
}

#[derive(serde::Serialize)]
struct AdminTourneePackagesRequest {
    sso_token: String,
    username: String,
    societe: String,
    date: String,
}

#[derive(serde::Serialize)]
struct CreateStatusChangeRequest {
    tracking_code: String,
    session_id: String,
    driver_matricule: String,
    notes: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct StatusChangeResponse {
    pub success: bool,
    pub request_id: String,
    pub message: String,
}

#[derive(serde::Serialize)]
struct ConfirmAndSendEmailRequest {
    request_id: String,
    admin_matricule: String,
}

// Extensiones de ApiClient para admin
impl ApiClient {
    /// Obtener dashboard admin
    pub async fn fetch_admin_dashboard(
        &self,
        username: &str,
        password: &str,
        societe: &str,
        date_debut: &str,
    ) -> Result<AdminDashboardResponse, String> {
        let url = format!("{}/v1/admin/dashboard", self.base_url);
        let request_body = AdminDashboardRequest {
            username: username.to_string(),
            password: password.to_string(),
            societe: societe.to_string(),
            date_debut: date_debut.to_string(),
        };
        
        let response = Request::post(&url)
            .json(&request_body)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
            
        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }
        
        response.json::<AdminDashboardResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
    
    /// Obtener paquetes de una tournée
    pub async fn fetch_tournee_packages(
        &self,
        code_tournee: &str,
        sso_token: &str,
        username: &str,
        societe: &str,
        date: &str,
    ) -> Result<Session, String> {
        let url = format!("{}/v1/admin/tournee/{}/packages", self.base_url, code_tournee);
        let request_body = AdminTourneePackagesRequest {
            sso_token: sso_token.to_string(),
            username: username.to_string(),
            societe: societe.to_string(),
            date: date.to_string(),
        };
        
        #[derive(serde::Deserialize)]
        struct FetchPackagesResponse {
            pub success: bool,
            pub session: Session,
        }
        
        let response = Request::post(&url)
            .json(&request_body)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
            
        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }
        
        let result: FetchPackagesResponse = response.json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
            
        Ok(result.session)
    }
    
    /// Crear request de cambio de status
    pub async fn create_status_change_request(
        &self,
        tracking_code: &str,
        session_id: &str,
        driver_matricule: &str,
        notes: Option<&str>,
    ) -> Result<StatusChangeResponse, String> {
        let url = format!("{}/v1/admin/status-change-request", self.base_url);
        let request_body = CreateStatusChangeRequest {
            tracking_code: tracking_code.to_string(),
            session_id: session_id.to_string(),
            driver_matricule: driver_matricule.to_string(),
            notes: notes.map(|s| s.to_string()),
        };
        
        let response = Request::post(&url)
            .json(&request_body)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
            
        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }
        
        response.json::<StatusChangeResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
    
    /// Obtener requests de cambio de status pendientes.
    /// `source`: identificador para debug (ej. "refresh_button", "notif_badge", "auto_refresh", "app_restore", "login", "confirm_modal").
    pub async fn fetch_status_requests(&self, source: &str) -> Result<Vec<StatusChangeRequest>, String> {
        let url = format!("{}/v1/admin/status-change-requests", self.base_url);
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("🔍 [FETCH] status_requests desde: {}", source)));
        
        let response = Request::get(&url)
            .header("X-Debug-Source", source)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
            
        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }
        
        response.json::<Vec<StatusChangeRequest>>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
    
    /// Confirmar y enviar email para un request
    /// Confirmar un request (sin enviar email) - se agrega al aperçu
    pub async fn confirm_request(
        &self,
        request_id: &str,
        admin_matricule: &str,
    ) -> Result<StatusChangeResponse, String> {
        let url = format!("{}/v1/admin/status-change-request/{}/confirm", self.base_url, request_id);
        let request_body = ConfirmAndSendEmailRequest {
            request_id: request_id.to_string(),
            admin_matricule: admin_matricule.to_string(),
        };
        
        let response = Request::post(&url)
            .json(&request_body)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
            
        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }
        
        response.json::<StatusChangeResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
    
    pub async fn confirm_and_send_email(
        &self,
        request_id: &str,
        admin_matricule: &str,
    ) -> Result<StatusChangeResponse, String> {
        let url = format!("{}/v1/admin/status-change-request/{}/confirm-and-send", self.base_url, request_id);
        let request_body = ConfirmAndSendEmailRequest {
            request_id: request_id.to_string(),
            admin_matricule: admin_matricule.to_string(),
        };
        
        let response = Request::post(&url)
            .json(&request_body)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
            
        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }
        
        response.json::<StatusChangeResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    /// Buscar tracking en todas las tournées
    pub async fn search_tracking(
        &self,
        request: &SearchTrackingRequest,
    ) -> Result<SearchTrackingResponse, String> {
        let url = format!("{}/v1/admin/search-tracking", self.base_url);
        let response = Request::post(&url)
            .json(request)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }
        response.json::<SearchTrackingResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    /// Cerrar el día: pasar confirmed a resolved + borrar sesiones de la societe
    pub async fn close_day(&self, societe: &str) -> Result<CloseDayResponse, String> {
        let url = format!("{}/v1/admin/status-change-requests/close-day", self.base_url);
        #[derive(serde::Serialize)]
        struct CloseDayRequest {
            societe: String,
        }
        let body = CloseDayRequest {
            societe: societe.to_string(),
        };
        let response = Request::post(&url)
            .json(&body)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }
        response.json::<CloseDayResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
    
    /// Obtener traçabilité de un paquete
    pub async fn fetch_package_traceability(
        &self,
        tracking_code: &str,
        sso_token: &str,
        username: &str,
        societe: &str,
    ) -> Result<PackageTraceabilityResponse, String> {
        let url = format!("{}/v1/admin/package-traceability", self.base_url);
        
        #[derive(serde::Serialize)]
        struct GetPackageTraceabilityRequest {
            tracking_code: String,
            sso_token: String,
            username: String,
            societe: String,
        }
        
        let request_body = GetPackageTraceabilityRequest {
            tracking_code: tracking_code.to_string(),
            sso_token: sso_token.to_string(),
            username: username.to_string(),
            societe: societe.to_string(),
        };
        
        let response = Request::post(&url)
            .json(&request_body)
            .map_err(|e| format!("Serialize error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
            
        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }
        
        response.json::<PackageTraceabilityResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }
}
