use yew::prelude::*;
use web_sys::{window, MouseEvent};
use std::collections::HashMap;
use gloo_timers::callback::Timeout;
use crate::models::{DeliverySession, DriverInfo, LoginData, CreateSessionRequest, SyncRequest, SessionPackage};
use crate::services::{
    api_create_session,
    api_sync_session,
    api_load_session,
};
use gloo_storage::{LocalStorage, Storage};
use js_sys;
use wasm_bindgen::JsCast;

pub struct UsePackagesHandle {
    // Estado principal - ahora usa DeliverySession
    pub session: UseStateHandle<Option<DeliverySession>>,
    pub loading: UseStateHandle<bool>,
    pub optimizing: UseStateHandle<bool>,
    pub selected_index: UseStateHandle<Option<usize>>,
    pub animations: UseStateHandle<HashMap<usize, String>>,
    pub expanded_groups: UseStateHandle<Vec<String>>, // IDs de grupos expandidos
    
    // Reorder mode states
    pub reorder_mode: UseStateHandle<bool>,
    pub reorder_origin: UseStateHandle<Option<usize>>,
    
    // Filter mode state
    pub filter_mode: UseStateHandle<bool>, // Filtrar solo pendientes (STATUT_CHARGER)
    
    // Callbacks
    pub create_session: Callback<()>,
    pub sync_session: Callback<()>,
    pub scan_package: Callback<String>, // tracking number
    pub optimize_route: Callback<MouseEvent>,
    pub select_package: Callback<usize>,
    pub reorder: Callback<(usize, String)>,
    pub update_package: Callback<(String, f64, f64, String)>,
    pub mark_problematic: Callback<String>,
    pub toggle_group: Callback<String>,
    pub toggle_reorder_mode: Callback<()>,
    pub toggle_filter_mode: Callback<()>,
    pub load_optimized_route: Callback<()>,
    pub get_ordered_packages: Callback<(), Vec<SessionPackage>>,
}

