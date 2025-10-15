use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::hooks::RegisterData;

#[derive(Properties, PartialEq)]
pub struct RegisterViewProps {
    pub on_back_to_login: Callback<()>,
    pub on_register: Callback<RegisterData>,
}

#[function_component(RegisterView)]
pub fn register_view(props: &RegisterViewProps) -> Html {
    let company_name_ref = use_node_ref();
    let company_address_ref = use_node_ref();
    let company_siret_ref = use_node_ref();
    let admin_name_ref = use_node_ref();
    let admin_email_ref = use_node_ref();
    let admin_password_ref = use_node_ref();
    let admin_password_confirm_ref = use_node_ref();
    
    let on_submit = {
        let company_name_ref = company_name_ref.clone();
        let company_address_ref = company_address_ref.clone();
        let company_siret_ref = company_siret_ref.clone();
        let admin_name_ref = admin_name_ref.clone();
        let admin_email_ref = admin_email_ref.clone();
        let admin_password_ref = admin_password_ref.clone();
        let admin_password_confirm_ref = admin_password_confirm_ref.clone();
        let on_register = props.on_register.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            if let (
                Some(company_name_input),
                Some(company_address_input),
                Some(company_siret_input),
                Some(admin_name_input),
                Some(admin_email_input),
                Some(admin_password_input),
                Some(admin_password_confirm_input),
            ) = (
                company_name_ref.cast::<HtmlInputElement>(),
                company_address_ref.cast::<HtmlInputElement>(),
                company_siret_ref.cast::<HtmlInputElement>(),
                admin_name_ref.cast::<HtmlInputElement>(),
                admin_email_ref.cast::<HtmlInputElement>(),
                admin_password_ref.cast::<HtmlInputElement>(),
                admin_password_confirm_ref.cast::<HtmlInputElement>(),
            ) {
                let company_name = company_name_input.value();
                let company_address = company_address_input.value();
                let company_siret = company_siret_input.value();
                let admin_full_name = admin_name_input.value();
                let admin_email = admin_email_input.value();
                let admin_password = admin_password_input.value();
                let admin_password_confirm = admin_password_confirm_input.value();
                
                // Validate required fields
                if company_name.is_empty() || company_address.is_empty() || 
                   admin_full_name.is_empty() || admin_email.is_empty() || 
                   admin_password.is_empty() {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message("Por favor, completa todos los campos obligatorios")
                        .ok();
                    return;
                }
                
                // Validate password match
                if admin_password != admin_password_confirm {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message("Las contraseñas no coinciden")
                        .ok();
                    return;
                }
                
                // Validate email format
                if !admin_email.contains('@') {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message("Email inválido")
                        .ok();
                    return;
                }
                
                // Validate SIRET if provided (14 digits)
                if !company_siret.is_empty() && (company_siret.len() != 14 || !company_siret.chars().all(char::is_numeric)) {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message("SIRET debe tener 14 dígitos")
                        .ok();
                    return;
                }
                
                on_register.emit(RegisterData {
                    company_name,
                    company_address,
                    company_siret,
                    admin_full_name,
                    admin_email,
                    admin_password,
                });
            }
        })
    };
    
    html! {
        <div class="login-screen">
            <div class="login-container register-container">
                <div class="login-header">
                    <button 
                        class="btn-back"
                        onclick={props.on_back_to_login.reform(|_| ())}
                    >
                        {"← Retour"}
                    </button>
                    <div class="login-logo">
                        <div class="logo-icon">{"📦"}</div>
                    </div>
                    <h1>{"Inscription Entreprise"}</h1>
                    <p>{"Commencez votre essai gratuit"}</p>
                </div>
                
                <form class="login-form register-form" onsubmit={on_submit}>
                    <div class="form-section">
                        <h3 class="section-title">{"Informations de l'entreprise"}</h3>
                        
                        <div class="form-group">
                            <label for="company-name">{"Nom de l'entreprise"}<span class="required">{"*"}</span></label>
                            <input
                                type="text"
                                id="company-name"
                                name="company-name"
                                placeholder="Ex: Transport Express Paris"
                                ref={company_name_ref}
                                required=true
                            />
                        </div>
                        
                        <div class="form-group">
                            <label for="company-address">{"Adresse"}<span class="required">{"*"}</span></label>
                            <input
                                type="text"
                                id="company-address"
                                name="company-address"
                                placeholder="Ex: 123 Rue de la Paix, 75001 Paris"
                                ref={company_address_ref}
                                required=true
                            />
                        </div>
                        
                        <div class="form-group">
                            <label for="company-siret">{"SIRET (optionnel)"}</label>
                            <input
                                type="text"
                                id="company-siret"
                                name="company-siret"
                                placeholder="Ex: 12345678901234"
                                ref={company_siret_ref}
                                maxlength="14"
                            />
                        </div>
                    </div>
                    
                    <div class="form-section">
                        <h3 class="section-title">{"Administrateur du compte"}</h3>
                        
                        <div class="form-group">
                            <label for="admin-name">{"Nom complet"}<span class="required">{"*"}</span></label>
                            <input
                                type="text"
                                id="admin-name"
                                name="admin-name"
                                placeholder="Ex: Jean Dupont"
                                ref={admin_name_ref}
                                required=true
                            />
                        </div>
                        
                        <div class="form-group">
                            <label for="admin-email">{"Email"}<span class="required">{"*"}</span></label>
                            <input
                                type="email"
                                id="admin-email"
                                name="admin-email"
                                placeholder="Ex: jean.dupont@entreprise.fr"
                                ref={admin_email_ref}
                                required=true
                            />
                        </div>
                        
                        <div class="form-group">
                            <label for="admin-password">{"Mot de passe"}<span class="required">{"*"}</span></label>
                            <input
                                type="password"
                                id="admin-password"
                                name="admin-password"
                                placeholder="Minimum 8 caractères"
                                ref={admin_password_ref}
                                required=true
                                minlength="8"
                            />
                        </div>
                        
                        <div class="form-group">
                            <label for="admin-password-confirm">{"Confirmer mot de passe"}<span class="required">{"*"}</span></label>
                            <input
                                type="password"
                                id="admin-password-confirm"
                                name="admin-password-confirm"
                                placeholder="Répéter le mot de passe"
                                ref={admin_password_confirm_ref}
                                required=true
                                minlength="8"
                            />
                        </div>
                    </div>
                    
                    <button type="submit" class="btn-login">
                        <span class="btn-text">{"Créer mon compte"}</span>
                    </button>
                    
                    <div class="login-footer">
                        <p class="register-info">
                            {"En vous inscrivant, vous acceptez nos "}
                            <a href="#" class="link">{"Conditions d'utilisation"}</a>
                            {" et notre "}
                            <a href="#" class="link">{"Politique de confidentialité"}</a>
                        </p>
                    </div>
                </form>
            </div>
        </div>
    }
}

