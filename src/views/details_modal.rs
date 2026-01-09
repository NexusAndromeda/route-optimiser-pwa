// ============================================================================
// DETAILS MODAL VIEW - Modal de detalles del paquete (Rust puro)
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::{Element, console};
use wasm_bindgen::closure::Closure;
use std::rc::Rc;
use crate::dom::{ElementBuilder, append_child, set_attribute, add_class, create_element};
use crate::models::package::Package;
use crate::models::address::Address;
use crate::state::app_state::AppState;
use crate::utils::i18n::t;
use crate::models::admin::PackageTraceabilityResponse;

/// Renderizar modal de detalles
pub fn render_details_modal(
    pkg: &Package,
    addr: &Address,
    state: &AppState,
    on_close: Rc<dyn Fn()>,
    on_edit_address: Option<Rc<dyn Fn(String)>>,
    on_edit_door_code: Option<Rc<dyn Fn(String)>>,
    on_edit_mailbox: Option<Rc<dyn Fn(bool)>>,
    on_edit_driver_notes: Option<Rc<dyn Fn(String)>>,
    on_mark_problematic: Option<Rc<dyn Fn()>>,
) -> Result<Element, JsValue> {
    let lang = state.language.borrow().clone();
    
    // Inicializar estados de edición con valores actuales
    state.init_edit_states(addr);
    
    // Modal container - debe tener clase "active" para ser visible (siempre se renderiza cuando show_details es true)
    let modal = ElementBuilder::new("div")?
        .id("details-modal")?
        .class("modal active") // Agregar "active" siempre porque solo se renderiza cuando debe mostrarse
        .build();
    
    // Overlay (cierra al hacer click)
    let overlay = ElementBuilder::new("div")?
        .class("modal-overlay")
        .build();
    
    {
        let on_close_clone = on_close.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            on_close_clone();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        overlay.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&modal, &overlay)?;
    
    // Modal content (previene cierre al click dentro)
    let content = ElementBuilder::new("div")?
        .class("modal-content")
        .build();
    
    {
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        content.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    // Header
    let header = ElementBuilder::new("div")?
        .class("modal-header")
        .build();
    
    let title_text = if lang == "ES" {
        format!("Paquete {}", pkg.tracking)
    } else {
        format!("Colis {}", pkg.tracking)
    };
    
    let title = ElementBuilder::new("h2")?
        .text(&title_text)
        .build();
    
    let close_btn = ElementBuilder::new("button")?
        .class("btn-close")
        .text("✕")
        .build();
    
    {
        let on_close_clone = on_close.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            on_close_clone();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        close_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&header, &title)?;
    append_child(&header, &close_btn)?;
    append_child(&content, &header)?;
    
    // Body
    let body = ElementBuilder::new("div")?
        .class("modal-body")
        .build();
    
    // Mensaje de error (si existe)
    if let Some(error_msg) = state.edit_error_message.borrow().as_ref() {
        let error_div = ElementBuilder::new("div")?
            .class("error-message")
            .build();
        
        set_attribute(&error_div, "style", "color: red; padding: 10px; margin-bottom: 10px; background: #ffe6e6; border-radius: 4px;")?;
        crate::dom::set_text_content(&error_div, error_msg);
        append_child(&body, &error_div)?;
    }
    
    // Destinataire
    let section_dest = create_detail_section(
        &t("destinataire", &lang),
        &pkg.customer_name,
        None,
        false,
    )?;
    append_child(&body, &section_dest)?;
    
    // Adresse (editable)
    let section_addr = create_editable_address_section(
        state,
        addr,
        &lang,
        on_edit_address,
    )?;
    append_child(&body, &section_addr)?;
    
    // Téléphone
    let phone_value = if let Some(phone) = &pkg.phone_number {
        format!(r#"<a href="tel:{}" class="phone-link">{}</a>"#, phone, phone)
    } else {
        format!(r#"<span class="empty-value">{}</span>"#, t("non_renseigne", &lang))
    };
    let section_phone = create_detail_section(
        &t("telephone", &lang),
        &phone_value,
        Some(true), // HTML
        false,
    )?;
    append_child(&body, &section_phone)?;
    
    // Codes de porte (editable)
    let section_door = create_editable_door_code_section(
        state,
        addr,
        &lang,
        on_edit_door_code,
    )?;
    append_child(&body, &section_door)?;
    
    // Accès BAL (toggle editable)
    let has_mailbox = addr.mailbox_access.is_some() && addr.mailbox_access.as_ref().unwrap() == "true";
    log::info!("📬 [MODAL] Renderizando modal - address_id={}, mailbox_access={:?}, has_mailbox={}", 
               addr.address_id, addr.mailbox_access, has_mailbox);
    let section_bal = create_editable_mailbox_section(
        state,
        addr,
        &lang,
        has_mailbox,
        on_edit_mailbox,
    )?;
    append_child(&body, &section_bal)?;
    
    // Indications client (solo lectura)
    let client_indication = pkg.customer_indication.as_ref()
        .map(|s| format!("\"{}\"", s))
        .unwrap_or_else(|| format!("<span class=\"empty-value\">{}</span>", t("non_renseigne", &lang)));
    let section_indications = create_detail_section_with_action(
        &t("indications_client", &lang),
        &client_indication,
        Some(true), // HTML
        false, // solo lectura
    )?;
    append_child(&body, &section_indications)?;
    
    // Notes chauffeur (editable)
    let section_notes = create_editable_driver_notes_section(
        state,
        addr,
        &lang,
        on_edit_driver_notes,
    )?;
    append_child(&body, &section_notes)?;
    
    // Marcar como problemático
    let section_problematic = create_problematic_section(
        &t("marquer_problematique", &lang),
        pkg.is_problematic,
        &lang,
        on_mark_problematic,
    )?;
    append_child(&body, &section_problematic)?;
    
    // Botón para solicitar cambio de status
    let section_status_change = create_status_change_section(state, &pkg.tracking, &lang)?;
    append_child(&body, &section_status_change)?;
    
    // Traçabilité (solo en modo admin)
    if *state.admin_mode.borrow() {
        if let Some(ref traceability) = *state.package_traceability.borrow() {
            let section_traceability = create_traceability_section(traceability, &lang)?;
            append_child(&body, &section_traceability)?;
        } else {
            // Mostrar loading mientras se obtiene la traçabilité
            let loading_section = ElementBuilder::new("div")?
                .class("detail-section")
                .build();
            let loading_label = ElementBuilder::new("div")?
                .class("detail-label")
                .text(&t("traçabilité", &lang))
                .build();
            let loading_value = ElementBuilder::new("div")?
                .class("detail-value")
                .text("Chargement...")
                .build();
            append_child(&loading_section, &loading_label)?;
            append_child(&loading_section, &loading_value)?;
            append_child(&body, &loading_section)?;
        }
    }
    
    append_child(&content, &body)?;
    append_child(&modal, &content)?;
    
    Ok(modal)
}

/// Crear sección de traçabilité
fn create_traceability_section(
    traceability: &crate::models::admin::PackageTraceabilityResponse,
    lang: &str,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("detail-section traceability-section")
        .build();
    
    let label = ElementBuilder::new("div")?
        .class("detail-label")
        .text(&t("traçabilité", lang))
        .build();
    append_child(&section, &label)?;
    
    // Container para las acciones
    let actions_container = ElementBuilder::new("div")?
        .class("traceability-actions")
        .build();
    
    // Ordenar acciones por fecha (más reciente primero)
    let mut actions = traceability.actions.clone();
    actions.sort_by(|a, b| b.date_action.cmp(&a.date_action));
    
    for action in actions {
        let action_item = ElementBuilder::new("div")?
            .class("traceability-action-item")
            .build();
        
        // Fecha y tipo de acción
        let action_header = ElementBuilder::new("div")?
            .class("traceability-action-header")
            .build();
        
        // Formatear fecha (formato ISO 8601: "2026-01-08T11:10:55.5278814")
        let date_str = {
            // Intentar parsear el formato ISO con o sin timezone
            if let Some(t_idx) = action.date_action.find('T') {
                let date_part = &action.date_action[..t_idx];
                let time_part = if let Some(dot_idx) = action.date_action[t_idx+1..].find('.') {
                    &action.date_action[t_idx+1..t_idx+1+dot_idx]
                } else if let Some(z_idx) = action.date_action[t_idx+1..].find('Z') {
                    &action.date_action[t_idx+1..t_idx+1+z_idx]
                } else {
                    &action.date_action[t_idx+1..]
                };
                format!("{} {}", date_part, time_part)
            } else {
                action.date_action.clone()
            }
        };
        
        let date_el = ElementBuilder::new("span")?
            .class("traceability-date")
            .text(&date_str)
            .build();
        
        let type_el = ElementBuilder::new("span")?
            .class("traceability-type")
            .text(&action.type_action)
            .build();
        
        append_child(&action_header, &date_el)?;
        append_child(&action_header, &type_el)?;
        append_child(&action_item, &action_header)?;
        
        // Descripción
        if !action.description.is_empty() {
            let desc_el = ElementBuilder::new("div")?
                .class("traceability-description")
                .text(&action.description)
                .build();
            append_child(&action_item, &desc_el)?;
        }
        
        // Comentario
        if !action.commentaire.is_empty() {
            let comment_el = ElementBuilder::new("div")?
                .class("traceability-comment")
                .text(&action.commentaire)
                .build();
            append_child(&action_item, &comment_el)?;
        }
        
        // Origen de la acción
        if let Some(origine) = &action.origine_action {
            let origine_el = ElementBuilder::new("div")?
                .class("traceability-origin")
                .text(&format!("Origine: {}", origine))
                .build();
            append_child(&action_item, &origine_el)?;
        }
        
        append_child(&actions_container, &action_item)?;
    }
    
    append_child(&section, &actions_container)?;
    
    Ok(section)
}

/// Crear sección de dirección editable
fn create_editable_address_section(
    state: &AppState,
    addr: &Address,
    lang: &str,
    on_edit: Option<Rc<dyn Fn(String)>>,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("detail-section editable")
        .build();
    
    let label_el = ElementBuilder::new("div")?
        .class("detail-label")
        .text(&t("adresse", lang))
        .build();
    
    let value_container = ElementBuilder::new("div")?
        .class("detail-value-with-action")
        .build();
    
    let editing = *state.editing_address.borrow();
    
    if editing {
        // Modo edición: mostrar input
        let input_group = ElementBuilder::new("div")?
            .class("edit-input-group")
            .build();
        
        let input = create_element("input")?;
        set_attribute(&input, "type", "text")?;
        set_attribute(&input, "class", "edit-input")?;
        set_attribute(&input, "placeholder", &t("nouvelle_adresse", lang))?;
        set_attribute(&input, "autofocus", "true")?;
        
        let current_value = state.address_input_value.borrow().clone();
        set_attribute(&input, "value", &current_value)?;
        
        // Event listener para input (actualizar estado)
        {
            let state_clone = state.clone();
            let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                if let Some(input_el) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                    *state_clone.address_input_value.borrow_mut() = input_el.value();
                }
            }) as Box<dyn FnMut(web_sys::Event)>);
            input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        // Event listener para Enter/Escape
        {
            let state_clone = state.clone();
            let on_edit_clone = on_edit.clone();
            let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
                if e.key() == "Enter" {
                    e.prevent_default();
                    let value = state_clone.address_input_value.borrow().trim().to_string();
                    if let Some(cb) = &on_edit_clone {
                        cb(value);
                    }
                    // El callback manejará la actualización incremental
                } else if e.key() == "Escape" {
                    e.prevent_default();
                    *state_clone.address_input_value.borrow_mut() = addr.label.clone();
                    *state_clone.editing_address.borrow_mut() = false;
                    // Usar actualización incremental en lugar de re-render completo
                    crate::dom::incremental::toggle_edit_mode_address(&state_clone, false);
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
            input.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        append_child(&input_group, &input)?;
        append_child(&value_container, &input_group)?;
    } else {
        // Modo visualización: mostrar valor + botón editar
        let span = ElementBuilder::new("span")?
            .text(&addr.label)
            .build();
        
        let edit_btn = ElementBuilder::new("button")?
            .class("btn-icon")
            .attr("title", &t("modifier", lang))?
            .text("⚙️")
            .build();
        
        {
            let state_clone = state.clone();
            let addr_label = addr.label.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                *state_clone.address_input_value.borrow_mut() = addr_label.clone();
                *state_clone.editing_address.borrow_mut() = true;
                // Usar actualización incremental en lugar de re-render completo
                crate::dom::incremental::toggle_edit_mode_address(&state_clone, true);
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            edit_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        append_child(&value_container, &span)?;
        append_child(&value_container, &edit_btn)?;
    }
    
    append_child(&section, &label_el)?;
    append_child(&section, &value_container)?;
    
    Ok(section)
}

/// Crear sección de código de puerta editable
fn create_editable_door_code_section(
    state: &AppState,
    addr: &Address,
    lang: &str,
    on_edit: Option<Rc<dyn Fn(String)>>,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("detail-section editable")
        .build();
    
    let label_el = ElementBuilder::new("div")?
        .class("detail-label")
        .text(&t("codes_porte", lang))
        .build();
    
    let value_container = ElementBuilder::new("div")?
        .class("detail-value-with-action")
        .build();
    
    let editing = *state.editing_door_code.borrow();
    
    if editing {
        // Modo edición
        let input_group = ElementBuilder::new("div")?
            .class("edit-input-group")
            .build();
        
        let input = create_element("input")?;
        set_attribute(&input, "type", "text")?;
        set_attribute(&input, "class", "edit-input")?;
        set_attribute(&input, "placeholder", &t("code_de_porte", lang))?;
        set_attribute(&input, "autofocus", "true")?;
        
        let current_value = state.door_code_input_value.borrow().clone();
        set_attribute(&input, "value", &current_value)?;
        
        // Event listener para input
        {
            let state_clone = state.clone();
            let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                if let Some(input_el) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                    *state_clone.door_code_input_value.borrow_mut() = input_el.value();
                }
            }) as Box<dyn FnMut(web_sys::Event)>);
            input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        // Event listener para Enter/Escape
        {
            let state_clone = state.clone();
            let on_edit_clone = on_edit.clone();
            let current_code = addr.door_code.clone().unwrap_or_default();
            let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
                if e.key() == "Enter" {
                    e.prevent_default();
                    let value = state_clone.door_code_input_value.borrow().trim().to_string();
                    if let Some(cb) = &on_edit_clone {
                        cb(value);
                    }
                    *state_clone.editing_door_code.borrow_mut() = false;
                    crate::dom::incremental::toggle_edit_mode_door_code(&state_clone, false);
                } else if e.key() == "Escape" {
                    e.prevent_default();
                    *state_clone.door_code_input_value.borrow_mut() = current_code.clone();
                    *state_clone.editing_door_code.borrow_mut() = false;
                    crate::dom::incremental::toggle_edit_mode_door_code(&state_clone, false);
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
            input.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        append_child(&input_group, &input)?;
        append_child(&value_container, &input_group)?;
    } else {
        // Modo visualización
        let span = if let Some(code) = &addr.door_code {
            ElementBuilder::new("span")?
                .text(code)
                .build()
        } else {
            ElementBuilder::new("span")?
                .class("empty-value")
                .text(&t("non_renseigne", lang))
                .build()
        };
        
        let edit_btn = ElementBuilder::new("button")?
            .class("btn-icon-edit")
            .attr("title", &t("modifier", lang))?
            .text("✏️")
            .build();
        
        {
            let state_clone = state.clone();
            let current_code = addr.door_code.clone().unwrap_or_default();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                *state_clone.door_code_input_value.borrow_mut() = current_code.clone();
                *state_clone.editing_door_code.borrow_mut() = true;
                crate::dom::incremental::toggle_edit_mode_door_code(&state_clone, true);
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            edit_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        append_child(&value_container, &span)?;
        append_child(&value_container, &edit_btn)?;
    }
    
    append_child(&section, &label_el)?;
    append_child(&section, &value_container)?;
    
    Ok(section)
}

/// Crear sección de acceso BAL editable (toggle)
fn create_editable_mailbox_section(
    state: &AppState,
    addr: &Address,
    lang: &str,
    has_mailbox: bool,
    on_edit: Option<Rc<dyn Fn(bool)>>,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("detail-section editable")
        .build();
    
    let label_el = ElementBuilder::new("div")?
        .class("detail-label")
        .text(&t("acces_bal", lang))
        .build();
    
    let value_container = ElementBuilder::new("div")?
        .class("detail-value-with-action")
        .build();
    
    let bal_text = if has_mailbox {
        format!("✅ {}", t("oui_capital", lang))
    } else {
        format!("❌ {}", t("non_capital", lang))
    };
    
    let span = ElementBuilder::new("span")?
        .text(&bal_text)
        .build();
    
    let toggle_container = ElementBuilder::new("label")?
        .class("toggle-switch")
        .build();
    
    let toggle_input = create_element("input")?;
    set_attribute(&toggle_input, "type", "checkbox")?;
    if has_mailbox {
        set_attribute(&toggle_input, "checked", "checked")?;
    }
    
    // Deshabilitar toggle durante guardado
    let saving = *state.saving_mailbox.borrow();
    if saving {
        set_attribute(&toggle_input, "disabled", "true")?;
    }
    
    // Event listener para toggle
    {
        let on_edit_clone = on_edit.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
            if let Some(input_el) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                if let Some(cb) = &on_edit_clone {
                    cb(input_el.checked());
                }
            }
        }) as Box<dyn FnMut(web_sys::Event)>);
        toggle_input.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    let toggle_slider = ElementBuilder::new("span")?
        .class("toggle-slider")
        .build();
    
    append_child(&toggle_container, &toggle_input)?;
    append_child(&toggle_container, &toggle_slider)?;
    
    append_child(&value_container, &span)?;
    append_child(&value_container, &toggle_container)?;
    
    append_child(&section, &label_el)?;
    append_child(&section, &value_container)?;
    
    Ok(section)
}

