// ============================================================================
// LOGIN VIEW - Convertida a Rust puro
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlInputElement, console};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;
use crate::dom::{ElementBuilder, create_element, set_class_name, set_text_content, append_child, set_attribute, set_inner_html, on_click, on_input, remove_class, add_class};
use crate::state::app_state::AppState;
use crate::models::company::Company;
use crate::services::api_client::ApiClient;
use crate::viewmodels::SessionViewModel;
use std::cell::RefCell;
use std::rc::Rc;

/// Renderizar vista de login
pub fn render_login(state: &AppState) -> Result<Element, JsValue> {
    console::log_1(&JsValue::from_str("🎬 [LOGIN] render_login() llamado - INICIO"));
    log::info!("🎬 [LOGIN] render_login() llamado");
    
    // Estado local del formulario (en closures)
    let username = Rc::new(RefCell::new(String::new()));
    let password = Rc::new(RefCell::new(String::new()));
    let societe = Rc::new(RefCell::new(String::new()));
    let error = Rc::new(RefCell::new(None::<String>));
    let loading = Rc::new(RefCell::new(false));
    let companies = Rc::new(RefCell::new(Vec::<Company>::new()));
    let show_company_modal = Rc::new(RefCell::new(false));
    let company_query = Rc::new(RefCell::new(String::new()));
    
    // Cargar empresas al inicializar
    {
        let companies_clone = companies.clone();
        let societe_clone = societe.clone();
        
        console::log_1(&JsValue::from_str("🔍 [LOGIN] Iniciando carga de empresas..."));
        
        spawn_local(async move {
            console::log_1(&JsValue::from_str("🌐 [LOGIN] Llamando a API get_companies..."));
            
                let api = ApiClient::new();
                match api.get_companies().await {
                    Ok(list) => {
                    let msg = format!("✅ [LOGIN] Empresas recibidas del API: {} empresas", list.len());
                    console::log_1(&JsValue::from_str(&msg));
                    
                        let default_code = list.get(0).map(|c| c.code.clone()).unwrap_or_default();
                    *companies_clone.borrow_mut() = list;
                    
                    let count = companies_clone.borrow().len();
                    let msg2 = format!("💾 [LOGIN] Empresas guardadas en estado. Total: {}", count);
                    console::log_1(&JsValue::from_str(&msg2));
                    
                        if !default_code.is_empty() {
                        let default_code_clone = default_code.clone();
                        *societe_clone.borrow_mut() = default_code;
                        let msg3 = format!("🏢 [LOGIN] Código por defecto: {}", default_code_clone);
                        console::log_1(&JsValue::from_str(&msg3));
                        }
                    }
                    Err(e) => {
                    let msg = format!("❌ [LOGIN] Error: {}", e);
                    console::error_1(&JsValue::from_str(&msg));
                    }
                }
        });
    }
    
    // Container principal
    let login_screen = ElementBuilder::new("div")?
        .class("login-screen")
        .build();
    
    let login_container = ElementBuilder::new("div")?
        .class("login-container")
        .build();
    
    // Header
    let login_header = ElementBuilder::new("div")?
        .class("login-header")
        .build();
    
    let logo = ElementBuilder::new("div")?
        .class("login-logo")
        .build();
    
    let logo_icon = ElementBuilder::new("div")?
        .class("logo-icon")
        .text("📦")
        .build();
    
    append_child(&logo, &logo_icon)?;
    
    let title = ElementBuilder::new("h1")?
        .text("Route Optimizer")
        .build();
    
    let subtitle = ElementBuilder::new("p")?
        .text("Optimisation de Routes de Livraison")
        .build();
    
    append_child(&login_header, &logo)?;
    append_child(&login_header, &title)?;
    append_child(&login_header, &subtitle)?;
    
    // Formulario
    let form = create_element("form")?;
    set_class_name(&form, "login-form");
    
    // Input username
    let username_group = create_form_group(
        "username",
        "Utilisateur",
        "Entrez votre nom d'utilisateur",
        username.clone(),
        loading.clone(),
    )?;
    
    // Input password
    let password_group = create_password_group(
        "password",
        "Mot de passe",
        "Entrez votre mot de passe",
        password.clone(),
        loading.clone(),
    )?;
    
    // Company selector
    let company_group = create_company_selector(
        societe.clone(),
        companies.clone(),
        show_company_modal.clone(),
        company_query.clone(),
        loading.clone(),
    )?;
    
    // Submit button
    let submit_btn = ElementBuilder::new("button")?
        .attr("type", "submit")?
        .class("btn-login")
        .build();
    
    let btn_text = ElementBuilder::new("span")?
        .class("btn-text")
        .text("Se connecter")
        .build();
    
    append_child(&submit_btn, &btn_text)?;
    
    // Event listener para submit
    {
        let username_clone = username.clone();
        let password_clone = password.clone();
        let societe_clone = societe.clone();
        let error_clone = error.clone();
        let loading_clone = loading.clone();
        let state_clone = state.clone();
        
        let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
            e.prevent_default();
            
            let username_val = username_clone.borrow().clone();
            let password_val = password_clone.borrow().clone();
            let societe_val = societe_clone.borrow().clone();
            
            if username_val.is_empty() || password_val.is_empty() || societe_val.is_empty() {
                *error_clone.borrow_mut() = Some("Veuillez remplir tous les champs".to_string());
                return;
            }
            
            *loading_clone.borrow_mut() = true;
            *error_clone.borrow_mut() = None;
            
            let state_clone = state_clone.clone();
            let loading_clone = loading_clone.clone();
            let error_clone = error_clone.clone();
            
            spawn_local(async move {
                let vm = SessionViewModel::new();
                
                log::info!("🔐 [LOGIN] Iniciando login...");
                
                match vm.login_smart(username_val.clone(), password_val.clone(), societe_val.clone()).await {
                    Ok(session) => {
                        let msg = format!("✅ [LOGIN] Login exitoso, sesión creada con {} paquetes", session.stats.total_packages);
                        console::log_1(&JsValue::from_str(&msg));
                        log::info!("{}", msg);
                        
                                // Guardar sesión en storage
                                use crate::services::OfflineService;
                                let offline_service = OfflineService::new();
                                if let Err(e) = offline_service.save_session(&session) {
                                    log::error!("❌ [LOGIN] Error guardando sesión en storage: {}", e);
                                } else {
                                    log::info!("✅ [LOGIN] Sesión guardada en storage");
                                }
                        
                        // Actualizar estado
                        state_clone.session.set_session(Some(session));
                        state_clone.session.set_loading(false);
                        state_clone.auth.set_logged_in(true);
                        state_clone.auth.set_username(Some(username_val));
                        state_clone.auth.set_company_id(Some(societe_val));
                        
                                // Invalidar grupos memo para recalcular
                                state_clone.invalidate_groups_memo();
                        
                        console::log_1(&JsValue::from_str("🔄 [LOGIN] Estado actualizado, disparando evento loggedIn..."));
                        
                        // Notificar a la app para re-renderizar
                        if let Some(win) = web_sys::window() {
                            // Disparar evento personalizado (el listener en main() lo capturará)
                            if let Ok(event) = web_sys::Event::new("loggedIn") {
                                if win.dispatch_event(&event).is_ok() {
                                    console::log_1(&JsValue::from_str("✅ [LOGIN] Evento loggedIn disparado exitosamente"));
                                } else {
                                    console::warn_1(&JsValue::from_str("⚠️ [LOGIN] Error disparando evento loggedIn"));
                                }
                            }
                        }
                        
                        *loading_clone.borrow_mut() = false;
                    }
                    Err(e) => {
                        log::error!("❌ Error en login: {}", e);
                        *error_clone.borrow_mut() = Some(format!("Error: {}", e));
                        *loading_clone.borrow_mut() = false;
                    }
                }
            });
        }) as Box<dyn FnMut(web_sys::Event)>);
        
        form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    // Ensamblar formulario
    append_child(&form, &username_group)?;
    append_child(&form, &password_group)?;
    append_child(&form, &company_group)?;
    append_child(&form, &submit_btn)?;
    
    // Ensamblar container
    append_child(&login_container, &login_header)?;
    append_child(&login_container, &form)?;
    append_child(&login_screen, &login_container)?;
    
    // Modal de empresas (siempre renderizado, controlado por CSS)
    let company_modal = create_company_modal(
        societe.clone(),
        companies.clone(),
        show_company_modal.clone(),
        company_query.clone(),
    )?;
    append_child(&login_screen, &company_modal)?;
    
    
    Ok(login_screen)
}

