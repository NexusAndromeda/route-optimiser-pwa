use yew::prelude::*;
use crate::hooks::{use_auth, use_delivery_session, use_map, use_sheet, use_map_selection_listener};
use crate::views::auth::{LoginView, RegisterView, CompanySelector};
use crate::views::packages::PackageList;
use crate::components::details_modal::DetailsModal;
use crate::components::scanner::Scanner;
use crate::views::shared::{SettingsPopup, BalModal};
use crate::context::get_text;
use crate::models::LegacyPackage as Package;
use crate::services::DeliverySessionConverter;
use gloo_timers::callback::Timeout;

#[function_component(App)]
pub fn app() -> Html {
    // Use custom hooks
    let auth = use_auth();
    let delivery_session = use_delivery_session();
    let map = use_map();
    let sheet = use_sheet();
    
    // UI state
    let show_details = use_state(|| false);
    let details_package_index = use_state(|| None::<usize>);
    let details_package = use_state(|| None::<Package>);
    let show_bal_modal = use_state(|| false);
    let show_settings = use_state(|| false);
    let show_scanner = use_state(|| false);
    
    // Filter mode state (como en main)
    let filter_mode = use_state(|| false);
    let selected_index = use_state(|| None::<usize>);
    let expanded_groups = use_state(|| Vec::<String>::new());
    
    // Initialize map when delivery session is available (FIX: solo UNA VEZ)
    let map_init_attempted = use_state(|| false);
    
    {
        let map_initialized = map.state.initialized;
        let map_init_attempted = map_init_attempted.clone();
        let map_init = map.initialize_map.clone();
        let session_available = delivery_session.state.session.is_some();
        
        // Monitorear cuando la sesión esté disponible
        use_effect_with(session_available, move |session_ready| {
            // Solo inicializar UNA VEZ si hay sesión y el mapa no está inicializado
            if *session_ready && !*map_init_attempted && !map_initialized {
                log::info!("🗺️ Inicializando mapa UNA VEZ (session available: {})...", session_ready);
                map_init_attempted.set(true);
                
                // Initialize map immediately
                map_init.emit(());
            }
            || ()
        });
    }
    
    // Load optimized route when delivery session is available
    {
        let session_available = delivery_session.state.session.is_some();
        let refresh_session = delivery_session.refresh_session.clone();
        
        use_effect_with(session_available, move |session_ready| {
            if *session_ready {
                log::info!("💾 Refrescando sesión desde servidor...");
                refresh_session.emit(());
            }
            || ()
        });
    }
    
    // Convert DeliverySession to packages and update map (como en main)
    {
        let session = delivery_session.state.session.clone();
        let filter_mode_state = filter_mode.clone();
        let map_initialized = map.state.initialized;
        let map_update = map.update_packages.clone();
        
        use_effect_with((session.clone(), *filter_mode_state), move |(session_opt, filter_enabled)| {
            if let Some(session) = session_opt {
                // Convert DeliverySession to packages
                let packages = DeliverySessionConverter::convert_to_packages(&session);
                
                // Apply filter (como en main)
                let filtered = DeliverySessionConverter::apply_filters(&packages, *filter_enabled);
                
                // Filter packages for map: only those with valid coordinates (not 0,0)
                let packages_for_map: Vec<crate::models::package::Package> = filtered.iter()
                    .filter(|p| {
                        if let Some(coords) = p.coords {
                            // Excluir paquetes con coordenadas [0.0, 0.0] - aparecen en África
                            coords[0] != 0.0 || coords[1] != 0.0
                        } else {
                            false // Si no hay coords, no ir al mapa
                        }
                    })
                    .cloned()
                    .collect();
                
                log::info!("📦 Total: {}, Filtrados: {}, Map: {} (excluyendo problemáticos)", 
                           packages.len(), filtered.len(), packages_for_map.len());
                
                // Save filtered packages to window (for map and other JS functions)
                use wasm_bindgen::JsValue;
                if let Some(window) = web_sys::window() {
                    if let Ok(js_packages) = serde_wasm_bindgen::to_value(&filtered) {
                        let _ = js_sys::Reflect::set(
                            &window,
                            &JsValue::from_str("currentPackages"),
                            &js_packages
                        );
                    }
                }
                
                // If map is initialized, update packages immediately
                if map_initialized {
                    log::info!("📍 Actualizando mapa con {} paquetes (válidos para mapa: {})", filtered.len(), packages_for_map.len());
                    Timeout::new(100, move || {
                        map_update.emit(packages_for_map);
                    }).forget();
                }
            }
            
            || ()
        });
    }
    
    // Open bottom sheet when packages are loaded
    {
        let session_available = delivery_session.state.session.is_some();
        let set_half = sheet.set_half.clone();
        
        use_effect_with(session_available, move |session_ready| {
            if *session_ready {
                log::info!("📱 Abriendo bottom sheet automáticamente...");
                set_half.emit(());
            }
            || ()
        });
    }
    
    // Update selected package on map (como en main)
    {
        let selected_idx = selected_index.clone();
        let map_initialized = map.state.initialized;
        let map_select = map.select_package.clone();
        
        use_effect_with(selected_idx, move |idx_opt| {
            if map_initialized {
                if let Some(idx) = &**idx_opt {
                    log::info!("📍 Seleccionando paquete en mapa: {}", idx);
                    map_select.emit(*idx);
                }
            }
            || ()
        });
    }
    
    // Listen for package selection from map
    let on_map_select = {
        let selected_idx = selected_index.clone();
        let sheet_state = sheet.state.clone();
        let set_half = sheet.set_half.clone();
        
        Callback::from(move |package_index: usize| {
            log::info!("🖱️ Paquete seleccionado desde mapa: {}", package_index);
            // Solo actualizar selected_index, el use_effect_with se encarga del resto
            selected_idx.set(Some(package_index));
            
            // Open bottom sheet to half if collapsed
            if matches!(*sheet_state, crate::hooks::SheetState::Collapsed) {
                set_half.emit(());
            }
        })
    };
    use_map_selection_listener(on_map_select);
    
    // Show details handler (para paquetes individuales por índice)
    let on_show_details = {
        let show_details = show_details.clone();
        let details_package = details_package.clone();
        let delivery_session = delivery_session.state.session.clone();
        let filter_mode_state = filter_mode.clone();
        
        Callback::from(move |index: usize| {
            log::info!("📦 Mostrando detalles del paquete en índice: {}", index);
            
            if let Some(session) = &delivery_session {
                // Convertir DeliverySession a packages
                let packages = DeliverySessionConverter::convert_to_packages(session);
                // Aplicar filtros
                let filtered = DeliverySessionConverter::apply_filters(&packages, *filter_mode_state);
                
                if let Some(package) = filtered.get(index) {
                    log::info!("📦 Paquete encontrado: {} - {}", package.id, package.recipient);
                    details_package.set(Some(package.clone()));
                    show_details.set(true);
                } else {
                    log::warn!("⚠️ Paquete no encontrado en índice: {}", index);
                }
            }
        })
    };
    
    // Show details handler (para paquetes completos)
    let on_show_package_details = {
        let show_details = show_details.clone();
        let details_package = details_package.clone();
        Callback::from(move |package: Package| {
            log::info!("📦 Mostrando detalles del paquete: {}", package.id);
            details_package.set(Some(package));
            show_details.set(true);
        })
    };
    
    // Navigate handler
    let on_navigate = Callback::from(move |address_id: String| {
        log::info!("🧭 Navigate to address {}", address_id);
    });
    
    // Toggle group expansion handler
    let on_toggle_group = {
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
    
    // Toggle settings
    let toggle_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_: MouseEvent| {
            show_settings.set(!*show_settings);
        })
    };
    
    // Toggle scanner
    let toggle_scanner = {
        let show_scanner = show_scanner.clone();
        Callback::from(move |_: MouseEvent| {
            show_scanner.set(!*show_scanner);
        })
    };
    
    // Handle barcode detection
    let on_barcode_detected = {
        let show_scanner = show_scanner.clone();
        Callback::from(move |barcode: String| {
            log::info!("📷 Código detectado: {}", barcode);
            // Aquí puedes agregar lógica para buscar el paquete con ese código
            show_scanner.set(false);
            // Por ahora solo cerramos el scanner
        })
    };
    
    
    // Enhanced logout that clears delivery session
    let on_logout = {
        let logout = auth.logout.clone();
        let clear_session = delivery_session.clear_session.clone();
        let show_settings = show_settings.clone();
        let reset_map = map.reset_map.clone();
        let map_init_attempted = map_init_attempted.clone();
        
        Callback::from(move |_| {
            // Clear delivery session
            clear_session.emit(());
            
            // Reset map state
            reset_map.emit(());
            
            // Reset map init attempted flag to allow re-initialization on next login
            map_init_attempted.set(false);
            log::info!("🔄 map_init_attempted reseteado para permitir reinicialización");
            
            show_settings.set(false);
            logout.emit(());
        })
    };
    
    // Calculate stats from delivery session (como en main)
    let (total, treated, percentage) = if let Some(session) = &delivery_session.state.session {
        let packages = DeliverySessionConverter::convert_to_packages(session);
        DeliverySessionConverter::get_stats(&packages)
    } else {
        (0, 0, 0)
    };
    
    // Get filtered packages for display
    let filtered_packages = if let Some(session) = &delivery_session.state.session {
        let packages = DeliverySessionConverter::convert_to_packages(session);
        DeliverySessionConverter::apply_filters(&packages, *filter_mode)
    } else {
        Vec::new()
    };
    
    // Debug: Log current state
    log::info!("🔍 App render - delivery_session.state.session.is_some(): {}", delivery_session.state.session.is_some());
    log::info!("🔍 App render - auth.state.is_logged_in: {}", auth.state.is_logged_in);
    
    // Render login screen if no delivery session
    if delivery_session.state.session.is_none() {
        return html! {
            <>
                if auth.state.show_register {
                    <RegisterView
                        on_back_to_login={auth.back_to_login.clone()}
                        on_register={auth.register.clone()}
                    />
                } else {
                    <>
                        <LoginView
                            on_show_companies={auth.show_companies.clone()}
                            selected_company={auth.state.selected_company.clone()}
                            saved_credentials={auth.state.saved_credentials.clone()}
                            on_login={Callback::from(move |(username, password, societe): (String, String, String)| {
                                // Usar las credenciales que el usuario ingresó
                                delivery_session.login_and_fetch.emit((username, password, societe));
                            })}
                            on_show_register={auth.show_register.clone()}
                        />
                        <CompanySelector
                            show={auth.state.show_company_modal}
                            companies={auth.state.companies.clone()}
                            on_close={auth.close_companies.clone()}
                            on_select={auth.select_company.clone()}
                            loading={auth.state.companies_loading}
                        />
                    </>
                }
            </>
        };
    }
    
    // Main app
    html! {
        <>
            <header class="app-header">
                <h1>{"Route Optimizer"}</h1>
                <div class="header-actions">
                    <button 
                        class="btn-optimize-mini" 
                        onclick={Callback::from(|_| log::info!("🎯 Optimize route"))}
                        disabled={delivery_session.state.loading}
                        title={get_text("optimize")}
                    >
                        {"🎯"}
                    </button>
                    <button 
                        class="btn-scanner" 
                        onclick={toggle_scanner.clone()}
                        disabled={delivery_session.state.loading}
                        title="Escáner de código de barras"
                    >
                        {"📷"}
                    </button>
                    <button 
                        class="btn-refresh" 
                        onclick={Callback::from(move |_| delivery_session.fetch_packages.emit(()))}
                        disabled={delivery_session.state.loading}
                        title={get_text("refresh")}
                    >
                        {if delivery_session.state.loading { "⏳" } else { "🔄" }}
                    </button>
                    <button class="btn-settings" onclick={toggle_settings}>
                        {"⚙️"}
                    </button>
                </div>
            </header>
            
            <div id="map" class="map-container"></div>
            
            <div id="package-container" class="package-container">
                <div id="backdrop" class="backdrop" onclick={sheet.close.clone()}></div>
                
                <div id="bottom-sheet" class={(*sheet.state).to_class()}>
                    <PackageList
                        packages={filtered_packages}
                        selected_index={*selected_index}
                        total={total}
                        delivered={treated}
                        percentage={percentage}
                        sheet_state={(*sheet.state).to_str()}
                        on_toggle={sheet.toggle.clone()}
                        on_select={Callback::from(move |index: usize| {
                            selected_index.set(Some(index));
                        })}
                        on_show_details={on_show_details.clone()}
                        on_navigate={Callback::from(move |index: usize| {
                            log::info!("🧭 Navegando al paquete: {}", index);
                        })}
                        on_reorder={Callback::from(|_| log::info!("🔄 Reorder"))}
                        animations={std::collections::HashMap::new()}
                        loading={delivery_session.state.loading}
                        expanded_groups={(*expanded_groups).clone()}
                        on_toggle_group={Some(on_toggle_group.clone())}
                        on_show_package_details={Some(on_show_package_details.clone())}
                        reorder_mode={false}
                        reorder_origin={None}
                        on_optimize={Some(Callback::from(|_| log::info!("🎯 Optimize")))}
                    />
                </div>
            </div>
            
            {
                if *show_details {
                    if let Some(pkg) = (*details_package).clone() {
                        html! {
                            <DetailsModal
                                package={pkg}
                                on_close={Callback::from({
                                    let show_details = show_details.clone();
                                    move |_| show_details.set(false)
                                })}
                                on_edit_bal={Callback::from({
                                    let show_bal_modal = show_bal_modal.clone();
                                    let show_details = show_details.clone();
                                    move |_| {
                                        show_details.set(false);
                                        show_bal_modal.set(true);
                                    }
                                })}
                                on_update_package={Callback::from(|_| log::info!("📝 Update package"))}
                                on_mark_problematic={Callback::from(|_| log::info!("⚠️ Mark problematic"))}
                            />
                        }
                    } else {
                        html! {}
                    }
                } else {
                    html! {}
                }
            }
            
            {
                if *show_bal_modal {
                    html! {
                        <BalModal
                            on_close={Callback::from({
                                let show_bal_modal = show_bal_modal.clone();
                                move |_| show_bal_modal.set(false)
                            })}
                            on_select={Callback::from(move |has_access: bool| {
                                log::info!("📬 BAL access: {}", has_access);
                            })}
                        />
                    }
                } else {
                    html! {}
                }
            }
            
            {
                if *show_settings {
                    html! {
                        <SettingsPopup
                            on_close={Callback::from({
                                let show_settings = show_settings.clone();
                                move |_| show_settings.set(false)
                            })}
                            on_logout={on_logout.clone()}
                            reorder_mode={false}
                            on_toggle_reorder={Callback::from(|_| log::info!("🔄 Toggle reorder"))}
                            filter_mode={false}
                            on_toggle_filter={Callback::from(|_| log::info!("🔍 Toggle filter"))}
                        />
                    }
                } else {
                    html! {}
                }
            }
            
            {
                if *show_scanner {
                    html! {
                        <Scanner
                            show={*show_scanner}
                            on_close={Callback::from({
                                let show_scanner = show_scanner.clone();
                                move |_| show_scanner.set(false)
                            })}
                            on_barcode_detected={on_barcode_detected.clone()}
                        />
                    }
                } else {
                    html! {}
                }
            }
        </>
    }
}
