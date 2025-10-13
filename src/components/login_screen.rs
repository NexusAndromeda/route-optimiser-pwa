use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::models::Company;

#[derive(Properties, PartialEq)]
pub struct LoginScreenProps {
    pub on_show_companies: Callback<()>,
    pub selected_company: Option<Company>,
    pub on_login: Callback<(String, String)>,
    pub on_show_register: Callback<()>,
}

#[function_component(LoginScreen)]
pub fn login_screen(props: &LoginScreenProps) -> Html {
    let username_ref = use_node_ref();
    let password_ref = use_node_ref();
    
    let on_submit = {
        let username_ref = username_ref.clone();
        let password_ref = password_ref.clone();
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
            
            if let (Some(username_input), Some(password_input)) = (
                username_ref.cast::<HtmlInputElement>(),
                password_ref.cast::<HtmlInputElement>()
            ) {
                let username = username_input.value();
                let password = password_input.value();
                
                // Validate fields
                if username.is_empty() || password.is_empty() {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message("Por favor, completa todos los campos")
                        .ok();
                    return;
                }
                
                on_login.emit((username, password));
            }
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
                            ref={username_ref}
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
                            ref={password_ref}
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

