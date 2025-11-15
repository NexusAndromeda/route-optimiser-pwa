// ============================================================================
// APP VIEW - COMPONENTE PRINCIPAL
// ============================================================================
// ✅ HTML/CSS EXACTO DEL ORIGINAL preservado
// Usa hooks nativos de Yew en lugar de Yewdux (compatibilidad Rust 1.90)
// ============================================================================

use yew::prelude::*;
use crate::hooks::{use_session, use_sync_state, use_auth, group_packages, GroupBy, use_map, use_map_selection_listener, SessionContextProvider, PackageGroup};
use crate::components::{SyncIndicator, Scanner, DraggablePackageList, SettingsPopup, PackageList, DetailsModal};
use crate::views::login::LoginView;
use crate::viewmodels::{SessionViewModel, MapViewModel};
use crate::models::{package::Package, address::Address};
use crate::services::{SyncService, OfflineService};
use crate::utils::t;
use wasm_bindgen::{JsCast, JsValue};
use js_sys::Reflect;
use wasm_bindgen::closure::Closure;
use gloo_timers::callback::Timeout;
use web_sys::HtmlInputElement;

// ============================================================================
// HELPER FUNCTION: Encontrar group_idx por tracking
// ============================================================================
/// Busca el índice del grupo (group_idx) que contiene el paquete con el tracking dado
fn find_group_idx_by_tracking(
    tracking: &str,
    groups: &[PackageGroup],
) -> Option<usize> {
    groups.iter()
        .enumerate()
        .find(|(_, group)| {
            group.packages.iter().any(|p| p.tracking == tracking)
        })
        .map(|(idx, _)| idx)
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <SessionContextProvider>
            <AppContent />
        </SessionContextProvider>
    }
}

