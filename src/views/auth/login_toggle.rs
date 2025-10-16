use yew::prelude::*;
use web_sys::HtmlInputElement;

#[derive(PartialEq, Clone)]
pub enum UserType {
    Livreur,
    Admin,
}

impl UserType {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserType::Livreur => "livreur",
            UserType::Admin => "admin",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            UserType::Livreur => "Livreur",
            UserType::Admin => "Admin",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct LoginToggleProps {
    pub on_login: Callback<(String, String, UserType)>,
    pub on_show_register: Callback<()>,
}

#[function_component(LoginToggle)]
pub fn login_toggle(props: &LoginToggleProps) -> Html {
    let username_ref = use_node_ref();
    let password_ref = use_node_ref();
    let user_type = use_state(|| UserType::Livreur);
    
    let on_toggle_user_type = {
        let user_type = user_type.clone();
        Callback::from(move |_| {
            let new_type = match *user_type {
                UserType::Livreur => UserType::Admin,
                UserType::Admin => UserType::Livreur,
            };
            user_type.set(new_type);
        })
    };
    
    let on_submit = {
        let username_ref = username_ref.clone();
        let password_ref = password_ref.clone();
        let on_login = props.on_login.clone();
        let user_type = user_type.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
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
                
                on_login.emit((username, password, (*user_type).clone()));
            }
        })
    };
    
    let current_user_type = (*user_type).clone();
    let is_livreur = matches!(current_user_type, UserType::Livreur);
    
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
                    // Toggle de tipo de usuario
                    <div class="user-type-toggle">
                        <div class="toggle-container">
                            <button
                                type="button"
                                class={classes!(
                                    "toggle-option",
                                    is_livreur.then_some("active")
                                )}
                                onclick={on_toggle_user_type.clone()}
                            >
                                {"Livreur"}
                            </button>
                            <button
                                type="button"
                                class={classes!(
                                    "toggle-option",
                                    (!is_livreur).then_some("active")
                                )}
                                onclick={on_toggle_user_type.clone()}
                            >
                                {"Admin"}
                            </button>
                        </div>
                        <div class="toggle-description">
                            {if is_livreur {
                                "Conectarse como conductor para gestionar paquetes"
                            } else {
                                "Conectarse como administrador para monitorear tournées"
                            }}
                        </div>
                    </div>
                    
                    <div class="form-group">
                        <label for="username">
                            {if is_livreur { "Usuario (A187518)" } else { "Usuario Admin" }}
                        </label>
                        <input
                            type="text"
                            id="username"
                            name="username"
                            placeholder={if is_livreur { "A187518" } else { "admin_inti" }}
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
                    
                    <button type="submit" class="btn-login">
                        <span class="btn-text">
                            {if is_livreur { "Iniciar como Livreur" } else { "Iniciar como Admin" }}
                        </span>
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
