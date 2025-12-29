// ============================================================================
// PACKAGE CARD VIEW - Convertida de componente Yew a función Rust puro
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::Element;
use wasm_bindgen::closure::Closure;
use std::rc::Rc;
use crate::dom::{ElementBuilder, append_child};
use crate::models::package::Package;

/// Renderizar package card
pub fn render_package_card(
    pkg: &Package,
    index: usize,
    address: Option<&str>,
    is_selected: bool,
    is_expanded: bool, // Para mostrar paquetes internos si es grupo
    on_select: Rc<dyn Fn(usize)>,
    on_info: Rc<dyn Fn(String)>,
    on_toggle_expand: Option<Rc<dyn Fn(usize)>>, // Toggle para expandir/colapsar (solo para grupos)
) -> Result<Element, JsValue> {
    // Determinar clases
    let mut classes = vec!["package-card"];
    
    // Tipo de entrega
    match pkg.delivery_type {
        crate::models::package::DeliveryType::PickupPoint => classes.push("type-relais"),
        crate::models::package::DeliveryType::Rcs => classes.push("type-rcs"),
        _ => classes.push("type-domicile"),
    }
    
    if pkg.is_problematic {
        classes.push("problematic");
    }
    
    if is_selected {
        classes.push("selected");
    }
    
    // Crear card
    let card_el = ElementBuilder::new("div")?
        .class(&classes.join(" "))
        .attr("data-index", &index.to_string())?
        .build();
    
    // Event listener para click en card
    {
        let card_el_clone = card_el.clone();
        let index_clone = index;
        let on_select_clone = on_select.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            on_select_clone(index_clone);
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        
        card_el_clone.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    // Header
    let header = ElementBuilder::new("div")?
        .class("package-header")
        .build();
    
    // Número del paquete
    let number_class = match pkg.status.as_str() {
        s if s.contains("RECEPTIONNER") => "package-number package-number-yellow",
        s if s.contains("LIVRER") => "package-number package-number-green",
        s if s.contains("NONLIV") => "package-number package-number-red",
        _ => "package-number package-number-normal",
    };
    
    let number = ElementBuilder::new("div")?
        .class(number_class)
        .text(&(index + 1).to_string())
        .build();
    
    // Botón info
    let info_btn = ElementBuilder::new("button")?
        .class("btn-info")
        .text("i")
        .build();
    
    {
        let tracking = pkg.tracking.clone();
        let on_info_clone = on_info.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            e.prevent_default(); // Prevenir comportamiento por defecto también
            on_info_clone(tracking.clone());
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        
        info_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&header, &number)?;
    append_child(&header, &info_btn)?;
    
    // Main content
    let main = ElementBuilder::new("div")?
        .class("package-main")
        .build();
    
    let info = ElementBuilder::new("div")?
        .class("package-info")
        .build();
    
    let recipient_row = ElementBuilder::new("div")?
        .class("package-recipient-row")
        .build();
    
    let recipient = ElementBuilder::new("div")?
        .class("package-recipient")
        .text(&pkg.customer_name)
        .build();
    
    append_child(&recipient_row, &recipient)?;
    
    if is_selected {
        let nav_btn = ElementBuilder::new("button")?
            .class("btn-navigate")
            .text("Go")
            .build();
        append_child(&recipient_row, &nav_btn)?;
    }
    
    append_child(&info, &recipient_row)?;
    
    if let Some(addr) = address {
        let addr_el = ElementBuilder::new("div")?
            .class("package-address")
            .text(addr)
            .build();
        append_child(&info, &addr_el)?;
    }
    
    append_child(&main, &info)?;
    
    append_child(&card_el, &header)?;
    append_child(&card_el, &main)?;
    
    // PAQUETES EXPANDIDOS - solo para grupos Y cuando está expandido
    if pkg.is_group && is_expanded {
        if let Some(packages) = &pkg.group_packages {
            let expanded_container = ElementBuilder::new("div")?
                .class("packages-expanded")
                .build();
            
            for (idx, pkg_inner) in packages.iter().enumerate() {
                let pkg_status_color = match pkg_inner.status.as_str() {
                    s if s.contains("RECEPTIONNER") => "package-number-yellow",
                    s if s.contains("LIVRER") => "package-number-green",
                    s if s.contains("NONLIV") => "package-number-red",
                    _ => "package-number-normal",
                };
                
                let pkg_type_class = match pkg_inner.delivery_type {
                    crate::models::package::DeliveryType::PickupPoint => "type-relais",
                    crate::models::package::DeliveryType::Rcs => "type-rcs",
                    _ => "type-domicile",
                };
                
                let package_item = ElementBuilder::new("div")?
                    .class(&format!("package-item {}", pkg_type_class))
                    .build();
                
                let item_header = ElementBuilder::new("div")?
                    .class("package-item-header")
                    .build();
                
                let pkg_number = ElementBuilder::new("span")?
                    .class(&format!("package-number {}", pkg_status_color))
                    .text(&(idx + 1).to_string())
                    .build();
                append_child(&item_header, &pkg_number)?;
                
                match pkg_inner.delivery_type {
                    // RELAIS: Solo tracking
                    crate::models::package::DeliveryType::PickupPoint => {
                        let content = ElementBuilder::new("div")?
                            .class("package-item-content")
                            .build();
                        
                        let tracking_label = ElementBuilder::new("span")?
                            .class("tracking-label")
                            .text("📦 ")
                            .build();
                        let tracking_value = ElementBuilder::new("span")?
                            .class("tracking-value")
                            .text(&pkg_inner.tracking)
                            .build();
                        
                        append_child(&content, &tracking_label)?;
                        append_child(&content, &tracking_value)?;
                        append_child(&item_header, &content)?;
                    },
                    // NO RELAIS: Nombre del cliente + botón detalles
                    _ => {
                        let customer_name = ElementBuilder::new("strong")?
                            .class("package-customer")
                            .text(&pkg_inner.customer_name)
                            .build();
                        append_child(&item_header, &customer_name)?;
                        
                        let details_btn = ElementBuilder::new("button")?
                            .class("btn-package-details")
                            .text("i")
                            .build();
                        
                        {
                            let tracking = pkg_inner.tracking.clone();
                            let on_info_clone = on_info.clone();
                            let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                                e.stop_propagation();
                                e.prevent_default(); // Prevenir comportamiento por defecto también
                                on_info_clone(tracking.clone());
                            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
                            
                            details_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
                            closure.forget();
                        }
                        
                        append_child(&item_header, &details_btn)?;
                    }
                }
                
                append_child(&package_item, &item_header)?;
                append_child(&expanded_container, &package_item)?;
            }
            
            append_child(&card_el, &expanded_container)?;
        }
    }
    
    // EXPAND HANDLE - solo para grupos Y cuando está seleccionado (igual que Yew)
    if pkg.is_group && is_selected {
        if let Some(on_toggle) = on_toggle_expand {
            let mut expand_handle_classes = vec!["expand-handle"];
            if !is_expanded {
                expand_handle_classes.push("pulse");
            }
            let expand_handle = ElementBuilder::new("div")?
                .class(&expand_handle_classes.join(" "))
                .build();
            
            let expand_indicator = ElementBuilder::new("div")?
                .class("expand-indicator")
                .build();
            append_child(&expand_handle, &expand_indicator)?;
            
            {
                let index_clone = index;
                let on_toggle_clone = on_toggle.clone();
                let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                    e.stop_propagation();
                    e.prevent_default();
                    on_toggle_clone(index_clone);
                }) as Box<dyn FnMut(web_sys::MouseEvent)>);
                
                expand_handle.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
                closure.forget();
            }
            
            append_child(&card_el, &expand_handle)?;
        }
    }
    
    Ok(card_el)
}

