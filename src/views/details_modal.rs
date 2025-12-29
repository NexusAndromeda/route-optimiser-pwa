// ============================================================================
// DETAILS MODAL VIEW - Modal de detalles del paquete (Rust puro)
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::Element;
use wasm_bindgen::closure::Closure;
use std::rc::Rc;
use crate::dom::{ElementBuilder, append_child, set_attribute, add_class, create_element};
use crate::models::package::Package;
use crate::models::address::Address;
use crate::state::app_state::AppState;
use crate::utils::i18n::t;

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
    
    append_child(&content, &body)?;
    append_child(&modal, &content)?;
    
    Ok(modal)
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