/// Helper para crear form group
fn create_form_group(
    id: &str,
    label_text: &str,
    placeholder: &str,
    value: Rc<RefCell<String>>,
    _loading: Rc<RefCell<bool>>,
) -> Result<Element, JsValue> {
    
    let group = ElementBuilder::new("div")?
        .class("form-group")
        .build();
    
    let label = ElementBuilder::new("label")?
        .attr("for", id)?
        .text(label_text)
        .build();
    
    let input = create_element("input")?;
    set_attribute(&input, "type", "text")?;
    set_attribute(&input, "id", id)?;
    set_attribute(&input, "name", id)?;
    set_attribute(&input, "placeholder", placeholder)?;
    set_class_name(&input, "form-input");
    
    // Event listener para input
    {
        let value_clone = value.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::InputEvent| {
            if let Some(target) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                *value_clone.borrow_mut() = target.value();
            }
        }) as Box<dyn FnMut(web_sys::InputEvent)>);
        
        input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&group, &label)?;
    append_child(&group, &input)?;
    
    Ok(group)
}

/// Helper para crear password group
fn create_password_group(
    id: &str,
    label_text: &str,
    placeholder: &str,
    value: Rc<RefCell<String>>,
    _loading: Rc<RefCell<bool>>,
) -> Result<Element, JsValue> {
    let group = ElementBuilder::new("div")?
        .class("form-group")
        .build();
    
    let label = ElementBuilder::new("label")?
        .attr("for", id)?
        .text(label_text)
        .build();
    
    let input = create_element("input")?;
    set_attribute(&input, "type", "password")?;
    set_attribute(&input, "id", id)?;
    set_attribute(&input, "name", id)?;
    set_attribute(&input, "placeholder", placeholder)?;
    set_class_name(&input, "form-input");
    
    // Event listener para input
    {
        let value_clone = value.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::InputEvent| {
            if let Some(target) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                *value_clone.borrow_mut() = target.value();
            }
        }) as Box<dyn FnMut(web_sys::InputEvent)>);
        
        input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&group, &label)?;
    append_child(&group, &input)?;
    
    Ok(group)
}

