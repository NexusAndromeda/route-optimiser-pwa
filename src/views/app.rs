use yew::prelude::*;
use crate::hooks::{use_auth, use_packages, use_map, use_sheet, use_map_selection_listener, clear_packages_cache, use_auto_sync};
use crate::views::auth::{LoginView, RegisterView, CompanySelector};
use crate::views::packages::{PackageList, PackageDetails};
use crate::views::shared::{SettingsPopup, BalModal};
use crate::context::get_text;
use gloo_timers::callback::Timeout;

#[function_component(App)]
pub fn app() -> Html {
    // Use custom hooks
    let auth = use_auth();
    let packages_hook = use_packages(auth.state.login_data.clone());
    let map = use_map();
    let sheet = use_sheet();
    
    // Detectar actividad del usuario para ajustar frecuencia de sync
    let user_active = use_state(|| false);
    
    // Auto-sincronización
    let _auto_sync = use_auto_sync(
        auth.state.login_data.clone(),
        packages_hook.packages.clone(),
        *user_active
    );
    
    // UI state
    let show_details = use_state(|| false);
    let details_package_index = use_state(|| None::<usize>);
    let show_bal_modal = use_state(|| false);
    let show_settings = use_state(|| false);
    
    // Initialize map when user logs in (only if not already initialized)
    {
        let is_logged_in = auth.state.is_logged_in;
        let map_initialized = map.state.initialized;
        let map_init = map.initialize_map.clone();
        
        use_effect_with(is_logged_in, move |logged_in| {
            if *logged_in && !map_initialized {
                log::info!("🗺️ Preparando mapa...");
                Timeout::new(100, move || {
                    let map_init = map_init.clone();
                    // Delay más largo para asegurar que el contenedor esté listo
                    Timeout::new(500, move || {
                        map_init.emit(());
                    }).forget();
                }).forget();
            }
            || ()
        });
    }
    
    // Update map when packages change - EXACTLY like original
    {
        let packages = (*packages_hook.packages).clone();
        let map_initialized = map.state.initialized;
        let map_update = map.update_packages.clone();
        
        use_effect_with(packages.clone(), move |pkgs| {
            let pkgs_clone = pkgs.clone();
            
            // Filter out problematic packages for the map
            let map_packages: Vec<_> = pkgs_clone.iter()
                .filter(|p| !p.is_problematic)
                .cloned()
                .collect();
            
            // Save packages to window immediately (for map load event and other JS functions)
            use wasm_bindgen::JsValue;
            if let Some(window) = web_sys::window() {
                if let Ok(js_packages) = serde_wasm_bindgen::to_value(&map_packages) {
                    let _ = js_sys::Reflect::set(
                        &window,
                        &JsValue::from_str("currentPackages"),
                        &js_packages
                    );
                }
            }
            
            // If map is initialized, update packages immediately
            if map_initialized {
                Timeout::new(100, move || {
                    map_update.emit(map_packages);
                }).forget();
            }
            
            || ()
        });
    }
    
    // Update selected package on map
    {
        let selected_index = *packages_hook.selected_index;
        let map_initialized = map.state.initialized;
        let map_select = map.select_package.clone();
        
        use_effect_with(selected_index, move |idx| {
            if map_initialized {
                if let Some(index) = *idx {
                    map_select.emit(index);
                }
            }
            || ()
        });
    }
    
    // Listen for package selection from map
    {
        let select_package = packages_hook.select_package.clone();
        let sheet_state = sheet.state.clone();
        let set_half = sheet.set_half.clone();
        
        let on_map_select = Callback::from(move |index: usize| {
            log::info!("🖱️ Paquete seleccionado desde mapa: {}", index);
            select_package.emit(index);
            
            // Open bottom sheet to half if collapsed
            if matches!(*sheet_state, crate::hooks::SheetState::Collapsed) {
                set_half.emit(());
            }
        });
        
        use_map_selection_listener(on_map_select);
    }
    
    // Show details handler
    let on_show_details = {
        let show_details = show_details.clone();
        let details_package_index = details_package_index.clone();
        let user_active = user_active.clone();
        
        Callback::from(move |index: usize| {
            details_package_index.set(Some(index));
            show_details.set(true);
            user_active.set(true);
            
            // Resetear actividad después de 2 minutos
            let user_active = user_active.clone();
            gloo_timers::callback::Timeout::new(120_000, move || {
                user_active.set(false);
            }).forget();
        })
    };
    
    // Navigate handler
    let on_navigate = Callback::from(move |index: usize| {
        log::info!("🧭 Navigate to package {}", index);
    });
    
    // Toggle settings
    let toggle_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_: MouseEvent| {
            show_settings.set(!*show_settings);
        })
    };
    
    // Enhanced logout that clears package cache
    let on_logout = {
        let logout = auth.logout.clone();
        let login_data = auth.state.login_data.clone();
        let selected_company = auth.state.selected_company.clone();
        let show_settings = show_settings.clone();
        let reset_map = map.reset_map.clone();
        
        Callback::from(move |_| {
            // Clear packages cache
            if let (Some(login), Some(company)) = (login_data.as_ref(), selected_company.as_ref()) {
                clear_packages_cache(&company.code, &login.username);
            }
            
            // Reset map state
            reset_map.emit(());
            
            show_settings.set(false);
            logout.emit(());
        })
    };
    
    // Calculate stats
    let total = packages_hook.packages.len();
    let delivered = packages_hook.packages.iter().filter(|p| {
        p.code_statut_article.as_ref()
            .map(|code| code.starts_with("STATUT_LIVRER_"))
            .unwrap_or(false)
    }).count();
    let percentage = if total > 0 { (delivered * 100) / total } else { 0 };
    
    // Render login screen if not logged in
    if !auth.state.is_logged_in {
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
                            on_login={auth.login.clone()}
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
                        class="btn-optimize" 
                        onclick={packages_hook.optimize.clone()}
                        disabled={*packages_hook.optimizing}
                    >
                        {if *packages_hook.optimizing { 
                            format!("⏳ {}...", get_text("loading")) 
                        } else { 
                            format!("🎯 {}", get_text("optimize")) 
                        }}
                    </button>
                    <button 
                        class="btn-refresh" 
                        onclick={packages_hook.refresh.clone()}
                        disabled={*packages_hook.loading}
                        title={get_text("refresh")}
                    >
                        {if *packages_hook.loading { "⏳" } else { "🔄" }}
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
                        packages={(*packages_hook.packages).clone()}
                        selected_index={*packages_hook.selected_index}
                        total={total}
                        delivered={delivered}
                        percentage={percentage}
                        sheet_state={(*sheet.state).to_str()}
                        on_toggle={sheet.toggle.clone()}
                        on_select={packages_hook.select_package.clone()}
                        on_show_details={on_show_details}
                        on_navigate={on_navigate}
                        on_reorder={packages_hook.reorder.clone()}
                        animations={(*packages_hook.animations).clone()}
                        loading={*packages_hook.loading}
                        expanded_groups={(*packages_hook.expanded_groups).clone()}
                        on_toggle_group={Some(packages_hook.toggle_group.clone())}
                        reorder_mode={*packages_hook.reorder_mode}
                        reorder_origin={*packages_hook.reorder_origin}
                    />
                </div>
            </div>
            
            {
                if *show_details {
                    if let Some(idx) = *details_package_index {
                        if let Some(pkg) = packages_hook.packages.get(idx) {
                            html! {
                                <PackageDetails
                                    package={pkg.clone()}
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
                                    on_update_package={packages_hook.update_package.clone()}
                                    on_mark_problematic={packages_hook.mark_problematic.clone()}
                                />
                            }
                        } else {
                            html! {}
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
                            reorder_mode={*packages_hook.reorder_mode}
                            on_toggle_reorder={packages_hook.toggle_reorder_mode.clone()}
                        />
                    }
                } else {
                    html! {}
                }
            }
        </>
    }
}
