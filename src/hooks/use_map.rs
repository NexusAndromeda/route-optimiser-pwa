// ============================================================================
// USE MAP HOOK - Gestión de estado del mapa
// ============================================================================
// Hook nativo de Yew - Delega lógica al ViewModel
// ============================================================================

use yew::prelude::*;
use crate::viewmodels::{MapViewModel, map_viewmodel::MapPackage};

/// Store local del hook (NO es un Store global)
#[derive(Clone, PartialEq)]
pub struct MapState {
    pub initialized: bool,
}

/// Handle del hook
#[derive(Clone)]
pub struct UseMapHandle {
    pub state: UseStateHandle<MapState>,
    pub initialize: Callback<()>,
    pub update_packages: Callback<Vec<MapPackage>>,
    pub select_package: Callback<usize>,
    pub center_on_package: Callback<usize>,
}

#[hook]
pub fn use_map() -> UseMapHandle {
    let state = use_state(|| MapState { initialized: false });
    
    // Inicializar mapa
    let initialize = {
        let state = state.clone();
        Callback::from(move |_| {
            if !(*state).initialized {
                log::info!("🗺️ Hook: Inicializando mapa...");
                
                // Delegar a ViewModel
                MapViewModel::initialize_map();
                
                // Esperar 1.5 segundos para que el mapa cargue completamente
                let state_clone = state.clone();
                use gloo_timers::callback::Timeout;
                Timeout::new(1500, move || {
                    let mut new_state = (*state_clone).clone();
                    new_state.initialized = true;
                    state_clone.set(new_state);
                    
                    log::info!("✅ Hook: Mapa marcado como inicializado (después de espera)");
                }).forget();
            }
        })
    };
    
    // Actualizar paquetes
    let update_packages = {
        let state = state.clone();
        Callback::from(move |packages: Vec<MapPackage>| {
            if (*state).initialized {
                log::info!("🗺️ Hook: Actualizando {} paquetes en el mapa", packages.len());
                
                // Guardar en window para acceso desde JS
                if let Some(window) = web_sys::window() {
                    if let Ok(js_pkg) = serde_wasm_bindgen::to_value(&packages) {
                        let _ = js_sys::Reflect::set(
                            &window,
                            &wasm_bindgen::JsValue::from_str("currentPackages"),
                            &js_pkg
                        );
                    }
                }
                
                // Delegar a ViewModel
                MapViewModel::update_map_packages(packages);
            } else {
                log::warn!("⚠️ Hook: Mapa no inicializado, no se pueden actualizar paquetes");
            }
        })
    };
    
    // Seleccionar paquete en el mapa
    let select_package = {
        let state = state.clone();
        Callback::from(move |index: usize| {
            if (*state).initialized {
                log::info!("🗺️ Hook: Seleccionando paquete {} desde el mapa", index);
                
                // Llamar a funciones JavaScript
                if let Some(window) = web_sys::window() {
                    // Actualizar selección en el mapa
                    let update_fn = js_sys::Function::new_no_args(&format!(
                        "if (window.updateSelectedPackage) window.updateSelectedPackage({});",
                        index as i32
                    ));
                    let _ = update_fn.call0(&window.into());
                    
                    // Scroll al card seleccionado con delay de 150ms (más corto para map->sheet)
                    use gloo_timers::callback::Timeout;
                    Timeout::new(150, move || {
                        if let Some(window) = web_sys::window() {
                            let scroll_fn = js_sys::Function::new_no_args(&format!(
                                "if (window.scrollToSelectedPackage) window.scrollToSelectedPackage({});",
                                index
                            ));
                            let _ = scroll_fn.call0(&window.into());
                        }
                    }).forget();
                }
            }
        })
    };
    
    // Centrar mapa en un paquete
    let center_on_package = {
        let state = state.clone();
        Callback::from(move |index: usize| {
            if (*state).initialized {
                log::info!("🗺️ Hook: Centrando mapa en paquete {}", index);
                
                // PRIMERO: Actualizar selección (inicia pulse animation)
                if let Some(window) = web_sys::window() {
                    let update_fn = js_sys::Function::new_no_args(&format!(
                        "if (window.updateSelectedPackage) window.updateSelectedPackage({});",
                        index
                    ));
                    let _ = update_fn.call0(&window.into());
                }
                
                // DESPUÉS: Centrar el mapa
                if let Some(window) = web_sys::window() {
                    let center_fn = js_sys::Function::new_no_args(&format!(
                        "if (window.centerMapOnPackage) window.centerMapOnPackage({});",
                        index
                    ));
                    let _ = center_fn.call0(&window.into());
                }
            }
        })
    };
    
    UseMapHandle {
        state,
        initialize,
        update_packages,
        select_package,
        center_on_package,
    }
}

