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
        log::info!("🔐 [VIEWMODEL] Iniciando login_and_fetch para usuario: {} (societe: {})", username, societe);
        
        // 2. Crear sesión (login)
        log::info!("🔐 [VIEWMODEL] Llamando a api_client.create_session...");
        let create_response = match self.api_client.create_session(&username, &password, &societe).await {
            Ok(response) => {
                log::info!("✅ [VIEWMODEL] create_session respuesta recibida: success={}", response.success);
                response
            },
            Err(e) => {
                log::error!("❌ [VIEWMODEL] Error en create_session: {}", e);
                return Err(e);
            }
        };
        
        if !create_response.success {
            let error = create_response.error.unwrap_or_else(|| "Error creando sesión".to_string());
            log::error!("❌ [VIEWMODEL] create_session falló: {}", error);
            return Err(error);
        }
        
        let session = match create_response.session {
            Some(s) => {
                log::info!("✅ [VIEWMODEL] Sesión recibida: {}", s.session_id);
                
                // Log de direcciones con mailbox_access después de crear sesión
                for (addr_id, addr) in &s.addresses {
                    if addr.mailbox_access.is_some() {
                        log::info!("📬 [VIEWMODEL] Dirección {} tiene mailbox_access={:?} al crear sesión", addr_id, addr.mailbox_access);
                    }
                }
                
                s
            },
            None => {
                log::error!("❌ [VIEWMODEL] No se recibió sesión en la respuesta");
                return Err("No se recibió sesión en la respuesta".to_string());
            }
        };
        
        log::info!("✅ [VIEWMODEL] Sesión creada exitosamente: {} ({} paquetes)", 
            session.session_id, session.stats.total_packages);
        
        // Guardar sesión inicial
        log::info!("💾 [VIEWMODEL] Guardando sesión en localStorage...");
        if let Err(e) = self.offline_service.save_session(&session) {
            log::error!("❌ [VIEWMODEL] Error guardando sesión: {}", e);
        } else {
            log::info!("✅ [VIEWMODEL] Sesión guardada en localStorage exitosamente");
        }
        
        // 3. Fetch automático de paquetes
        log::info!("📦 [VIEWMODEL] Obteniendo paquetes automáticamente...");
        let fetch_response = match self.api_client.fetch_packages(
            &session.session_id,
            &username,
            &password,
            &societe,
        ).await {
            Ok(response) => {
                log::info!("✅ [VIEWMODEL] fetch_packages respuesta recibida: success={}, new_packages={:?}", 
                    response.success, response.new_packages_count);
                response
            },
            Err(e) => {
                log::error!("❌ [VIEWMODEL] Error obteniendo paquetes: {}", e);
                return Err(e);
            }
        };
        
        if !fetch_response.success {
            let error = fetch_response.error.unwrap_or_else(|| "Error obteniendo paquetes".to_string());
            log::error!("❌ [VIEWMODEL] fetch_packages falló: {}", error);
            return Err(error);
        }
        
        let updated_session = match fetch_response.session {
            Some(s) => {
                log::info!("✅ [VIEWMODEL] Sesión actualizada recibida: {} ({} paquetes)", 
                    s.session_id, s.stats.total_packages);
                
                // Log de direcciones con mailbox_access
                for (addr_id, addr) in &s.addresses {
                    if addr.mailbox_access.is_some() {
                        log::info!("📬 [VIEWMODEL] Dirección {} tiene mailbox_access={:?}", addr_id, addr.mailbox_access);
                    }
                }
                
                s
            },
            None => {
                log::error!("❌ [VIEWMODEL] No se recibió sesión actualizada");
                return Err("No se recibió sesión actualizada".to_string());
            }
        };
        
        log::info!("✅ [VIEWMODEL] Paquetes obtenidos: {} nuevos", 
                   fetch_response.new_packages_count.unwrap_or(0));
        
        // Guardar sesión actualizada
        log::info!("💾 [VIEWMODEL] Guardando sesión actualizada en localStorage...");
        if let Err(e) = self.offline_service.save_session(&updated_session) {
            log::error!("❌ [VIEWMODEL] Error guardando sesión actualizada: {}", e);
        } else {
            log::info!("✅ [VIEWMODEL] Sesión actualizada guardada en localStorage exitosamente");
        }
        
        log::info!("✅ [VIEWMODEL] login_and_fetch completado exitosamente");
        Ok(updated_session)
    }
    
    /// Login inteligente: verifica sesión local + backend antes de crear nueva
    /// Si encuentra sesión existente por driver_id + company_id, la recupera y hace sync incremental (solo cambios nuevos)
    pub async fn login_smart(
        &self,
        username: String,
        password: String,
        societe: String,
    ) -> Result<DeliverySession, String> {
        log::info!("🔐 [LOGIN_SMART] Iniciando login inteligente para usuario: {} (societe: {})", username, societe);
        
        // 1. Verificar si existe sesión LOCAL con estos credenciales
        let local_session_opt = match self.offline_service.load_session() {
            Ok(Some(session)) => {
                if session.driver.driver_id == username && session.driver.company_id == societe {
                    log::info!("✅ [LOGIN_SMART] Sesión local encontrada: {} ({} paquetes)", 
                        session.session_id, session.stats.total_packages);
                    Some(session)
                } else {
                    log::info!("⚠️ [LOGIN_SMART] Sesión local con credenciales diferentes, ignorando");
                    None
                }
            }
            Ok(None) => {
                log::info!("📋 [LOGIN_SMART] No hay sesión local");
                None
            }
            Err(e) => {
                log::warn!("⚠️ [LOGIN_SMART] Error cargando sesión local: {}", e);
                None
            }
        };
        
        // 2. Verificar si existe sesión en BACKEND (por driver_id + company_id)
        log::info!("🔍 [LOGIN_SMART] Verificando sesión en backend...");
        match self.api_client.find_session_by_driver(&username, &societe).await {
            Ok(Some(backend_session)) => {
                // ✅ Sesión existe en backend
                log::info!("✅ [LOGIN_SMART] Sesión encontrada en backend: {} ({} paquetes)", 
                    backend_session.session_id, backend_session.stats.total_packages);
                
                // ⚠️ NUEVO: Primero refrescar token para obtener token nuevo
                log::info!("🔐 [LOGIN_SMART] Refrescando token...");
                match self.api_client.refresh_token(
                    &backend_session.session_id,
                    &username,
                    &password,
                    &societe,
                ).await {
                    Ok(response) => {
                        if response.success {
                            let session_with_new_token = response.session;
                            log::info!("✅ [LOGIN_SMART] Token actualizado exitosamente");
                            
                            // Guardar sesión con token nuevo en local
                            if let Err(e) = self.offline_service.save_session(&session_with_new_token) {
                                log::warn!("⚠️ [LOGIN_SMART] Error guardando sesión con token nuevo: {}", e);
                            } else {
                                log::info!("💾 [LOGIN_SMART] Sesión con token nuevo guardada en local");
                            }
                            
                            // Ahora hacer sync incremental con token nuevo
                            log::info!("🔄 [LOGIN_SMART] Ejecutando sync incremental con token nuevo...");
                            match self.sync_incremental(&session_with_new_token.session_id, &username, &societe, None).await {
                                Ok(updated_session) => {
                                    log::info!("✅ [LOGIN_SMART] Sync incremental completado: {} paquetes", 
                                        updated_session.stats.total_packages);
                                    Ok(updated_session)
                                }
                                Err(e) => {
                                    log::warn!("⚠️ [LOGIN_SMART] Error en sync incremental: {}, usando sesión con token nuevo", e);
                                    Ok(session_with_new_token)
                                }
                            }
                        } else {
                            log::warn!("⚠️ [LOGIN_SMART] Respuesta de refresh_token no exitosa, usando sesión existente");
                            // Fallback: usar sesión existente y hacer sync incremental
                            if let Err(e) = self.offline_service.save_session(&backend_session) {
                                log::warn!("⚠️ [LOGIN_SMART] Error guardando sesión: {}", e);
                            }
                            self.sync_incremental(&backend_session.session_id, &username, &societe, None).await
                        }
                    }
                    Err(e) => {
                        log::warn!("⚠️ [LOGIN_SMART] Error refrescando token: {}, usando sesión existente", e);
                        // Fallback: usar sesión existente y hacer sync incremental
                        if let Err(e) = self.offline_service.save_session(&backend_session) {
                            log::warn!("⚠️ [LOGIN_SMART] Error guardando sesión: {}", e);
                        } else {
                            log::info!("💾 [LOGIN_SMART] Sesión del backend guardada en local");
                        }
                        match self.sync_incremental(&backend_session.session_id, &username, &societe, None).await {
                            Ok(updated_session) => {
                                log::info!("✅ [LOGIN_SMART] Sync incremental completado: {} paquetes", 
                                    updated_session.stats.total_packages);
                                Ok(updated_session)
                            }
                            Err(e) => {
                                log::warn!("⚠️ [LOGIN_SMART] Error en sync incremental: {}, usando sesión del backend sin actualizar", e);
                                Ok(backend_session)
                            }
                        }
                    }
                }
            }
            Ok(None) => {
                // No existe en backend - crear nueva sesión
                log::info!("📋 [LOGIN_SMART] No hay sesión en backend, creando nueva sesión");
                self.login_and_fetch(username, password, societe).await
            }
            Err(e) => {
                log::warn!("⚠️ [LOGIN_SMART] Error verificando backend: {}, procediendo con login normal", e);
                self.login_and_fetch(username, password, societe).await
            }
        }
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
    
    /// Actualizar solo campos específicos de dirección
    pub async fn update_address_fields(
        &self,
        session_id: &str,
        address_id: &str,
        door_code: Option<String>,
        has_mailbox_access: Option<bool>,
        driver_notes: Option<String>,
    ) -> Result<DeliverySession, String> {
        log::info!("📝 [VIEWMODEL] Actualizando campos de dirección: {} en sesión: {}", address_id, session_id);
        log::info!("📬 [VIEWMODEL] Valores a actualizar - door_code={:?}, has_mailbox_access={:?}, driver_notes={:?}", 
                   door_code.is_some(), has_mailbox_access, driver_notes.is_some());
        
        let response = self.api_client.update_address_fields(
            session_id,
            address_id,
            door_code,
            has_mailbox_access,
            driver_notes,
        ).await?;
        
        if !response.success {
            log::error!("❌ [VIEWMODEL] La respuesta del API indicó success=false");
            return Err("Error actualizando campos de dirección".to_string());
        }
        
        let updated_session = response.session;
        
        // Verificar que la dirección se actualizó correctamente en la sesión
        if let Some(addr) = updated_session.addresses.get(address_id) {
            log::info!("📬 [VIEWMODEL] Dirección después de actualizar - mailbox_access={:?}, door_code={:?}, driver_notes={:?}",
                      addr.mailbox_access, addr.door_code.is_some(), addr.driver_notes.is_some());
        } else {
            log::warn!("⚠️ [VIEWMODEL] Dirección no encontrada en sesión actualizada: {}", address_id);
        }
        
        // Guardar sesión actualizada
        if let Err(e) = self.offline_service.save_session(&updated_session) {
            log::error!("❌ [VIEWMODEL] Error guardando sesión actualizada: {}", e);
        } else {
            log::info!("💾 [VIEWMODEL] Sesión actualizada guardada en storage local");
        }
        
        log::info!("✅ [VIEWMODEL] Campos de dirección actualizados exitosamente");
        Ok(updated_session)
    }
    
    /// Actualizar dirección completa (para direcciones problemáticas)
    pub async fn update_address(
        &self,
        session_id: &str,
        address_id: &str,
        new_label: String,
    ) -> Result<DeliverySession, String> {
        log::info!("📍 Actualizando dirección: {} → {}", address_id, new_label);
        
        let response = self.api_client.update_address(
            session_id,
            address_id,
            new_label.clone(),
            0.0, // El backend hará geocoding si es necesario
            0.0,
        ).await?;
        
        if !response.success {
            return Err("Error actualizando dirección".to_string());
        }
        
        let updated_session = response.session;
        
        // Guardar sesión actualizada
        if let Err(e) = self.offline_service.save_session(&updated_session) {
            log::error!("❌ Error guardando sesión actualizada: {}", e);
        }
        
        log::info!("✅ Dirección actualizada exitosamente: {}", new_label);
        Ok(updated_session)
    }
    
    /// Marcar paquete como problemático (coordenadas 0.0, 0.0)
    pub async fn mark_as_problematic(
        &self,
        session_id: &str,
        address_id: &str,
    ) -> Result<DeliverySession, String> {
        log::info!("⚠️ Marcando dirección como problemática: {}", address_id);
        
        // Obtener sesión actual para obtener el label original de la dirección
        let session = self.offline_service.load_session()
            .map_err(|e| format!("Error cargando sesión: {}", e))?
            .ok_or_else(|| "Sesión no encontrada".to_string())?;
        
        // Verificar que la sesión cargada coincide con el session_id proporcionado
        if session.session_id != session_id {
            return Err(format!("Session ID mismatch: expected {}, got {}", session_id, session.session_id));
        }
        
        // Obtener dirección actual para mantener el label original
        let address = session.addresses.get(address_id)
            .ok_or_else(|| "Dirección no encontrada".to_string())?;
        
        let original_label = address.label.clone();
        
        log::info!("📍 Manteniendo dirección original: '{}'", original_label);
        log::info!("📍 Estableciendo coordenadas a 0.0, 0.0 para marcar como problemática");
        
        // Actualizar dirección con coordenadas 0.0, 0.0 (mantener label original)
        let response = self.api_client.update_address(
            session_id,
            address_id,
            original_label, // Mantener dirección original
            0.0, // Coordenadas a 0.0, 0.0 para marcar como problemática
            0.0,
        ).await?;
        
        if !response.success {
            return Err("Error marcando como problemática".to_string());
        }
        
        let updated_session = response.session;
        
        // Guardar sesión actualizada
        if let Err(e) = self.offline_service.save_session(&updated_session) {
            log::error!("❌ Error guardando sesión actualizada: {}", e);
        }
        
        log::info!("✅ Dirección marcada como problemática exitosamente");
        Ok(updated_session)
    }
    
    /// Sincronización incremental
    pub async fn sync_incremental(
        &self,
        session_id: &str,
        username: &str,
        societe: &str,
        date: Option<&str>,
    ) -> Result<DeliverySession, String> {
        log::info!("🔄 Iniciando sincronización incremental para sesión: {}", session_id);
        
        let response = self.api_client.sync_incremental(
            session_id,
            username,
            societe,
            date,
        ).await?;
        
        if !response.success {
            return Err("Error en sincronización incremental".to_string());
        }
        
        let updated_session = response.session;
        
        // Log de direcciones con mailbox_access después de sync
        for (addr_id, addr) in &updated_session.addresses {
            if addr.mailbox_access.is_some() {
                log::info!("📬 [SYNC] Dirección {} tiene mailbox_access={:?} después de sync", addr_id, addr.mailbox_access);
            }
        }
        
        // Aplicar deltas a sesión local si es necesario
        // Por ahora, simplemente usar la sesión actualizada del backend
        // TODO: En el futuro, aplicar deltas de forma más granular
        
        // Guardar sesión actualizada
        if let Err(e) = self.offline_service.save_session(&updated_session) {
            log::error!("❌ Error guardando sesión actualizada: {}", e);
        } else {
            log::info!("💾 [SYNC] Sesión actualizada guardada en storage local");
        }
        
        log::info!("✅ Sincronización incremental completada: {} nuevos, {} actualizados, {} eliminados",
            response.delta.added.len(), response.delta.updated.len(), response.delta.removed.len());
        
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
