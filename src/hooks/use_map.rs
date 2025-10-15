use yew::prelude::*;
use gloo_timers::callback::Timeout;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::models::Package;
use crate::utils::{init_mapbox, add_packages_to_map, update_selected_package as update_selected_package_ffi, center_map_on_package, scroll_to_selected_package};

#[derive(Clone, PartialEq)]
pub struct MapState {
    pub initialized: bool,
}

pub struct UseMapHandle {
    pub state: UseStateHandle<MapState>,
    pub initialize_map: Callback<()>,
    pub update_packages: Callback<Vec<Package>>,
    pub select_package: Callback<usize>,
}

/// Detect dark mode preference
fn is_dark_mode() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
        .flatten()
        .map(|mq| mq.matches())
        .unwrap_or(false)
}

#[hook]
pub fn use_map() -> UseMapHandle {
    let state = use_state(|| MapState {
        initialized: false,
    });
    
    // Initialize map
    let initialize_map = {
        let state = state.clone();
        Callback::from(move |_| {
            if !(*state).initialized {
                log::info!("🗺️ Inicializando mapa...");
                
                // Delay to ensure map container is ready
                let state = state.clone();
                Timeout::new(500, move || {
                    let is_dark = is_dark_mode();
                    log::info!("🎨 Modo mapa: {}", if is_dark { "oscuro" } else { "claro" });
                    init_mapbox("map", is_dark);
                    
                    let mut current_state = (*state).clone();
                    current_state.initialized = true;
                    state.set(current_state);
                }).forget();
            }
        })
    };
    
    // Update packages on map
    let update_packages = {
        let state = state.clone();
        Callback::from(move |packages: Vec<Package>| {
            log::info!("🗺️ use_map: Actualizando paquetes (initialized: {}, count: {})", (*state).initialized, packages.len());
            
            // Save packages to window for JS access
            use wasm_bindgen::JsValue;
            if let Some(window) = web_sys::window() {
                if let Ok(js_packages) = serde_wasm_bindgen::to_value(&packages) {
                    let _ = js_sys::Reflect::set(
                        &window,
                        &JsValue::from_str("currentPackages"),
                        &js_packages
                    );
                    log::info!("✅ Paquetes guardados en window.currentPackages");
                }
            }
            
            // If map is initialized, update packages immediately
            if (*state).initialized {
                log::info!("🎯 Mapa inicializado, enviando paquetes al mapa...");
                Timeout::new(100, move || {
                    let packages_json = serde_json::to_string(&packages).unwrap_or_default();
                    log::info!("📤 Llamando add_packages_to_map con {} paquetes", packages.len());
                    add_packages_to_map(&packages_json);
                }).forget();
            } else {
                log::warn!("⚠️ Mapa no inicializado todavía, esperando...");
            }
        })
    };
    
    // Select package on map
    let select_package = {
        let state = state.clone();
        Callback::from(move |index: usize| {
            if (*state).initialized {
                update_selected_package_ffi(index as i32);
                
                // Center map and scroll to package
                Timeout::new(100, move || {
                    center_map_on_package(index);
                    
                    Timeout::new(300, move || {
                        scroll_to_selected_package(index);
                    }).forget();
                }).forget();
            }
        })
    };
    
    UseMapHandle {
        state,
        initialize_map,
        update_packages,
        select_package,
    }
}

/// Setup event listener for package selection from map
#[hook]
pub fn use_map_selection_listener(on_select: Callback<usize>) -> () {
    use_effect_with(on_select.clone(), move |on_select_cb| {
        let on_select_cb = on_select_cb.clone();
        
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            // Get detail.index from custom event
            if let Ok(detail) = js_sys::Reflect::get(&event, &JsValue::from_str("detail")) {
                if let Ok(index_val) = js_sys::Reflect::get(&detail, &JsValue::from_str("index")) {
                    if let Some(index) = index_val.as_f64() {
                        log::info!("📍 Evento packageSelected recibido: index {}", index);
                        on_select_cb.emit(index as usize);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "packageSelected",
                callback.as_ref().unchecked_ref()
            );
            log::info!("✅ Event listener 'packageSelected' registrado");
        }
        
        move || {
            if let Some(window) = web_sys::window() {
                // Removemos el listener anterior
                log::info!("🗑️ Removiendo event listener anterior");
            }
            callback.forget();
        }
    });
}

