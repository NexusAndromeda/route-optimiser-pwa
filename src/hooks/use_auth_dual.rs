use yew::prelude::*;
use web_sys::window;
use crate::models::{Company, LoginData};
use crate::services::{load_companies, register_company};
use crate::utils::{save_to_storage, load_from_storage, remove_from_storage, STORAGE_KEY_LOGIN_DATA, STORAGE_KEY_SELECTED_COMPANY};
use crate::views::auth::UserType;

#[derive(Clone, PartialEq)]
pub struct AuthState {
    pub is_logged_in: bool,
    pub login_data: Option<LoginData>,
    pub selected_company: Option<Company>,
    pub companies: Vec<Company>,
    pub companies_loading: bool,
    pub show_company_modal: bool,
    pub show_register: bool,
    pub user_type: UserType,
    pub jwt_token: Option<String>,
}

pub struct UseAuthHandle {
    pub state: UseStateHandle<AuthState>,
    pub login: Callback<(String, String, UserType)>,
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

#[derive(serde::Serialize, serde::Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    user_type: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LoginResponseData {
    success: bool,
    data: Option<LoginResponseDataInner>,
    message: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LoginResponseDataInner {
    token: Option<String>,
    user_info: Option<UserInfo>,
    expires_at: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UserInfo {
    id: String,
    username: String,
    role: String,
    company_id: Option<String>,
    tournee_id: Option<String>,
    permissions: Vec<String>,
}

#[hook]
pub fn use_auth_dual() -> UseAuthHandle {
    let state = use_state(|| AuthState {
        is_logged_in: false,
        login_data: None,
        selected_company: None,
        companies: Vec::new(),
        companies_loading: false,
        show_company_modal: false,
        show_register: false,
        user_type: UserType::Livreur,
        jwt_token: None,
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
    
    // Check login status on mount
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            if let Some(saved_login) = load_from_storage::<LoginData>(STORAGE_KEY_LOGIN_DATA) {
                log::info!("✅ Login data encontrada: {}", saved_login.username);
                
                if let Some(saved_company) = load_from_storage::<Company>(STORAGE_KEY_SELECTED_COMPANY) {
                    let mut current_state = (*state).clone();
                    current_state.login_data = Some(saved_login);
                    current_state.selected_company = Some(saved_company);
                    current_state.is_logged_in = true;
                    state.set(current_state);
                }
            }
            || ()
        });
    }
    
    // Login callback
    let login = {
        let state = state.clone();
        Callback::from(move |(username, password, user_type): (String, String, UserType)| {
            let state = state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match perform_dual_login(&username, &password, &user_type).await {
                    Ok((jwt_token, user_info)) => {
                        handle_dual_login_response(jwt_token, user_info, username, user_type, state).await;
                    }
                    Err(e) => {
                        log::error!("❌ Error en login: {}", e);
                        if let Some(win) = window() {
                            let _ = win.alert_with_message(&format!("Error de login: {}", e));
                        }
                    }
                }
            });
        })
    };
    
    // Logout callback
    let logout = {
        let state = state.clone();
        Callback::from(move |_| {
            // Clear storage
            let _ = remove_from_storage(STORAGE_KEY_LOGIN_DATA);
            let _ = remove_from_storage(STORAGE_KEY_SELECTED_COMPANY);
            
            log::info!("👋 Logout");
            
            // Reset state
            let mut new_state = AuthState {
                is_logged_in: false,
                login_data: None,
                selected_company: None,
                companies: Vec::new(),
                companies_loading: false,
                show_company_modal: false,
                show_register: false,
                user_type: UserType::Livreur,
                jwt_token: None,
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

async fn perform_dual_login(
    username: &str,
    password: &str,
    user_type: &UserType,
) -> Result<(String, UserInfo), String> {
    let request = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
        user_type: user_type.as_str().to_string(),
    };
    
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8080/api/auth/login")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Error de conexión: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Error HTTP: {}", response.status()));
    }
    
    let login_response: LoginResponseData = response
        .json()
        .await
        .map_err(|e| format!("Error parseando respuesta: {}", e))?;
    
    if !login_response.success {
        return Err(login_response.message.unwrap_or_else(|| "Error de autenticación".to_string()));
    }
    
    let data = login_response.data.ok_or("No data in response")?;
    let token = data.token.ok_or("No token in response")?;
    let user_info = data.user_info.ok_or("No user info in response")?;
    
    Ok((token, user_info))
}

async fn handle_dual_login_response(
    jwt_token: String,
    user_info: UserInfo,
    username: String,
    user_type: UserType,
    state: UseStateHandle<AuthState>,
) {
    // Para livreurs, necesitamos la empresa seleccionada
    let current_state = (*state).clone();
    let company = if matches!(user_type, UserType::Livreur) {
        current_state.selected_company.clone()
    } else {
        // Para admins, crear una empresa ficticia
        Some(Company {
            code: user_info.company_id.unwrap_or_else(|| "ADMIN".to_string()),
            name: "Administración".to_string(),
            description: Some("Sistema de Administración".to_string()),
        })
    };
    
    if let Some(company) = company {
        let login_data = LoginData {
            username: username.clone(),
            token: jwt_token.clone(),
            company: company.clone(),
        };
        
        // Save to storage
        let _ = save_to_storage(STORAGE_KEY_LOGIN_DATA, &login_data);
        let _ = save_to_storage(STORAGE_KEY_SELECTED_COMPANY, &company);
        
        log::info!("✅ Login exitoso: {} como {}", username, user_type.as_str());
        
        let mut current_state = (*state).clone();
        current_state.login_data = Some(login_data);
        current_state.is_logged_in = true;
        current_state.user_type = user_type;
        current_state.jwt_token = Some(jwt_token);
        state.set(current_state);
    } else {
        log::error!("❌ No hay empresa seleccionada para livreur");
        if let Some(win) = window() {
            let _ = win.alert_with_message("Por favor, selecciona una empresa");
        }
    }
}
