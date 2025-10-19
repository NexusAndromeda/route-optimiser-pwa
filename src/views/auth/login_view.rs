use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::models::{Company, SavedCredentials};

#[derive(Properties, PartialEq)]
pub struct LoginViewProps {
    pub on_show_companies: Callback<()>,
    pub selected_company: Option<Company>,
    pub saved_credentials: Option<SavedCredentials>,
    pub on_login: Callback<(String, String)>,
    pub on_show_register: Callback<()>,
}

#[function_component(LoginView)]
pub fn login_view(props: &LoginViewProps) -> Html {
    // Estados para los valores de los inputs
    let username = use_state(|| {
        props.saved_credentials.as_ref()
            .map(|c| c.username.clone())
            .unwrap_or_default()
    });
    
    let password = use_state(|| {
        props.saved_credentials.as_ref()
            .map(|c| c.password.clone())
            .unwrap_or_default()
    });
    
    let on_username_change = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };
    
    let on_password_change = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };
    
    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let on_login = props.on_login.clone();
        let selected_company = props.selected_company.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            // Validate company selection
            if selected_company.is_none() {
                web_sys::window()
                    .unwrap()
                    .alert_with_message("Por favor, selecciona una empresa")
                    .ok();
                return;
            }
            
            let username_val = (*username).clone();
            let password_val = (*password).clone();
            
            // Validate fields
            if username_val.is_empty() || password_val.is_empty() {
                web_sys::window()
                    .unwrap()
                    .alert_with_message("Por favor, completa todos los campos")
                    .ok();
                return;
            }
            
            on_login.emit((username_val, password_val));
        })
    };
    
    let company_text = match &props.selected_company {
        Some(company) => company.name.clone(),
        None => "Seleccionar empresa".to_string(),
    };
    
    html! {
        <div class="login-screen">
            <div class="login-container">
                <div class="login-header">
                    <div class="login-logo">
                        <div class="logo-icon">{"📦"}</div>
                    </div>
                    <h1>{"Route Optimizer"}</h1>
                    <p>{"Optimización de Rutas de Entrega"}</p>
                </div>
                
                <form class="login-form" onsubmit={on_submit}>
                    <div class="form-group">
                        <label for="username">{"Usuario"}</label>
                        <input
                            type="text"
                            id="username"
                            name="username"
                            placeholder="Ingresa tu usuario"
                            value={(*username).clone()}
                            oninput={on_username_change}
                            required=true
                        />
                    </div>
                    
                    <div class="form-group">
                        <label for="password">{"Contraseña"}</label>
                        <input
                            type="password"
                            id="password"
                            name="password"
                            placeholder="Ingresa tu contraseña"
                            value={(*password).clone()}
                            oninput={on_password_change}
                            required=true
                        />
                    </div>
                    
                    <div class="form-group">
                        <label for="company">{"Empresa"}</label>
                        <button
                            type="button"
                            class="company-selector"
                            onclick={props.on_show_companies.reform(|_| ())}
                        >
                            <span id="company-text">{company_text}</span>
                            <span class="chevron">{"▼"}</span>
                        </button>
                    </div>
                    
                    <button type="submit" class="btn-login">
                        <span class="btn-text">{"Iniciar Sesión"}</span>
                    </button>
                    
                    <div class="login-footer">
                        <p class="register-text">{"Votre entreprise souhaite utiliser Route Optimizer ?"}</p>
                        <button 
                            type="button" 
                            class="btn-register-link"
                            onclick={props.on_show_register.reform(|_| ())}
                        >
                            {"Contactez-nous pour un essai gratuit"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

