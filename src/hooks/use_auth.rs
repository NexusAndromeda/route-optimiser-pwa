use yew::prelude::*;
use web_sys::window;
use crate::models::{Company, LoginData, LoginResponse, SavedCredentials, DriverInfo};
use crate::services::{load_companies, perform_login, register_company};
use crate::utils::{save_to_storage, load_from_storage, remove_from_storage, STORAGE_KEY_LOGIN_DATA, STORAGE_KEY_SELECTED_COMPANY, STORAGE_KEY_SAVED_CREDENTIALS};

#[derive(Clone, PartialEq)]
pub struct AuthState {
    pub is_logged_in: bool,
    pub login_data: Option<LoginData>,
    pub selected_company: Option<Company>,
    pub saved_credentials: Option<SavedCredentials>,
    pub companies: Vec<Company>,
    pub companies_loading: bool,
    pub show_company_modal: bool,
    pub show_register: bool,
}

pub struct UseAuthHandle {
    pub state: UseStateHandle<AuthState>,
    pub login: Callback<(String, String)>,
    pub logout: Callback<()>,
    pub select_company: Callback<Company>,
    pub show_companies: Callback<()>,
    pub close_companies: Callback<()>,
    pub show_register: Callback<()>,
    pub back_to_login: Callback<()>,
    pub register: Callback<RegisterData>,
}

#[derive(Clone, PartialEq)]
pub struct RegisterData {
    pub company_name: String,
    pub company_address: String,
    pub company_siret: String,
    pub admin_full_name: String,
    pub admin_email: String,
    pub admin_password: String,
}

#[hook]
pub fn use_auth() -> UseAuthHandle {
    let state = use_state(|| AuthState {
        is_logged_in: false,
        login_data: None,
        selected_company: None,
        saved_credentials: None,
        companies: Vec::new(),
        companies_loading: false,
        show_company_modal: false,
        show_register: false,
    });
    
    // Load companies on mount if not logged in
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            let has_saved_login = load_from_storage::<LoginData>(STORAGE_KEY_LOGIN_DATA).is_some();
            
            if !has_saved_login {
                log::info!("📋 Cargando lista de empresas...");
                let state = state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut current_state = (*state).clone();
                    current_state.companies_loading = true;
                    state.set(current_state);
                    
                    match load_companies().await {
                        Ok(loaded_companies) => {
                            log::info!("✅ Empresas cargadas: {}", loaded_companies.len());
                            let mut current_state = (*state).clone();
                            current_state.companies = loaded_companies;
                            current_state.companies_loading = false;
                            state.set(current_state);
                        }
                        Err(e) => {
                            log::error!("❌ Error cargando empresas: {}", e);
                            let mut current_state = (*state).clone();
                            current_state.companies_loading = false;
                            state.set(current_state);
                        }
                    }
                });
            } else {
                log::info!("ℹ️ Usuario ya logueado, no se cargan empresas");
            }
            || ()
        });
    }
    
    // Check login status and load saved credentials on mount
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            let mut current_state = (*state).clone();
            let mut updated = false;
            
            // Check if user is already logged in
            if let Some(saved_login) = load_from_storage::<LoginData>(STORAGE_KEY_LOGIN_DATA) {
                log::info!("✅ Login data encontrada: {}", saved_login.username);
                
                if let Some(saved_company) = load_from_storage::<Company>(STORAGE_KEY_SELECTED_COMPANY) {
                    current_state.login_data = Some(saved_login);
                    current_state.selected_company = Some(saved_company.clone());
                    current_state.is_logged_in = true;
                    updated = true;
                }
            }
            
            // Load saved credentials (username, password, company) for auto-fill
            if let Some(saved_creds) = load_from_storage::<SavedCredentials>(STORAGE_KEY_SAVED_CREDENTIALS) {
                log::info!("✅ Credenciales guardadas encontradas: {}", saved_creds.username);
                current_state.saved_credentials = Some(saved_creds.clone());
                
                // Si no está logueado, pre-seleccionar la empresa guardada
                if !current_state.is_logged_in {
                    current_state.selected_company = Some(saved_creds.company);
                }
                updated = true;
            }
            
            if updated {
                state.set(current_state);
            }
            
            || ()
        });
    }
    
    // Login callback
    let login = {
        let state = state.clone();
        Callback::from(move |(username, password): (String, String)| {
            let current_state = (*state).clone();
            if let Some(company) = current_state.selected_company {
                let state = state.clone();
                let password_clone = password.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match perform_login(&username, &password, &company.code).await {
                        Ok(response) => {
                            handle_login_response(response, username, password_clone, company, state).await;
                        }
                        Err(e) => {
                            log::error!("❌ Error en login: {}", e);
                            if let Some(win) = window() {
                                let _ = win.alert_with_message(&format!("Error de login: {}", e));
                            }
                        }
                    }
                });
            } else {
                log::error!("❌ No hay empresa seleccionada");
            }
        })
    };
    
    // Logout callback
    let logout = {
        let state = state.clone();
        Callback::from(move |_| {
            // Clear login storage (pero NO las credenciales guardadas)
            let _ = remove_from_storage(STORAGE_KEY_LOGIN_DATA);
            let _ = remove_from_storage(STORAGE_KEY_SELECTED_COMPANY);
            
            log::info!("👋 Logout");
            
            // Mantener credenciales guardadas para próximo login
            let saved_creds = load_from_storage::<SavedCredentials>(STORAGE_KEY_SAVED_CREDENTIALS);
            let saved_company = saved_creds.as_ref().map(|c| c.company.clone());
            
            // Reset state
            let mut new_state = AuthState {
                is_logged_in: false,
                login_data: None,
                selected_company: saved_company,
                saved_credentials: saved_creds,
                companies: Vec::new(),
                companies_loading: false,
                show_company_modal: false,
                show_register: false,
            };
            
            // Reload companies
            let state_clone = state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                log::info!("📋 Recargando lista de empresas...");
                new_state.companies_loading = true;
                state_clone.set(new_state.clone());
                
                match load_companies().await {
                    Ok(loaded_companies) => {
                        log::info!("✅ Empresas recargadas: {}", loaded_companies.len());
                        new_state.companies = loaded_companies;
                        new_state.companies_loading = false;
                        state_clone.set(new_state);
                    }
                    Err(e) => {
                        log::error!("❌ Error recargando empresas: {}", e);
                        new_state.companies_loading = false;
                        state_clone.set(new_state);
                    }
                }
            });
        })
    };
    
    // Select company
    let select_company = {
        let state = state.clone();
        Callback::from(move |company: Company| {
            log::info!("✅ Empresa seleccionada: {:?}", company);
            let mut current_state = (*state).clone();
            current_state.selected_company = Some(company);
            current_state.show_company_modal = false;
            state.set(current_state);
        })
    };
    
    // Show companies modal
    let show_companies = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut current_state = (*state).clone();
            current_state.show_company_modal = true;
            state.set(current_state);
        })
    };
    
    // Close companies modal
    let close_companies = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut current_state = (*state).clone();
            current_state.show_company_modal = false;
            state.set(current_state);
        })
    };
    
    // Show register
    let show_register = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut current_state = (*state).clone();
            current_state.show_register = true;
            state.set(current_state);
        })
    };
    
    // Back to login
    let back_to_login = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut current_state = (*state).clone();
            current_state.show_register = false;
            state.set(current_state);
        })
    };
    
    // Register
    let register = {
        Callback::from(move |data: RegisterData| {
            log::info!("📝 Registro de empresa: {}", data.company_name);
            
            wasm_bindgen_futures::spawn_local(async move {
                let siret = if data.company_siret.is_empty() { None } else { Some(data.company_siret) };
                
                match register_company(
                    data.company_name,
                    data.company_address,
                    siret,
                    data.admin_full_name,
                    data.admin_email,
                    data.admin_password,
                ).await {
                    Ok(_) => {
                        log::info!("✅ Registro exitoso");
                        if let Some(win) = window() {
                            let _ = win.alert_with_message("✅ Registro exitoso!\n\nRecibirá un email de confirmación en breve.\n\nNuestro equipo se pondrá en contacto con usted.");
                        }
                    }
                    Err(e) => {
                        log::error!("❌ Error en registro: {}", e);
                        if let Some(win) = window() {
                            let _ = win.alert_with_message(&format!("❌ Error en el registro:\n{}", e));
                        }
                    }
                }
            });
        })
    };
    
    UseAuthHandle {
        state,
        login,
        logout,
        select_company,
        show_companies,
        close_companies,
        show_register,
        back_to_login,
        register,
    }
}

