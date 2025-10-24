use crate::models::{
    DeliverySession, CreateSessionRequest, SyncRequest, SyncResponse, 
    InitialFetchResponse, LoadSessionParams
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;

const API_BASE_URL: &str = "http://localhost:3000";

/// Crear nueva sesión de delivery
pub async fn api_create_session(request: CreateSessionRequest) -> Result<InitialFetchResponse, String> {
    let url = format!("{}/session/create", API_BASE_URL);
    
    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("Error serializando request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Error enviando request: {}", e))?;
    
    if response.ok() {
        let session: InitialFetchResponse = response
            .json()
            .await
            .map_err(|e| format!("Error parseando respuesta: {}", e))?;
        Ok(session)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Error desconocido".to_string());
        Err(format!("Error del servidor ({}): {}", response.status(), error_text))
    }
}

/// Cargar sesión existente
pub async fn api_load_session(session_id: &str) -> Result<DeliverySession, String> {
    let url = format!("{}/session/load?session_id={}", API_BASE_URL, session_id);
    
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Error enviando request: {}", e))?;
    
    if response.ok() {
        let session: DeliverySession = response
            .json()
            .await
            .map_err(|e| format!("Error parseando respuesta: {}", e))?;
        Ok(session)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Error desconocido".to_string());
        Err(format!("Error del servidor ({}): {}", response.status(), error_text))
    }
}

/// Sincronizar sesión
pub async fn api_sync_session(request: SyncRequest) -> Result<SyncResponse, String> {
    let url = format!("{}/session/sync", API_BASE_URL);
    
    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("Error serializando request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Error enviando request: {}", e))?;
    
    if response.ok() {
        let sync_response: SyncResponse = response
            .json()
            .await
            .map_err(|e| format!("Error parseando respuesta: {}", e))?;
        Ok(sync_response)
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Error desconocido".to_string());
        Err(format!("Error del servidor ({}): {}", response.status(), error_text))
    }
}
