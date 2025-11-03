// ============================================================================
// SESSION VIEWMODEL - LÓGICA DE SESIÓN
// ============================================================================
// Lógica de negocio de sesión - SIN yewdux
// Devuelve valores, los hooks actualizan el estado
// ============================================================================

use crate::services::{ApiClient, OfflineService};
use crate::services::api_client::OptimizeRouteResponse;
use crate::models::{session::DeliverySession, sync::Change};
use wasm_bindgen::JsCast;

/// ViewModel de sesión - SOLO lógica de negocio
pub struct SessionViewModel {
    api_client: ApiClient,
    offline_service: OfflineService,
}

impl SessionViewModel {
    pub fn new() -> Self {
        Self {
            api_client: ApiClient::new(),
            offline_service: OfflineService::new(),
        }
    }
    
    /// Login y fetch automático de paquetes
    pub async fn login_and_fetch(
        &self,
        username: String,
        password: String,
        societe: String,
    ) -> Result<DeliverySession, String> {
        log::info!("🔐 Iniciando login y fetch de paquetes...");
        
        // 2. Crear sesión (login)
        let create_response = match self.api_client.create_session(&username, &password, &societe).await {
            Ok(response) => response,
            Err(e) => return Err(e),
        };
        
        if !create_response.success {
            let error = create_response.error.unwrap_or_else(|| "Error creando sesión".to_string());
            return Err(error);
        }
        
        let session = match create_response.session {
            Some(s) => s,
            None => return Err("No se recibió sesión en la respuesta".to_string()),
        };
        
        log::info!("✅ Sesión creada exitosamente: {}", session.session_id);
        
        // Guardar sesión inicial
        if let Err(e) = self.offline_service.save_session(&session) {
            log::error!("❌ Error guardando sesión: {}", e);
        }
        
        // 3. Fetch automático de paquetes
        log::info!("📦 Obteniendo paquetes automáticamente...");
        let fetch_response = match self.api_client.fetch_packages(
            &session.session_id,
            &username,
            &password,
            &societe,
        ).await {
            Ok(response) => response,
            Err(e) => {
                log::error!("❌ Error obteniendo paquetes: {}", e);
                return Err(e);
            }
        };
        
        if !fetch_response.success {
            let error = fetch_response.error.unwrap_or_else(|| "Error obteniendo paquetes".to_string());
            return Err(error);
        }
        
        let updated_session = match fetch_response.session {
            Some(s) => s,
            None => return Err("No se recibió sesión actualizada".to_string()),
        };
        
        log::info!("✅ Paquetes obtenidos: {} nuevos", 
                   fetch_response.new_packages_count.unwrap_or(0));
        
        // Guardar sesión actualizada
        if let Err(e) = self.offline_service.save_session(&updated_session) {
            log::error!("❌ Error guardando sesión actualizada: {}", e);
        }
        
        Ok(updated_session)
    }
    
    /// Fetch manual de paquetes
    pub async fn fetch_packages(&self) -> Result<DeliverySession, String> {
        // Necesitamos la sesión actual - esto debe venir del hook
        // Por ahora retornamos error
        Err("fetch_packages necesita sesión actual del hook".to_string())
    }
    
    /// Escanear paquete (Optimistic UI)
    pub async fn scan_package(&self, tracking: &str, current_session: &DeliverySession) -> Result<(DeliverySession, Change), String> {
        let mut session = current_session.clone();
        
        match session.find_by_tracking(tracking) {
            Some(_) => {
                if let Err(e) = session.mark_scanned(tracking) {
                    return Err(e);
                }
                
                // Guardar en localStorage
                if let Err(e) = self.offline_service.save_session(&session) {
                    log::error!("❌ Error guardando sesión: {}", e);
                }
                
                // Crear cambio pendiente
                let change = Change::PackageScanned {
                    tracking: tracking.to_string(),
                    new_status: "STATUT_SCANNED".to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                };
                
                log::info!("✅ Paquete {} escaneado localmente, pendiente de sync", tracking);
                Ok((session, change))
            }
            None => {
                Err(format!("Paquete no encontrado: {}", tracking))
            }
        }
    }
    
    /// Limpiar sesión
    pub fn clear_session(&self) {
        log::info!("🗑️ Limpiando sesión");
        let _ = self.offline_service.clear_pending_changes();
    }
    
