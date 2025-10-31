// ============================================================================
// LOGIN VIEW
// ============================================================================
// ✅ COPIADO DEL ORIGINAL - Preserva UI/UX exacta
// Usa hooks nativos de Yew en lugar de Yewdux
// ============================================================================

use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;
use crate::hooks::{use_session, use_auth};
use crate::models::company::Company;
use crate::services::api_client::ApiClient;

#[function_component(LoginView)]
pub fn login_view() -> Html {
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let societe = use_state(|| String::new());
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    let companies = use_state(|| Vec::<Company>::new());
    let show_company_modal = use_state(|| false);
    let company_query = use_state(|| String::new());

    let session_handle = use_session();
    let auth_handle = use_auth();

    // Cargar empresas al montar
    {
        let companies = companies.clone();
        let societe = societe.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let api = ApiClient::new();
                match api.get_companies().await {
                    Ok(list) => {
                        let default_code = list.get(0).map(|c| c.code.clone()).unwrap_or_default();
                        companies.set(list);
                        if !default_code.is_empty() {
                            societe.set(default_code);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to load companies: {}", e);
                    }
                }
            });
            || ()
        });
    }

    // Cargar al abrir si está vacío
    {
        let companies = companies.clone();
        let show = show_company_modal.clone();
        use_effect_with((*show).clone(), move |open| {
            if *open && companies.len() == 0 {
                let companies = companies.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let api = ApiClient::new();
                    if let Ok(list) = api.get_companies().await {
                        companies.set(list);
                    }
                });
            }
            || ()
        });
    }

    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let societe = societe.clone();
        let error = error.clone();
        let loading = loading.clone();
        let session_state = session_handle.state.clone();
        let auth_state = auth_handle.state.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            let username_val = (*username).clone();
            let password_val = (*password).clone();
            let societe_val = (*societe).clone();
            
            if username_val.is_empty() || password_val.is_empty() || societe_val.is_empty() {
                error.set(Some("Veuillez remplir tous les champs".to_string()));
                return;
            }
            
            loading.set(true);
            error.set(None);
            
            let session_state_clone = session_state.clone();
            let auth_state_clone = auth_state.clone();
            let loading_clone = loading.clone();
            let error_clone = error.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                use crate::viewmodels::SessionViewModel;
                let vm = SessionViewModel::new();
                
                match vm.login_and_fetch(username_val.clone(), password_val.clone(), societe_val.clone()).await {
                    Ok(session) => {
                        log::info!("✅ Login exitoso, sesión creada con {} paquetes", session.stats.total_packages);
                        
                        // Actualizar estado de sesión
                        let mut new_session_state = (*session_state_clone).clone();
                        new_session_state.session = Some(session);
                        new_session_state.loading = false;
                        session_state_clone.set(new_session_state);
                        
                        // Actualizar estado de autenticación
                        let mut new_auth_state = (*auth_state_clone).clone();
                        new_auth_state.is_logged_in = true;
                        new_auth_state.username = Some(username_val);
                        new_auth_state.company_id = Some(societe_val);
                        auth_state_clone.set(new_auth_state);
                        
                        // Notificar a la app (sin refrescar)
                        if let Some(win) = web_sys::window() {
                            if let Ok(event) = web_sys::Event::new("loggedIn") {
                                let _ = win.dispatch_event(&event);
                            }
                        }
                        
                        loading_clone.set(false);
                    }
                    Err(e) => {
                        log::error!("❌ Error en login: {}", e);
                        error_clone.set(Some(format!("Error: {}", e)));
                        loading_clone.set(false);
                    }
                }
            });
        })
    };

    let company_text = {
        if let Some(selected) = (*companies).iter().find(|c| c.code == *societe) {
            selected.name.clone()
        } else {
            "Sélectionner l'entreprise".to_string()
        }
    };

    let filtered: Vec<Company> = {
        let q = (*company_query).to_lowercase();
        if q.is_empty() { (*companies).clone() } else {
            (*companies).iter().filter(|c| {
                c.name.to_lowercase().contains(&q) || c.code.to_lowercase().contains(&q)
            }).cloned().collect()
        }
    };

    // Clases del modal original
    let modal_class = if *show_company_modal { "company-modal show" } else { "company-modal" };

    html! {
        <div class="login-screen">
            <div class="login-container">
                <div class="login-header">
                    <div class="login-logo">
                        <div class="logo-icon">{"📦"}</div>
                    </div>
                    <h1>{"Route Optimizer"}</h1>
                    <p>{"Optimisation de Routes de Livraison"}</p>
                </div>
                
                {
                    if let Some(ref err) = *error {
                        html! {
                            <div class="error-message">
                                <span>{"❌ "}</span>
                                <span>{err}</span>
                            </div>
                        }
                    } else { html!{} }
                }
                
                <form class="login-form" onsubmit={on_submit}>
                    <div class="form-group">
                        <label for="username">{"Utilisateur"}</label>
                        <input
                            type="text"
                            id="username"
                            name="username"
                            placeholder="Entrez votre nom d'utilisateur"
                            value={(*username).clone()}
                            oninput={Callback::from({
                                let username = username.clone();
                                move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    username.set(input.value());
                                }
                            })}
                            required=true
                            disabled={*loading}
                        />
                    </div>
                    
                    <div class="form-group">
                        <label for="password">{"Mot de passe"}</label>
                        <input
                            type="password"
                            id="password"
                            name="password"
                            placeholder="Entrez votre mot de passe"
                            value={(*password).clone()}
                            oninput={Callback::from({
                                let password = password.clone();
                                move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    password.set(input.value());
                                }
                            })}
                            required=true
                            disabled={*loading}
                        />
                    </div>
                    
                    <div class="form-group">
                        <label for="company">{"Entreprise"}</label>
                        <button
                            type="button"
                            class="company-selector"
                            onclick={Callback::from({
                                let show_company_modal = show_company_modal.clone();
                                move |_| { show_company_modal.set(true); }
                            })}
                            disabled={*loading}
                        >
                            <span id="company-text">{company_text}</span>
                            <span class="chevron">{"▼"}</span>
                        </button>
                    </div>
                    
                    <button type="submit" class="btn-login" disabled={*loading}>
                        <span class="btn-text">{ if *loading { "⏳ Connexion..." } else { "Se connecter" } }</span>
                    </button>
                </form>

                // Modal de empresas (estructura/clases del original)
                <div class={modal_class} onclick={Callback::from({
                    let show_company_modal = show_company_modal.clone();
                    move |_| { if *show_company_modal { show_company_modal.set(false); } }
                })}>
                    <div class="company-modal-content" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                        <div class="company-modal-header">
                            <h3>{"Seleccionar Empresa"}</h3>
                            <button type="button" class="btn-close" onclick={Callback::from({
                                let show_company_modal = show_company_modal.clone();
                                move |_| { show_company_modal.set(false); }
                            })}>{"✕"}</button>
                        </div>
                        <div class="company-search">
                            <input
                                type="text"
                                id="company-search"
                                placeholder="Buscar empresa..."
                                value={(*company_query).clone()}
                                oninput={Callback::from({
                                    let company_query = company_query.clone();
                                    move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        company_query.set(input.value());
                                    }
                                })}
                            />
                        </div>
                        <div class="company-list">
                            {
                                if companies.len() == 0 {
                                    html!{ <div class="company-loading">{"⏳ Cargando empresas..."}</div> }
                                } else if filtered.len() == 0 {
                                    html!{ <div class="company-empty">{"No se encontraron empresas"}</div> }
                                } else {
                                    html!{ for filtered.iter().map(|c| {
                                        let code = c.code.clone();
                                        let name = c.name.clone();
                                        let on_click = Callback::from({
                                            let show_company_modal = show_company_modal.clone();
                                            let societe = societe.clone();
                                            move |_| { societe.set(code.clone()); show_company_modal.set(false); }
                                        });
                                        html!{
                                            <div class="company-item" onclick={on_click}>
                                                <div class="company-name">{name}</div>
                                                <div class="company-code">{&c.code}</div>
                                            </div>
                                        }
                                    }) }
                                }
                            }
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
