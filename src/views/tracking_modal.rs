// ============================================================================
// TRACKING MODAL - Modal de búsqueda de trackings
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::Element;
use std::rc::Rc;
use crate::dom::{ElementBuilder, append_child, set_attribute, add_class, remove_class, set_inner_html, set_text_content};
use crate::dom::events::on_click;
use crate::models::session::DeliverySession;
use crate::utils::i18n::t;

/// Renderizar modal de búsqueda de trackings
pub fn render_tracking_modal(
    session: &DeliverySession,
    on_tracking_selected: Rc<dyn Fn(String)>,
    on_close: Rc<dyn Fn()>,
    lang: &str,
) -> Result<Element, JsValue> {
    // Modal overlay - inicialmente oculto (se mostrará con CSS cuando tenga clase "show")
    let modal = ElementBuilder::new("div")?
        .id("tracking-modal")?
        .class("company-modal") // Reutilizar estilos del modal de empresas (sin "show" inicialmente)
        .build();
    
    // Event listener para cerrar al hacer click fuera
    {
        let on_close_clone = on_close.clone();
        let modal_clone = modal.clone();
        on_click(&modal, move |_| {
            on_close_clone();
        })?;
    }
    
    // Modal content
    let modal_content = ElementBuilder::new("div")?
        .class("company-modal-content")
        .build();
    
    // Prevenir que el click dentro del contenido cierre el modal
    {
        let modal_content_clone = modal_content.clone();
        on_click(&modal_content, move |e: web_sys::MouseEvent| {
            e.stop_propagation();
        })?;
    }
    
    // Header
    let modal_header = ElementBuilder::new("div")?
        .class("company-modal-header")
        .build();
    
    let header_title = ElementBuilder::new("h3")?
        .text(&t("buscar_tracking_title", lang))
        .build();
    
    let close_btn = ElementBuilder::new("button")?
        .attr("type", "button")?
        .attr("class", "btn-close")?
        .text("✕")
        .build();
    
    {
        let on_close_clone = on_close.clone();
        on_click(&close_btn, move |_| {
            on_close_clone();
        })?;
    }
    
    append_child(&modal_header, &header_title)?;
    append_child(&modal_header, &close_btn)?;
    
    // Search input
    let search_container = ElementBuilder::new("div")?
        .class("company-search")
        .build();
    
    let search_input = crate::dom::create_element("input")?;
    set_attribute(&search_input, "type", "text")?;
    set_attribute(&search_input, "id", "tracking-search")?;
    set_attribute(&search_input, "placeholder", &t("buscar_tracking_placeholder", lang))?;
    
    // Lista de trackings (container)
    let tracking_list = ElementBuilder::new("div")?
        .id("tracking-list-container")?
        .class("company-list")
        .build();
    
    // Obtener lista de trackings
    let trackings: Vec<String> = session.packages.keys().cloned().collect();
    
    // Renderizar lista inicial (todos los trackings)
    render_tracking_list(&tracking_list, &trackings, &on_tracking_selected, &on_close, lang)?;
    
    // Event listener para búsqueda
    {
        let trackings_clone = trackings.clone();
        let list_clone = tracking_list.clone();
        let on_select_clone = on_tracking_selected.clone();
        let on_close_clone = on_close.clone();
        let lang_tracking = lang.to_string();
        
        crate::dom::events::on_input(&search_input, move |e: web_sys::InputEvent| {
            if let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                let query = target.value().to_lowercase();
                
                // Filtrar trackings
                let filtered: Vec<String> = if query.is_empty() {
                    trackings_clone.clone()
                } else {
                    trackings_clone.iter()
                        .filter(|tracking| tracking.to_lowercase().contains(&query))
                        .cloned()
                        .collect()
                };
                
                // Re-renderizar lista
                let _ = render_tracking_list(&list_clone, &filtered, &on_select_clone, &on_close_clone, &lang_tracking);
            }
        })?;
    }
    
    append_child(&search_container, &search_input)?;
    
    append_child(&modal_content, &modal_header)?;
    append_child(&modal_content, &search_container)?;
    append_child(&modal_content, &tracking_list)?;
    
    append_child(&modal, &modal_content)?;
    
    Ok(modal)
}

/// Renderizar lista de trackings
fn render_tracking_list(
    container: &Element,
    trackings: &[String],
    on_select: &Rc<dyn Fn(String)>,
    on_close: &Rc<dyn Fn()>,
    lang: &str,
) -> Result<(), JsValue> {
    // Limpiar lista anterior
    set_inner_html(container, "");
    
    if trackings.is_empty() {
        let empty_msg = ElementBuilder::new("div")?
            .class("company-empty")
            .text(&t("aucun_tracking", lang))
            .build();
        append_child(container, &empty_msg)?;
        return Ok(());
    }
    
    for tracking in trackings {
        let item = ElementBuilder::new("div")?
            .class("company-item")
            .build();
        
        let tracking_text = ElementBuilder::new("div")?
            .class("company-name")
            .text(tracking)
            .build();
        
        append_child(&item, &tracking_text)?;
        
        // Event listener para seleccionar tracking
        {
            let tracking_clone = tracking.clone();
            let on_select_clone = on_select.clone();
            let on_close_clone = on_close.clone();
            on_click(&item, move |_| {
                on_select_clone(tracking_clone.clone());
                on_close_clone();
            })?;
        }
        
        append_child(container, &item)?;
    }
    
    Ok(())
}