async fn handle_login_response(
    response: LoginResponse,
    username: String,
    password: String,
    company: Company,
    state: UseStateHandle<AuthState>,
) {
    if !response.success {
        let error_msg = if let Some(error) = response.error {
            error.message.unwrap_or_else(|| "Error de autenticación".to_string())
        } else if let Some(auth) = response.authentication {
            auth.message.unwrap_or_else(|| "Error de autenticación".to_string())
        } else {
            "Error de autenticación".to_string()
        };
        
        log::error!("❌ Login fallido: {}", error_msg);
        if let Some(win) = window() {
            let _ = win.alert_with_message(&format!("Error: {}", error_msg));
        }
        return;
    }
    
    // Get authentication info
    let auth = match response.authentication {
        Some(a) => a,
        None => {
            log::error!("❌ No authentication data");
            return;
        }
    };
    
    let token = auth.token.unwrap_or_default();
    let full_username = format!("{}_{}", company.code, username);
    
    let login_data = LoginData {
        username: full_username.clone(),
        token: token.clone(),
        company: company.clone(),
    };
    
    // Crear DriverInfo para la nueva estructura
    let driver_info = DriverInfo {
        driver_id: username.clone(),
        name: username.clone(), // Por ahora usamos el username como nombre
        company_id: company.code.clone(),
        vehicle_id: Some("DEFAULT_VEHICLE".to_string()), // Hardcoded por ahora
    };
    
    // Save to storage
    let _ = save_to_storage(STORAGE_KEY_LOGIN_DATA, &login_data);
    let _ = save_to_storage(STORAGE_KEY_SELECTED_COMPANY, &company);
    let _ = save_to_storage("driver_info", &driver_info);
    
    // Save credentials for next login (auto-fill)
    let saved_credentials = SavedCredentials {
        username: username.clone(),
        password,
        company: company.clone(),
    };
    let _ = save_to_storage(STORAGE_KEY_SAVED_CREDENTIALS, &saved_credentials);
    log::info!("💾 Credenciales guardadas para próximo login");
    
    log::info!("✅ Login exitoso: {}", username);
    
    let mut current_state = (*state).clone();
    current_state.login_data = Some(login_data);
    current_state.saved_credentials = Some(saved_credentials);
    current_state.is_logged_in = true;
    state.set(current_state);
}

