use yew::prelude::*;
use web_sys::{window, Storage};
use gloo_net::http::Request;
use crate::models::{Package, Company, CompaniesResponse, LoginRequest, LoginResponse, LoginData, PackageRequest, PackagesCache, OptimizationRequest, OptimizationResponse};
use super::{PackageList, DetailsModal, BalModal, SettingsPopup, LoginScreen, CompanyModal, RegisterScreen, RegisterData};
use gloo_timers::callback::Timeout;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

const BACKEND_URL: &str = "https://api.delivery.nexuslabs.one";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = initMapbox)]
    fn init_mapbox(container_id: &str, is_dark: bool);
    
    #[wasm_bindgen(js_name = addPackagesToMap)]
    fn add_packages_to_map(packages_json: &str);
    
    #[wasm_bindgen(js_name = updateSelectedPackage)]
    fn update_selected_package(selected_index: i32);
}

#[derive(Clone, Copy, PartialEq)]
pub enum SheetState {
    Collapsed,
    Half,
    Full,
}

#[function_component(App)]
pub fn app() -> Html {
    // Auth state
    let is_logged_in = use_state(|| false);
    let login_data = use_state(|| None::<LoginData>);
    let companies = use_state(|| Vec::<Company>::new());
    let selected_company = use_state(|| None::<Company>);
    let show_company_modal = use_state(|| false);
    let companies_loading = use_state(|| false);
    let show_register = use_state(|| false);
    
    // App state
    let packages = use_state(|| Vec::<Package>::new());
    let packages_loading = use_state(|| false);
    let optimizing = use_state(|| false);
    let selected_index = use_state(|| None::<usize>);
    let sheet_state = use_state(|| SheetState::Collapsed);
    let show_details = use_state(|| false);
    let details_package_index = use_state(|| None::<usize>);
    let show_bal_modal = use_state(|| false);
    let show_settings = use_state(|| false);
    let animations = use_state(|| HashMap::<usize, String>::new());
    let map_initialized = use_state(|| false);
    
    // Load companies on mount
    {
        let companies = companies.clone();
        let companies_loading = companies_loading.clone();
        
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                companies_loading.set(true);
                match load_companies().await {
                    Ok(loaded_companies) => {
                        log::info!("✅ Empresas cargadas: {}", loaded_companies.len());
                        companies.set(loaded_companies);
                    }
                    Err(e) => {
                        log::error!("❌ Error cargando empresas: {}", e);
                    }
                }
                companies_loading.set(false);
            });
            || ()
        });
    }
    
    // Check login status on mount
    {
        let is_logged_in = is_logged_in.clone();
        let login_data = login_data.clone();
        let selected_company = selected_company.clone();
        let packages = packages.clone();
        let map_initialized = map_initialized.clone();
        let packages_loading = packages_loading.clone();
        
        use_effect_with((), move |_| {
            if let Some(storage) = get_local_storage() {
                if let (Ok(Some(saved_login)), Ok(Some(saved_company))) = (
                    storage.get_item("routeOptimizer_loginData"),
                    storage.get_item("routeOptimizer_selectedCompany")
                ) {
                    if let (Ok(login), Ok(company)) = (
                        serde_json::from_str::<LoginData>(&saved_login),
                        serde_json::from_str::<Company>(&saved_company)
                    ) {
                        log::info!("✅ Usuario ya logueado: {}", login.username);
                        login_data.set(Some(login.clone()));
                        selected_company.set(Some(company.clone()));
                        is_logged_in.set(true);
                        
                        // Initialize map after login
                        Timeout::new(100, {
                            let map_initialized = map_initialized.clone();
                            move || {
                                let is_dark = window()
                                    .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
                                    .flatten()
                                    .map(|mq| mq.matches())
                                    .unwrap_or(false);
                                init_mapbox("map", is_dark);
                                map_initialized.set(true);
                            }
                        }).forget();
                        
                        // Fetch packages for already logged in user
                        let packages = packages.clone();
                        let packages_loading = packages_loading.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            packages_loading.set(true);
                            match fetch_packages(&login.username, &company.code, false).await {
                                Ok(fetched_packages) => {
                                    log::info!("📦 Paquetes obtenidos: {}", fetched_packages.len());
                                    packages.set(fetched_packages);
                                }
                                Err(e) => {
                                    log::error!("❌ Error obteniendo paquetes: {}", e);
                                }
                            }
                            packages_loading.set(false);
                        });
                    }
                }
            }
            || ()
        });
    }
    
    // Initialize map when user logs in (only if not already initialized)
    {
        let is_logged_in = *is_logged_in;
        let map_initialized = map_initialized.clone();
        
        use_effect_with(is_logged_in, move |logged_in| {
            if *logged_in && !*map_initialized {
                Timeout::new(100, {
                    let map_initialized = map_initialized.clone();
                    move || {
                        let is_dark = window()
                            .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
                            .flatten()
                            .map(|mq| mq.matches())
                            .unwrap_or(false);
                        init_mapbox("map", is_dark);
                        map_initialized.set(true);
                    }
                }).forget();
            }
            || ()
        });
    }
    
    // Update map when packages change
    {
        let packages = (*packages).clone();
        let map_initialized = *map_initialized;
        
        use_effect_with(packages.clone(), move |pkgs| {
            let pkgs_clone = pkgs.clone();
            
            // Save packages to window immediately (for map load event and other JS functions)
            use wasm_bindgen::JsValue;
            if let Ok(window) = web_sys::window().ok_or("no window") {
                let js_packages = serde_wasm_bindgen::to_value(&pkgs_clone).unwrap_or(JsValue::NULL);
                let _ = js_sys::Reflect::set(
                    &window,
                    &JsValue::from_str("currentPackages"),
                    &js_packages
                );
            }
            
            // If map is initialized, update packages immediately
            if map_initialized {
                Timeout::new(100, move || {
                    let packages_json = serde_json::to_string(&pkgs_clone).unwrap_or_default();
                    add_packages_to_map(&packages_json);
                }).forget();
            }
            
            || ()
        });
    }
    
    // Update selected package on map and center
    {
        let selected_index = *selected_index;
        let map_initialized = *map_initialized;
        
        use_effect_with(selected_index, move |idx| {
            if map_initialized {
                if let Some(index) = *idx {
                    // Update visual selection
                    update_selected_package(index as i32);
                    
                    // Center map on package
                    Timeout::new(100, move || {
                        if let Some(win) = web_sys::window() {
                            use wasm_bindgen::JsCast;
                            let function = js_sys::Function::new_no_args(&format!(
                                "if (window.centerMapOnPackage) window.centerMapOnPackage({});",
                                index
                            ));
                            let _ = function.call0(&win.clone().into());
                            
                            // Scroll to package with flash animation
                            let function2 = js_sys::Function::new_no_args(&format!(
                                "if (window.scrollToSelectedPackage) window.scrollToSelectedPackage({});",
                                index
                            ));
                            Timeout::new(300, move || {
                                let _ = function2.call0(&win.into());
                            }).forget();
                        }
                    }).forget();
                }
            }
            || ()
        });
    }
    
    // Listen for package selection from map
    {
        let selected_index = selected_index.clone();
        let sheet_state = sheet_state.clone();
        
        use_effect_with((), move |_| {
            use wasm_bindgen::prelude::*;
            use wasm_bindgen::JsCast;
            
            let selected_index_clone = selected_index.clone();
            let sheet_state_clone = sheet_state.clone();
            
            let callback = Closure::wrap(Box::new(move |event: JsValue| {
                // Get detail.index from custom event
                if let Ok(detail) = js_sys::Reflect::get(&event, &JsValue::from_str("detail")) {
                    if let Ok(index_val) = js_sys::Reflect::get(&detail, &JsValue::from_str("index")) {
                        if let Some(index) = index_val.as_f64() {
                            let index = index as usize;
                            selected_index_clone.set(Some(index));
                            
                            // Open bottom sheet to half if collapsed
                            if matches!(*sheet_state_clone, SheetState::Collapsed) {
                                sheet_state_clone.set(SheetState::Half);
                                
                                // Activate backdrop
                                if let Some(window) = web_sys::window() {
                                    if let Some(document) = window.document() {
                                        if let Some(backdrop) = document.get_element_by_id("backdrop") {
                                            let _ = backdrop.class_list().add_1("active");
                                        }
                                    }
                                }
                            }
                            
                            // Center map and scroll (using JS functions)
                            let index_for_closure = index;
                            Timeout::new(100, move || {
                                use wasm_bindgen::JsCast;
                                if let Some(window) = web_sys::window() {
                                    let function = js_sys::Function::new_no_args(&format!(
                                        "if (window.centerMapOnPackage) window.centerMapOnPackage({});",
                                        index_for_closure
                                    ));
                                    let _ = function.call0(&window.clone().into());
                                    
                                    let function2 = js_sys::Function::new_no_args(&format!(
                                        "if (window.scrollToSelectedPackage) window.scrollToSelectedPackage({});",
                                        index_for_closure
                                    ));
                                    Timeout::new(300, move || {
                                        let _ = function2.call0(&window.into());
                                    }).forget();
                                }
                            }).forget();
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);
            
            if let Some(window) = web_sys::window() {
                let _ = window.add_event_listener_with_callback(
                    "packageSelected",
                    callback.as_ref().unchecked_ref()
                );
            }
            
            move || {
                callback.forget();
            }
        });
    }
    
    // Show companies modal
    let on_show_companies = {
        let show_company_modal = show_company_modal.clone();
        Callback::from(move |_| {
            show_company_modal.set(true);
        })
    };
    
    // Close companies modal
    let on_close_companies = {
        let show_company_modal = show_company_modal.clone();
        Callback::from(move |_| {
            show_company_modal.set(false);
        })
    };
    
    // Select company
    let on_select_company = {
        let selected_company = selected_company.clone();
        let show_company_modal = show_company_modal.clone();
        
        Callback::from(move |company: Company| {
            log::info!("✅ Empresa seleccionada: {:?}", company);
            selected_company.set(Some(company));
            show_company_modal.set(false);
        })
    };
    
    // Show register screen
    let on_show_register = {
        let show_register = show_register.clone();
        Callback::from(move |_| {
            show_register.set(true);
        })
    };
    
    // Back to login from register
    let on_back_to_login = {
        let show_register = show_register.clone();
        Callback::from(move |_| {
            show_register.set(false);
        })
    };
    
    // Handle registration
    let on_register = {
        Callback::from(move |register_data: RegisterData| {
            log::info!("📝 Registro de empresa: {}", register_data.company_name);
            
            // TODO: Send to backend
            wasm_bindgen_futures::spawn_local(async move {
                match Request::post(&format!("{}/register", BACKEND_URL))
                    .json(&serde_json::json!({
                        "company_name": register_data.company_name,
                        "company_address": register_data.company_address,
                        "company_siret": register_data.company_siret,
                        "admin_full_name": register_data.admin_full_name,
                        "admin_email": register_data.admin_email,
                        "admin_password": register_data.admin_password,
                    }))
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.ok() {
                            log::info!("✅ Registro exitoso");
                            if let Some(win) = window() {
                                let _ = win.alert_with_message("✅ Registro exitoso!\n\nRecibirá un email de confirmación en breve.\n\nNuestro equipo se pondrá en contacto con usted.");
                            }
                        } else {
                            log::error!("❌ Error en registro: {}", response.status());
                            if let Some(win) = window() {
                                let _ = win.alert_with_message("❌ Error en el registro. Por favor, intente nuevamente.");
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("❌ Error en registro: {}", e);
                        if let Some(win) = window() {
                            let _ = win.alert_with_message("❌ Error de conexión. Por favor, intente nuevamente.");
                        }
                    }
                }
            });
        })
    };
    
    // Perform login
    let on_login = {
        let selected_company = selected_company.clone();
        let login_data = login_data.clone();
        let is_logged_in = is_logged_in.clone();
        let packages = packages.clone();
        let packages_loading = packages_loading.clone();
        
        Callback::from(move |(username, password): (String, String)| {
            if let Some(company) = (*selected_company).clone() {
                let username = username.clone();
                let password = password.clone();
                let company_clone = company.clone();
                let login_data = login_data.clone();
                let is_logged_in = is_logged_in.clone();
                let _selected_company = selected_company.clone();
                let packages = packages.clone();
                let packages_loading = packages_loading.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    match perform_login(&username, &password, &company_clone.code).await {
                        Ok(response) => {
                            if !response.success {
                                let error_msg = if let Some(error) = response.error {
                                    error.message.unwrap_or_else(|| "Error de autenticación".to_string())
                                } else if let Some(auth) = response.authentication {
                                    auth.message.unwrap_or_else(|| "Error de autenticación".to_string())
                                } else {
                                    "Error de autenticación".to_string()
                                };
                                
                                log::error!("❌ Login fallido: {}", error_msg);
                                window().unwrap().alert_with_message(&format!("Error: {}", error_msg)).ok();
                                return;
                            }
                            
                            // Get authentication info
                            let auth = response.authentication
                                .ok_or_else(|| "No authentication data".to_string())
                                .unwrap();
                            
                            let token = auth.token
                                .unwrap_or_default();
                            
                            // El username que guardamos es el formato "SOCIETE_MATRICULE"
                            // como en el prototipo
                            let full_username = format!("{}_{}", company_clone.code, username);
                            
                            let data = LoginData {
                                username: full_username.clone(),
                                token: token.clone(),
                                company: company_clone.clone(),
                            };
                            
                            // Save to localStorage
                            if let Some(storage) = get_local_storage() {
                                let _ = storage.set_item(
                                    "routeOptimizer_loginData",
                                    &serde_json::to_string(&data).unwrap_or_default()
                                );
                                let _ = storage.set_item(
                                    "routeOptimizer_selectedCompany",
                                    &serde_json::to_string(&company_clone).unwrap_or_default()
                                );
                            }
                            
                            log::info!("✅ Login exitoso: {}", username);
                            login_data.set(Some(data.clone()));
                            is_logged_in.set(true);
                            
                            // Fetch packages after successful login
                            let packages_loading = packages_loading.clone();
                            packages_loading.set(true);
                            match fetch_packages(&full_username, &company_clone.code, false).await {
                                Ok(fetched_packages) => {
                                    log::info!("📦 Paquetes obtenidos: {}", fetched_packages.len());
                                    packages.set(fetched_packages);
                                }
                                Err(e) => {
                                    log::error!("❌ Error obteniendo paquetes: {}", e);
                                }
                            }
                            packages_loading.set(false);
                        }
                        Err(e) => {
                            log::error!("❌ Error en login: {}", e);
                            window().unwrap().alert_with_message(&format!("Error de login: {}", e)).ok();
                        }
                    }
                });
            } else {
                log::error!("❌ No hay empresa seleccionada");
            }
        })
    };
    
    // Calculate stats
    let total = packages.len();
    let delivered = packages.iter().filter(|p| p.status == "delivered").count();
    let percentage = if total > 0 { (delivered * 100) / total } else { 0 };
    
    // Toggle bottom sheet
    let toggle_sheet = {
        let sheet_state = sheet_state.clone();
        Callback::from(move |_: MouseEvent| {
            let new_state = match *sheet_state {
                SheetState::Collapsed => SheetState::Half,
                SheetState::Half => SheetState::Full,
                SheetState::Full => SheetState::Collapsed,
            };
            sheet_state.set(new_state);
            
            // Update backdrop visibility
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(backdrop) = document.get_element_by_id("backdrop") {
                        match new_state {
                            SheetState::Collapsed => {
                                let _ = backdrop.class_list().remove_1("active");
                            }
                            SheetState::Half | SheetState::Full => {
                                let _ = backdrop.class_list().add_1("active");
                            }
                        }
                    }
                }
            }
        })
    };
    
    // Close on backdrop click
    let close_sheet = {
        let sheet_state = sheet_state.clone();
        Callback::from(move |_: MouseEvent| {
            sheet_state.set(SheetState::Collapsed);
            
            // Remove backdrop active class
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(backdrop) = document.get_element_by_id("backdrop") {
                        let _ = backdrop.class_list().remove_1("active");
                    }
                }
            }
        })
    };
    
    // Select package
    let on_select = {
        let selected_index = selected_index.clone();
        let sheet_state = sheet_state.clone();
        
        Callback::from(move |index: usize| {
            // Set selected index
            selected_index.set(Some(index));
            
            // Open bottom sheet to half if collapsed
            if matches!(*sheet_state, SheetState::Collapsed) {
                sheet_state.set(SheetState::Half);
                
                // Activate backdrop
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(backdrop) = document.get_element_by_id("backdrop") {
                            let _ = backdrop.class_list().add_1("active");
                        }
                    }
                }
            }
        })
    };
    
    // Show details
    let on_show_details = {
        let show_details = show_details.clone();
        let details_package_index = details_package_index.clone();
        Callback::from(move |index: usize| {
            details_package_index.set(Some(index));
            show_details.set(true);
        })
    };
    
    // Navigate
    let on_navigate = Callback::from(move |index: usize| {
        log::info!("🧭 Navigate to package {}", index);
    });
    
    // Reorder
    let on_reorder = {
        let packages = packages.clone();
        let animations = animations.clone();
        let selected_index = selected_index.clone();
        
        Callback::from(move |(index, direction): (usize, String)| {
            let pkgs = (*packages).clone();
            let mut anims = (*animations).clone();
            
            if direction == "up" && index > 0 {
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
            }
        })
    };
    
    // Toggle settings
    let toggle_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_: MouseEvent| {
            show_settings.set(!*show_settings);
        })
    };
    
    // Optimize route
    let on_optimize = {
        let login_data = login_data.clone();
        let selected_company = selected_company.clone();
        let packages = packages.clone();
        let optimizing = optimizing.clone();
        
        Callback::from(move |_: MouseEvent| {
            if let (Some(login), Some(company)) = ((*login_data).clone(), (*selected_company).clone()) {
                let login_clone = login.clone();
                let company_clone = company.clone();
                let packages = packages.clone();
                let optimizing = optimizing.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    optimizing.set(true);
                    log::info!("🎯 Iniciando optimización de ruta...");
                    
                    match optimize_route(&login_clone.username, &company_clone.code).await {
                        Ok(response) => {
                            if response.success {
                                if let Some(data) = response.data {
                                    log::info!("✅ Ruta optimizada: {} paquetes", data.lst_lieu_article.len());
                                    
                                    // Reordenar paquetes según la optimización
                                    let mut current_packages = (*packages).clone();
                                    let mut optimized_packages = Vec::new();
                                    
                                    // Mapear paquetes optimizados
                                    for opt_pkg in data.lst_lieu_article {
                                        // Buscar el paquete en la lista actual por referencia
                                        if let Some(ref_colis) = opt_pkg.reference_colis {
                                            if let Some(found) = current_packages.iter().find(|p| p.id == ref_colis) {
                                                optimized_packages.push(found.clone());
                                            }
                                        }
                                    }
                                    
                                    // Si encontramos paquetes optimizados, actualizar
                                    if !optimized_packages.is_empty() {
                                        packages.set(optimized_packages);
                                        log::info!("📦 Paquetes reordenados según optimización");
                                        
                                        // Mostrar mensaje de éxito
                                        if let Some(window) = web_sys::window() {
                                            let _ = window.alert_with_message("✅ Ruta optimizada exitosamente");
                                        }
                                    }
                                } else {
                                    log::error!("❌ No se recibieron datos de optimización");
                                }
                            } else {
                                let msg = response.message.unwrap_or_else(|| "Error desconocido".to_string());
                                log::error!("❌ Error en optimización: {}", msg);
                                if let Some(window) = web_sys::window() {
                                    let _ = window.alert_with_message(&format!("Error: {}", msg));
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("❌ Error llamando API de optimización: {}", e);
                            if let Some(window) = web_sys::window() {
                                let _ = window.alert_with_message(&format!("Error: {}", e));
                            }
                        }
                    }
                    
                    optimizing.set(false);
                });
            }
        })
    };
    
    // Logout
    let on_logout = {
        let is_logged_in = is_logged_in.clone();
        let login_data = login_data.clone();
        let selected_company = selected_company.clone();
        let show_settings = show_settings.clone();
        let packages = packages.clone();
        let packages_loading = packages_loading.clone();
        
        Callback::from(move |_| {
            // Clear packages cache
            if let (Some(storage), Some(login), Some(company)) = (
                get_local_storage(),
                (*login_data).as_ref(),
                (*selected_company).as_ref()
            ) {
                let cache_key = format!("routeOptimizer_packages_{}_{}", company.code, login.username);
                let _ = storage.remove_item(&cache_key);
                log::info!("🗑️ Cache de paquetes eliminado");
            }
            
            // Clear login data from localStorage
            if let Some(storage) = get_local_storage() {
                let _ = storage.remove_item("routeOptimizer_loginData");
                let _ = storage.remove_item("routeOptimizer_selectedCompany");
            }
            
            log::info!("👋 Logout");
            login_data.set(None);
            selected_company.set(None);
            is_logged_in.set(false);
            show_settings.set(false);
            packages.set(Vec::new());
            packages_loading.set(false);
        })
    };
    
    // Render login screen or main app
    if !*is_logged_in {
        return html! {
            <>
                if *show_register {
                    <RegisterScreen
                        on_back_to_login={on_back_to_login}
                        on_register={on_register}
                    />
                } else {
                    <>
                        <LoginScreen
                            on_show_companies={on_show_companies}
                            selected_company={(*selected_company).clone()}
                            on_login={on_login}
                            on_show_register={on_show_register}
                        />
                        <CompanyModal
                            show={*show_company_modal}
                            companies={(*companies).clone()}
                            on_close={on_close_companies}
                            on_select={on_select_company}
                            loading={*companies_loading}
                        />
                    </>
                }
            </>
        };
    }
    
    // Main app
    let sheet_class = match *sheet_state {
        SheetState::Collapsed => "bottom-sheet collapsed",
        SheetState::Half => "bottom-sheet half",
        SheetState::Full => "bottom-sheet full",
    };
    
    let sheet_state_str = match *sheet_state {
        SheetState::Collapsed => "collapsed",
        SheetState::Half => "half",
        SheetState::Full => "full",
    };
    
    html! {
        <>
            <header class="app-header">
                <h1>{"Route Optimizer"}</h1>
                <div class="header-actions">
                    <button 
                        class="btn-optimize" 
                        onclick={on_optimize}
                        disabled={*optimizing}
                    >
                        {if *optimizing { "⏳ Optimizando..." } else { "🎯 Optimizar" }}
                    </button>
                    <button class="btn-settings" onclick={toggle_settings}>
                        {"⚙️"}
                    </button>
                </div>
            </header>
            
            <div id="map" class="map-container"></div>
            
            <div id="package-container" class="package-container">
                <div id="backdrop" class="backdrop" onclick={close_sheet}></div>
                
                <div id="bottom-sheet" class={sheet_class}>
                    <PackageList
                        packages={(*packages).clone()}
                        selected_index={*selected_index}
                        total={total}
                        delivered={delivered}
                        percentage={percentage}
                        sheet_state={sheet_state_str}
                        on_toggle={toggle_sheet}
                        on_select={on_select}
                        on_show_details={on_show_details}
                        on_navigate={on_navigate}
                        on_reorder={on_reorder}
                        animations={(*animations).clone()}
                        loading={*packages_loading}
                    />
                </div>
            </div>
            
            {
                if *show_details {
                    if let Some(idx) = *details_package_index {
                        if let Some(pkg) = packages.get(idx) {
                            html! {
                                <DetailsModal
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
                        />
                    }
                } else {
                    html! {}
                }
            }
        </>
    }
}

// Helper functions
fn get_local_storage() -> Option<Storage> {
    window()?.local_storage().ok()?
}

async fn load_companies() -> Result<Vec<Company>, String> {
    let url = format!("{}/colis-prive/companies", BACKEND_URL);
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    let companies_response = response
        .json::<CompaniesResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    Ok(companies_response.companies)
}

async fn perform_login(username: &str, password: &str, societe: &str) -> Result<LoginResponse, String> {
    let url = format!("{}/colis-prive/auth", BACKEND_URL);
    let request_body = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
        societe: societe.to_string(),
    };
    
    let response = Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    response
        .json::<LoginResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

async fn optimize_route(username: &str, societe: &str) -> Result<OptimizationResponse, String> {
    // Extract just the username part (without SOCIETE prefix)
    // username viene como "PCP0010699_C187518", extraemos "C187518"
    let matricule_only = if let Some(underscore_pos) = username.rfind('_') {
        &username[underscore_pos + 1..]
    } else {
        username
    };
    
    let url = format!("{}/colis-prive/optimize", BACKEND_URL);
    let request_body = OptimizationRequest {
        matricule: matricule_only.to_string(),
        societe: societe.to_string(),
    };
    
    log::info!("🎯 Optimizando ruta para: {} en societe: {}", matricule_only, societe);
    
    let response = Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    response
        .json::<OptimizationResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

async fn fetch_packages(username: &str, societe: &str, force_refresh: bool) -> Result<Vec<Package>, String> {
    log::info!("📦 Obteniendo paquetes de Colis Privé...");
    
    // Extract matricule from username (format: "COMPANY_CODE_MATRICULE")
    let matricule = if let Some(underscore_pos) = username.rfind('_') {
        &username[underscore_pos + 1..]
    } else {
        username
    };
    
    let cache_key = format!("routeOptimizer_packages_{}_{}", societe, username);
    
    // 🎯 Check cache first
    if !force_refresh {
        if let Some(storage) = get_local_storage() {
            if let Ok(Some(cached_data)) = storage.get_item(&cache_key) {
                if let Ok(cache) = serde_json::from_str::<PackagesCache>(&cached_data) {
                    // Check cache age
                    if let Ok(cache_time) = chrono::DateTime::parse_from_rfc3339(&cache.timestamp) {
                        let now = chrono::Utc::now();
                        let cache_age = now.signed_duration_since(cache_time.with_timezone(&chrono::Utc));
                        let cache_age_minutes = cache_age.num_minutes();
                        
                        // Cache valid for 30 minutes
                        if cache_age_minutes < 30 {
                            log::info!("📦 Usando paquetes del cache ({} min de antigüedad)", cache_age_minutes);
                            return Ok(cache.packages);
                        } else {
                            log::info!("📦 Cache expirado, obteniendo datos frescos...");
                        }
                    }
                }
            }
        }
    }
    
    // 🎯 Fetch from API
    let url = format!("{}/colis-prive/packages", BACKEND_URL);
    let request_body = PackageRequest {
        matricule: matricule.to_string(),
        societe: societe.to_string(),
        date: None,
    };
    
    log::info!("📤 Request: matricule={}, societe={}", matricule, societe);
    
    match Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
    {
        Ok(response) => {
            if !response.ok() {
                // Try to use cache as fallback
                if let Some(storage) = get_local_storage() {
                    if let Ok(Some(cached_data)) = storage.get_item(&cache_key) {
                        if let Ok(cache) = serde_json::from_str::<PackagesCache>(&cached_data) {
                            log::info!("📦 Error en API, usando cache como fallback...");
                            return Ok(cache.packages);
                        }
                    }
                }
                return Err(format!("HTTP error: {}", response.status()));
            }
            
            let packages_response = response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| format!("Parse error: {}", e))?;
            
            // Parse packages from response
            if let Some(success) = packages_response.get("success").and_then(|s| s.as_bool()) {
                if success {
                    if let Some(packages_array) = packages_response.get("packages").and_then(|p| p.as_array()) {
                        let packages: Result<Vec<Package>, String> = packages_array
                            .iter()
                            .enumerate()
                            .map(|(index, pkg)| {
                                Ok(Package {
                                    id: pkg.get("tracking_number")
                                        .and_then(|t| t.as_str())
                                        .unwrap_or(&format!("PKG-{}", index + 1))
                                        .to_string(),
                                    recipient: pkg.get("recipient_name")
                                        .and_then(|r| r.as_str())
                                        .unwrap_or("Destinatario desconocido")
                                        .to_string(),
                                    address: pkg.get("formatted_address")
                                        .and_then(|a| a.as_str())
                                        .or_else(|| pkg.get("address").and_then(|a| a.as_str()))
                                        .unwrap_or("Dirección no disponible")
                                        .to_string(),
                                    status: if pkg.get("status")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or("pending") == "delivered" {
                                        "delivered".to_string()
                                    } else {
                                        "pending".to_string()
                                    },
                                    coords: if let (Some(lat), Some(lng)) = (
                                        pkg.get("latitude").and_then(|l| l.as_f64()),
                                        pkg.get("longitude").and_then(|l| l.as_f64())
                                    ) {
                                        Some([lng, lat])
                                    } else {
                                        None
                                    },
                                })
                            })
                            .collect();
                        
                        let packages = packages?;
                        
                        log::info!("✅ Paquetes obtenidos: {} paquetes", packages.len());
                        let with_coords = packages.iter().filter(|p| p.coords.is_some()).count();
                        log::info!("📍 Paquetes con coordenadas: {} / {}", with_coords, packages.len());
                        
                        // 🎯 Save to cache
                        if let Some(storage) = get_local_storage() {
                            let cache = PackagesCache {
                                packages: packages.clone(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            if let Ok(cache_json) = serde_json::to_string(&cache) {
                                let _ = storage.set_item(&cache_key, &cache_json);
                                log::info!("💾 Paquetes guardados en cache");
                            }
                        }
                        
                        return Ok(packages);
                    }
                }
            }
            
            log::info!("⚠️ No hay paquetes disponibles");
            Ok(Vec::new())
        }
        Err(e) => {
            log::error!("❌ Error obteniendo paquetes: {}", e);
            
            // 🎯 Try to use cache as fallback
            if let Some(storage) = get_local_storage() {
                if let Ok(Some(cached_data)) = storage.get_item(&cache_key) {
                    if let Ok(cache) = serde_json::from_str::<PackagesCache>(&cached_data) {
                        log::info!("📦 Error en API, usando cache como fallback...");
                        return Ok(cache.packages);
                    }
                }
            }
            
            Err(format!("Request error: {}", e))
        }
    }
}