#[function_component(AppContent)]
fn app_content() -> Html {
    let session_handle = use_session();
    let sync_handle = use_sync_state();
    let auth_handle = use_auth();
    let map_handle = use_map();
    
    let session_state = session_handle.state.clone();
    let auth_state = auth_handle.state.clone();
    
    let is_logged_in = auth_state.is_logged_in;
    let loading = session_state.loading;
    
    // Log del estado de autenticación para debugging
    {
        let auth_state_log = auth_state.clone();
        let session_state_log = session_state.clone();
        use_effect_with(is_logged_in, move |logged| {
            log::info!("🔍 [APP] Estado de is_logged_in cambió: {}", logged);
            log::info!("🔍 [APP] auth_state.is_logged_in: {}, session: {:?}", 
                auth_state_log.is_logged_in, 
                session_state_log.session.as_ref().map(|s| s.session_id.clone()));
            || ()
        });
    }
    
    // Cargar sesión al iniciar (localStorage)
    {
        let session_state = session_handle.state.clone();
        let auth_state = auth_handle.state.clone();
        use_effect_with((), move |_| {
            let vm = SessionViewModel::new();
            let session_state = session_state.clone();
            let auth_state = auth_state.clone();

            // Listener de 'loggedIn'
            log::info!("🔍 [APP] Configurando listener para evento 'loggedIn'...");
            let win = web_sys::window().unwrap();
            let session_for_event = session_state.clone();
            let auth_for_event = auth_state.clone();
            let on_logged = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                log::info!("🔔 [APP] Evento 'loggedIn' recibido!");
                let vm = SessionViewModel::new();
                let session_state_in = session_for_event.clone();
                let auth_state_in = auth_for_event.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("🔔 [APP] Cargando sesión desde storage...");
                    match vm.load_session_from_storage().await {
                        Ok(Some(session)) => {
                            log::info!("✅ [APP] Sesión cargada desde storage: {} paquetes", session.stats.total_packages);
                        let mut new_session_state = (*session_state_in).clone();
                        new_session_state.session = Some(session);
                        session_state_in.set(new_session_state);
                            log::info!("🔔 [APP] Estado de sesión actualizado");
                            
                        let mut new_auth_state = (*auth_state_in).clone();
                            log::info!("🔔 [APP] Estado auth antes: is_logged_in={}", new_auth_state.is_logged_in);
                        new_auth_state.is_logged_in = true;
                            let is_logged_in_after = new_auth_state.is_logged_in;
                        auth_state_in.set(new_auth_state);
                            log::info!("✅ [APP] Estado auth actualizado: is_logged_in={}", is_logged_in_after);
                        }
                        Ok(None) => {
                            log::warn!("⚠️ [APP] No hay sesión en storage");
                        }
                        Err(e) => {
                            log::error!("❌ [APP] Error cargando sesión desde storage: {}", e);
                        }
                    }
                });
            }) as Box<dyn FnMut(_)>);
            match win.add_event_listener_with_callback("loggedIn", on_logged.as_ref().unchecked_ref()) {
                Ok(_) => log::info!("✅ [APP] Listener 'loggedIn' registrado exitosamente"),
                Err(e) => log::error!("❌ [APP] Error registrando listener 'loggedIn': {:?}", e),
            }

            // Carga inicial
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(Some(session)) = vm.load_session_from_storage().await {
                    log::info!("📋 Sesión cargada desde storage: {} paquetes", session.stats.total_packages);
                    let mut new_session_state = (*session_state).clone();
                    new_session_state.session = Some(session.clone());
                    session_state.set(new_session_state);
                    let mut new_auth_state = (*auth_state).clone();
                    new_auth_state.is_logged_in = true;
                    auth_state.set(new_auth_state);
                }
            });

            // Mantener closure vivo
            on_logged.forget();
            || ()
        });
    }
    
    let show_scanner = use_state(|| false);
    let show_params = use_state(|| false);
    let sheet_state = use_state(|| String::from("half")); // collapsed | half | full
    let selected_package_index = use_state(|| None::<usize>); // Índice del paquete seleccionado
    let show_details = use_state(|| false);
    let details_package = use_state(|| None::<(Package, Address)>); // Paquete y dirección para el modal
    
    // Estados para modal de búsqueda de trackings
    let show_tracking_modal = use_state(|| false);
    let tracking_query = use_state(|| String::new());
    
    // Estados para idioma, filtro y modo edición
    let language = use_state(|| {
        // Cargar desde localStorage si existe
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(lang)) = storage.get_item("language") {
                    return lang;
                }
            }
        }
        "ES".to_string() // Por defecto español
    });
    let filter_mode = use_state(|| false); // Filtrar solo STATUT_CHARGER
    let edit_mode = use_state(|| false); // Modo edición para reordenar
    let edit_origin_idx = use_state(|| None::<usize>); // Índice del paquete/grupo seleccionado como origen
    
    // Actualizar details_package cuando la sesión se actualiza
    {
        let session_state = session_handle.state.clone();
        let details_package = details_package.clone();
        use_effect_with(session_state.session.clone(), move |session_opt| {
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("🔄 EFECTO: Sesión cambió, intentando actualizar details_package");
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            if let Some(session) = session_opt {
                log::info!("   📊 Sesión tiene {} paquetes y {} direcciones", 
                          session.packages.len(), session.addresses.len());
                
                // Leer el estado actual de details_package sin incluirlo en dependencias
                if let Some((pkg, addr)) = (*details_package).clone() {
                    log::info!("   🔍 details_package actual:");
                    log::info!("      - tracking: {}", pkg.tracking);
                    log::info!("      - address_id: {}", addr.address_id);
                    log::info!("      - label actual: '{}'", addr.label);
                    log::info!("      - is_problematic actual: {}", pkg.is_problematic);
                    
                    // Buscar paquete y dirección actualizados en la sesión
                    if let Some(updated_pkg) = session.packages.get(&pkg.tracking) {
                        log::info!("      ✅ Paquete encontrado en sesión:");
                        log::info!("         - is_problematic: {}", updated_pkg.is_problematic);
                        
                        if let Some(updated_addr) = session.addresses.get(&addr.address_id) {
                            log::info!("      ✅ Dirección encontrada en sesión:");
                            log::info!("         - label: '{}' (len: {}, is_empty: {})", 
                                      updated_addr.label, updated_addr.label.len(), 
                                      updated_addr.label.trim().is_empty());
                            log::info!("         - lat: {}, lng: {}", 
                                      updated_addr.latitude, updated_addr.longitude);
                            
                            // Verificar si hay cambios
                            let pkg_changed = updated_pkg.is_problematic != pkg.is_problematic;
                            let addr_changed = updated_addr.label != addr.label || 
                                             updated_addr.latitude != addr.latitude || 
                                             updated_addr.longitude != addr.longitude;
                            
                            if pkg_changed || addr_changed {
                                log::info!("      🔄 Cambios detectados! Actualizando details_package...");
                                log::info!("         - is_problematic cambió: {} → {}", pkg.is_problematic, updated_pkg.is_problematic);
                                log::info!("         - label cambió: '{}' → '{}'", addr.label, updated_addr.label);
                                
                                details_package.set(Some((updated_pkg.clone(), updated_addr.clone())));
                                log::info!("   ✅✅✅ details_package ACTUALIZADO DESDE EFECTO");
                            } else {
                                log::info!("      ℹ️ No hay cambios, no se actualiza details_package");
                            }
                        } else {
                            log::warn!("      ❌ Dirección NO encontrada en sesión.addresses!");
                            log::info!("      🔍 Addresses disponibles: {:?}", 
                                      session.addresses.keys().take(5).collect::<Vec<_>>());
                        }
                    } else {
                        log::warn!("      ❌ Paquete NO encontrado en sesión.packages!");
                        log::info!("      🔍 Tracking buscado: {}", pkg.tracking);
                    }
                } else {
                    log::warn!("   ⚠️ details_package es None");
                }
            } else {
                log::warn!("   ⚠️ session_opt es None");
            }
            
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            || ()
        });
    }
    
    let toggle_scanner = {
        let show_scanner = show_scanner.clone();
        Callback::from(move |_| {
            show_scanner.set(!*show_scanner);
        })
    };
    let toggle_params = {
        let show_params = show_params.clone();
        Callback::from(move |_| {
            show_params.set(!*show_params);
        })
    };

    let on_close_settings = {
        let show_params = show_params.clone();
        Callback::from(move |_| show_params.set(false))
    };
    
    let on_logout = {
        let session_state = session_handle.state.clone();
        let auth_state = auth_handle.state.clone();
        let show_params = show_params.clone();
        let reset_map = map_handle.reset.clone();
        
        Callback::from(move |_| {
            log::info!("👋 Logout iniciado");
            
            // Limpiar con ViewModel
            let vm = SessionViewModel::new();
            if let Err(e) = vm.logout() {
                log::error!("❌ Error en logout: {}", e);
            }
            
            // Resetear estado de sesión
            let mut new_session_state = (*session_state).clone();
            new_session_state.session = None;
            new_session_state.loading = false;
            new_session_state.error = None;
            session_state.set(new_session_state);
            
            // Resetear estado de auth
            let mut new_auth_state = (*auth_state).clone();
            new_auth_state.is_logged_in = false;
            new_auth_state.username = None;
            new_auth_state.token = None;
            new_auth_state.company_id = None;
            auth_state.set(new_auth_state);
            
            // Resetear estado del mapa (patrón de app-backup)
            reset_map.emit(());
            
            // Cerrar popup de settings
            show_params.set(false);
            
            log::info!("✅ Logout completado");
        })
    };
    
    let on_retry = Callback::from(|_| log::info!("retry map"));

    let toggle_sheet_size = {
        let sheet_state = sheet_state.clone();
        Callback::from(move |_| {
            let cur = sheet_state.as_str().to_string();
            let next = if cur == "collapsed" { "half" } else if cur == "half" { "full" } else { "collapsed" };
            sheet_state.set(next.to_string());
        })
    };

    let close_sheet = {
        let sheet_state = sheet_state.clone();
        Callback::from(move |_| sheet_state.set("collapsed".to_string()))
    };
    
    // Memorizar grupos de paquetes para evitar reordenamientos en cada render
    let filter_mode_clone = filter_mode.clone();
    let groups_memo = use_memo(
        (session_state.session.clone(), *filter_mode),
        |(session_opt, filter)| {
            session_opt.as_ref().map(|session| {
                let mut items: Vec<_> = session.packages.values()
                    .cloned()
                    .collect();
                
                // Aplicar filtro si está activado
                if *filter {
                    items.retain(|p| p.status.starts_with("STATUT_CHARGER"));
                }
                
                items.sort_by_key(|p| p.route_order.unwrap_or(p.original_order));
                let groups = group_packages(items, GroupBy::Address);
                
                log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                log::info!("📦 AGRUPACIÓN DE PAQUETES");
                log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                log::info!("   📊 Total paquetes en sesión: {}", session.packages.len());
                log::info!("   📦 Total grupos creados: {}", groups.len());
                
                // Log de los primeros 10 grupos
                for (idx, group) in groups.iter().take(10).enumerate() {
                    log::info!("   [{idx}] group_idx={}, count={}, address_id={}", 
                              idx, group.count, group.title);
                }
                if groups.len() > 10 {
                    log::info!("   ... y {} grupos más", groups.len() - 10);
                }
                log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                
                groups
            })
        }
    );
    
    // Callback cuando se detecta un código de barras
    let on_barcode_detected = {
        let scan_package = session_handle.scan_package.clone();
        let show_scanner = show_scanner.clone();
        let sheet_state = sheet_state.clone();
        let selected_package_index = selected_package_index.clone();
        let center_on_package = map_handle.center_on_package.clone();
        let session_state = session_handle.state.clone();
        let groups_memo = groups_memo.clone();
        
        Callback::from(move |tracking: String| {
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("📱 CÓDIGO DE BARRAS DETECTADO");
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("   📦 Tracking: {}", tracking);
            
            // 1. Emitir scan_package (actualiza estado en backend)
            scan_package.emit(tracking.clone());
            
            // 2. Cerrar scanner
            show_scanner.set(false);
            
            // 3. Buscar group_idx del paquete
            let group_idx_opt = if let Some(session) = &(*session_state).session {
                if let Some(groups) = (*groups_memo).as_ref() {
                    find_group_idx_by_tracking(&tracking, groups)
                } else {
                    log::warn!("   ⚠️ No hay grupos disponibles en groups_memo");
                    None
                }
            } else {
                log::warn!("   ⚠️ No hay sesión activa");
                None
            };
            
            if let Some(group_idx) = group_idx_opt {
                log::info!("   ✅ group_idx encontrado: {}", group_idx);
                
                // 4. Abrir bottom sheet si está colapsado
                let current_state = (*sheet_state).clone();
                log::info!("   📱 Estado actual del sheet: {}", current_state);
                if current_state == "collapsed" {
                    sheet_state.set("half".to_string());
                    log::info!("   📱 Bottom sheet abierto desde colapsado → half");
                }
                
                // 5. Actualizar índice seleccionado
                selected_package_index.set(Some(group_idx));
                log::info!("   ✅ selected_package_index actualizado: {:?}", Some(group_idx));
                
                // 6. Centrar mapa en el paquete (con delay para que el sheet se abra primero)
                let center_on_package_clone = center_on_package.clone();
                Timeout::new(300, move || {
                    log::info!("   🗺️ Centrando mapa en grupo {}...", group_idx);
                    center_on_package_clone.emit(group_idx);
                    
                    // 7. Hacer scroll al card (con delay adicional para que el mapa se centre)
                    Timeout::new(100, move || {
                        if let Some(window) = web_sys::window() {
                            let scroll_fn = js_sys::Function::new_no_args(&format!(
                                "if (window.scrollToSelectedPackage) window.scrollToSelectedPackage({});",
                                group_idx
                            ));
                            let _ = scroll_fn.call0(&window.into());
                            log::info!("   📜 Scroll completado para grupo {}", group_idx);
                        }
                    }).forget();
                }).forget();
            } else {
                log::warn!("   ⚠️ No se encontró group_idx para tracking: {}", tracking);
            }
            
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        })
    };
    
    // Callback cuando se selecciona un tracking desde el modal de búsqueda
    let on_tracking_selected = {
        let scan_package = session_handle.scan_package.clone();
        let show_tracking_modal = show_tracking_modal.clone();
        let sheet_state = sheet_state.clone();
        let selected_package_index = selected_package_index.clone();
        let center_on_package = map_handle.center_on_package.clone();
        let session_state = session_handle.state.clone();
        let groups_memo = groups_memo.clone();
        
        Callback::from(move |tracking: String| {
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("🔍 TRACKING SELECCIONADO DESDE MODAL");
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("   📦 Tracking: {}", tracking);
            
            // 1. Emitir scan_package (actualiza estado en backend)
            scan_package.emit(tracking.clone());
            
            // 2. Cerrar modal
            show_tracking_modal.set(false);
            
            // 3. Buscar group_idx del paquete
            let group_idx_opt = if let Some(session) = &(*session_state).session {
                if let Some(groups) = (*groups_memo).as_ref() {
                    find_group_idx_by_tracking(&tracking, groups)
                } else {
                    log::warn!("   ⚠️ No hay grupos disponibles en groups_memo");
                    None
                }
            } else {
                log::warn!("   ⚠️ No hay sesión activa");
                None
            };
            
            if let Some(group_idx) = group_idx_opt {
                log::info!("   ✅ group_idx encontrado: {}", group_idx);
                
                // 4. Abrir bottom sheet si está colapsado
                let current_state = (*sheet_state).clone();
                log::info!("   📱 Estado actual del sheet: {}", current_state);
                if current_state == "collapsed" {
                    sheet_state.set("half".to_string());
                    log::info!("   📱 Bottom sheet abierto desde colapsado → half");
                }
                
                // 5. Actualizar índice seleccionado
                selected_package_index.set(Some(group_idx));
                log::info!("   ✅ selected_package_index actualizado: {:?}", Some(group_idx));
                
                // 6. Centrar mapa en el paquete (con delay para que el sheet se abra primero)
                let center_on_package_clone = center_on_package.clone();
                Timeout::new(300, move || {
                    log::info!("   🗺️ Centrando mapa en grupo {}...", group_idx);
                    center_on_package_clone.emit(group_idx);
                    
                    // 7. Hacer scroll al card (con delay adicional para que el mapa se centre)
                    Timeout::new(100, move || {
                        if let Some(window) = web_sys::window() {
                            let scroll_fn = js_sys::Function::new_no_args(&format!(
                                "if (window.scrollToSelectedPackage) window.scrollToSelectedPackage({});",
                                group_idx
                            ));
                            let _ = scroll_fn.call0(&window.into());
                            log::info!("   📜 Scroll completado para grupo {}", group_idx);
                        }
                    }).forget();
                }).forget();
            } else {
                log::warn!("   ⚠️ No se encontró group_idx para tracking: {}", tracking);
            }
            
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        })
    };
    
    // Si no está logueado, mostrar login
    log::info!("🔍 [APP] Renderizando: is_logged_in={}, loading={}, tiene_session={}", 
        is_logged_in, loading, session_state.session.is_some());
    
    if !is_logged_in {
        log::info!("🔍 [APP] Usuario no logueado, mostrando LoginView");
        return html! {
            <LoginView />
        };
    }
    
    log::info!("🔍 [APP] Usuario logueado, mostrando vista principal");
    
    // Inicializar mapa cuando se hace login (MVVM)
    // IMPORTANTE: Se inicializa cuando el componente se monta Y está logueado,
    // o cuando is_logged_in cambia de false a true
    {
        let map_init = map_handle.initialize.clone();
        let map_initialized = map_handle.state.initialized.clone();
        let is_logged = is_logged_in;
        
        // Efecto que se ejecuta al montar Y cuando cambian las dependencias
        use_effect_with((is_logged, map_initialized.clone()), move |(logged, initialized)| {
            log::info!("🔍 use_effect_with ejecutado: is_logged={}, map_initialized={}", logged, initialized);
            
            // Solo inicializar si está logueado Y el mapa no está inicializado
            if *logged && !*initialized {
                log::info!("🗺️ Usuario logueado y mapa no inicializado, inicializando... (is_logged: {}, initialized: {})", 
                          logged, initialized);
                
                // Pequeño delay para asegurar que el DOM está listo (especialmente después de logout/login)
                use gloo_timers::callback::Timeout;
                Timeout::new(200, move || {
                    log::info!("🗺️ Llamando a initialize después del delay (200ms)...");
                    map_init.emit(());
                }).forget();
            } else {
                log::debug!("🗺️ Condiciones no cumplidas para inicializar mapa (is_logged: {}, initialized: {})", 
                           logged, initialized);
            }
            || ()
        });
    }
    
    // Enviar paquetes al mapa cuando cambia la sesión (MVVM)
    {
        let session_opt = session_state.session.clone();
        let map_update = map_handle.update_packages.clone();
        let map_handle_state = map_handle.state.clone();
        let filter_mode_for_map = filter_mode.clone();
        
        use_effect_with((session_opt.clone(), *filter_mode, map_handle_state.initialized), move |(session_opt, filter, initialized)| {
            if let Some(session) = session_opt {
                if *initialized {
                    log::info!("📦 Sesión actualizada, preparando paquetes para el mapa...");
                    
                    // Convertir HashMap a Vec y aplicar filtro si está activado
                    let mut packages_vec: Vec<_> = session.packages.values().cloned().collect();
                    if *filter {
                        packages_vec.retain(|p| p.status.starts_with("STATUT_CHARGER"));
                    }
                    let packages_count = packages_vec.len();
                    
                    // Preparar grupos
                    let groups = group_packages(packages_vec, GroupBy::Address);
                    log::info!("📦 Grupos preparados: {} grupos de {} paquetes", 
                              groups.len(), packages_count);
                    
                    // Preparar paquetes para el mapa (usando ViewModel)
                    let packages_for_map = MapViewModel::prepare_packages_for_map(&groups, &session);
                    log::info!("🗺️ Paquetes preparados para mapa: {} de {} grupos", 
                              packages_for_map.len(), groups.len());
                    
                    // Enviar al mapa con delay
                    Timeout::new(200, move || {
                        map_update.emit(packages_for_map);
                    }).forget();
                }
            }
            || ()
        });
    }

    // Re-enviar paquetes cuando el mapa se marca como inicializado
    {
        let session_state_clone = session_state.clone();
        let map_update = map_handle.update_packages.clone();
        let map_initialized = map_handle.state.initialized;
        let filter_mode_for_map_init = filter_mode.clone();
        
        use_effect_with((map_initialized.clone(), *filter_mode), move |(initialized, filter)| {
            if *initialized {
                if let Some(session) = &session_state_clone.session {
                    log::info!("🗺️ Mapa ahora inicializado, re-enviando paquetes...");
                    
                    // Convertir HashMap a Vec y aplicar filtro si está activado
                    let mut packages_vec: Vec<_> = session.packages.values().cloned().collect();
                    if *filter {
                        packages_vec.retain(|p| p.status.starts_with("STATUT_CHARGER"));
                    }
                    
                    // Preparar grupos
                    let groups = group_packages(packages_vec, GroupBy::Address);
                    
                    // Preparar paquetes para el mapa
                    let packages_for_map = MapViewModel::prepare_packages_for_map(&groups, session);
                    
                    // Enviar al mapa con delay más largo
                    Timeout::new(500, move || {
                        log::info!("📤 Enviando {} paquetes al mapa...", packages_for_map.len());
                        map_update.emit(packages_for_map);
                    }).forget();
                }
            }
            || ()
        });
    }
    
    // Escuchar clicks en el mapa (SINCRONIZACIÓN: Mapa → Bottom Sheet)
    // Usa el hook dedicado que garantiza registro único (patrón de app-backup)
    let on_map_select = {
        let map_select = map_handle.select_package.clone();
        let sheet_state = sheet_state.clone();
        let selected_package_index = selected_package_index.clone();
        let edit_mode_clone = edit_mode.clone();
        let edit_origin_idx_clone = edit_origin_idx.clone();
        let session_handle_clone = session_handle.clone();
        let groups_memo_clone = groups_memo.clone();
        let groups_count = if let Some(ref groups) = *groups_memo {
            groups.len()
        } else {
            0
        };
        
        Callback::from(move |package_index: usize| {
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("🖱️ CLICK EN MAPA RECIBIDO EN APP");
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("   📍 group_idx recibido: {}", package_index);
            log::info!("   📦 Total grupos disponibles: {}", groups_count);
            log::info!("   ✏️ Modo edición: {}", *edit_mode_clone);
            
            if package_index >= groups_count {
                log::warn!("⚠️  group_idx {} >= grupos disponibles {}, ignorando", 
                          package_index, groups_count);
                return;
            }
            
            log::info!("   ✅ group_idx válido, actualizando selección...");
            
            // Si está en modo edición, manejar reordenamiento
            if *edit_mode_clone {
                if let Some(origin_idx) = *edit_origin_idx_clone {
                    // Ya tenemos origen, este es el destino - ejecutar reordenamiento
                    if origin_idx != package_index {
                        log::info!("   🔄 Reordenando desde mapa: origen {} → destino {}", origin_idx, package_index);
                        
                        // Reordenar grupos y actualizar route_order
                        if let Some(mut groups) = (*groups_memo_clone).as_ref().cloned() {
                            if origin_idx < groups.len() && package_index < groups.len() {
                                let group_to_move = groups.remove(origin_idx);
                                let dest_idx = if package_index > origin_idx { package_index - 1 } else { package_index };
                                groups.insert(dest_idx.min(groups.len()), group_to_move);
                                
                                // Actualizar route_order de todos los paquetes
                                let mut new_order = 0;
                                let mut trackings_order: Vec<(String, usize)> = Vec::new();
                                for group in &groups {
                                    for pkg in &group.packages {
                                        trackings_order.push((pkg.tracking.clone(), new_order));
                                    }
                                    new_order += 1;
                                }
                                
                                // Actualizar sesión con nuevo orden
                                if let Some(session) = &session_handle_clone.state.session {
                                    let mut updated_session = session.clone();
                                    
                                    // Guardar posiciones anteriores antes de actualizar
                                    let mut old_positions: Vec<(String, usize)> = Vec::new();
                                    for (tracking, _) in &trackings_order {
                                        if let Some(pkg) = updated_session.packages.get(tracking) {
                                            let old_pos = pkg.route_order.unwrap_or(pkg.visual_position);
                                            old_positions.push((tracking.clone(), old_pos));
                                        }
                                    }
                                    
                                    // Actualizar route_order y visual_position
                                    for (tracking, order) in &trackings_order {
                                        if let Some(pkg) = updated_session.packages.get_mut(tracking) {
                                            pkg.route_order = Some(*order);
                                            pkg.visual_position = *order;
                                        }
                                    }
                                    
                                    // Guardar sesión actualizada en localStorage
                                    let offline_service = OfflineService::new();
                                    if let Err(e) = offline_service.save_session(&updated_session) {
                                        log::error!("❌ Error guardando sesión en localStorage: {}", e);
                                    } else {
                                        log::info!("✅ Sesión guardada en localStorage");
                                    }
                                    
                                    // Guardar sesión actualizada
                                    let session_id = updated_session.session_id.clone();
                                    let sync_service = SyncService::new();
                                    let trackings_order_clone = trackings_order.clone();
                                    let session_handle_for_sync = session_handle_clone.clone();
                                    
                                    // Crear cambios de sincronización con old_position correcto
                                    let mut changes = Vec::new();
                                    for (tracking, new_pos) in &trackings_order_clone {
                                        // Buscar la posición anterior del paquete
                                        let old_pos = old_positions.iter()
                                            .find(|(t, _)| t == tracking)
                                            .map(|(_, pos)| *pos)
                                            .unwrap_or(origin_idx);
                                        
                                        changes.push(crate::models::sync::Change::OrderChanged {
                                            package_internal_id: tracking.clone(),
                                            old_position: old_pos,
                                            new_position: *new_pos,
                                            timestamp: chrono::Utc::now().timestamp(),
                                        });
                                    }
                                    
                                    // Guardar cambios pendientes y sincronizar
                                    wasm_bindgen_futures::spawn_local(async move {
                                        if let Err(e) = sync_service.save_pending_changes(&changes) {
                                            log::error!("❌ Error guardando cambios pendientes: {}", e);
                                        }
                                        
                                        // Procesar queue automáticamente
                                        if let Err(e) = sync_service.process_pending_queue().await {
                                            log::warn!("⚠️ Error sincronizando cambios: {}", e);
                                        }
                                    });
                                    
                                    // Actualizar estado local
                                    let mut new_state = (*session_handle_for_sync.state).clone();
                                    new_state.session = Some(updated_session);
                                    session_handle_for_sync.state.set(new_state);
                                    
                                    log::info!("   ✅ Reordenamiento completado y sincronizado");
                                }
                            }
                        }
                        
                        // Limpiar origen
                        edit_origin_idx_clone.set(None);
                    } else {
                        // Mismo índice, cancelar
                        edit_origin_idx_clone.set(None);
                    }
                } else {
                    // Primer click - establecer como origen
                    edit_origin_idx_clone.set(Some(package_index));
                    selected_package_index.set(Some(package_index));
                    log::info!("   📍 Origen establecido: {}", package_index);
                    
                    // Mostrar animación flash
                    map_select.clone().emit(package_index);
                }
            } else {
                // Modo normal - comportamiento original
            // Actualizar índice seleccionado
            selected_package_index.set(Some(package_index));
            log::info!("   ✅ selected_package_index actualizado: {:?}", Some(package_index));
            
            // Abrir bottom sheet si está colapsado (patrón de app-backup)
            let current_state = (*sheet_state).clone();
            log::info!("   📱 Estado actual del sheet: {}", current_state);
            if current_state == "collapsed" {
                sheet_state.set("half".to_string());
                log::info!("   📱 Bottom sheet abierto desde colapsado → half");
            }
            
            // Hacer scroll y animación flash (con delay para que el sheet se abra primero)
            let map_select_clone = map_select.clone();
            use gloo_timers::callback::Timeout;
            Timeout::new(300, move || {
                log::info!("   ⏱️  Delay completado, emitiendo select_package con group_idx: {}", 
                          package_index);
                map_select_clone.emit(package_index);
            }).forget();
            }
            
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        })
    };
    use_map_selection_listener(on_map_select);
    
    // Callback cuando se selecciona un paquete en el bottom sheet
    let on_package_selected = {
        let center_on_package = map_handle.center_on_package.clone();
        let selected_package_index = selected_package_index.clone();
        let edit_mode_clone = edit_mode.clone();
        let edit_origin_idx_clone = edit_origin_idx.clone();
        let session_handle_clone = session_handle.clone();
        let groups_memo_clone = groups_memo.clone();
        
        Callback::from(move |index: usize| {
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("📦 PAQUETE SELECCIONADO EN BOTTOM SHEET");
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("   📍 group_idx: {}", index);
            log::info!("   ✏️ Modo edición: {}", *edit_mode_clone);
            
            // Si está en modo edición, manejar reordenamiento
            if *edit_mode_clone {
                if let Some(origin_idx) = *edit_origin_idx_clone {
                    // Ya tenemos origen, este es el destino - ejecutar reordenamiento
                    if origin_idx != index {
                        log::info!("   🔄 Reordenando desde bottom sheet: origen {} → destino {}", origin_idx, index);
                        
                        // Reordenar grupos y actualizar route_order
                        if let Some(mut groups) = (*groups_memo_clone).as_ref().cloned() {
                            if origin_idx < groups.len() && index < groups.len() {
                                let group_to_move = groups.remove(origin_idx);
                                let dest_idx = if index > origin_idx { index - 1 } else { index };
                                groups.insert(dest_idx.min(groups.len()), group_to_move);
                                
                                // Actualizar route_order de todos los paquetes
                                let mut new_order = 0;
                                let mut trackings_order: Vec<(String, usize)> = Vec::new();
                                for group in &groups {
                                    for pkg in &group.packages {
                                        trackings_order.push((pkg.tracking.clone(), new_order));
                                    }
                                    new_order += 1;
                                }
                                
                                // Actualizar sesión con nuevo orden
                                if let Some(session) = &session_handle_clone.state.session {
                                    let mut updated_session = session.clone();
                                    
                                    // Guardar posiciones anteriores antes de actualizar
                                    let mut old_positions: Vec<(String, usize)> = Vec::new();
                                    for (tracking, _) in &trackings_order {
                                        if let Some(pkg) = updated_session.packages.get(tracking) {
                                            let old_pos = pkg.route_order.unwrap_or(pkg.visual_position);
                                            old_positions.push((tracking.clone(), old_pos));
                                        }
                                    }
                                    
                                    let trackings_order_clone = trackings_order.clone();
                                    
                                    // Actualizar route_order y visual_position
                                    for (tracking, order) in trackings_order {
                                        if let Some(pkg) = updated_session.packages.get_mut(&tracking) {
                                            pkg.route_order = Some(order);
                                            pkg.visual_position = order;
                                        }
                                    }
                                    
                                    // Guardar sesión actualizada en localStorage
                                    let offline_service = OfflineService::new();
                                    if let Err(e) = offline_service.save_session(&updated_session) {
                                        log::error!("❌ Error guardando sesión en localStorage: {}", e);
                                    } else {
                                        log::info!("✅ Sesión guardada en localStorage");
                                    }
                                    
                                    // Guardar sesión actualizada
                                    let session_id = updated_session.session_id.clone();
                                    let sync_service = SyncService::new();
                                    
                                    // Crear cambios de sincronización con old_position correcto
                                    let mut changes = Vec::new();
                                    for (tracking, new_pos) in trackings_order_clone {
                                        // Buscar la posición anterior del paquete
                                        let old_pos = old_positions.iter()
                                            .find(|(t, _)| t == &tracking)
                                            .map(|(_, pos)| *pos)
                                            .unwrap_or(origin_idx);
                                        
                                        changes.push(crate::models::sync::Change::OrderChanged {
                                            package_internal_id: tracking.clone(),
                                            old_position: old_pos,
                                            new_position: new_pos,
                                            timestamp: chrono::Utc::now().timestamp(),
                                        });
                                    }
                                    
                                    // Guardar cambios pendientes y sincronizar
                                    wasm_bindgen_futures::spawn_local(async move {
                                        if let Err(e) = sync_service.save_pending_changes(&changes) {
                                            log::error!("❌ Error guardando cambios pendientes: {}", e);
                                        }
                                        
                                        // Procesar queue automáticamente
                                        if let Err(e) = sync_service.process_pending_queue().await {
                                            log::warn!("⚠️ Error sincronizando cambios: {}", e);
                                        }
                                    });
                                    
                                    // Actualizar estado local
                                    let mut new_state = (*session_handle_clone.state).clone();
                                    new_state.session = Some(updated_session);
                                    session_handle_clone.state.set(new_state);
                                    
                                    log::info!("   ✅ Reordenamiento completado y sincronizado");
                                }
                            }
                        }
                        
                        // Limpiar origen
                        edit_origin_idx_clone.set(None);
                    } else {
                        // Mismo índice, cancelar
                        edit_origin_idx_clone.set(None);
                    }
                } else {
                    // Primer click - establecer como origen
                    edit_origin_idx_clone.set(Some(index));
                    selected_package_index.set(Some(index));
                    log::info!("   📍 Origen establecido desde bottom sheet: {}", index);
                }
            } else {
                // Modo normal - comportamiento original
            log::info!("   🗺️  Centrando mapa en grupo {}...", index);
            
            // Actualizar índice seleccionado
            selected_package_index.set(Some(index));
            
            // Centrar mapa en el paquete seleccionado (con pulse animation)
            center_on_package.emit(index);
            
            // ⭐ NUEVO: Scroll al card seleccionado (para centrarlo visualmente en el bottom sheet)
            // Usar delay de 300ms como en el prototipo para dar tiempo al render
            Timeout::new(300, move || {
                if let Some(window) = web_sys::window() {
                    let scroll_fn = js_sys::Function::new_no_args(&format!(
                        "if (window.scrollToSelectedPackage) window.scrollToSelectedPackage({});",
                        index
                    ));
                    let _ = scroll_fn.call0(&window.into());
                }
            }).forget();
            }
        })
    };

    // Preparar datos para el modal de trackings
    let session_state_for_modal = session_handle.state.clone();
    let tracking_query_for_modal = tracking_query.clone();
    let show_tracking_modal_for_modal = show_tracking_modal.clone();
    let on_tracking_selected_for_modal = on_tracking_selected.clone();
    
    let modal_class = if *show_tracking_modal { "company-modal show" } else { "company-modal" };
    
    // Obtener trackings de la sesión
    let trackings: Vec<String> = if let Some(session) = &(*session_state_for_modal).session {
        session.packages.keys().cloned().collect()
    } else {
        Vec::new()
    };
    
    // Filtrar trackings basado en la query
    let filtered_trackings: Vec<String> = {
        let q = (*tracking_query_for_modal).to_lowercase();
        if q.is_empty() {
            trackings.clone()
        } else {
            trackings.iter()
                .filter(|tracking| tracking.to_lowercase().contains(&q))
                .cloned()
                .collect()
        }
    };
    
    // Obtener información adicional de los paquetes para mostrar
    let tracking_items: Vec<(String, Option<String>, Option<String>)> = if let Some(session) = &(*session_state_for_modal).session {
        filtered_trackings.iter()
            .filter_map(|tracking| {
                session.packages.get(tracking).map(|pkg| {
                    (tracking.clone(), Some(pkg.customer_name.clone()), Some(pkg.status.clone()))
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    html! {
        <>
            <header class="app-header">
                <h1>{t("route_optimizer", &(*language))}</h1>
                <div class="header-actions">
                    <button 
                        class="btn-icon-header btn-optimize-mini" 
                        onclick={Callback::from({
                            let session_state = session_handle.state.clone();
                            move |_| {
                                let session_state = session_state.clone();
                                
                                // Obtener sesión actual
                                let current_session = {
                                    let session = (*session_state).clone();
                                    session.session.clone()
                                };
                                
                                if let Some(session) = current_session {
                                    let session_id = session.session_id.clone();
                                    let vm = SessionViewModel::new();
                                    
                                    // Marcar como cargando
                                    let mut new_state = (*session_state).clone();
                                    new_state.loading = true;
                                    new_state.error = None;
                                    session_state.set(new_state);
                                    
                                    wasm_bindgen_futures::spawn_local(async move {
                                        match vm.optimize_route(&session_id).await {
                                            Ok(updated_session) => {
                                                log::info!("✅ Optimización completada: {} paquetes en orden optimizado", 
                                                          updated_session.stats.total_packages);
                                                
                                                // Actualizar estado con sesión optimizada
                                                let mut new_state = (*session_state).clone();
                                                new_state.session = Some(updated_session);
                                                new_state.loading = false;
                                                new_state.error = None;
                                                session_state.set(new_state);
                                            }
                                            Err(e) => {
                                                log::error!("❌ Error optimizando ruta: {}", e);
                                                
                                                // Mostrar alert si el error es por falta de localización
                                                let error_lower = e.to_lowercase();
                                                if error_lower.contains("geolocalización") || 
                                                   error_lower.contains("ubicación") || 
                                                   error_lower.contains("localización") ||
                                                   error_lower.contains("location") {
                                                    if let Some(window) = web_sys::window() {
                                                        let _ = window.alert_with_message(
                                                            "⚠️ Debes activar tu localización primero.\n\nPor favor, haz clic en el botón de geolocalización (📍) en el mapa para activar tu ubicación antes de optimizar la ruta."
                                                        );
                                                    }
                                                }
                                                
                                                // Mostrar error en estado
                                                let mut new_state = (*session_state).clone();
                                                new_state.loading = false;
                                                new_state.error = Some(e);
                                                session_state.set(new_state);
                                            }
                                        }
                                    });
                                } else {
                                    log::warn!("⚠️ No hay sesión activa para optimizar");
                                }
                            }
                        })}
                        disabled={loading}
                        title={t("optimiser", &(*language))}
                    >
                        {"🎯"}
                    </button>
                    <button 
                        class="btn-icon-header btn-tracking-search" 
                        onclick={Callback::from({
                            let show_tracking_modal = show_tracking_modal.clone();
                            move |_| {
                                show_tracking_modal.set(true);
                            }
                        })}
                        disabled={loading || session_state.session.is_none()}
                        title="Buscar tracking"
                    >
                        {"🔍"}
                    </button>
                    <button 
                        class="btn-icon-header btn-scanner" 
                        onclick={toggle_scanner.clone()}
                        disabled={loading}
                        title={t("scanner", &(*language))}
                    >
                        {"📷"}
                    </button>
                    <button 
                        class="btn-icon-header btn-refresh" 
                        onclick={Callback::from({
                            let session_state = session_handle.state.clone();
                            let map_handle_refresh = map_handle.update_packages.clone();
                            let filter_mode_refresh = filter_mode.clone();
                            move |_| {
                                let session_state = session_state.clone();
                                let map_update = map_handle_refresh.clone();
                                let filter_mode_for_refresh = filter_mode_refresh.clone();
                                
                                // Obtener sesión actual
                                let current_session = {
                                    let session = (*session_state).clone();
                                    session.session.clone()
                                };
                                
                                if let Some(session) = current_session {
                                    let session_id = session.session_id.clone();
                                    let username = session.driver.driver_id.clone();
                                    let societe = session.driver.company_id.clone();
                                    let vm = SessionViewModel::new();
                                    let sync_service = SyncService::new();
                                    
                                    // Marcar como cargando
                                    let mut new_state = (*session_state).clone();
                                    new_state.loading = true;
                                    new_state.error = None;
                                    session_state.set(new_state);
                                    
                                    wasm_bindgen_futures::spawn_local(async move {
                                        // 1. Primero procesar cambios pendientes
                                        log::info!("🔄 Procesando cambios pendientes antes de refrescar...");
                                        if let Err(e) = sync_service.process_pending_queue().await {
                                            log::warn!("⚠️ Error procesando cambios pendientes: {}", e);
                                        }
                                        
                                        // 2. Luego hacer sync incremental
                                        match vm.sync_incremental(&session_id, &username, &societe, None).await {
                                            Ok(updated_session) => {
                                                log::info!("✅ Sincronización incremental completada: {} paquetes", 
                                                          updated_session.stats.total_packages);
                                                
                                                // Actualizar estado con sesión actualizada
                                                let mut new_state = (*session_state).clone();
                                                new_state.session = Some(updated_session.clone());
                                                new_state.loading = false;
                                                new_state.error = None;
                                                session_state.set(new_state);
                                                
                                                // ⭐ Actualizar mapa explícitamente
                                                use gloo_timers::callback::Timeout;
                                                use crate::viewmodels::map_viewmodel::MapViewModel;
                                                
                                                // Preparar paquetes para el mapa
                                                let mut packages_vec: Vec<_> = updated_session.packages.values().cloned().collect();
                                                if *filter_mode_for_refresh {
                                                    packages_vec.retain(|p| p.status.starts_with("STATUT_CHARGER"));
                                                }
                                                
                                                let groups = group_packages(packages_vec, GroupBy::Address);
                                                let packages_for_map = MapViewModel::prepare_packages_for_map(&groups, &updated_session);
                                                
                                                Timeout::new(300, move || {
                                                    log::info!("🗺️ Actualizando mapa después de refresh: {} paquetes", packages_for_map.len());
                                                    map_update.emit(packages_for_map);
                                                }).forget();
                                            }
                                            Err(e) => {
                                                log::error!("❌ Error en sincronización incremental: {}", e);
                                                
                                                // Mostrar error
                                                let mut new_state = (*session_state).clone();
                                                new_state.loading = false;
                                                new_state.error = Some(e);
                                                session_state.set(new_state);
                                            }
                                        }
                                    });
                                } else {
                                    log::warn!("⚠️ No hay sesión activa para refrescar");
                                }
                            }
                        })}
                        disabled={loading}
                        title={t("rafraichir", &(*language))}
                    >
                        {if loading { "⏳" } else { "🔄" }}
                    </button>
                    <button class="btn-icon-header btn-settings" onclick={toggle_params.clone()}>{"⚙️"}</button>
                </div>
            </header>
            
            <div id="map" class="map-container"></div>
            
            <div id="package-container" class="package-container">
                <div id="backdrop" class={classes!("backdrop", if sheet_state.as_str() != "collapsed" { Some("active") } else { None })} onclick={close_sheet.clone()}></div>
                
                <div id="bottom-sheet" class={
                    let cls = sheet_state.as_str().to_string();
                    classes!("bottom-sheet", cls)
                }>
                    <div class="drag-handle-container" onclick={toggle_sheet_size.clone()}>
                        <div class="drag-handle"></div>
                        {
                            if let Some(ref session) = session_state.session {
                                // ========== CONTADOR DE DIRECCIONES TRATADAS ==========
                                let total_addresses = session.stats.total_addresses;
                                let completed_addresses = session.addresses.values()
                                    .filter(|address| {
                                        // Dirección tratada = TODOS los paquetes están hechos (no CHARGER)
                                        !address.package_ids.is_empty() && address.package_ids.iter().all(|pkg_id| {
                                            session.packages.get(pkg_id)
                                                .map(|pkg| !pkg.status.starts_with("STATUT_CHARGER"))
                                                .unwrap_or(false)
                                        })
                                    })
                                    .count();
                                
                                // ========== CONTADOR DE PAQUETES ==========
                                let total_packages = session.stats.total_packages;
                                
                                // Paquetes entregados (LIVRER)
                                let delivered_packages = session.packages.values()
                                    .filter(|p| p.status.contains("LIVRER"))
                                    .count();
                                
                                // Paquetes fallidos (NONLIV)
                                let failed_packages = session.packages.values()
                                    .filter(|p| p.status.contains("NONLIV"))
                                    .count();
                                
                                // Porcentajes para la barra de progreso
                                let delivered_percent = if total_packages > 0 { 
                                    (delivered_packages * 100) / total_packages 
                                } else { 
                                    0 
                                };
                                
                                let failed_percent = if total_packages > 0 { 
                                    (failed_packages * 100) / total_packages 
                                } else { 
                                    0 
                                };
                                
                                html!{
                                    <>
                                    <div class="progress-info">
                                        <div class="progress-text">
                                                <span class="progress-count">
                                                    {format!("✓ {}/{} {}", completed_addresses, total_addresses, t("traitees", &(*language)))}
                                                </span>
                                            </div>
                                            <div class="progress-packages">
                                                <span class="packages-count">
                                                    {format!("{}/{} {}", delivered_packages, total_packages, t("paquets", &(*language)))}
                                                </span>
                                            </div>
                                        </div>
                                        <div class="progress-bar-container">
                                            // Barra verde (entregados)
                                            <div 
                                                class="progress-bar progress-bar-delivered" 
                                                style={format!("width: {}%", delivered_percent)}
                                            ></div>
                                            // Barra roja (fallidos) - se superpone después de la verde
                                            <div 
                                                class="progress-bar progress-bar-failed" 
                                                style={format!("width: {}%; left: {}%", failed_percent, delivered_percent)}
                                            ></div>
                                    </div>
                                    </>
                                }
                            } else { html!{} }
                        }
                    </div>
                    {
                        if let Some(ref session) = session_state.session {
                            if session.packages.is_empty() {
                                html!{
                                    <div class="no-packages">
                                        <div class="no-packages-icon">{"📦"}</div>
                                        <div class="no-packages-text">{t("aucun_colis", &(*language))}</div>
                                        <div class="no-packages-subtitle">{t("veuillez_rafraichir", &(*language))}</div>
                                    </div>
                                }
                            } else {
                                // Usar los grupos memorizados
                                if let Some(groups) = (*groups_memo).as_ref() {
                                    let addresses: std::collections::HashMap<String, String> = session.addresses.iter().map(|(id, a)| (id.clone(), a.label.clone())).collect();
                                    
                                    // Callback para abrir modal de detalles
                                    let show_details_state = show_details.clone();
                                    let details_package_state = details_package.clone();
                                    let session_packages = session.packages.clone();
                                    let session_addresses = session.addresses.clone();
                                    let on_info = Callback::from(move |tracking: String| {
                                        log::info!("📦 Abriendo detalles para tracking: {}", tracking);
                                        
                                        // Buscar el paquete en la sesión
                                        if let Some(pkg) = session_packages.get(&tracking) {
                                            // Obtener la dirección asociada
                                            if let Some(addr) = session_addresses.get(&pkg.address_id) {
                                                log::info!("✅ Paquete y dirección encontrados");
                                                details_package_state.set(Some((pkg.clone(), addr.clone())));
                                                show_details_state.set(true);
                                            } else {
                                                log::warn!("⚠️ Dirección no encontrada para address_id: {}", pkg.address_id);
                                            }
                                        } else {
                                            log::warn!("⚠️ Paquete no encontrado: {}", tracking);
                                        }
                                    });
                                    
                                    html!{ 
                                        <PackageList 
                                            groups={groups.clone()} 
                                            addresses={addresses} 
                                            on_info={on_info} 
                                            on_package_selected={Some(on_package_selected.clone())}
                                            selected_index={*selected_package_index}
                                        /> 
                                    }
                                } else {
                                    html!{}
                                }
                            }
                        } else { html!{} }
                    }
                </div>
            </div>
            
            {
                if *show_scanner {
                    html! {
                        <Scanner
                            show={true}
                            on_close={Callback::from(move |_| {
                                show_scanner.set(false);
                            })}
                            on_barcode_detected={on_barcode_detected.clone()}
                        />
                    }
                } else {
                    html! {}
                }
            }

            <SettingsPopup 
                active={*show_params} 
                on_close={on_close_settings} 
                on_logout={on_logout} 
                on_retry_map={on_retry}
                language={(*language).clone()}
                on_toggle_language={Some(Callback::from({
                    let language = language.clone();
                    move |new_lang: String| {
                        language.set(new_lang.clone());
                        // Guardar en localStorage
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.set_item("language", &new_lang);
                            }
                        }
                    }
                }))}
                edit_mode={*edit_mode}
                on_toggle_edit_mode={Some(Callback::from({
                    let edit_mode = edit_mode.clone();
                    let edit_origin_idx = edit_origin_idx.clone();
                    move |enabled: bool| {
                        edit_mode.set(enabled);
                        if !enabled {
                            // Si se desactiva el modo edición, limpiar origen
                            edit_origin_idx.set(None);
                        }
                    }
                }))}
                filter_mode={*filter_mode}
                on_toggle_filter={Some(Callback::from({
                    let filter_mode = filter_mode.clone();
                    move |enabled: bool| {
                        filter_mode.set(enabled);
                    }
                }))}
            />
            
            // Modal de detalles
            {
                if *show_details {
                    if let Some((pkg, addr)) = (*details_package).clone() {
                        let session_state_for_modal = session_handle.state.clone();
                        let address_id = addr.address_id.clone();
                        let session_id = if let Some(ref session) = session_state_for_modal.session {
                            session.session_id.clone()
                        } else {
                            String::new()
                        };
                        let username = if let Some(ref session) = session_state_for_modal.session {
                            session.driver.driver_id.clone()
                        } else {
                            String::new()
                        };
                        let societe = if let Some(ref session) = session_state_for_modal.session {
                            session.driver.company_id.clone()
                        } else {
                            String::new()
                        };
                        
                        html! {
                            <DetailsModal
                                package={pkg}
                                address={addr}
                                language={(*language).clone()}
                                on_close={Callback::from({
                                    let show_details = show_details.clone();
                                    move |_| show_details.set(false)
                                })}
                                on_edit_address={Some(Callback::from({
                                    let session_state = session_state_for_modal.clone();
                                    let address_id = address_id.clone();
                                    let session_id = session_id.clone();
                                    let details_package_state = details_package.clone();
                                    move |new_label: String| {
                                        log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                        log::info!("📝 ON_EDIT_ADDRESS CALLBACK EJECUTADO");
                                        log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                        log::info!("   📍 address_id: {}", address_id);
                                        log::info!("   📝 new_label: '{}'", new_label);
                                        
                                        let session_state = session_state.clone();
                                        let address_id = address_id.clone();
                                        let session_id = session_id.clone();
                                        let details_package = details_package_state.clone();
                                        let vm = SessionViewModel::new();
                                        
                                        // Log estado actual de details_package
                                        if let Some((pkg, addr)) = (*details_package).clone() {
                                            log::info!("   📦 details_package actual: tracking={}, address_id={}, label='{}'", 
                                                      pkg.tracking, addr.address_id, addr.label);
                                            log::info!("   📦 is_problematic: {}", pkg.is_problematic);
                                        } else {
                                            log::warn!("   ⚠️ details_package es None");
                                        }
                                        
                                        // Marcar como cargando
                                        let mut new_state = (*session_state).clone();
                                        new_state.loading = true;
                                        new_state.error = None;
                                        session_state.set(new_state);
                                        
                                        wasm_bindgen_futures::spawn_local(async move {
                                            log::info!("   🔄 Llamando a vm.update_address...");
                                            match vm.update_address(&session_id, &address_id, new_label.clone()).await {
                                                Ok(updated_session) => {
                                                    log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                                    log::info!("✅ RESPUESTA DEL BACKEND - Dirección actualizada");
                                                    log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                                    
                                                    // Verificar dirección actualizada en la sesión
                                                    if let Some(updated_addr) = updated_session.addresses.get(&address_id) {
                                                        log::info!("   📍 Dirección actualizada en sesión:");
                                                        log::info!("      - address_id: {}", address_id);
                                                        log::info!("      - label: '{}'", updated_addr.label);
                                                        log::info!("      - lat: {}, lng: {}", 
                                                                  updated_addr.latitude, updated_addr.longitude);
                                                    } else {
                                                        log::error!("   ❌ Dirección NO encontrada en updated_session.addresses!");
                                                    }
                                                    
                                                    // Verificar paquetes problemáticos
                                                    let problematic_packages: Vec<_> = updated_session.packages.iter()
                                                        .filter(|(_, p)| p.is_problematic)
                                                        .collect();
                                                    log::info!("   📦 Paquetes problemáticos en sesión: {}", problematic_packages.len());
                                                    for (tracking, pkg) in problematic_packages.iter() {
                                                        log::info!("      - tracking: {}, is_problematic: {}", tracking, pkg.is_problematic);
                                                    }
                                                    
                                                    // Actualizar estado con sesión actualizada
                                                    let mut new_state = (*session_state).clone();
                                                    new_state.session = Some(updated_session.clone());
                                                    new_state.loading = false;
                                                    new_state.error = None;
                                                    session_state.set(new_state);
                                                    log::info!("   ✅ session_state actualizado");
                                                    
                                                    // Actualizar también details_package inmediatamente si está abierto
                                                    if let Some((pkg, old_addr)) = (*details_package).clone() {
                                                        log::info!("   🔍 Buscando paquete y dirección actualizados en details_package...");
                                                        log::info!("      - tracking buscado: {}", pkg.tracking);
                                                        log::info!("      - address_id buscado: {}", address_id);
                                                        
                                                        if let Some(updated_pkg) = updated_session.packages.get(&pkg.tracking) {
                                                            log::info!("      ✅ Paquete encontrado: tracking={}, is_problematic={}", 
                                                                      updated_pkg.tracking, updated_pkg.is_problematic);
                                                            
                                                            if let Some(updated_addr) = updated_session.addresses.get(&address_id) {
                                                                log::info!("      ✅ Dirección encontrada: address_id={}, label='{}', lat={}, lng={}", 
                                                                          address_id, updated_addr.label, 
                                                                          updated_addr.latitude, updated_addr.longitude);
                                                                
                                                                details_package.set(Some((updated_pkg.clone(), updated_addr.clone())));
                                                                log::info!("   ✅✅✅ details_package ACTUALIZADO INMEDIATAMENTE");
                                                                log::info!("      - Nuevo is_problematic: {}", updated_pkg.is_problematic);
                                                                log::info!("      - Nuevo label: '{}'", updated_addr.label);
                                                            } else {
                                                                log::error!("      ❌ Dirección NO encontrada en updated_session.addresses!");
                                                                log::info!("      🔍 Addresses disponibles: {:?}", 
                                                                          updated_session.addresses.keys().collect::<Vec<_>>());
                                                            }
                                                        } else {
                                                            log::error!("      ❌ Paquete NO encontrado en updated_session.packages!");
                                                            log::info!("      🔍 Tracking buscado: {}", pkg.tracking);
                                                        }
                                                    } else {
                                                        log::warn!("   ⚠️ details_package es None, no se puede actualizar");
                                                    }
                                                    
                                                    log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                                }
                                                Err(e) => {
                                                    log::error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                                    log::error!("❌ ERROR ACTUALIZANDO DIRECCIÓN");
                                                    log::error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                                    log::error!("   Error: {}", e);
                                                    
                                                    // Mostrar error
                                                    let mut new_state = (*session_state).clone();
                                                    new_state.loading = false;
                                                    new_state.error = Some(e);
                                                    session_state.set(new_state);
                                                }
                                            }
                                        });
                                    }
                                }))}
                                on_edit_door_code={Some(Callback::from({
                                    let session_state = session_state_for_modal.clone();
                                    let address_id = address_id.clone();
                                    let session_id = session_id.clone();
                                    let details_package_state = details_package.clone();
                                    move |new_code: String| {
                                        let session_state = session_state.clone();
                                        let address_id = address_id.clone();
                                        let session_id = session_id.clone();
                                        let details_package = details_package_state.clone();
                                        let vm = SessionViewModel::new();
                                        
                                        // Marcar como cargando
                                        let mut new_state = (*session_state).clone();
                                        new_state.loading = true;
                                        new_state.error = None;
                                        session_state.set(new_state);
                                        
                                        wasm_bindgen_futures::spawn_local(async move {
                                            // Enviar Some("") para borrar campo, el backend lo convertirá a None
                                            let door_code = Some(new_code.trim().to_string());
                                            
                                            match vm.update_address_fields(&session_id, &address_id, door_code, None, None).await {
                                                Ok(updated_session) => {
                                                    log::info!("✅ Código de puerta actualizado exitosamente");
                                                    
                                                    // Actualizar estado con sesión actualizada
                                                    let mut new_state = (*session_state).clone();
                                                    new_state.session = Some(updated_session.clone());
                                                    new_state.loading = false;
                                                    new_state.error = None;
                                                    session_state.set(new_state);
                                                    
                                                    // ⭐ Actualizar details_package inmediatamente si está abierto
                                                    if let Some((pkg, old_addr)) = (*details_package).clone() {
                                                        if let Some(updated_addr) = updated_session.addresses.get(&address_id) {
                                                            details_package.set(Some((pkg.clone(), updated_addr.clone())));
                                                            log::info!("✅ details_package actualizado inmediatamente para door_code");
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    log::error!("❌ Error actualizando código de puerta: {}", e);
                                                    
                                                    // Mostrar error
                                                    let mut new_state = (*session_state).clone();
                                                    new_state.loading = false;
                                                    new_state.error = Some(e);
                                                    session_state.set(new_state);
                                                }
                                            }
                                        });
                                    }
                                }))}
                                on_toggle_mailbox={Some(Callback::from({
                                    let session_state = session_state_for_modal.clone();
                                    let address_id = address_id.clone();
                                    let session_id = session_id.clone();
                                    move |new_value: bool| {
                                        let session_state = session_state.clone();
                                        let address_id = address_id.clone();
                                        let session_id = session_id.clone();
                                        let vm = SessionViewModel::new();
                                        
                                        // Marcar como cargando
                                        let mut new_state = (*session_state).clone();
                                        new_state.loading = true;
                                        new_state.error = None;
                                        session_state.set(new_state);
                                        
                                        wasm_bindgen_futures::spawn_local(async move {
                                            match vm.update_address_fields(&session_id, &address_id, None, Some(new_value), None).await {
                                                Ok(updated_session) => {
                                                    log::info!("✅ Acceso BAL actualizado exitosamente: {}", new_value);
                                                    
                                                    // Actualizar estado con sesión actualizada
                                                    let mut new_state = (*session_state).clone();
                                                    new_state.session = Some(updated_session);
                                                    new_state.loading = false;
                                                    new_state.error = None;
                                                    session_state.set(new_state);
                                                }
                                                Err(e) => {
                                                    log::error!("❌ Error actualizando acceso BAL: {}", e);
                                                    
                                                    // Mostrar error
                                                    let mut new_state = (*session_state).clone();
                                                    new_state.loading = false;
                                                    new_state.error = Some(e);
                                                    session_state.set(new_state);
                                                }
                                            }
                                        });
                                    }
                                }))}
                                on_edit_driver_notes={Some(Callback::from({
                                    let session_state = session_state_for_modal.clone();
                                    let address_id = address_id.clone();
                                    let session_id = session_id.clone();
                                    let details_package_state = details_package.clone();
                                    move |new_notes: String| {
                                        let session_state = session_state.clone();
                                        let address_id = address_id.clone();
                                        let session_id = session_id.clone();
                                        let details_package = details_package_state.clone();
                                        let vm = SessionViewModel::new();
                                        
                                        // Marcar como cargando
                                        let mut new_state = (*session_state).clone();
                                        new_state.loading = true;
                                        new_state.error = None;
                                        session_state.set(new_state);
                                        
                                        wasm_bindgen_futures::spawn_local(async move {
                                            // Enviar Some("") para borrar campo, el backend lo convertirá a None
                                            let driver_notes = Some(new_notes.trim().to_string());
                                            
                                            match vm.update_address_fields(&session_id, &address_id, None, None, driver_notes).await {
                                                Ok(updated_session) => {
                                                    log::info!("✅ Notas chofer actualizadas exitosamente");
                                                    
                                                    // Actualizar estado con sesión actualizada
                                                    let mut new_state = (*session_state).clone();
                                                    new_state.session = Some(updated_session.clone());
                                                    new_state.loading = false;
                                                    new_state.error = None;
                                                    session_state.set(new_state);
                                                    
                                                    // ⭐ Actualizar details_package inmediatamente si está abierto
                                                    if let Some((pkg, old_addr)) = (*details_package).clone() {
                                                        if let Some(updated_addr) = updated_session.addresses.get(&address_id) {
                                                            details_package.set(Some((pkg.clone(), updated_addr.clone())));
                                                            log::info!("✅ details_package actualizado inmediatamente para driver_notes");
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    log::error!("❌ Error actualizando notas chofer: {}", e);
                                                    
                                                    // Mostrar error
                                                    let mut new_state = (*session_state).clone();
                                                    new_state.loading = false;
                                                    new_state.error = Some(e);
                                                    session_state.set(new_state);
                                                }
                                            }
                                        });
                                    }
                                }))}
                                on_mark_problematic={Some(Callback::from({
                                    let session_state = session_state_for_modal.clone();
                                    let address_id = address_id.clone();
                                    let session_id = session_id.clone();
                                    let details_package_state = details_package.clone();
                                    move |_| {
                                        log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                        log::info!("⚠️ MARCAR COMO PROBLEMÁTICO");
                                        log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                        log::info!("   📍 address_id: {}", address_id);
                                        
                                        let session_state = session_state.clone();
                                        let address_id = address_id.clone();
                                        let session_id = session_id.clone();
                                        let details_package = details_package_state.clone();
                                        let vm = SessionViewModel::new();
                                        
                                        // Marcar como cargando
                                        let mut new_state = (*session_state).clone();
                                        new_state.loading = true;
                                        new_state.error = None;
                                        session_state.set(new_state);
                                        
                                        wasm_bindgen_futures::spawn_local(async move {
                                            log::info!("   🔄 Llamando a vm.mark_as_problematic...");
                                            match vm.mark_as_problematic(&session_id, &address_id).await {
                                                Ok(updated_session) => {
                                                    log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                                    log::info!("✅ Dirección marcada como problemática exitosamente");
                                                    log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                                    
                                                    // Actualizar estado con sesión actualizada
                                                    let mut new_state = (*session_state).clone();
                                                    new_state.session = Some(updated_session.clone());
                                                    new_state.loading = false;
                                                    new_state.error = None;
                                                    session_state.set(new_state);
                                                    
                                                    // Actualizar details_package
                                                    if let Some((pkg, addr)) = (*details_package).clone() {
                                                        if let Some(updated_pkg) = updated_session.packages.get(&pkg.tracking) {
                                                            if let Some(updated_addr) = updated_session.addresses.get(&address_id) {
                                                                details_package.set(Some((updated_pkg.clone(), updated_addr.clone())));
                                                                log::info!("   ✅ details_package actualizado");
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    log::error!("❌ Error marcando como problemática: {}", e);
                                                    let mut new_state = (*session_state).clone();
                                                    new_state.loading = false;
                                                    new_state.error = Some(e);
                                                    session_state.set(new_state);
                                                }
                                            }
                                        });
                                    }
                                }))}
                            />
                        }
                    } else {
                        html! {}
                    }
                } else {
                    html! {}
                }
            }
            
            <SyncIndicator />
            
            // Modal de búsqueda de trackings
            {
                html! {
                    <div class={modal_class} onclick={Callback::from({
                        let show_tracking_modal = show_tracking_modal_for_modal.clone();
                        move |_| { if *show_tracking_modal { show_tracking_modal.set(false); } }
                    })}>
                        <div class="company-modal-content" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                            <div class="company-modal-header">
                                <h3>{"Buscar Tracking"}</h3>
                                <button type="button" class="btn-close" onclick={Callback::from({
                                    let show_tracking_modal = show_tracking_modal_for_modal.clone();
                                    move |_| { show_tracking_modal.set(false); }
                                })}>{"✕"}</button>
                            </div>
                            <div class="company-search">
                                <input
                                    type="text"
                                    id="tracking-search"
                                    placeholder="Buscar tracking..."
                                    value={(*tracking_query_for_modal).clone()}
                                    oninput={Callback::from({
                                        let tracking_query = tracking_query_for_modal.clone();
                                        move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            tracking_query.set(input.value());
                                        }
                                    })}
                                />
                            </div>
                            <div class="company-list">
                                {
                                    if trackings.is_empty() {
                                        html!{ <div class="company-loading">{"⏳ No hay trackings disponibles"}</div> }
                                    } else if filtered_trackings.is_empty() {
                                        html!{ <div class="company-empty">{"No se encontraron trackings"}</div> }
                                    } else {
                                        html!{ for tracking_items.iter().map(|(tracking, customer_name, status)| {
                                            let tracking_clone = tracking.clone();
                                            let on_click = Callback::from({
                                                let on_tracking_selected = on_tracking_selected_for_modal.clone();
                                                let show_tracking_modal = show_tracking_modal_for_modal.clone();
                                                move |_| { 
                                                    on_tracking_selected.emit(tracking_clone.clone());
                                                }
                                            });
                                            html!{
                                                <div class="company-item" onclick={on_click}>
                                                    <div>
                                                        <div class="company-name">{tracking}</div>
                                                        {
                                                            if let Some(name) = customer_name {
                                                                html!{ <div class="company-code">{name}</div> }
                                                            } else {
                                                                html!{}
                                                            }
                                                        }
                                                        {
                                                            if let Some(stat) = status {
                                                                html!{ <div class="company-code" style="font-size: 11px; color: #999;">{stat}</div> }
                                                            } else {
                                                                html!{}
                                                            }
                                                        }
                                                    </div>
                                                </div>
                                            }
                                        }) }
                                    }
                                }
                            </div>
                        </div>
                    </div>
                }
            }
        </>
    }
}