/// Helper para crear company selector
fn create_company_selector(
    societe: Rc<RefCell<String>>,
    companies: Rc<RefCell<Vec<Company>>>,
    show_modal: Rc<RefCell<bool>>,
    company_query: Rc<RefCell<String>>,
    loading: Rc<RefCell<bool>>,
) -> Result<Element, JsValue> {
    let group = ElementBuilder::new("div")?
        .class("form-group")
        .build();
    
    let label = ElementBuilder::new("label")?
        .attr("for", "company")?
        .text("Entreprise")
        .build();
    
    let selector = ElementBuilder::new("button")?
        .attr("type", "button")?
        .class("company-selector")
        .build();
    
    // Texto del botón (span para company-text)
    let company_text_span = ElementBuilder::new("span")?
        .attr("id", "company-text")?
        .build();
    
    // Función helper para actualizar el texto del botón
    let update_button_text = {
        let companies_clone = companies.clone();
        let societe_clone = societe.clone();
        let text_span = company_text_span.clone();
        Rc::new(move || {
            let companies_list = companies_clone.borrow();
            let societe_val = societe_clone.borrow();
            
            let display_text = if let Some(selected) = companies_list.iter().find(|c| c.code == *societe_val) {
                selected.name.clone()
            } else {
                "Sélectionner l'entreprise".to_string()
            };
            
            set_text_content(&text_span, &display_text);
        })
    };
    
    // Actualizar texto inicial
    update_button_text();
    
    append_child(&selector, &company_text_span)?;
    
    // Chevron
    let chevron = ElementBuilder::new("span")?
        .class("chevron")
        .text("▼")
        .build();
    append_child(&selector, &chevron)?;
    
    // Event listener para abrir modal
    {
        let show_modal_clone = show_modal.clone();
        let companies_clone = companies.clone();
        let company_query_clone = company_query.clone();
        let societe_clone = societe.clone();
        
        on_click(&selector, move |_| {
            console::log_1(&JsValue::from_str("🖱️ [LOGIN] Click en selector de empresa - abriendo modal"));
            log::info!("🖱️ [LOGIN] Click en selector de empresa - abriendo modal");
            *show_modal_clone.borrow_mut() = true;
            
            let companies_count = companies_clone.borrow().len();
            let msg = format!("📊 [LOGIN] Empresas en estado: {} empresas", companies_count);
            console::log_1(&JsValue::from_str(&msg));
            log::info!("{}", msg);
            
            // Actualizar clase del modal para mostrarlo
            if let Some(modal) = crate::dom::get_element_by_id("company-modal") {
                console::log_1(&JsValue::from_str("✅ [LOGIN] Modal encontrado, mostrándolo..."));
                log::info!("✅ [LOGIN] Modal encontrado, mostrándolo...");
                let _ = add_class(&modal, "show");
                
                // Re-renderizar lista cuando se abre el modal
                if let Some(list_el) = crate::dom::get_element_by_id("company-list-container") {
                    console::log_1(&JsValue::from_str("🔄 [LOGIN] Re-renderizando lista de empresas..."));
                    log::info!("🔄 [LOGIN] Re-renderizando lista de empresas...");
                    match render_company_list_internal(
                        &list_el,
                        &companies_clone,
                        &company_query_clone,
                        &societe_clone,
                        &show_modal_clone,
                        &modal,
                    ) {
                        Ok(_) => {
                            console::log_1(&JsValue::from_str("✅ [LOGIN] Lista re-renderizada exitosamente"));
                            log::info!("✅ [LOGIN] Lista re-renderizada exitosamente");
                        },
                        Err(e) => {
                            let msg = format!("❌ [LOGIN] Error re-renderizando lista: {:?}", e);
                            console::error_1(&JsValue::from_str(&msg));
                            log::error!("{}", msg);
                        },
                    }
                } else {
                    console::error_1(&JsValue::from_str("❌ [LOGIN] company-list-container no encontrado!"));
                    log::error!("❌ [LOGIN] company-list-container no encontrado!");
                }
            } else {
                console::error_1(&JsValue::from_str("❌ [LOGIN] Modal no encontrado!"));
                log::error!("❌ [LOGIN] Modal no encontrado!");
            }
        })?;
    }
    
    // Deshabilitar si está cargando
    if *loading.borrow() {
        selector.set_attribute("disabled", "true")?;
    }
    
    append_child(&group, &label)?;
    append_child(&group, &selector)?;
    
    Ok(group)
}

