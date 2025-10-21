use yew::prelude::*;
use web_sys::{window, MouseEvent};
use std::collections::HashMap;
use gloo_timers::callback::Timeout;
use crate::models::{Package, LoginData};
use crate::services::{
    fetch_packages,
    optimization_service::{
        optimize_route as mapbox_optimize_route,
        PackageLocation, 
        DepotLocation
    }
};
use crate::utils::{get_local_storage, STORAGE_KEY_PACKAGES_PREFIX};
use js_sys;
use wasm_bindgen::JsCast;

pub struct UsePackagesHandle {
    // Separate states - exactly like original version
    pub packages: UseStateHandle<Vec<Package>>,
    pub loading: UseStateHandle<bool>,
    pub optimizing: UseStateHandle<bool>,
    pub selected_index: UseStateHandle<Option<usize>>,
    pub animations: UseStateHandle<HashMap<usize, String>>,
    pub expanded_groups: UseStateHandle<Vec<String>>, // IDs de grupos expandidos
    
    // Reorder mode states
    pub reorder_mode: UseStateHandle<bool>, // Modo reordenar activado/desactivado
    pub reorder_origin: UseStateHandle<Option<usize>>, // Primer paquete seleccionado para swap
    
    // Filter mode state
    pub filter_mode: UseStateHandle<bool>, // Filtrar solo pendientes (STATUT_CHARGER)
    
    // Callbacks
    pub refresh: Callback<MouseEvent>,
    pub optimize: Callback<MouseEvent>,
    pub select_package: Callback<usize>,
    pub reorder: Callback<(usize, String)>,
    pub update_package: Callback<(String, f64, f64, String)>,
    pub mark_problematic: Callback<String>, // Marcar paquete como problemático
    pub toggle_group: Callback<String>, // Toggle expand/collapse de grupo
    pub toggle_reorder_mode: Callback<()>, // Toggle modo reordenar
    pub toggle_filter_mode: Callback<()>, // Toggle filtrar pendientes
    pub optimize_route: Callback<MouseEvent>, // Optimizar ruta con Mapbox
}

