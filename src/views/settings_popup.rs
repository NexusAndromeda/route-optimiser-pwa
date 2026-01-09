// ============================================================================
// SETTINGS POPUP VIEW - Popup de configuración (Rust puro)
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::Element;
use wasm_bindgen::closure::Closure;
use std::rc::Rc;
use crate::dom::{ElementBuilder, append_child, set_attribute, add_class};
use crate::state::app_state::AppState;
use crate::utils::i18n::t;

/// Renderizar popup de configuración
pub fn render_settings_popup(
    state: &AppState,
    on_close: Rc<dyn Fn()>,
    on_logout: Rc<dyn Fn()>,
) -> Result<Element, JsValue> {
    let lang = state.language.borrow().clone();
    let map_enabled = *state.map_enabled.borrow();
    let edit_mode = *state.edit_mode.borrow();
    let filter_mode = *state.filter_mode.borrow();
    
    // Popup container - inicialmente oculto (se mostrará con CSS cuando tenga clase "active")
    let popup = ElementBuilder::new("div")?
        .id("settings-popup")?
        .class("settings-popup") // Sin "active" inicialmente
        .build();
    
    // Prevenir cierre al click dentro
    {
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        popup.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    // Content
    let content = ElementBuilder::new("div")?
        .class("settings-content")
        .build();
    
    // Header
    let header = ElementBuilder::new("div")?
        .class("settings-header")
        .build();
    
    let title = ElementBuilder::new("h3")?
        .text(&t("parametres", &lang))
        .build();
    
    let close_btn = ElementBuilder::new("button")?
        .class("btn-close-settings")
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
        .class("settings-body")
        .build();
    
    // Language section
    let lang_section = create_language_section(&lang, state, &on_close)?;
    append_child(&body, &lang_section)?;
    
    // Map toggle - solo mostrar si NO está en modo admin
    let is_admin_mode = *state.admin_mode.borrow();
    if !is_admin_mode {
        let map_section = create_toggle_section(
            "Mapa",
            map_enabled,
            {
                let state_clone = state.clone();
                Rc::new(move |enabled: bool| {
                    state_clone.set_map_enabled(enabled);
                    state_clone.notify_subscribers();
                })
            },
        )?;
        append_child(&body, &map_section)?;
    }
    
    // Edit mode toggle
    let edit_section = create_toggle_section(
        &t("mode_edition", &lang),
        edit_mode,
        {
            let state_clone = state.clone();
            Rc::new(move |enabled: bool| {
                state_clone.set_edit_mode(enabled);
            })
        },
    )?;
    append_child(&body, &edit_section)?;
    
    // Filter mode toggle
    let filter_section = create_toggle_section(
        &t("filtrer", &lang),
        filter_mode,
        {
            let state_clone = state.clone();
            Rc::new(move |enabled: bool| {
                state_clone.set_filter_mode(enabled);
            })
        },
    )?;
    append_child(&body, &filter_section)?;
    
    // Color codes section
    let color_section = create_color_codes_section(&lang)?;
    append_child(&body, &color_section)?;
    
    // Logout button
    let logout_btn = ElementBuilder::new("button")?
        .class("btn-logout")
        .text(&t("deconnexion", &lang))
        .build();
    
    {
        let on_logout_clone = on_logout.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            on_logout_clone();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        logout_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&body, &logout_btn)?;
    append_child(&content, &body)?;
    append_child(&popup, &content)?;
    
    Ok(popup)
}

/// Crear sección de idioma
fn create_language_section(
    lang: &str,
    state: &AppState,
    _on_close: &Rc<dyn Fn()>,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("language-section")
        .build();
    
    let label = ElementBuilder::new("div")?
        .class("language-label")
        .text(&t("langue", lang))
        .build();
    
    let toggle_container = ElementBuilder::new("div")?
        .class("language-toggle")
        .build();
    
    // Botón FR
    let current_lang = state.language.borrow().clone();
    let fr_active = current_lang == "FR";
    let fr_btn = ElementBuilder::new("button")?
        .class(if fr_active { "toggle-btn active" } else { "toggle-btn" })
        .text("FR")
        .build();
    
    {
        let state_clone = state.clone();
        let on_close_clone = _on_close.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            state_clone.set_language("FR".to_string());
            state_clone.notify_subscribers();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        fr_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    // Botón ES
    let es_active = current_lang == "ES";
    let es_btn = ElementBuilder::new("button")?
        .class(if es_active { "toggle-btn active" } else { "toggle-btn" })
        .text("ES")
        .build();
    
    {
        let state_clone = state.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            state_clone.set_language("ES".to_string());
            state_clone.notify_subscribers();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        es_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&toggle_container, &fr_btn)?;
    append_child(&toggle_container, &es_btn)?;
    append_child(&section, &label)?;
    append_child(&section, &toggle_container)?;
    
    Ok(section)
}

/// Crear sección con toggle
fn create_toggle_section(
    label: &str,
    checked: bool,
    on_change: Rc<dyn Fn(bool)>,
) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("reorder-mode-section")
        .build();
    
    let label_el = ElementBuilder::new("span")?
        .class("reorder-mode-label")
        .text(label)
        .build();
    
    let toggle_container = ElementBuilder::new("label")?
        .class("toggle-switch")
        .build();
    
    let toggle_input = crate::dom::create_element("input")?;
    set_attribute(&toggle_input, "type", "checkbox")?;
    if checked {
        set_attribute(&toggle_input, "checked", "checked")?;
    }
    
    {
        let on_change_clone = on_change.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
            if let Some(input) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                on_change_clone(input.checked());
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
    
    append_child(&section, &label_el)?;
    append_child(&section, &toggle_container)?;
    
    Ok(section)
}

/// Crear sección de códigos de color
fn create_color_codes_section(lang: &str) -> Result<Element, JsValue> {
    let section = ElementBuilder::new("div")?
        .class("color-codes-section")
        .build();
    
    let label = ElementBuilder::new("div")?
        .class("color-codes-label")
        .text(&t("codes_couleur", lang))
        .build();
    
    let list = ElementBuilder::new("div")?
        .class("color-codes-list")
        .build();
    
    // Lista de códigos de color
    let color_items = vec![
        ("relais", t("relais", lang)),
        ("rcs", t("rcs_premium", lang)),
        ("green", t("livre", lang)),
        ("red", t("non_livre", lang)),
        ("blue", t("en_transit", lang)),
        ("cyan", t("receptionne", lang)),
        ("magenta", t("en_collecte", lang)),
    ];
    
    for (color_class, description) in color_items {
        let item = ElementBuilder::new("div")?
            .class("color-code-item")
            .build();
        
        let indicator = ElementBuilder::new("div")?
            .class(&format!("color-indicator {}", color_class))
            .build();
        
        let desc_span = ElementBuilder::new("span")?
            .class("color-description")
            .text(&description)
            .build();
        
        append_child(&item, &indicator)?;
        append_child(&item, &desc_span)?;
        append_child(&list, &item)?;
    }
    
    append_child(&section, &label)?;
    append_child(&section, &list)?;
    
    Ok(section)
}