/// Helper para crear modal de empresas
fn create_company_modal(
    societe: Rc<RefCell<String>>,
    companies: Rc<RefCell<Vec<Company>>>,
    show_modal: Rc<RefCell<bool>>,
    company_query: Rc<RefCell<String>>,
) -> Result<Element, JsValue> {
    let modal = ElementBuilder::new("div")?
        .attr("id", "company-modal")?
        .class("company-modal")
        .build();
    
    let modal_content = ElementBuilder::new("div")?
        .class("company-modal-content")
        .build();
    
    // Header del modal
    let modal_header = ElementBuilder::new("div")?
        .class("company-modal-header")
        .build();
    
    let header_title = ElementBuilder::new("h3")?
        .text("Seleccionar Empresa")
        .build();
    
    let close_btn = ElementBuilder::new("button")?
        .attr("type", "button")?
        .class("btn-close")
        .text("✕")
        .build();
    
    // Cerrar modal al hacer click en el botón
    {
        let show_modal_clone = show_modal.clone();
        let modal_clone = modal.clone();
        on_click(&close_btn, move |_| {
            *show_modal_clone.borrow_mut() = false;
            let _ = remove_class(&modal_clone, "show");
        })?;
    }
    
    // Cerrar modal al hacer click fuera del contenido
    {
        let show_modal_clone = show_modal.clone();
        let modal_clone = modal.clone();
        on_click(&modal, move |_| {
            if *show_modal_clone.borrow() {
                *show_modal_clone.borrow_mut() = false;
                let _ = remove_class(&modal_clone, "show");
            }
        })?;
    }
    
    // Prevenir que el click dentro del contenido cierre el modal
    {
        let modal_content_clone = modal_content.clone();
        on_click(&modal_content, move |e: web_sys::MouseEvent| {
            e.stop_propagation();
        })?;
    }
    
    append_child(&modal_header, &header_title)?;
    append_child(&modal_header, &close_btn)?;
    
    // Búsqueda
    let company_search = ElementBuilder::new("div")?
        .class("company-search")
        .build();
    
    let search_input = create_element("input")?;
    set_attribute(&search_input, "type", "text")?;
    set_attribute(&search_input, "id", "company-search")?;
    set_attribute(&search_input, "placeholder", "Buscar empresa...")?;
    
    // Lista de empresas (crear antes del event listener)
    let company_list = ElementBuilder::new("div")?
        .attr("id", "company-list-container")?
        .class("company-list")
        .build();
    
    // Event listener para búsqueda - actualiza la lista cuando cambia el texto
    {
        let query_clone = company_query.clone();
        let companies_clone = companies.clone();
        let societe_clone = societe.clone();
        let show_modal_clone = show_modal.clone();
        let modal_clone = modal.clone();
        let list_clone = company_list.clone();
        
        on_input(&search_input, move |e: web_sys::InputEvent| {
            if let Some(target) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                *query_clone.borrow_mut() = target.value();
                
                // Re-renderizar lista filtrada
                let _ = render_company_list_internal(
                    &list_clone,
                    &companies_clone,
                    &query_clone,
                    &societe_clone,
                    &show_modal_clone,
                    &modal_clone,
                );
            }
        })?;
    }
    
    append_child(&company_search, &search_input)?;
    
    // Renderizar lista inicial
    render_company_list_internal(
        &company_list,
        &companies,
        &company_query,
        &societe,
        &show_modal,
        &modal,
    )?;
    
    // Re-renderizar lista cuando cambie la búsqueda o las empresas
    // Nota: Esto requeriría un sistema de observación más complejo.
    // Por ahora, se renderizará cuando se abra el modal.
    
    append_child(&modal_content, &modal_header)?;
    append_child(&modal_content, &company_search)?;
    append_child(&modal_content, &company_list)?;
    
    append_child(&modal, &modal_content)?;
    
    Ok(modal)
}

