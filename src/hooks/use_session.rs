// ============================================================================
// USE SESSION HOOK - Reemplazo de Yewdux Store
// ============================================================================
// Hook nativo de Yew para manejar estado de sesión
// Compatible con Rust 1.90 (sin yewdux)
// ============================================================================

use yew::prelude::*;
use crate::stores::SessionStore;
use crate::viewmodels::SessionViewModel;

#[derive(Clone)]
pub struct UseSessionHandle {
    pub state: UseStateHandle<SessionStore>,
    pub login_and_fetch: Callback<(String, String, String)>,
    pub fetch_packages: Callback<()>,
    pub scan_package: Callback<String>,
    pub clear_session: Callback<()>,
    pub refresh_session: Callback<()>,
}

#[hook]
pub fn use_session() -> UseSessionHandle {
    let state = use_state(|| SessionStore::default());
    // Login and fetch
    let login_and_fetch = {
        let state = state.clone();
        Callback::from(move |(username, password, societe): (String, String, String)| {
            let state = state.clone();
            let vm = SessionViewModel::new(); // Crear nuevo en lugar de clonar
            wasm_bindgen_futures::spawn_local(async move {
                let result = vm.login_and_fetch(username, password, societe).await;
                if let Ok(session) = result {
                    let mut new_state = (*state).clone();
                    new_state.session = Some(session);
                    new_state.loading = false;
                    state.set(new_state);
                } else {
                    let mut new_state = (*state).clone();
                    new_state.error = Some("Error en login".to_string());
                    new_state.loading = false;
                    state.set(new_state);
                }
            });
        })
    };
    
    // Fetch packages - necesita credenciales, por ahora placeholder
    let fetch_packages = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            log::warn!("⚠️ fetch_packages necesita implementación completa con credenciales");
            let mut new_state = (*state).clone();
            new_state.loading = false;
            state.set(new_state);
        })
    };
    
    // Scan package
    let scan_package = {
        let state = state.clone();
        Callback::from(move |tracking: String| {
            let state = state.clone();
            let vm = SessionViewModel::new(); // Crear nuevo
            wasm_bindgen_futures::spawn_local(async move {
                let current_session = (*state).clone();
                if let Some(session) = current_session.session.clone() {
                    match vm.scan_package(&tracking, &session).await {
                        Ok((updated_session, _change)) => {
                            // Actualizar sesión
                            let mut new_state = (*state).clone();
                            new_state.session = Some(updated_session);
                            state.set(new_state);
                            
                            // Agregar cambio pendiente (se hace en use_sync_state)
                            log::info!("✅ Paquete escaneado, cambio pendiente agregado");
                        }
                        Err(e) => {
                            log::error!("❌ Error escaneando: {}", e);
                            let mut new_state = (*state).clone();
                            new_state.error = Some(e);
                            state.set(new_state);
                        }
                    }
                }
            });
        })
    };
    
    // Clear session
    let clear_session = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            let vm = SessionViewModel::new(); // Crear nuevo
            vm.clear_session();
            state.set(SessionStore::default());
        })
    };
    
    // Refresh session
    let refresh_session = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            let vm = SessionViewModel::new(); // Crear nuevo
            wasm_bindgen_futures::spawn_local(async move {
                let current_session = (*state).clone();
                if let Some(session) = current_session.session.clone() {
                    match vm.refresh_session(&session.session_id).await {
                        Ok(updated_session) => {
                            let mut new_state = (*state).clone();
                            new_state.session = Some(updated_session);
                            state.set(new_state);
                        }
                        Err(e) => {
                            log::error!("❌ Error refrescando: {}", e);
                        }
                    }
                }
            });
        })
    };
    
    UseSessionHandle {
        state,
        login_and_fetch,
        fetch_packages,
        scan_package,
        clear_session,
        refresh_session,
    }
}