    /// Logout completo - limpia toda la sesión y cambios pendientes
    pub fn logout(&self) -> Result<(), String> {
        log::info!("👋 Logout - limpiando toda la sesión");
        
        // Limpiar localStorage
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                // Limpiar sesión
                let _ = storage.remove_item("delivery_session");
                // Limpiar cambios pendientes
                let _ = storage.remove_item("pending_changes");
                // Limpiar cualquier auth data
                let _ = storage.remove_item("auth_state");
                log::info!("✅ LocalStorage limpiado");
            }
        }
        
        // Limpiar cambios pendientes
        if let Err(e) = self.offline_service.clear_pending_changes() {
            log::warn!("⚠️ Error limpiando cambios pendientes: {}", e);
        }
        
        Ok(())
    }
    
    /// Refrescar sesión desde backend
    pub async fn refresh_session(&self, session_id: &str) -> Result<DeliverySession, String> {
        log::info!("🔄 Refrescando sesión desde backend...");
        
        match self.api_client.get_session(session_id).await {
            Ok(updated_session) => {
                if let Err(e) = self.offline_service.save_session(&updated_session) {
                    log::error!("❌ Error guardando sesión: {}", e);
                }
                Ok(updated_session)
            }
            Err(e) => Err(e)
        }
    }
    
    /// Cargar sesión desde storage al iniciar
    pub async fn load_session_from_storage(&self) -> Result<Option<DeliverySession>, String> {
        self.offline_service.load_session()
    }
    
    /// Agregar cambio pendiente (helper para hooks)
    pub async fn add_pending_change(&self, change: Change) {
        // Guardar en queue persistente
        if let Ok(Some(mut queue)) = self.offline_service.load_pending_changes() {
            queue.changes.push(change);
            let _ = self.offline_service.save_pending_changes(&queue.changes);
        } else {
            let changes = vec![change];
            let _ = self.offline_service.save_pending_changes(&changes);
        }
    }
    
    /// Optimizar ruta usando la localización del chofer desde Mapbox
    pub async fn optimize_route(&self, session_id: &str) -> Result<DeliverySession, String> {
        log::info!("🗺️ Iniciando optimización de ruta para sesión: {}", session_id);
        
        // 1. Obtener localización del chofer desde Mapbox
        let driver_location = get_driver_location_from_mapbox()
            .ok_or_else(|| {
                "No hay ubicación del chofer disponible. Por favor, activa la geolocalización primero.".to_string()
            })?;
        
        log::info!("📍 Ubicación del chofer obtenida: ({}, {})", 
                   driver_location.latitude, driver_location.longitude);
        
        // 2. Llamar al backend para optimizar
        let response = self.api_client.optimize_route(
            session_id,
            driver_location.latitude,
            driver_location.longitude,
        ).await?;
        
        if !response.success {
            return Err("Error optimizando ruta en el backend".to_string());
        }
        
        // 3. Guardar sesión actualizada con el orden optimizado
        let updated_session = response.session;
        if let Err(e) = self.offline_service.save_session(&updated_session) {
            log::error!("❌ Error guardando sesión optimizada: {}", e);
        }
        
        log::info!("✅ Ruta optimizada: {} paradas, tiempo estimado: {} minutos", 
                   response.total_stops, response.estimated_time_seconds / 60);
        
        Ok(updated_session)
    }
}

/// Obtener localización del chofer desde Mapbox JavaScript
fn get_driver_location_from_mapbox() -> Option<DriverLocation> {
    let window = web_sys::window()?;
    
    // Llamar a window.getDriverLocation()
    let get_driver_location = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("getDriverLocation"))
        .ok()?;
    
    let func = get_driver_location.dyn_ref::<js_sys::Function>()?;
    let result = func.call0(&wasm_bindgen::JsValue::NULL).ok()?;
    
    // Si es null, no hay ubicación
    if result.is_null() {
        return None;
    }
    
    // Parsear el objeto {latitude, longitude}
    let latitude = js_sys::Reflect::get(&result, &wasm_bindgen::JsValue::from_str("latitude"))
        .ok()?
        .as_f64()?;
    let longitude = js_sys::Reflect::get(&result, &wasm_bindgen::JsValue::from_str("longitude"))
        .ok()?
        .as_f64()?;
    
    Some(DriverLocation {
        latitude,
        longitude,
    })
}

#[derive(Debug)]
struct DriverLocation {
    latitude: f64,
    longitude: f64,
}

impl Default for SessionViewModel {
    fn default() -> Self {
        Self::new()
    }
}