/// Helper interno para renderizar la lista de empresas en el modal
fn render_company_list_internal(
    list_container: &Element,
    companies: &Rc<RefCell<Vec<Company>>>,
    company_query: &Rc<RefCell<String>>,
    societe: &Rc<RefCell<String>>,
    show_modal: &Rc<RefCell<bool>>,
    modal: &Element,
) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str("🎨 [RENDER] Iniciando renderizado de lista de empresas"));
    log::info!("🎨 [RENDER] Iniciando renderizado de lista de empresas");
    
    // Limpiar lista anterior
    set_inner_html(list_container, "");
    
    let companies_list = companies.borrow();
    let query = company_query.borrow();
    
    let count = companies_list.len();
    let msg = format!("📊 [RENDER] Empresas en estado: {} empresas", count);
    console::log_1(&JsValue::from_str(&msg));
    log::info!("{}", msg);
    
    let query_msg = format!("🔍 [RENDER] Query de búsqueda: '{}'", query);
    console::log_1(&JsValue::from_str(&query_msg));
    log::info!("{}", query_msg);
    
    if companies_list.is_empty() {
        console::warn_1(&JsValue::from_str("⚠️ [RENDER] Lista de empresas vacía - mostrando mensaje de carga"));
        log::warn!("⚠️ [RENDER] Lista de empresas vacía - mostrando mensaje de carga");
        let loading_msg = ElementBuilder::new("div")?
            .class("company-loading")
            .text("⏳ Cargando empresas...")
            .build();
        append_child(list_container, &loading_msg)?;
    } else {
        let render_msg = format!("✅ [RENDER] Renderizando {} empresas", companies_list.len());
        console::log_1(&JsValue::from_str(&render_msg));
        log::info!("{}", render_msg);
        // Filtrar empresas
        let filtered: Vec<Company> = if query.is_empty() {
            log::info!("📋 [RENDER] Sin filtro - mostrando todas las empresas");
            companies_list.clone()
        } else {
            let q = query.to_lowercase();
            let filtered_list: Vec<Company> = companies_list.iter()
                .filter(|c| {
                    c.name.to_lowercase().contains(&q) || c.code.to_lowercase().contains(&q)
                })
                .cloned()
                .collect();
            log::info!("🔍 [RENDER] Filtrado: {} empresas encontradas con query '{}'", filtered_list.len(), q);
            filtered_list
        };
        
        if filtered.is_empty() {
            log::warn!("⚠️ [RENDER] No hay empresas que mostrar después del filtro");
            let empty_msg = ElementBuilder::new("div")?
                .class("company-empty")
                .text("No se encontraron empresas")
                .build();
            append_child(list_container, &empty_msg)?;
        } else {
            log::info!("🎯 [RENDER] Creando {} elementos de empresa", filtered.len());
            for company in filtered.iter() {
                let company_item = ElementBuilder::new("div")?
                    .class("company-item")
                    .build();
                
                let company_name = ElementBuilder::new("div")?
                    .class("company-name")
                    .text(&company.name)
                    .build();
                
                let company_code = ElementBuilder::new("div")?
                    .class("company-code")
                    .text(&company.code)
                    .build();
                
                append_child(&company_item, &company_name)?;
                append_child(&company_item, &company_code)?;
                
                // Event listener para seleccionar empresa
                {
                    let code = company.code.clone();
                    let societe_clone = societe.clone();
                    let show_modal_clone = show_modal.clone();
                    let modal_clone = modal.clone();
                    let companies_clone = companies.clone();
                    
                    on_click(&company_item, move |_| {
                        *societe_clone.borrow_mut() = code.clone();
                        *show_modal_clone.borrow_mut() = false;
                        let _ = remove_class(&modal_clone, "show");
                        
                        // Actualizar texto del botón
                        if let Some(btn_text) = crate::dom::get_element_by_id("company-text") {
                            let companies_list = companies_clone.borrow();
                            if let Some(selected) = companies_list.iter().find(|c| c.code == code) {
                                crate::dom::set_text_content(&btn_text, &selected.name);
                            } else {
                                crate::dom::set_text_content(&btn_text, "Sélectionner l'entreprise");
                            }
                        }
                    })?;
                }
                
                append_child(list_container, &company_item)?;
            }
            log::info!("✅ [RENDER] {} elementos de empresa creados exitosamente", filtered.len());
        }
    }
    
    log::info!("✅ [RENDER] Renderizado completado");
    Ok(())
}
