// ============================================================================
// USE MAP HOOK - Gestión de estado del mapa
// ============================================================================
// Hook nativo de Yew - Delega lógica al ViewModel
// ============================================================================

use yew::prelude::*;
use wasm_bindgen::JsCast;
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
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("🗺️ HOOK: SELECT_PACKAGE LLAMADO");
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("   📍 group_idx: {}", index);
            log::info!("   🗺️  Mapa inicializado: {}", (*state).initialized);
            
            if (*state).initialized {
                log::info!("   ✅ Mapa inicializado, ejecutando selección...");
                
                // Llamar a funciones JavaScript
                if let Some(window) = web_sys::window() {
                    log::info!("   🔧 Llamando window.updateSelectedPackage({})", index);
                    
                    // Actualizar selección en el mapa
                    let update_fn = js_sys::Function::new_no_args(&format!(
                        "if (window.updateSelectedPackage) window.updateSelectedPackage({});",
                        index as i32
                    ));
                    let _ = update_fn.call0(&window.into());
                    
                    // Scroll al card seleccionado con delay de 150ms (más corto para map->sheet)
                    use gloo_timers::callback::Timeout;
                    Timeout::new(150, move || {
                        log::info!("   ⏱️  Delay completado, llamando scrollToSelectedPackage({})", index);
                        if let Some(window) = web_sys::window() {
                            let scroll_fn = js_sys::Function::new_no_args(&format!(
                                "if (window.scrollToSelectedPackage) window.scrollToSelectedPackage({});",
                                index
                            ));
                            let _ = scroll_fn.call0(&window.into());
                            log::info!("   ✅ scrollToSelectedPackage llamado");
                        }
                    }).forget();
                } else {
                    log::warn!("   ⚠️  No se pudo obtener window");
                }
            } else {
                log::warn!("   ⚠️  Mapa no inicializado, ignorando selección");
            }
            
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        })
    };
    
    // Centrar mapa en un paquete
    let center_on_package = {
        let state = state.clone();
        Callback::from(move |index: usize| {
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("🗺️ HOOK: CENTER_ON_PACKAGE LLAMADO");
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("   📍 group_idx: {}", index);
            log::info!("   🗺️  Mapa inicializado: {}", (*state).initialized);
            
            if (*state).initialized {
                log::info!("   ✅ Mapa inicializado, centrando...");
                
                // PRIMERO: Actualizar selección (inicia pulse animation)
                if let Some(window) = web_sys::window() {
                    log::info!("   🔧 Llamando window.updateSelectedPackage({})", index);
                    let update_fn = js_sys::Function::new_no_args(&format!(
                        "if (window.updateSelectedPackage) window.updateSelectedPackage({});",
                        index
                    ));
                    let _ = update_fn.call0(&window.into());
                    log::info!("   ✅ updateSelectedPackage llamado");
                }
                
                // DESPUÉS: Centrar el mapa
                if let Some(window) = web_sys::window() {
                    log::info!("   🔧 Llamando window.centerMapOnPackage({})", index);
                    let center_fn = js_sys::Function::new_no_args(&format!(
                        "if (window.centerMapOnPackage) window.centerMapOnPackage({});",
                        index
                    ));
                    let _ = center_fn.call0(&window.into());
                    log::info!("   ✅ centerMapOnPackage llamado");
                }
            } else {
                log::warn!("   ⚠️  Mapa no inicializado, ignorando centrado");
            }
            
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
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

/// Setup event listener for package selection from map
/// Actualiza el listener cuando el callback cambia para mantener siempre la versión más reciente
#[hook]
pub fn use_map_selection_listener(on_select: Callback<usize>) -> () {
    // Usar use_effect_with con el callback como dependencia
    // Esto asegura que el listener siempre use el callback más reciente
    use_effect_with(on_select.clone(), move |callback| {
        log::info!("🔗 Registrando/actualizando listener de selección del mapa");
        
        let on_select_cb = callback.clone();
        
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: wasm_bindgen::JsValue| {
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("📡 EVENTO 'packageSelected' RECIBIDO EN LISTENER");
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("   📦 Evento completo: {:?}", event);
            
            // Get detail.index from custom event
            if let Ok(detail) = js_sys::Reflect::get(&event, &wasm_bindgen::JsValue::from_str("detail")) {
                log::info!("   ✅ 'detail' obtenido: {:?}", detail);
                
                if let Ok(index_val) = js_sys::Reflect::get(&detail, &wasm_bindgen::JsValue::from_str("index")) {
                    log::info!("   ✅ 'index' obtenido: {:?}", index_val);
                    
                    if let Some(index) = index_val.as_f64() {
                        let idx = index as usize;
                        log::info!("   📍 group_idx extraído: {} (number)", idx);
                        log::info!("   📤 Emitiendo callback on_select_cb con group_idx: {}", idx);
                        on_select_cb.emit(idx);
                        log::info!("   ✅ Callback 'on_map_select' emitido exitosamente");
                    } else {
                        log::warn!("   ⚠️  index_val no es un número: {:?}", index_val);
                    }
                } else {
                    log::warn!("   ⚠️  No se pudo obtener 'index' del detail: {:?}", detail);
                }
            } else {
                log::warn!("   ⚠️  No se pudo obtener 'detail' del evento");
            }
            
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }) as Box<dyn FnMut(_)>);
        
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "packageSelected",
                closure.as_ref().unchecked_ref()
            );
            log::info!("✅ Event listener 'packageSelected' registrado/actualizado");
        } else {
            log::error!("❌ No se pudo obtener window");
        }
        
        // Cleanup: se ejecuta cuando el callback cambia o el componente se desmonta
        move || {
            log::info!("🗑️ Limpiando event listener anterior");
            closure.forget();
        }
    });
}

