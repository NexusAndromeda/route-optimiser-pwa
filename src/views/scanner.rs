// ============================================================================
// SCANNER VIEW - Scanner de códigos de barras con QuaggaJS (Rust puro)
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::Element;
use wasm_bindgen::closure::Closure;
use std::rc::Rc;
use crate::dom::{ElementBuilder, append_child, set_attribute};
use crate::utils::barcode_ffi;

/// Renderizar scanner
pub fn render_scanner(
    on_close: Rc<dyn Fn()>,
    on_barcode: Rc<dyn Fn(String)>,
) -> Result<Element, JsValue> {
    // Modal container
    let modal = ElementBuilder::new("div")?
        .id("scanner-modal")?
        .class("scanner-modal active")
        .build();
    
    // Overlay (cierra al hacer click)
    let overlay = ElementBuilder::new("div")?
        .class("scanner-overlay")
        .build();
    
    {
        let on_close_clone = on_close.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            // Detener scanner antes de cerrar
            barcode_ffi::stop_barcode_scanner();
            on_close_clone();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        overlay.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&modal, &overlay)?;
    
    // Content
    let content = ElementBuilder::new("div")?
        .class("scanner-content")
        .build();
    
    // Prevenir cierre al click dentro
    {
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        content.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    // Header
    let header = ElementBuilder::new("div")?
        .class("scanner-header")
        .build();
    
    let title = ElementBuilder::new("h2")?
        .text("Scanner")
        .build();
    
    let close_btn = ElementBuilder::new("button")?
        .class("btn-close")
        .text("✕")
        .build();
    
    {
        let on_close_clone = on_close.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            // Detener scanner antes de cerrar
            barcode_ffi::stop_barcode_scanner();
            on_close_clone();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        close_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&header, &title)?;
    append_child(&header, &close_btn)?;
    append_child(&content, &header)?;
    
    // Container para QuaggaJS (ID debe coincidir con barcode_scanner.js)
    let scanner_container_id = "scanner-viewport";
    let scanner_container = ElementBuilder::new("div")?
        .attr("id", scanner_container_id)?
        .class("scanner-viewport")
        .build();
    
    append_child(&content, &scanner_container)?;
    
    // Inicializar QuaggaJS con delay (como en Yew) para asegurar que el DOM esté listo
    {
        let on_barcode_clone = on_barcode.clone();
        
        // Callback cuando se detecta un código
        let on_detected_closure = Closure::wrap(Box::new(move |barcode: JsValue| {
            if let Some(barcode_str) = barcode.as_string() {
                log::info!("📱 [SCANNER] Código detectado: {}", barcode_str);
                on_barcode_clone(barcode_str);
            }
        }) as Box<dyn FnMut(JsValue)>);
        
        // Callback de error
        let on_error_closure = Closure::wrap(Box::new(move |_error: JsValue| {
            log::error!("❌ [SCANNER] Error en QuaggaJS");
            barcode_ffi::show_scanner_error();
        }) as Box<dyn FnMut(JsValue)>);
        
        // Callback cuando está listo
        let on_ready_closure = Closure::wrap(Box::new(move |_ready: JsValue| {
            log::info!("✅ [SCANNER] QuaggaJS listo");
            barcode_ffi::hide_scanner_error();
        }) as Box<dyn FnMut(JsValue)>);
        
        // Inicializar con delay (100ms como en Yew) para asegurar que el DOM esté listo
        use gloo_timers::callback::Timeout;
        
        // Usar Timeout en lugar de async/await
        Timeout::new(100, move || {
            log::info!("📷 [SCANNER] Inicializando QuaggaJS...");
        barcode_ffi::init_barcode_scanner_with_ready(
            scanner_container_id,
            on_detected_closure.as_ref().unchecked_ref(),
            on_error_closure.as_ref().unchecked_ref(),
            on_ready_closure.as_ref().unchecked_ref(),
        );
        
        // Mantener closures vivos (se liberarán cuando el elemento se destruya)
        on_detected_closure.forget();
        on_error_closure.forget();
        on_ready_closure.forget();
        }).forget();
    }
    
    append_child(&modal, &content)?;
    
    Ok(modal)
}