#[hook]
pub fn use_packages(login_data: Option<LoginData>) -> UsePackagesHandle {
    // Separate states - exactly like original version
    let packages = use_state(|| Vec::<Package>::new());
    let loading = use_state(|| false);
    let optimizing = use_state(|| false);
    let selected_index = use_state(|| None::<usize>);
    let animations = use_state(|| HashMap::<usize, String>::new());
    let expanded_groups = use_state(|| Vec::<String>::new());
    
    // Reorder mode states
    let reorder_mode = use_state(|| false);
    let reorder_origin = use_state(|| None::<usize>);
    
    // Filter mode state
    let filter_mode = use_state(|| false); // Por defecto: mostrar todos
    
    // Load packages on login
    {
        let packages = packages.clone();
        let loading = loading.clone();
        let login_data_clone = login_data.clone();
        
        use_effect_with(login_data_clone, move |login_opt| {
            if let Some(login) = login_opt {
                let packages = packages.clone();
                let loading = loading.clone();
                let username = login.username.clone();
                let company_code = login.company.code.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    loading.set(true);
                    
                    match fetch_packages(&username, &company_code, false).await {
                        Ok(fetched_packages) => {
                            log::info!("📦 Paquetes obtenidos: {}", fetched_packages.len());
                            let with_coords = fetched_packages.iter().filter(|p| p.coords.is_some()).count();
                            log::info!("📍 Paquetes con coordenadas: {}/{}", with_coords, fetched_packages.len());
                            packages.set(fetched_packages);
                            loading.set(false);
                        }
                        Err(e) => {
                            log::error!("❌ Error obteniendo paquetes: {}", e);
                            loading.set(false);
                        }
                    }
                });
            }
            || ()
        });
    }
    
    // Refresh packages
    let refresh = {
        let packages = packages.clone();
        let loading = loading.clone();
        let login_data_ref = login_data.clone();
        
        Callback::from(move |_: MouseEvent| {
            if let Some(login) = login_data_ref.as_ref() {
                let packages = packages.clone();
                let loading = loading.clone();
                let username = login.username.clone();
                let company_code = login.company.code.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("🔄 Refrescando paquetes...");
                    loading.set(true);
                    
                    match fetch_packages(&username, &company_code, true).await {
                        Ok(fetched_packages) => {
                            log::info!("✅ Paquetes refrescados: {}", fetched_packages.len());
                            packages.set(fetched_packages);
                            loading.set(false);
                        }
                        Err(e) => {
                            log::error!("❌ Error refrescando paquetes: {}", e);
                            loading.set(false);
                        }
                    }
                });
            }
        })
    };
    
    // Optimize route (OLD - usando endpoint legacy de Colis Privé)
    // Este callback será reemplazado por optimize_route que usa Mapbox Optimization API v2
    let optimize = {
        let packages = packages.clone();
        
        Callback::from(move |_: MouseEvent| {
            log::warn!("⚠️ Optimize callback antiguo llamado - usar optimize_route en su lugar");
            if let Some(win) = window() {
                let _ = win.alert_with_message("Por favor, usa el botón 🎯 de optimización");
            }
        })
    };
    
    // Select package (ahora maneja modo reordenar)
    let select_package = {
        let selected_index = selected_index.clone();
        let reorder_mode = reorder_mode.clone();
        let reorder_origin = reorder_origin.clone();
        let packages = packages.clone();
        let animations = animations.clone();
        
        Callback::from(move |index: usize| {
            log::info!("📍 use_packages: Seleccionando paquete index {}", index);
            
            // Si estamos en modo reordenar
            if *reorder_mode {
                if let Some(origin_idx) = *reorder_origin {
                    // Ya tenemos un origen, mover (insert) el paquete origen a la posición del segundo
                    if origin_idx != index {
                        log::info!("🔄 Moviendo paquete {} → posición {}", origin_idx, index);
                        
                        let mut pkgs = (*packages).clone();
                        
                        // Remover el paquete origen
                        let package_to_move = pkgs.remove(origin_idx);
                        
                        // Insertarlo en la nueva posición
                        // Si origin_idx < index, el índice se desplaza -1 después de remove
                        let insert_idx = if origin_idx < index { index } else { index };
                        pkgs.insert(insert_idx, package_to_move);
                        
                        // Animaciones
                        let mut anims = (*animations).clone();
                        if origin_idx < index {
                            // Moviendo hacia abajo
                            anims.insert(origin_idx, "moving-down".to_string());
                            // Los paquetes entre origin e index se mueven hacia arriba
                            for i in (origin_idx + 1)..=index {
                                anims.insert(i, "moving-up".to_string());
                            }
                        } else {
                            // Moviendo hacia arriba
                            anims.insert(origin_idx, "moving-up".to_string());
                            // Los paquetes entre index y origin se mueven hacia abajo
                            for i in index..origin_idx {
                                anims.insert(i, "moving-down".to_string());
                            }
                        }
                        animations.set(anims.clone());
                        
                        // Actualizar paquetes
                        packages.set(pkgs);
                        
                        // Limpiar animaciones después de 300ms
                        let animations_clear = animations.clone();
                        Timeout::new(300, move || {
                            let mut anims = (*animations_clear).clone();
                            anims.insert(insert_idx, "moved".to_string());
                            animations_clear.set(anims);
                            
                            Timeout::new(500, move || {
                                animations_clear.set(HashMap::new());
                            }).forget();
                        }).forget();
                        
                        // Resetear origen y seleccionar el paquete en su nueva posición
                        reorder_origin.set(None);
                        selected_index.set(Some(insert_idx));
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
    
    // Reorder packages - EXACTLY like original
    let reorder = {
        let packages = packages.clone();
        let animations = animations.clone();
        let selected_index = selected_index.clone();
        
        Callback::from(move |(index, direction): (usize, String)| {
            log::info!("🔄 Reordenando paquete {} hacia {}", index, direction);
            let pkgs = (*packages).clone();
            let mut anims = (*animations).clone();
            
            if direction == "up" && index > 0 {
                log::info!("⬆️ Moviendo paquete {} hacia arriba", index);
                anims.insert(index, "moving-up".to_string());
                anims.insert(index - 1, "moving-down".to_string());
                animations.set(anims.clone());
                
                let timeout = Timeout::new(200, {
                    let packages = packages.clone();
                    let animations = animations.clone();
                    let selected_index = selected_index.clone();
                    let mut pkgs = pkgs.clone();
                    
                    move || {
                        pkgs.swap(index, index - 1);
                        packages.set(pkgs.clone());
                        
                        let mut final_anims = HashMap::new();
                        final_anims.insert(index - 1, "moved".to_string());
                        animations.set(final_anims.clone());
                        
                        if let Some(sel) = *selected_index {
                            if sel == index {
                                selected_index.set(Some(index - 1));
                            } else if sel == index - 1 {
                                selected_index.set(Some(index));
                            }
                        }
                        
                        let timeout2 = Timeout::new(300, {
                            let animations = animations.clone();
                            move || {
                                animations.set(HashMap::new());
                            }
                        });
                        timeout2.forget();
                    }
                });
                timeout.forget();
            } else if direction == "down" && index < pkgs.len() - 1 {
                log::info!("⬇️ Moviendo paquete {} hacia abajo", index);
                anims.insert(index, "moving-down".to_string());
                anims.insert(index + 1, "moving-up".to_string());
                animations.set(anims.clone());
                
                let timeout = Timeout::new(200, {
                    let packages = packages.clone();
                    let animations = animations.clone();
                    let selected_index = selected_index.clone();
                    let mut pkgs = pkgs.clone();
                    
                    move || {
                        pkgs.swap(index, index + 1);
                        packages.set(pkgs.clone());
                        
                        let mut final_anims = HashMap::new();
                        final_anims.insert(index + 1, "moved".to_string());
                        animations.set(final_anims.clone());
                        
                        if let Some(sel) = *selected_index {
                            if sel == index {
                                selected_index.set(Some(index + 1));
                            } else if sel == index + 1 {
                                selected_index.set(Some(index));
                            }
                        }
                        
                        let timeout2 = Timeout::new(300, {
                            let animations = animations.clone();
                            move || {
                                animations.set(HashMap::new());
                            }
                        });
                        timeout2.forget();
                    }
                });
                timeout.forget();
            } else {
                log::warn!("⚠️ No se puede mover paquete {} hacia {} (límites: 0-{})", index, direction, pkgs.len() - 1);
            }
        })
    };
    
    // Update package
    let update_package = {
        let packages = packages.clone();
        Callback::from(move |(id, lat, lng, new_address): (String, f64, f64, String)| {
            let mut pkgs = (*packages).clone();
            if let Some(pkg) = pkgs.iter_mut().find(|p| p.id == id) {
                pkg.address = new_address;
                pkg.coords = Some([lng, lat]);
                pkg.is_problematic = false; // Ya no es problemático si se geocodificó correctamente
                log::info!("✅ Paquete {} actualizado y removido de problemáticos", id);
            }
            
            // Reordenar: paquetes problemáticos al final
            pkgs.sort_by(|a, b| {
                match (a.is_problematic, b.is_problematic) {
                    (true, false) => std::cmp::Ordering::Greater,  // a va después de b
                    (false, true) => std::cmp::Ordering::Less,     // a va antes de b
                    _ => std::cmp::Ordering::Equal,                // mantener orden original
                }
            });
            
            packages.set(pkgs);
        })
    };
    
    // Mark package as problematic
    let mark_problematic = {
        let packages = packages.clone();
        Callback::from(move |package_id: String| {
            let mut pkgs = (*packages).clone();
            if let Some(pkg) = pkgs.iter_mut().find(|p| p.id == package_id) {
                pkg.is_problematic = true;
                pkg.coords = None; // Quitar coordenadas para que no aparezca en el mapa
                log::info!("⚠️ Paquete {} marcado como problemático", package_id);
                
                // Remover del mapa usando JavaScript
                if let Some(window) = web_sys::window() {
                    if let Ok(js_value) = js_sys::Reflect::get(&window, &"removePackageFromMap".into()) {
                        if let Ok(remove_func) = js_value.dyn_into::<js_sys::Function>() {
                            let package_id_clone = package_id.clone();
                            let _ = remove_func.call1(&window, &package_id_clone.into());
                            log::info!("🗑️ Paquete {} removido del mapa via JavaScript", package_id);
                        }
                    }
                }
            }
            
            // Reordenar: paquetes problemáticos al final
            pkgs.sort_by(|a, b| {
                match (a.is_problematic, b.is_problematic) {
                    (true, false) => std::cmp::Ordering::Greater,  // a va después de b
                    (false, true) => std::cmp::Ordering::Less,     // a va antes de b
                    _ => std::cmp::Ordering::Equal,                // mantener orden original
                }
            });
            
            packages.set(pkgs);
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
    
    // Optimize route with Mapbox
    let optimize_route = {
        let packages = packages.clone();
        let optimizing = optimizing.clone();
        
        Callback::from(move |_: MouseEvent| {
            // Get driver location from JavaScript
            if let Some(window) = web_sys::window() {
                if let Ok(get_location_fn) = js_sys::Reflect::get(&window, &"getDriverLocation".into()) {
                    if let Ok(func) = get_location_fn.dyn_into::<js_sys::Function>() {
                        match func.call0(&window) {
                            Ok(location_value) => {
                                if location_value.is_null() || location_value.is_undefined() {
                                    // No location available
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
                                    let packages = packages.clone();
                                    let optimizing = optimizing.clone();
                                    
                                    wasm_bindgen_futures::spawn_local(async move {
                                        optimizing.set(true);
                                        log::info!("🎯 Iniciando optimización de ruta...");
                                        
                                        // Extract locations from packages
                                        let locations: Vec<PackageLocation> = packages.iter()
                                            .filter_map(|p| {
                                                let coords = p.coords?;
                                                Some(PackageLocation {
                                                    id: p.id.clone(),
                                                    latitude: coords[1], // [lng, lat]
                                                    longitude: coords[0],
                                                    type_livraison: Some(p.type_livraison.clone().unwrap_or_else(|| "DOMICILE".to_string())),
                                                })
                                            })
                                            .collect();
                                        
                                        if locations.is_empty() {
                                            log::error!("❌ No hay paquetes con coordenadas");
                                            optimizing.set(false);
                                            if let Some(win) = web_sys::window() {
                                                let _ = win.alert_with_message("No hay paquetes con coordenadas para optimizar");
                                            }
                                            return;
                                        }
                                        
                                        let depot = DepotLocation { latitude, longitude };
                                        
                                        log::info!("📍 Optimizando {} ubicaciones desde ({}, {})", 
                                            locations.len(), latitude, longitude);
                                        
                                        // El backend hace todo el polling internamente
                                        // Solo esperamos la respuesta final
                                        match mapbox_optimize_route(locations, depot).await {
                                            Ok(response) => {
                                                optimizing.set(false);
                                                
                                                if response.success {
                                                    log::info!("✅ Optimización completa");
                                                    
                                                    // Reordenar paquetes según el orden optimizado
                                                    if let Some(optimized_order) = response.optimized_order {
                                                        let current_packages = (*packages).clone();
                                                        
                                                        // Crear mapa ID -> Package
                                                        let package_map: HashMap<String, Package> = current_packages
                                                            .into_iter()
                                                            .map(|p| (p.id.clone(), p))
                                                            .collect();
                                                        
                                                        // Reordenar según el orden optimizado
                                                        let mut reordered = Vec::new();
                                                        for id in &optimized_order {
                                                            if let Some(pkg) = package_map.get(id) {
                                                                reordered.push(pkg.clone());
                                                            }
                                                        }
                                                        
                                                        // Agregar paquetes que no estaban en el orden (por si acaso)
                                                        for (id, pkg) in package_map {
                                                            if !reordered.iter().any(|p| p.id == id) {
                                                                reordered.push(pkg);
                                                            }
                                                        }
                                                        
                                                        log::info!("✅ {} paquetes reordenados", reordered.len());
                                                        packages.set(reordered);
                                                        
                                                        if let Some(win) = web_sys::window() {
                                                            let _ = win.alert_with_message("✅ Ruta optimizada exitosamente");
                                                        }
                                                    } else {
                                                        log::warn!("⚠️ No se recibió orden optimizado");
                                                        if let Some(win) = web_sys::window() {
                                                            let _ = win.alert_with_message("✅ Optimización completa pero sin orden");
                                                        }
                                                    }
                                                } else {
                                                    log::error!("❌ Error: {:?}", response.message);
                                                    if let Some(win) = web_sys::window() {
                                                        let msg = response.message.unwrap_or_else(|| "Error desconocido".to_string());
                                                        let _ = win.alert_with_message(&format!("Error: {}", msg));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                log::error!("❌ Error optimizando: {}", e);
                                                optimizing.set(false);
                                                if let Some(win) = web_sys::window() {
                                                    let _ = win.alert_with_message(&format!("Error: {}", e));
                                                }
                                            }
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
        })
    };
    
    UsePackagesHandle {
        packages,
        loading,
        optimizing,
        selected_index,
        animations,
        expanded_groups,
        reorder_mode,
        reorder_origin,
        filter_mode,
        refresh,
        optimize,
        select_package,
        reorder,
        update_package,
        mark_problematic,
        toggle_group,
        toggle_reorder_mode,
        toggle_filter_mode,
        optimize_route,
    }
}

/// Clear packages cache
pub fn clear_packages_cache(company_code: &str, username: &str) {
    if let Some(storage) = get_local_storage() {
        let cache_key = format!("{}_{}", STORAGE_KEY_PACKAGES_PREFIX, format!("{}_{}", company_code, username));
        let _ = storage.remove_item(&cache_key);
        log::info!("🗑️ Cache de paquetes eliminado");
    }
}

