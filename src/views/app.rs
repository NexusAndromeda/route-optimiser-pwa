// ============================================================================
// APP VIEW - COMPONENTE PRINCIPAL
// ============================================================================
// ✅ HTML/CSS EXACTO DEL ORIGINAL preservado
// Usa hooks nativos de Yew en lugar de Yewdux (compatibilidad Rust 1.90)
// ============================================================================

use yew::prelude::*;
use crate::hooks::{use_session, use_sync_state, use_auth, group_packages, GroupBy, use_map, use_map_selection_listener};
use crate::components::{SyncIndicator, Scanner, DraggablePackageList, SettingsPopup, PackageList};
use crate::views::login::LoginView;
use crate::viewmodels::{SessionViewModel, MapViewModel};
use wasm_bindgen::{JsCast, JsValue};
use js_sys::Reflect;
use wasm_bindgen::closure::Closure;
use gloo_timers::callback::Timeout;

#[function_component(App)]
pub fn app() -> Html {
    let session_handle = use_session();
    let sync_handle = use_sync_state();
    let auth_handle = use_auth();
    let map_handle = use_map();
    
    let session_state = session_handle.state.clone();
    let auth_state = auth_handle.state.clone();
    
    let is_logged_in = auth_state.is_logged_in;
    let loading = session_state.loading;
    
    // Cargar sesión al iniciar (localStorage)
    {
        let session_state = session_handle.state.clone();
        let auth_state = auth_handle.state.clone();
        use_effect_with((), move |_| {
            let vm = SessionViewModel::new();
            let session_state = session_state.clone();
            let auth_state = auth_state.clone();

            // Listener de 'loggedIn'
            let win = web_sys::window().unwrap();
            let session_for_event = session_state.clone();
            let auth_for_event = auth_state.clone();
            let on_logged = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let vm = SessionViewModel::new();
                let session_state_in = session_for_event.clone();
                let auth_state_in = auth_for_event.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(Some(session)) = vm.load_session_from_storage().await {
                        let mut new_session_state = (*session_state_in).clone();
                        new_session_state.session = Some(session);
                        session_state_in.set(new_session_state);
                        let mut new_auth_state = (*auth_state_in).clone();
                        new_auth_state.is_logged_in = true;
                        auth_state_in.set(new_auth_state);
                    }
                });
            }) as Box<dyn FnMut(_)>);
            win.add_event_listener_with_callback("loggedIn", on_logged.as_ref().unchecked_ref()).ok();

            // Carga inicial
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(Some(session)) = vm.load_session_from_storage().await {
                    log::info!("📋 Sesión cargada desde storage: {} paquetes", session.stats.total_packages);
                    let mut new_session_state = (*session_state).clone();
                    new_session_state.session = Some(session);
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
    
    let on_barcode_detected = {
        let scan_package = session_handle.scan_package.clone();
        let show_scanner = show_scanner.clone();
        Callback::from(move |tracking: String| {
            scan_package.emit(tracking);
            show_scanner.set(false);
        })
    };

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
    let groups_memo = use_memo(
        session_state.session.clone(),
        |session_opt| {
            session_opt.as_ref().map(|session| {
                let mut items: Vec<_> = session.packages.values().cloned().collect();
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
    
    // Si no está logueado, mostrar login
    if !is_logged_in {
        return html! {
            <LoginView />
        };
    }
    
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
        let map_initialized = map_handle.state.initialized;
        
        use_effect_with(session_opt, move |session_opt| {
            if let Some(session) = session_opt {
                if map_initialized {
                    log::info!("📦 Sesión actualizada, preparando paquetes para el mapa...");
                    
                    // Convertir HashMap a Vec
                    let packages_vec: Vec<_> = session.packages.values().cloned().collect();
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
        
        use_effect_with(map_initialized, move |initialized| {
            if *initialized {
                if let Some(session) = &session_state_clone.session {
                    log::info!("🗺️ Mapa ahora inicializado, re-enviando paquetes...");
                    
                    // Convertir HashMap a Vec
                    let packages_vec: Vec<_> = session.packages.values().cloned().collect();
                    
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
            
            if package_index >= groups_count {
                log::warn!("⚠️  group_idx {} >= grupos disponibles {}, ignorando", 
                          package_index, groups_count);
                return;
            }
            
            log::info!("   ✅ group_idx válido, actualizando selección...");
            
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
            
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        })
    };
    use_map_selection_listener(on_map_select);
    
    // Callback cuando se selecciona un paquete en el bottom sheet
    let on_package_selected = {
        let center_on_package = map_handle.center_on_package.clone();
        let selected_package_index = selected_package_index.clone();
        
        Callback::from(move |index: usize| {
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("📦 PAQUETE SELECCIONADO EN BOTTOM SHEET");
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            log::info!("   📍 group_idx: {}", index);
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
        })
    };

    html! {
        <>
            <header class="app-header">
                <h1>{"Route Optimizer"}</h1>
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
                                                
                                                // Mostrar error
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
                        title="Optimiser"
                    >
                        {"🎯"}
                    </button>
                    <button 
                        class="btn-icon-header btn-scanner" 
                        onclick={toggle_scanner.clone()}
                        disabled={loading}
                        title="Scanner"
                    >
                        {"📷"}
                    </button>
                    <button 
                        class="btn-icon-header btn-refresh" 
                        onclick={Callback::from({
                            let fetch_packages = session_handle.fetch_packages.clone();
                            move |_| fetch_packages.emit(())
                        })}
                        disabled={loading}
                        title="Rafraîchir"
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
                                                    {format!("✓ {}/{} traitées", completed_addresses, total_addresses)}
                                                </span>
                                            </div>
                                            <div class="progress-packages">
                                                <span class="packages-count">
                                                    {format!("{}/{} paquets", delivered_packages, total_packages)}
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
                                        <div class="no-packages-text">{"Aucun colis dans la session"}</div>
                                        <div class="no-packages-subtitle">{"Veuillez rafraîchir ou recharger la tournée"}</div>
                                    </div>
                                }
                            } else {
                                // Usar los grupos memorizados
                                if let Some(groups) = (*groups_memo).as_ref() {
                                    let addresses: std::collections::HashMap<String, String> = session.addresses.iter().map(|(id, a)| (id.clone(), a.label.clone())).collect();
                                    let on_info = Callback::from(|tracking: String| log::info!("info {}", tracking));
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

            <SettingsPopup active={*show_params} on_close={on_close_settings} on_logout={on_logout} on_retry_map={on_retry} />
            
            <SyncIndicator />
        </>
    }
}
