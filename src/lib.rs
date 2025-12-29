// ============================================================================
// ROUTE OPTIMIZER APP - FRONTEND MVVM ESTRICTO (RUST PURO)
// ============================================================================
// Arquitectura MVVM estricta:
// - Views: Funciones que renderizan DOM (sin lógica)
// - ViewModels: Estado + Lógica UI
// - Services: SOLO comunicación API
// - State: State Management con Rc<RefCell>
// - Models: Estructuras compartidas con backend
// ============================================================================

mod models;
mod services;
mod viewmodels;
mod state;
mod dom;
mod views;
mod utils;
mod app;

use wasm_bindgen::prelude::*;
use wasm_logger::Config;
use console_error_panic_hook;
use crate::app::App;
use crate::state::app_state::{UpdateType, IncrementalUpdate};
use std::cell::RefCell;

// Variable estática global para mantener la instancia de App
thread_local! {
    static APP: RefCell<Option<App>> = RefCell::new(None);
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Inicializar panic hook para mejor debugging
    console_error_panic_hook::set_once();
    
    // Inicializar logging
    wasm_logger::init(Config::default());
    log::info!("🚀 Route Optimizer App - Rust Puro + MVVM");
    
    // Crear y renderizar app
    let mut app = App::new()?;
    app.render()?;
    
    // Guardar app en variable global
    APP.with(|app_cell| {
        *app_cell.borrow_mut() = Some(app);
    });
    
    // Escuchar evento "loggedIn" para re-renderizar
    // Nota: Este listener global solo se registra UNA VEZ en init(), por lo que es seguro.
    // Para listeners globales que pueden registrarse múltiples veces, usar protección (ver NetworkMonitor).
    if let Some(win) = web_sys::window() {
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            web_sys::console::log_1(&JsValue::from_str("🔄 [MAIN] Evento loggedIn recibido, re-renderizando app..."));
            rerender_app();
        }) as Box<dyn FnMut(web_sys::Event)>);
        
        win.add_event_listener_with_callback("loggedIn", closure.as_ref().unchecked_ref())?;
        // Nota: closure.forget() es necesario para mantener el closure vivo en Rust WASM.
        // Como este listener solo se registra una vez en init(), no hay riesgo de acumulación.
        closure.forget();
    }
    
    Ok(())
}

/// Función pública para re-renderizar la app (re-render completo)
pub fn rerender_app() {
    rerender_app_with_type(UpdateType::FullRender);
}

/// Función pública para actualizar la app con tipo específico
pub fn rerender_app_with_type(update_type: UpdateType) {
    APP.with(|app_cell| {
        match update_type {
            UpdateType::Incremental(inc_type) => {
                web_sys::console::log_1(&JsValue::from_str(&format!("🔄 [UPDATE] Actualización incremental: {:?}", inc_type)));
                // Primero intentamos actualización incremental
                let needs_full_render = {
                    if let Some(ref app) = *app_cell.borrow() {
                        match app.update_incremental(inc_type) {
                            Ok(()) => {
                                web_sys::console::log_1(&JsValue::from_str("✅ [UPDATE] Actualización incremental completada"));
                                false
                            }
                            Err(e) => {
                                // Si el error indica que necesita re-render completo (modal no existe)
                                let error_str = format!("{:?}", e);
                                if error_str.contains("needs full render") || error_str.contains("Modal not found") {
                                    web_sys::console::log_1(&JsValue::from_str("🔄 [UPDATE] Cambiando a re-render completo"));
                                    true
                                } else {
                                    web_sys::console::error_1(&JsValue::from_str(&format!("❌ Error en actualización incremental: {:?}", e)));
                                    false
                                }
                            }
                        }
                    } else {
                        web_sys::console::warn_1(&JsValue::from_str("⚠️ [UPDATE] App no está inicializada"));
                        false
                    }
                };
                
                // Si necesita re-render completo, liberamos el borrow anterior y hacemos el re-render
                if needs_full_render {
                    if let Some(ref mut app_mut) = *app_cell.borrow_mut() {
                        let _ = app_mut.render();
                    }
                }
            }
            UpdateType::FullRender => {
                web_sys::console::log_1(&JsValue::from_str("🔄 [RERENDER] Re-render completo"));
                if let Some(ref mut app_mut) = *app_cell.borrow_mut() {
                    if let Err(e) = app_mut.render() {
                        web_sys::console::error_1(&JsValue::from_str(&format!("❌ Error re-renderizando: {:?}", e)));
                    } else {
                        web_sys::console::log_1(&JsValue::from_str("✅ [RERENDER] App re-renderizada exitosamente"));
                    }
                } else {
                    web_sys::console::warn_1(&JsValue::from_str("⚠️ [RERENDER] App no está inicializada"));
                }
            }
        }
    });
}

/// Función pública WASM para re-renderizar la app (llamable desde JavaScript)
#[wasm_bindgen]
pub fn rerender_app_wasm() {
    rerender_app();
}

/// Función pública WASM para manejar el toggle de expand de grupos (llamable desde JavaScript)
#[wasm_bindgen]
pub fn handle_toggle_expand_group(index: usize) {
    // #region agent log
    // Logs removidos temporalmente para compilación
    // #endregion
    web_sys::console::log_1(&JsValue::from_str(&format!("🔄 [RUST] handle_toggle_expand_group llamado con index: {}", index)));
    
    APP.with(|app_cell| {
        if let Some(ref app) = *app_cell.borrow() {
            let state = app.state();
            
            // #region agent log
            // Logs removidos temporalmente para compilación
            // #endregion
            
            // Toggle del grupo expandido
            let was_expanded = {
                let mut expanded = state.expanded_groups.borrow_mut();
                if expanded.contains(&index) {
                    expanded.remove(&index);
                    true
                } else {
                    expanded.insert(index);
                    false
                }
            };
            
            // #region agent log
            // Logs removidos temporalmente para compilación
            // #endregion
            
            web_sys::console::log_1(&JsValue::from_str(&format!("✅ [RUST] Grupo {} {}", index, if was_expanded { "colapsado" } else { "expandido" })));
            
            // Obtener grupos y sesión para actualizar solo el card específico
            if let Some(session) = state.session.get_session() {
                // Obtener paquetes y agruparlos
                let packages: Vec<_> = session.packages.values().cloned().collect();
                let groups = crate::views::package_list::group_packages_by_address(packages);
                
                if let Some(group) = groups.get(index) {
                    let addresses_map: std::collections::HashMap<String, String> = session.addresses
                        .iter()
                        .map(|(k, v)| (k.clone(), v.label.clone()))
                        .collect();
                    
                    // Manipulación directa del DOM desde Rust - 100% Rust, máximo rendimiento
                    if let Err(_) = crate::dom::incremental::toggle_group_expand_direct_rust(
                        state,
                        index,
                        group,
                        &addresses_map,
                        &session,
                    ) {
                        // Fallback: si falla, hacer update completo de la lista
                        rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::PackageList));
                    }
                    // Si tiene éxito, no necesitamos hacer nada más
                } else {
                    // Grupo no encontrado, hacer update completo
                    rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::PackageList));
                }
            } else {
                // No hay sesión, hacer update completo
                rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::PackageList));
            }
        } else {
            web_sys::console::error_1(&JsValue::from_str("❌ [RUST] App no está inicializada"));
        }
    });
}