/// Crear sección de notas chofer editable
fn create_editable_driver_notes_section(
    state: &AppState,
    addr: &Address,
    lang: &str,
    on_edit: Option<Rc<dyn Fn(String)>>,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("detail-section editable")
        .build();
    
    let label_el = ElementBuilder::new("div")?
        .class("detail-label")
        .text(&t("notes_chauffeur", lang))
        .build();
    
    let value_container = ElementBuilder::new("div")?
        .class("detail-value-with-action")
        .build();
    
    let editing = *state.editing_driver_notes.borrow();
    
    if editing {
        // Modo edición
        let input_group = ElementBuilder::new("div")?
            .class("edit-input-group")
            .build();
        
        let input = create_element("input")?;
        set_attribute(&input, "type", "text")?;
        set_attribute(&input, "class", "edit-input")?;
        set_attribute(&input, "placeholder", &t("ajouter_note", lang))?;
        set_attribute(&input, "autofocus", "true")?;
        
        let current_value = state.driver_notes_input_value.borrow().clone();
        set_attribute(&input, "value", &current_value)?;
        
        // Event listener para input
        {
            let state_clone = state.clone();
            let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                if let Some(input_el) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                    *state_clone.driver_notes_input_value.borrow_mut() = input_el.value();
                }
            }) as Box<dyn FnMut(web_sys::Event)>);
            input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        // Event listener para Enter/Escape
        {
            let state_clone = state.clone();
            let on_edit_clone = on_edit.clone();
            let current_notes = addr.driver_notes.clone().unwrap_or_default();
            let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
                if e.key() == "Enter" {
                    e.prevent_default();
                    let value = state_clone.driver_notes_input_value.borrow().trim().to_string();
                    if let Some(cb) = &on_edit_clone {
                        cb(value);
                    }
                    *state_clone.editing_driver_notes.borrow_mut() = false;
                    crate::dom::incremental::toggle_edit_mode_driver_notes(&state_clone, false);
                } else if e.key() == "Escape" {
                    e.prevent_default();
                    *state_clone.driver_notes_input_value.borrow_mut() = current_notes.clone();
                    *state_clone.editing_driver_notes.borrow_mut() = false;
                    crate::dom::incremental::toggle_edit_mode_driver_notes(&state_clone, false);
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
            input.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        append_child(&input_group, &input)?;
        append_child(&value_container, &input_group)?;
    } else {
        // Modo visualización
        let span = if let Some(notes) = &addr.driver_notes {
            if !notes.is_empty() {
                ElementBuilder::new("span")?
                    .text(&format!("\"{}\"", notes))
                    .build()
            } else {
                ElementBuilder::new("span")?
                    .class("empty-value")
                    .text(&t("ajouter_note", lang))
                    .build()
            }
        } else {
            ElementBuilder::new("span")?
                .class("empty-value")
                .text(&t("ajouter_note", lang))
                .build()
        };
        
        let edit_btn = ElementBuilder::new("button")?
            .class("btn-icon-edit")
            .attr("title", &t("modifier", lang))?
            .text("✏️")
            .build();
        
        {
            let state_clone = state.clone();
            let current_notes = addr.driver_notes.clone().unwrap_or_default();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                *state_clone.driver_notes_input_value.borrow_mut() = current_notes.clone();
                *state_clone.editing_driver_notes.borrow_mut() = true;
                crate::dom::incremental::toggle_edit_mode_driver_notes(&state_clone, true);
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            edit_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        append_child(&value_container, &span)?;
        append_child(&value_container, &edit_btn)?;
    }
    
    append_child(&section, &label_el)?;
    append_child(&section, &value_container)?;
    
    Ok(section)
}

/// Crear sección de detalle simple
fn create_detail_section(
    label: &str,
    value: &str,
    is_html: Option<bool>,
    editable: bool,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("detail-section")
        .build();
    
    if editable {
        add_class(&section, "editable")?;
    }
    
    let label_el = ElementBuilder::new("div")?
        .class("detail-label")
        .text(label)
        .build();
    
    let value_el = ElementBuilder::new("div")?
        .class("detail-value")
        .build();
    
    if is_html.unwrap_or(false) {
        value_el.set_inner_html(value);
    } else {
        crate::dom::set_text_content(&value_el, value);
    }
    
    append_child(&section, &label_el)?;
    append_child(&section, &value_el)?;
    
    Ok(section)
}

/// Crear sección de detalle con botón de acción
fn create_detail_section_with_action(
    label: &str,
    value: &str,
    is_html: Option<bool>,
    editable: bool,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("detail-section")
        .build();
    
    if editable {
        add_class(&section, "editable")?;
    }
    
    let label_el = ElementBuilder::new("div")?
        .class("detail-label")
        .text(label)
        .build();
    
    let value_container = ElementBuilder::new("div")?
        .class("detail-value-with-action")
        .build();
    
    let value_span = ElementBuilder::new("span")?
        .build();
    
    if is_html.unwrap_or(false) {
        value_span.set_inner_html(value);
    } else {
        crate::dom::set_text_content(&value_span, value);
    }
    
    append_child(&value_container, &value_span)?;
    append_child(&section, &label_el)?;
    append_child(&section, &value_container)?;
    
    Ok(section)
}

/// Crear sección con toggle
fn create_detail_section_with_toggle(
    label: &str,
    value_text: &str,
    checked: bool,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("detail-section editable")
        .build();
    
    let label_el = ElementBuilder::new("div")?
        .class("detail-label")
        .text(label)
        .build();
    
    let value_container = ElementBuilder::new("div")?
        .class("detail-value-with-action")
        .build();
    
    let value_span = ElementBuilder::new("span")?
        .text(value_text)
        .build();
    
    let toggle_container = ElementBuilder::new("label")?
        .class("toggle-switch")
        .build();
    
    let toggle_input = crate::dom::create_element("input")?;
    set_attribute(&toggle_input, "type", "checkbox")?;
    if checked {
        set_attribute(&toggle_input, "checked", "checked")?;
    }
    
    let toggle_slider = ElementBuilder::new("span")?
        .class("toggle-slider")
        .build();
    
    append_child(&toggle_container, &toggle_input)?;
    append_child(&toggle_container, &toggle_slider)?;
    
    append_child(&value_container, &value_span)?;
    append_child(&value_container, &toggle_container)?;
    
    append_child(&section, &label_el)?;
    append_child(&section, &value_container)?;
    
    Ok(section)
}

/// Crear sección de problemático
fn create_problematic_section(
    label: &str,
    is_problematic: bool,
    lang: &str,
    on_mark_problematic: Option<Rc<dyn Fn()>>,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("detail-section")
        .build();
    
    let detail_row = ElementBuilder::new("div")?
        .class("detail-row")
        .build();
    
    let label_el = ElementBuilder::new("div")?
        .class("detail-label")
        .text(label)
        .build();
    
    let value_el = ElementBuilder::new("div")?
        .class("detail-value")
        .build();
    
    if is_problematic {
        let badge = ElementBuilder::new("span")?
            .class("problematic-badge")
            .text(&t("problematique", lang))
            .build();
        append_child(&value_el, &badge)?;
    } else {
        let btn = ElementBuilder::new("button")?
            .class("btn-problematic")
            .attr("title", label)?
            .build();
        
        let btn_text = format!("⚠️ {}", t("marquer_problematique", lang));
        crate::dom::set_text_content(&btn, &btn_text);
        
        // Event listener para marcar como problemático
        if let Some(callback) = on_mark_problematic {
            let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                callback();
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        append_child(&value_el, &btn)?;
    }
    
    append_child(&detail_row, &label_el)?;
    append_child(&detail_row, &value_el)?;
    append_child(&section, &detail_row)?;
    
    Ok(section)
}

/// Crear sección para cambio de status
fn create_status_change_section(
    state: &AppState,
    tracking: &str,
    lang: &str,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("detail-section")
        .build();
    
    let actions = ElementBuilder::new("div")?
        .class("detail-actions")
        .build();
    
    let status_change_btn = ElementBuilder::new("button")?
        .class("btn-status-change")
        .text("⚠️ Nécessite changement de statut")
        .build();
    
    {
        let state_clone = state.clone();
        let tracking_clone = tracking.to_string();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            *state_clone.show_status_change_modal.borrow_mut() = true;
            *state_clone.status_change_tracking.borrow_mut() = Some(tracking_clone.clone());
            crate::rerender_app();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        status_change_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&actions, &status_change_btn)?;
    append_child(&section, &actions)?;
    
    Ok(section)
}

/// Renderizar modal de cambio de status
pub fn render_status_change_modal(state: &AppState) -> Result<Option<Element>, JsValue> {
    if !*state.show_status_change_modal.borrow() {
        return Ok(None);
    }
    
    let tracking = state.status_change_tracking.borrow().clone()
        .unwrap_or_else(|| "UNKNOWN".to_string());
    
    // Obtener sesión actual para driver_matricule
    let driver_matricule = state.session.get_session()
        .map(|s| s.driver.driver_id.clone())
        .unwrap_or_else(|| "UNKNOWN".to_string());
    
    let session_id = state.session.get_session()
        .map(|s| s.session_id.clone())
        .unwrap_or_default();
    
    let modal = ElementBuilder::new("div")?
        .class("modal-overlay")
        .build();
    
    let modal_content = ElementBuilder::new("div")?
        .class("modal-content")
        .build();
    
    // Prevenir cierre al click dentro
    {
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        modal_content.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    let title = ElementBuilder::new("h3")?
        .text("📝 Notes pour changement de statut")
        .build();
    
    let textarea = create_element("textarea")?;
    set_attribute(&textarea, "class", "status-notes-textarea")?;
    set_attribute(&textarea, "placeholder", "Ajoutez vos notes (ex: code barre manquant, étiquette déchirée, lieu de dépôt...)")?;
    set_attribute(&textarea, "rows", "5")?;
    
    let actions = ElementBuilder::new("div")?
        .class("modal-actions")
        .build();
    
    let cancel_btn = ElementBuilder::new("button")?
        .class("btn-cancel")
        .text("Annuler")
        .build();
    
    {
        let state_clone = state.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            *state_clone.show_status_change_modal.borrow_mut() = false;
            *state_clone.status_change_tracking.borrow_mut() = None;
            crate::rerender_app();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        cancel_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    let confirm_btn = ElementBuilder::new("button")?
        .class("btn-confirm")
        .text("✅ Confirmer")
        .build();
    
    {
        let state_clone = state.clone();
        let textarea_clone = textarea.clone();
        let tracking_clone = tracking.clone();
        let driver_matricule_clone = driver_matricule.clone();
        let session_id_clone = session_id.clone();
        
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            // Obtener notes del textarea
            let notes = textarea_clone.dyn_ref::<web_sys::HtmlTextAreaElement>()
                .map(|t| t.value())
                .unwrap_or_default();
            
            // Enviar al backend
            let state_clone = state_clone.clone();
            let tracking_clone = tracking_clone.clone();
            let driver_matricule_clone = driver_matricule_clone.clone();
            let session_id_clone = session_id_clone.clone();
            let notes_clone = notes.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                use crate::services::api_client::ApiClient;
                let api = ApiClient::new();
                
                match api.create_status_change_request(
                    &tracking_clone,
                    &session_id_clone,
                    &driver_matricule_clone,
                    if notes_clone.is_empty() { None } else { Some(&notes_clone) },
                ).await {
                    Ok(_) => {
                        console::log_1(&JsValue::from_str("✅ Request de cambio de status creado"));
                        *state_clone.show_status_change_modal.borrow_mut() = false;
                        *state_clone.status_change_tracking.borrow_mut() = None;
                        crate::rerender_app();
                    }
                    Err(e) => {
                        console::error_1(&JsValue::from_str(&format!("❌ Error: {}", e)));
                    }
                }
            });
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        confirm_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    // Cerrar al click en overlay
    {
        let state_clone = state.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            *state_clone.show_status_change_modal.borrow_mut() = false;
            *state_clone.status_change_tracking.borrow_mut() = None;
            crate::rerender_app();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        modal.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&actions, &cancel_btn)?;
    append_child(&actions, &confirm_btn)?;
    
    append_child(&modal_content, &title)?;
    append_child(&modal_content, &textarea)?;
    append_child(&modal_content, &actions)?;
    append_child(&modal, &modal_content)?;
    
    Ok(Some(modal))
}
