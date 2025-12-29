// ============================================================================
// ELEMENT HELPERS - Funciones básicas para manipular DOM
// ============================================================================

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, Window, HtmlElement};

/// Obtener window global
pub fn window() -> Option<Window> {
    web_sys::window()
}

/// Obtener document
pub fn document() -> Option<Document> {
    window()?.document()
}

/// Obtener elemento por ID
pub fn get_element_by_id(id: &str) -> Option<Element> {
    document()?.get_element_by_id(id)
}

/// Crear elemento
pub fn create_element(tag: &str) -> Result<Element, JsValue> {
    document()
        .ok_or_else(|| JsValue::from_str("No document"))
        .and_then(|doc| doc.create_element(tag))
}

/// Establecer class name (reemplaza todas las clases)
pub fn set_class_name(element: &Element, class: &str) {
    element.set_class_name(class);
}

/// Agregar clase
pub fn add_class(element: &Element, class: &str) -> Result<(), JsValue> {
    element
        .dyn_ref::<HtmlElement>()
        .ok_or_else(|| JsValue::from_str("Element is not an HtmlElement"))?
        .class_list()
        .add_1(class)
}

/// Remover clase
pub fn remove_class(element: &Element, class: &str) -> Result<(), JsValue> {
    element
        .dyn_ref::<HtmlElement>()
        .ok_or_else(|| JsValue::from_str("Element is not an HtmlElement"))?
        .class_list()
        .remove_1(class)
}

/// Establecer text content
pub fn set_text_content(element: &Element, text: &str) {
    element.set_text_content(Some(text));
}

/// Establecer inner HTML
pub fn set_inner_html(element: &Element, html: &str) {
    element.set_inner_html(html);
}

/// Agregar hijo
pub fn append_child(parent: &Element, child: &Element) -> Result<(), JsValue> {
    parent.append_child(child).map(|_| ())
}

/// Remover hijo
pub fn remove_child(parent: &Element, child: &Element) -> Result<(), JsValue> {
    parent.remove_child(child).map(|_| ())
}

/// Establecer atributo
pub fn set_attribute(element: &Element, name: &str, value: &str) -> Result<(), JsValue> {
    element.set_attribute(name, value)
}

/// Obtener atributo
pub fn get_attribute(element: &Element, name: &str) -> Option<String> {
    element.get_attribute(name)
}

/// Remover atributo
pub fn remove_attribute(element: &Element, name: &str) -> Result<(), JsValue> {
    element.remove_attribute(name)
}

/// Verificar si tiene clase
pub fn has_class(element: &Element, class: &str) -> bool {
    element.class_list().contains(class)
}

/// Query selector (buscar elemento por selector CSS)
pub fn query_selector(selector: &str) -> Result<Option<Element>, JsValue> {
    document()
        .ok_or_else(|| JsValue::from_str("No document"))?
        .query_selector(selector)
}

/// Query selector all (buscar múltiples elementos por selector CSS)
/// Usa js_sys::eval para ejecutar querySelectorAll directamente
pub fn query_selector_all(selector: &str) -> Result<js_sys::Array, JsValue> {
    let js_code = format!("Array.from(document.querySelectorAll('{}'))", selector);
    let result = js_sys::eval(&js_code)?;
    if let Some(array) = result.dyn_ref::<js_sys::Array>() {
        Ok(array.clone())
    } else {
        Err(JsValue::from_str("querySelectorAll did not return an array"))
    }
}

