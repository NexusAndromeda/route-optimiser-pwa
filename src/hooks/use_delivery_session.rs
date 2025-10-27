use yew::prelude::*;
use web_sys::window;
use crate::models::{DeliverySession, Company, DriverInfo};
use crate::services::{
    create_session, get_session, scan_package,
    save_session_to_storage, load_session_from_storage, clear_session_from_storage
};
use crate::services::delivery_session_service::fetch_packages as fetch_delivery_packages;

#[derive(Clone, PartialEq)]
pub struct DeliverySessionState {
    pub session: Option<DeliverySession>,
    pub loading: bool,
    pub error: Option<String>,
    pub last_fetch_time: Option<i64>,
}

pub struct UseDeliverySessionHandle {
    pub state: UseStateHandle<DeliverySessionState>,
    pub login_and_fetch: Callback<(String, String, String)>,
    pub fetch_packages: Callback<()>,
    pub scan_package: Callback<String>,
    pub clear_session: Callback<()>,
    pub refresh_session: Callback<()>,
}

#[hook]
pub fn use_delivery_session() -> UseDeliverySessionHandle {
    let state = use_state(|| DeliverySessionState {
        session: None,
        loading: false,
        error: None,
        last_fetch_time: None,
    });
    
    // Cargar sesión guardada al montar
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            let state = state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match load_session_from_storage() {
                    Ok(Some(saved_session)) => {
                        log::info!("📋 Sesión cargada desde localStorage");
                        let mut current_state = (*state).clone();
                        current_state.session = Some(saved_session);
                        current_state.last_fetch_time = Some(chrono::Utc::now().timestamp());
                        state.set(current_state);
                    }
                    Ok(None) => {
                        log::info!("ℹ️ No hay sesión guardada");
                    }
                    Err(e) => {
                        log::error!("❌ Error cargando sesión: {}", e);
                        let mut current_state = (*state).clone();
                        current_state.error = Some(e);
                        state.set(current_state);
                    }
                }
            });
            || ()
        });
    }
    
    // Login y fetch automático de paquetes
    let login_and_fetch = {
        let state = state.clone();
        Callback::from(move |(username, password, societe): (String, String, String)| {
            let state = state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                log::info!("🔐 Iniciando login y fetch de paquetes...");
                
                // 1. Crear sesión (login)
                let mut current_state = (*state).clone();
                current_state.loading = true;
                current_state.error = None;
                state.set(current_state);
                
                match create_session(&username, &password, &societe).await {
                    Ok(response) => {
                        if response.success {
                            if let Some(session) = response.session {
                                log::info!("✅ Sesión creada exitosamente");
                                
                                // Guardar sesión en localStorage
                                if let Err(e) = save_session_to_storage(&session) {
                                    log::error!("❌ Error guardando sesión: {}", e);
                                }
                                
                                // 2. Fetch automático de paquetes
                                log::info!("📦 Obteniendo paquetes automáticamente...");
                                match fetch_delivery_packages(&session.session_id, &username, &password, &societe).await {
                                    Ok(fetch_response) => {
                                        if fetch_response.success {
                                            if let Some(updated_session) = fetch_response.session {
                                                log::info!("✅ Paquetes obtenidos: {} nuevos", 
                                                          fetch_response.new_packages_count.unwrap_or(0));
                                                
                                                // Guardar sesión actualizada
                                                if let Err(e) = save_session_to_storage(&updated_session) {
                                                    log::error!("❌ Error guardando sesión actualizada: {}", e);
                                                }
                                                
                                                let mut current_state = (*state).clone();
                                                current_state.session = Some(updated_session);
                                                current_state.loading = false;
                                                current_state.last_fetch_time = Some(chrono::Utc::now().timestamp());
                                                state.set(current_state);
                                            } else {
                                                log::error!("❌ No se recibió sesión actualizada");
                                                let mut current_state = (*state).clone();
                                                current_state.loading = false;
                                                current_state.error = Some("No se recibió sesión actualizada".to_string());
                                                state.set(current_state);
                                            }
                                        } else {
                                            log::error!("❌ Error obteniendo paquetes: {:?}", fetch_response.error);
                                            let mut current_state = (*state).clone();
                                            current_state.loading = false;
                                            current_state.error = fetch_response.error;
                                            state.set(current_state);
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("❌ Error en fetch de paquetes: {}", e);
                                        let mut current_state = (*state).clone();
                                        current_state.loading = false;
                                        current_state.error = Some(e);
                                        state.set(current_state);
                                    }
                                }
                            } else {
                                log::error!("❌ No se recibió sesión en la respuesta");
                                let mut current_state = (*state).clone();
                                current_state.loading = false;
                                current_state.error = Some("No se recibió sesión en la respuesta".to_string());
                                state.set(current_state);
                            }
                        } else {
                            log::error!("❌ Error creando sesión: {:?}", response.error);
                            let mut current_state = (*state).clone();
                            current_state.loading = false;
                            current_state.error = response.error;
                            state.set(current_state);
                        }
                    }
                    Err(e) => {
                        log::error!("❌ Error en login: {}", e);
                        let mut current_state = (*state).clone();
                        current_state.loading = false;
                        current_state.error = Some(e);
                        state.set(current_state);
                    }
                }
            });
        })
    };
    
    // Fetch manual de paquetes
    let fetch_packages = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            let current_session = (*state).session.clone();
            
            if let Some(session) = current_session {
                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("📦 Obteniendo paquetes manualmente...");
                    
                    let mut current_state = (*state).clone();
                    current_state.loading = true;
                    current_state.error = None;
                    state.set(current_state);
                    
                    // Necesitamos las credenciales del driver para el fetch
                    let username = session.driver.driver_id.clone();
                    let password = "".to_string(); // TODO: Necesitamos guardar la password
                    let societe = session.driver.company_id.clone();
                    
                    match fetch_delivery_packages(&session.session_id, &username, &password, &societe).await {
                        Ok(response) => {
                            if response.success {
                                if let Some(updated_session) = response.session {
                                    log::info!("✅ Paquetes actualizados: {} nuevos", 
                                              response.new_packages_count.unwrap_or(0));
                                    
                                    // Guardar sesión actualizada
                                    if let Err(e) = save_session_to_storage(&updated_session) {
                                        log::error!("❌ Error guardando sesión actualizada: {}", e);
                                    }
                                    
                                    let mut current_state = (*state).clone();
                                    current_state.session = Some(updated_session);
                                    current_state.loading = false;
                                    current_state.last_fetch_time = Some(chrono::Utc::now().timestamp());
                                    state.set(current_state);
                                } else {
                                    log::error!("❌ No se recibió sesión actualizada");
                                    let mut current_state = (*state).clone();
                                    current_state.loading = false;
                                    current_state.error = Some("No se recibió sesión actualizada".to_string());
                                    state.set(current_state);
                                }
                            } else {
                                log::error!("❌ Error obteniendo paquetes: {:?}", response.error);
                                let mut current_state = (*state).clone();
                                current_state.loading = false;
                                current_state.error = response.error;
                                state.set(current_state);
                            }
                        }
                        Err(e) => {
                            log::error!("❌ Error en fetch de paquetes: {}", e);
                            let mut current_state = (*state).clone();
                            current_state.loading = false;
                            current_state.error = Some(e);
                            state.set(current_state);
                        }
                    }
                });
            } else {
                log::error!("❌ No hay sesión activa para obtener paquetes");
                if let Some(win) = window() {
                    let _ = win.alert_with_message("No hay sesión activa. Por favor, inicie sesión primero.");
                }
            }
        })
    };
    
    // Escanear paquete
    let scan_package = {
        let state = state.clone();
        Callback::from(move |tracking: String| {
            let state = state.clone();
            let current_session = (*state).session.clone();
            
            if let Some(session) = current_session {
                let session_id = session.session_id.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("📱 Escaneando paquete: {}", tracking);
                    
                    match scan_package(&session_id, &tracking).await {
                        Ok(response) => {
                            if response.found {
                                log::info!("✅ Paquete escaneado: {} (posición: {:?})", 
                                          tracking, response.route_position);
                                
                                // Actualizar la sesión local con el paquete escaneado
                                let mut current_state = (*state).clone();
                                if let Some(mut session) = current_state.session {
                                    if let Some(package) = response.package {
                                        // Actualizar el paquete en la sesión
                                        session.packages.insert(package.internal_id.clone(), package);
                                        
                                        // Guardar sesión actualizada
                                        if let Err(e) = save_session_to_storage(&session) {
                                            log::error!("❌ Error guardando sesión actualizada: {}", e);
                                        }
                                        
                                        current_state.session = Some(session);
                                        state.set(current_state);
                                    }
                                }
                            } else {
                                log::warn!("⚠️ Paquete no encontrado: {}", tracking);
                                if let Some(win) = window() {
                                    let _ = win.alert_with_message(&format!("Paquete no encontrado: {}", tracking));
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("❌ Error escaneando paquete: {}", e);
                            if let Some(win) = window() {
                                let _ = win.alert_with_message(&format!("Error escaneando paquete: {}", e));
                            }
                        }
                    }
                });
            } else {
                log::error!("❌ No hay sesión activa para escanear");
                if let Some(win) = window() {
                    let _ = win.alert_with_message("No hay sesión activa. Por favor, inicie sesión primero.");
                }
            }
        })
    };
    
    // Limpiar sesión
    let clear_session = {
        let state = state.clone();
        Callback::from(move |_| {
            log::info!("🗑️ Limpiando sesión");
            clear_session_from_storage();
            
            let mut current_state = (*state).clone();
            current_state.session = None;
            current_state.loading = false;
            current_state.error = None;
            current_state.last_fetch_time = None;
            state.set(current_state);
        })
    };
    
    // Refrescar sesión desde el servidor
    let refresh_session = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            let current_session = (*state).session.clone();
            
            if let Some(session) = current_session {
                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("🔄 Refrescando sesión desde el servidor...");
                    
                    let mut current_state = (*state).clone();
                    current_state.loading = true;
                    current_state.error = None;
                    state.set(current_state);
                    
                    match get_session(&session.session_id).await {
                        Ok(updated_session) => {
                            log::info!("✅ Sesión refrescada: {} paquetes, {} direcciones", 
                                      updated_session.packages.len(), updated_session.addresses.len());
                            
                            // Guardar sesión actualizada
                            if let Err(e) = save_session_to_storage(&updated_session) {
                                log::error!("❌ Error guardando sesión actualizada: {}", e);
                            }
                            
                            let mut current_state = (*state).clone();
                            current_state.session = Some(updated_session);
                            current_state.loading = false;
                            current_state.last_fetch_time = Some(chrono::Utc::now().timestamp());
                            state.set(current_state);
                        }
                        Err(e) => {
                            log::error!("❌ Error refrescando sesión: {}", e);
                            let mut current_state = (*state).clone();
                            current_state.loading = false;
                            current_state.error = Some(e);
                            state.set(current_state);
                        }
                    }
                });
            } else {
                log::error!("❌ No hay sesión activa para refrescar");
            }
        })
    };
    
    UseDeliverySessionHandle {
        state,
        login_and_fetch,
        fetch_packages,
        scan_package,
        clear_session,
        refresh_session,
    }
}