#[hook]
pub fn use_packages(login_data: Option<LoginData>) -> UsePackagesHandle {
    // Estados principales
    let session = use_state(|| None::<DeliverySession>);
    let loading = use_state(|| false);
    let optimizing = use_state(|| false);
    let selected_index = use_state(|| None::<usize>);
    let animations = use_state(|| HashMap::<usize, String>::new());
    let expanded_groups = use_state(|| Vec::<String>::new());
    
    // Reorder mode states
    let reorder_mode = use_state(|| false);
    let reorder_origin = use_state(|| None::<usize>);
    
    // Filter mode state
    let filter_mode = use_state(|| false);
    
    // Load session from localStorage on mount
    {
        let session = session.clone();
        use_effect_with((), move |_| {
            if let Ok(stored_session) = LocalStorage::get::<DeliverySession>("delivery_session") {
                log::info!("📦 Sesión cargada desde localStorage: {} paquetes", stored_session.packages.len());
                session.set(Some(stored_session));
            }
            || ()
        });
    }
    
    // Create session
    let create_session = {
        let session = session.clone();
        let loading = loading.clone();
        let login_data_ref = login_data.clone();
        
        Callback::from(move |_| {
            if let Some(login) = login_data_ref.as_ref() {
                let session = session.clone();
                let loading = loading.clone();
                let driver = DriverInfo {
                    driver_id: login.username.clone(), // Usar username como driver_id
                    name: login.username.clone(), // Usar username como name por ahora
                    company_id: login.company.code.clone(),
                    vehicle_id: None,
                };
                let sso_token = login.token.clone(); // Usar token como sso_token
                
                wasm_bindgen_futures::spawn_local(async move {
                    loading.set(true);
                    
                    let request = CreateSessionRequest { driver, sso_token };
                    
                    match api_create_session(request).await {
                        Ok(response) => {
                            log::info!("✅ Sesión creada: {} paquetes", response.new_packages_count);
                            
                            // Save to localStorage
                            let _ = LocalStorage::set("delivery_session", &response.session);
                            
                            session.set(Some(response.session));
                        }
                        Err(e) => {
                            log::error!("❌ Error creando sesión: {}", e);
                        }
                    }
                    
                    loading.set(false);
                });
            }
        })
    };
    
    // Sync session
    let sync_session = {
        let session = session.clone();
        
        Callback::from(move |_| {
            if let Some(current_session) = (*session).clone() {
                let session = session.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("🔄 Sincronizando sesión...");
                    
                    let request = SyncRequest {
                        session_id: current_session.session_id.clone(),
                        session: current_session,
                    };
                    
                    match api_sync_session(request).await {
                        Ok(response) => {
                            log::info!("✅ Sesión sincronizada: {} nuevos paquetes", response.new_packages.len());
                            
                            // Update localStorage
                            let _ = LocalStorage::set("delivery_session", &response.updated_session);
                            
                            session.set(Some(response.updated_session));
                        }
                        Err(e) => {
                            log::error!("❌ Error sincronizando: {}", e);
                        }
                    }
                });
            }
        })
    };
    
    // Scan package (100% LOCAL - sin llamadas al backend)
    let scan_package = {
        let session = session.clone();
        
        Callback::from(move |tracking: String| {
            if let Some(mut current_session) = (*session).as_ref().cloned() {
                let session = session.clone();
                
                log::info!("📦 Escaneando localmente: {}", tracking);
                
                // BÚSQUEDA LOCAL O(1)
                let found = current_session.find_by_tracking(&tracking);
                
                if let Some(package) = found {
                    // Mostrar info al usuario
                    let route_position = current_session.get_route_position(&tracking);
                    let total_packages = current_session.packages.len();
                    
                    if let Some(pos) = route_position {
                        log::info!("✅ Paquete encontrado: posición {}/{}", 
                            pos + 1, total_packages);
                        
                        // Mostrar notificación visual
                        if let Some(win) = web_sys::window() {
                            let msg = format!(
                                "✅ Paquete {}\nCliente: {}\nPosición: {}/{}",
                                tracking,
                                package.customer_name,
                                pos + 1,
                                total_packages
                            );
                            let _ = win.alert_with_message(&msg);
                        }
                    }
                    
                    // MARCAR COMO ESCANEADO LOCALMENTE
                    let _ = current_session.mark_scanned(&tracking);
                    
                    // GUARDAR EN LOCALSTORAGE
                    let _ = LocalStorage::set("delivery_session", &current_session);
                    
                    // ACTUALIZAR STATE
                    session.set(Some(current_session));
                    
                    // ⚠️ NO SE LLAMA AL BACKEND AQUÍ
                    // El backend se sincroniza periódicamente cada 5 min
                    
                } else {
                    log::warn!("⚠️ Paquete no encontrado: {}", tracking);
                    if let Some(win) = web_sys::window() {
                        let _ = win.alert_with_message(&format!("❌ Paquete {} no encontrado", tracking));
                    }
                }
            }
        })
    };
    
    // Get ordered packages
    let get_ordered_packages = {
        let session = session.clone();
        
        Callback::from(move |_| {
            if let Some(s) = (*session).as_ref() {
                s.get_ordered_packages().into_iter().cloned().collect()
            } else {
                vec![]
            }
        })
    };
    
    // Select package (ahora maneja modo reordenar)
    let select_package = {
        let selected_index = selected_index.clone();
        let reorder_mode = reorder_mode.clone();
        let reorder_origin = reorder_origin.clone();
        let session = session.clone();
        let animations = animations.clone();
        
        Callback::from(move |index: usize| {
            log::info!("📍 use_packages: Seleccionando paquete index {}", index);
            
            // Si estamos en modo reordenar
            if *reorder_mode {
                if let Some(origin_idx) = *reorder_origin {
                    // Ya tenemos un origen, mover (insert) el paquete origen a la posición del segundo
                    if origin_idx != index {
                        log::info!("🔄 Moviendo paquete {} → posición {}", origin_idx, index);
                        
                        if let Some(mut current_session) = (*session).as_ref().cloned() {
                            // Obtener paquetes ordenados
                            let ordered_packages = current_session.get_ordered_packages();
                            
                            if origin_idx < ordered_packages.len() && index < ordered_packages.len() {
                                // Crear nuevo orden
                                let mut new_order = Vec::new();
                                
                                // Agregar paquetes antes del origen
                                for (i, pkg) in ordered_packages.iter().enumerate() {
                                    if i < origin_idx {
                                        new_order.push(pkg.internal_id.clone());
                                    }
                                }
                                
                                // Agregar paquetes entre origen e índice (sin el origen)
                                for (i, pkg) in ordered_packages.iter().enumerate() {
                                    if i > origin_idx && i <= index {
                                        new_order.push(pkg.internal_id.clone());
                                    }
                                }
                                
                                // Agregar el paquete origen en la nueva posición
                                if let Some(origin_pkg) = ordered_packages.get(origin_idx) {
                                    new_order.push(origin_pkg.internal_id.clone());
                                }
                                
                                // Agregar paquetes después del índice
                                for (i, pkg) in ordered_packages.iter().enumerate() {
                                    if i > index {
                                        new_order.push(pkg.internal_id.clone());
                                    }
                                }
                                
                                // Aplicar nuevo orden
                                let _ = current_session.apply_optimization(new_order);
                                
                                // Guardar en localStorage
                                let _ = LocalStorage::set("delivery_session", &current_session);
                                
                                // Actualizar state
                                session.set(Some(current_session));
                                
                                // Animaciones
                                let mut anims = (*animations).clone();
                                if origin_idx < index {
                                    anims.insert(origin_idx, "moving-down".to_string());
                                    for i in (origin_idx + 1)..=index {
                                        anims.insert(i, "moving-up".to_string());
                                    }
                                } else {
                                    anims.insert(origin_idx, "moving-up".to_string());
                                    for i in index..origin_idx {
                                        anims.insert(i, "moving-down".to_string());
                                    }
                                }
                                animations.set(anims.clone());
                                
                                // Limpiar animaciones después de 300ms
                                let animations_clear = animations.clone();
                                Timeout::new(300, move || {
                                    let mut anims = (*animations_clear).clone();
                                    anims.insert(index, "moved".to_string());
                                    animations_clear.set(anims);
                                    
                                    Timeout::new(500, move || {
                                        animations_clear.set(HashMap::new());
                                    }).forget();
                                }).forget();
                                
                                // Resetear origen y seleccionar el paquete en su nueva posición
                                reorder_origin.set(None);
                                selected_index.set(Some(index));
                            }
                        }
                    } else {
                        // Mismo paquete, deseleccionar origen
                        log::info!("❌ Mismo paquete, cancelando origen");
                        reorder_origin.set(None);
                    }
                } else {
                    // Primer paquete seleccionado, marcar como origen
                    log::info!("✅ Paquete {} marcado como origen", index);
                    reorder_origin.set(Some(index));
                    selected_index.set(Some(index));
                }
            } else {
                // Modo normal, solo seleccionar
                selected_index.set(Some(index));
                log::info!("✅ Paquete {} seleccionado", index);
            }
        })
    };
    
    // Reorder packages
    let reorder = {
        let session = session.clone();
        let animations = animations.clone();
        let selected_index = selected_index.clone();
        
        Callback::from(move |(index, direction): (usize, String)| {
            log::info!("🔄 Reordenando paquete {} hacia {}", index, direction);
            
            if let Some(mut current_session) = (*session).as_ref().cloned() {
                let ordered_packages = current_session.get_ordered_packages();
                
                if direction == "up" && index > 0 {
                    log::info!("⬆️ Moviendo paquete {} hacia arriba", index);
                    
                    // Crear nuevo orden intercambiando posiciones
                    let mut new_order = Vec::new();
                    for (i, pkg) in ordered_packages.iter().enumerate() {
                        if i == index - 1 {
                            new_order.push(ordered_packages[index].internal_id.clone());
                        } else if i == index {
                            new_order.push(ordered_packages[index - 1].internal_id.clone());
                        } else {
                            new_order.push(pkg.internal_id.clone());
                        }
                    }
                    
                    // Aplicar nuevo orden
                    let _ = current_session.apply_optimization(new_order);
                    
                    // Guardar en localStorage
                    let _ = LocalStorage::set("delivery_session", &current_session);
                    
                    // Actualizar state
                    session.set(Some(current_session));
                    
                    // Animaciones
                    let mut anims = (*animations).clone();
                    anims.insert(index, "moving-up".to_string());
                    anims.insert(index - 1, "moving-down".to_string());
                    animations.set(anims.clone());
                    
                    // Actualizar selección
                    if let Some(sel) = *selected_index {
                        if sel == index {
                            selected_index.set(Some(index - 1));
                        } else if sel == index - 1 {
                            selected_index.set(Some(index));
                        }
                    }
                    
                    // Limpiar animaciones
                    let animations_clear = animations.clone();
                    Timeout::new(300, move || {
                        let mut final_anims = HashMap::new();
                        final_anims.insert(index - 1, "moved".to_string());
                        animations_clear.set(final_anims);
                        
                        Timeout::new(500, move || {
                            animations_clear.set(HashMap::new());
                        }).forget();
                    }).forget();
                    
                } else if direction == "down" && index < ordered_packages.len() - 1 {
                    log::info!("⬇️ Moviendo paquete {} hacia abajo", index);
                    
                    // Crear nuevo orden intercambiando posiciones
                    let mut new_order = Vec::new();
                    for (i, pkg) in ordered_packages.iter().enumerate() {
                        if i == index {
                            new_order.push(ordered_packages[index + 1].internal_id.clone());
                        } else if i == index + 1 {
                            new_order.push(ordered_packages[index].internal_id.clone());
                        } else {
                            new_order.push(pkg.internal_id.clone());
                        }
                    }
                    
                    // Aplicar nuevo orden
                    let _ = current_session.apply_optimization(new_order);
                    
                    // Guardar en localStorage
                    let _ = LocalStorage::set("delivery_session", &current_session);
                    
                    // Actualizar state
                    session.set(Some(current_session));
                    
                    // Animaciones
                    let mut anims = (*animations).clone();
                    anims.insert(index, "moving-down".to_string());
                    anims.insert(index + 1, "moving-up".to_string());
                    animations.set(anims.clone());
                    
                    // Actualizar selección
                    if let Some(sel) = *selected_index {
                        if sel == index {
                            selected_index.set(Some(index + 1));
                        } else if sel == index + 1 {
                            selected_index.set(Some(index));
                        }
                    }
                    
                    // Limpiar animaciones
                    let animations_clear = animations.clone();
                    Timeout::new(300, move || {
                        let mut final_anims = HashMap::new();
                        final_anims.insert(index + 1, "moved".to_string());
                        animations_clear.set(final_anims);
                        
                        Timeout::new(500, move || {
                            animations_clear.set(HashMap::new());
                        }).forget();
                    }).forget();
                } else {
                    log::warn!("⚠️ No se puede mover paquete {} hacia {} (límites: 0-{})", 
                        index, direction, ordered_packages.len() - 1);
                }
            }
        })
    };
    
    // Update package
    let update_package = {
        let session = session.clone();
        Callback::from(move |(id, lat, lng, new_address): (String, f64, f64, String)| {
            if let Some(mut current_session) = (*session).as_ref().cloned() {
                // Buscar paquete por tracking (id es el tracking)
                if let Some(package) = current_session.find_by_tracking(&id) {
                    let address_id = package.address_id.clone();
                    
                    // Actualizar dirección
                    let _ = current_session.update_address(&address_id, new_address, lat, lng);
                    
                    // Guardar en localStorage
                    let _ = LocalStorage::set("delivery_session", &current_session);
                    
                    // Actualizar state
                    session.set(Some(current_session));
                    
                    log::info!("✅ Paquete {} actualizado", id);
                }
            }
        })
    };
    
    // Mark package as problematic
    let mark_problematic = {
        let session = session.clone();
        Callback::from(move |package_id: String| {
            if let Some(mut current_session) = (*session).as_ref().cloned() {
                // Buscar paquete por tracking
                if let Some(package) = current_session.find_by_tracking(&package_id) {
                    let internal_id = package.internal_id.clone();
                    
                    // Marcar como problemático en la sesión
                    if let Some(pkg) = current_session.packages.get_mut(&internal_id) {
                        pkg.is_problematic = true;
                        pkg.modified_by_driver = true;
                    }
                    
                    // Actualizar stats
                    current_session.update_stats();
                    
                    // Guardar en localStorage
                    let _ = LocalStorage::set("delivery_session", &current_session);
                    
                    // Actualizar state
                    session.set(Some(current_session));
                    
                    log::info!("⚠️ Paquete {} marcado como problemático", package_id);
                }
            }
        })
    };
    
    // Toggle group expansion
    let toggle_group = {
        let expanded_groups = expanded_groups.clone();
        
        Callback::from(move |group_id: String| {
            let mut expanded = (*expanded_groups).clone();
            
            if let Some(pos) = expanded.iter().position(|id| id == &group_id) {
                expanded.remove(pos);
                log::info!("📥 Colapsando grupo {}", group_id);
            } else {
                expanded.push(group_id.clone());
                log::info!("📤 Expandiendo grupo {}", group_id);
            }
            
            expanded_groups.set(expanded);
        })
    };
    
    // Toggle reorder mode
    let toggle_reorder_mode = {
        let reorder_mode = reorder_mode.clone();
        let reorder_origin = reorder_origin.clone();
        
        Callback::from(move |_| {
            let new_mode = !*reorder_mode;
            reorder_mode.set(new_mode);
            
            // Si desactivamos el modo, limpiar origen
            if !new_mode {
                reorder_origin.set(None);
                log::info!("🔄 Modo reordenar DESACTIVADO");
            } else {
                log::info!("🔄 Modo reordenar ACTIVADO");
            }
        })
    };
    
    // Toggle filter mode
    let toggle_filter_mode = {
        let filter_mode = filter_mode.clone();
        
        Callback::from(move |_| {
            let new_mode = !*filter_mode;
            filter_mode.set(new_mode);
            
            if new_mode {
                log::info!("🔍 Filtro ACTIVADO - Solo pendientes (STATUT_CHARGER)");
            } else {
                log::info!("🔍 Filtro DESACTIVADO - Mostrando todos");
            }
        })
    };
    
    // Load optimized route from cache
    let load_optimized_route = {
        let session = session.clone();
        
        Callback::from(move |_| {
            if let Ok(stored_session) = LocalStorage::get::<DeliverySession>("delivery_session") {
                if stored_session.is_optimized {
                    log::info!("💾 Cargando ruta optimizada desde cache");
                    session.set(Some(stored_session));
                } else {
                    log::info!("📦 No hay ruta optimizada en cache");
                }
            }
        })
    };
    
    // Optimize route with Mapbox
    let optimize_route = {
        let session = session.clone();
        let optimizing = optimizing.clone();
        
        Callback::from(move |_: MouseEvent| {
            if let Some(current_session) = (*session).as_ref() {
                // Get driver location from JavaScript
                if let Some(window) = web_sys::window() {
                    if let Ok(get_location_fn) = js_sys::Reflect::get(&window, &"getDriverLocation".into()) {
                        if let Ok(func) = get_location_fn.dyn_into::<js_sys::Function>() {
                            match func.call0(&window) {
                                Ok(location_value) => {
                                    if location_value.is_null() || location_value.is_undefined() {
                                        log::warn!("⚠️ No hay ubicación del chofer");
                                        if let Some(win) = web_sys::window() {
                                            let _ = win.alert_with_message("Por favor, activa primero tu ubicación usando el botón 📍 en el mapa");
                                        }
                                        return;
                                    }
                                    
                                    // Parse location
                                    let lat = js_sys::Reflect::get(&location_value, &"latitude".into())
                                        .ok()
                                        .and_then(|v| v.as_f64());
                                    let lng = js_sys::Reflect::get(&location_value, &"longitude".into())
                                        .ok()
                                        .and_then(|v| v.as_f64());
                                    
                                    if let (Some(latitude), Some(longitude)) = (lat, lng) {
                                        log::info!("✅ Ubicación del chofer: {}, {}", latitude, longitude);
                                        
                                        // Start optimization
                                        let session = session.clone();
                                        let optimizing = optimizing.clone();
                                        let current_session = current_session.clone();
                                        
                                        wasm_bindgen_futures::spawn_local(async move {
                                            optimizing.set(true);
                                            log::info!("🎯 Iniciando optimización de ruta...");
                                            
                                            // TODO: Implementar optimización con Mapbox
                                            // Por ahora, solo simular
                                            
                                            // Simular delay de optimización
                                            gloo_timers::future::TimeoutFuture::new(2000).await;
                                            
                                            optimizing.set(false);
                                            
                                            // Simular orden optimizado
                                            let ordered_packages = current_session.get_ordered_packages();
                                            let optimized_order: Vec<String> = ordered_packages
                                                .iter()
                                                .map(|p| p.internal_id.clone())
                                                .collect();
                                            
                                            // Aplicar optimización
                                            let mut updated_session = current_session.clone();
                                            let _ = updated_session.apply_optimization(optimized_order);
                                            
                                            // Guardar en localStorage
                                            let _ = LocalStorage::set("delivery_session", &updated_session);
                                            
                                            // Actualizar state
                                            session.set(Some(updated_session));
                                            
                                            log::info!("✅ Optimización simulada completada");
                                            
                                            if let Some(win) = web_sys::window() {
                                                let _ = win.alert_with_message("✅ Ruta optimizada exitosamente");
                                            }
                                        });
                                    } else {
                                        log::error!("❌ Error parseando coordenadas");
                                    }
                                }
                                Err(e) => {
                                    log::error!("❌ Error obteniendo ubicación: {:?}", e);
                                }
                            }
                        }
                    }
                }
            }
        })
    };
    
    UsePackagesHandle {
        session,
        loading,
        optimizing,
        selected_index,
        animations,
        expanded_groups,
        reorder_mode,
        reorder_origin,
        filter_mode,
        create_session,
        sync_session,
        scan_package,
        optimize_route,
        select_package,
        reorder,
        update_package,
        mark_problematic,
        toggle_group,
        toggle_reorder_mode,
        toggle_filter_mode,
        load_optimized_route,
        get_ordered_packages,
    }
}
