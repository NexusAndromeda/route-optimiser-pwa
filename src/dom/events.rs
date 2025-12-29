// ============================================================================
// EVENT HANDLING - Sistema de eventos
// ============================================================================
// GESTIÓN DE MEMORY LEAKS:
// - Para listeners en elementos del DOM: cuando el elemento se destruye (p.ej. con set_inner_html("")),
//   el navegador automáticamente limpia los listeners asociados. Por lo tanto, closure.forget() es
//   seguro para listeners locales.
// - Para listeners globales (window/document): solo deben registrarse UNA VEZ al inicio de la app.
//   Si se registran múltiples veces, se acumularán. Por eso, usar protección (flags) para prevenir
//   múltiples registros (ver NetworkMonitor como ejemplo).
// ============================================================================

use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, Event, MouseEvent, KeyboardEvent, InputEvent, DragEvent};
use std::cell::RefCell;
use std::rc::Rc;

/// Event listener helper (legacy, no se usa actualmente)
/// Los listeners se registran directamente usando Closure y forget()
pub struct EventListener {
    _closure: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
    element: Element,
    event_type: String,
}

impl EventListener {
    /// Crear event listener genérico
    pub fn new<F>(element: &Element, event_type: &str, handler: F) -> Result<Self, JsValue>
    where
        F: FnMut() + 'static,
    {
        let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut()>);
        element.add_event_listener_with_callback(
            event_type,
            closure.as_ref().unchecked_ref(),
        )?;
        // Nota: closure.forget() es necesario en Rust WASM para mantener el closure vivo.
        // Para listeners en elementos del DOM, cuando el elemento se destruye, el navegador
        // automáticamente limpia los listeners, por lo que no hay memory leak.
        closure.forget();
        Ok(Self {
            _closure: Rc::new(RefCell::new(None)), // Placeholder
            element: element.clone(),
            event_type: event_type.to_string(),
        })
    }
    
    /// Crear click listener (simplificado - no guarda el closure para evitar problemas de tipos)
    pub fn click<F>(_element: &Element, _handler: F) -> Result<Self, JsValue>
    where
        F: FnMut(MouseEvent) + 'static,
    {
        // Esta función no se usa actualmente - usamos Closure directamente en los componentes
        // Mantenemos la estructura por compatibilidad pero simplificamos la implementación
        Ok(Self {
            _closure: Rc::new(RefCell::new(None)),
            element: _element.clone(),
            event_type: "click".to_string(),
        })
    }
    
    /// Crear input listener
    pub fn input<F>(element: &Element, handler: F) -> Result<Self, JsValue>
    where
        F: FnMut(InputEvent) + 'static,
    {
        let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(InputEvent)>);
        element.add_event_listener_with_callback(
            "input",
            closure.as_ref().unchecked_ref(),
        )?;
        Ok(Self {
            _closure: Rc::new(RefCell::new(None)),
            element: element.clone(),
            event_type: "input".to_string(),
        })
    }
    
    /// Crear drag listener
    pub fn drag<F>(element: &Element, handler: F) -> Result<Self, JsValue>
    where
        F: FnMut(DragEvent) + 'static,
    {
        let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(DragEvent)>);
        element.add_event_listener_with_callback(
            "drag",
            closure.as_ref().unchecked_ref(),
        )?;
        Ok(Self {
            _closure: Rc::new(RefCell::new(None)),
            element: element.clone(),
            event_type: "drag".to_string(),
        })
    }
}

/// Helper para crear click handler simple
/// Nota: Cuando el elemento se destruye del DOM (p.ej. con set_inner_html("")),
/// el navegador automáticamente limpia los listeners, por lo que closure.forget() es seguro.
pub fn on_click<F>(element: &Element, handler: F) -> Result<(), JsValue>
where
    F: FnMut(MouseEvent) + 'static,
{
    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(MouseEvent)>);
    element.add_event_listener_with_callback(
        "click",
        closure.as_ref().unchecked_ref(),
    )?;
    // Nota: closure.forget() es necesario para mantener el closure vivo en Rust WASM
    closure.forget();
    Ok(())
}

/// Helper para crear input handler simple
/// Nota: Cuando el elemento se destruye del DOM, el navegador automáticamente limpia los listeners.
pub fn on_input<F>(element: &Element, handler: F) -> Result<(), JsValue>
where
    F: FnMut(InputEvent) + 'static,
{
    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(InputEvent)>);
    element.add_event_listener_with_callback(
        "input",
        closure.as_ref().unchecked_ref(),
    )?;
    // Nota: closure.forget() es necesario para mantener el closure vivo en Rust WASM
    closure.forget();
    Ok(())
}

