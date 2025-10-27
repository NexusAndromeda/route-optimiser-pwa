use gloo_net::http::Request;
use crate::models::{
    DeliverySession, CreateSessionRequest, CreateSessionResponse, 
    FetchPackagesRequest, FetchPackagesResponse, ScanRequest, ScanResponse
};
use crate::utils::BACKEND_URL;

/// Crear nueva sesión de entrega (login)
pub async fn create_session(username: &str, password: &str, societe: &str) -> Result<CreateSessionResponse, String> {
    let url = format!("{}/session/create", BACKEND_URL);
    let request_body = CreateSessionRequest {
        username: username.to_string(),
        password: password.to_string(),
        societe: societe.to_string(),
    };
    
    log::info!("🔐 Creando sesión para usuario: {}", username);
    
    let response = Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("HTTP error {}: {}", status, error_text));
    }
    
    let response_data = response
        .json::<CreateSessionResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    if response_data.success {
        log::info!("✅ Sesión creada exitosamente: {:?}", response_data.session_id);
    } else {
        log::error!("❌ Error creando sesión: {:?}", response_data.error);
    }
    
    Ok(response_data)
}

/// Obtener sesión por ID
pub async fn get_session(session_id: &str) -> Result<DeliverySession, String> {
    let url = format!("{}/session/{}", BACKEND_URL, session_id);
    
    log::info!("📋 Obteniendo sesión: {}", session_id);
    
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
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
pub async fn fetch_packages(session_id: &str, username: &str, password: &str, societe: &str) -> Result<FetchPackagesResponse, String> {
    let url = format!("{}/session/{}/fetch", BACKEND_URL, session_id);
    let request_body = FetchPackagesRequest {
        username: username.to_string(),
        password: password.to_string(),
        societe: societe.to_string(),
    };
    
    log::info!("📦 Obteniendo paquetes para sesión: {}", session_id);
    
    let response = Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
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

/// Escanear paquete
pub async fn scan_package(session_id: &str, tracking: &str) -> Result<ScanResponse, String> {
    let url = format!("{}/session/{}/scan", BACKEND_URL, session_id);
    let request_body = ScanRequest {
        session_id: session_id.to_string(),
        tracking: tracking.to_string(),
    };
    
    log::info!("📱 Escaneando paquete: {} en sesión: {}", tracking, session_id);
    
    let response = Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
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

/// Guardar sesión en localStorage
pub fn save_session_to_storage(session: &DeliverySession) -> Result<(), String> {
    use crate::utils::save_to_storage;
    
    let session_json = session.to_json()
        .map_err(|e| format!("Error serializando sesión: {}", e))?;
    
    save_to_storage("delivery_session", &session_json);
    log::info!("💾 Sesión guardada en localStorage");
    
    Ok(())
}

/// Cargar sesión desde localStorage
pub fn load_session_from_storage() -> Result<Option<DeliverySession>, String> {
    use crate::utils::load_from_storage;
    
    match load_from_storage::<String>("delivery_session") {
        Some(session_json) => {
            let session = DeliverySession::from_json(&session_json)
                .map_err(|e| format!("Error deserializando sesión: {}", e))?;
            
            log::info!("📋 Sesión cargada desde localStorage: {} paquetes", session.packages.len());
            Ok(Some(session))
        }
        None => {
            log::info!("ℹ️ No hay sesión guardada en localStorage");
            Ok(None)
        }
    }
}

/// Limpiar sesión del localStorage
pub fn clear_session_from_storage() {
    use crate::utils::remove_from_storage;
    
    remove_from_storage("delivery_session");
    log::info!("🗑️ Sesión eliminada del localStorage");
}
