// ============================================================================
// ELEMENT BUILDER - Builder pattern para crear elementos fácilmente
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::Element;
use crate::dom::{create_element, set_class_name, set_text_content, append_child, set_attribute};

pub struct ElementBuilder {
    element: Element,
}

impl ElementBuilder {
    /// Crear nuevo builder para un elemento
    pub fn new(tag: &str) -> Result<Self, JsValue> {
        Ok(Self {
            element: create_element(tag)?,
        })
    }
    
    /// Establecer class name (reemplaza todas las clases)
    pub fn class(mut self, class: &str) -> Self {
        set_class_name(&self.element, class);
        self
    }
    
    /// Agregar clases adicionales (sin reemplazar)
    pub fn add_class(mut self, class: &str) -> Result<Self, JsValue> {
        self.element.class_list().add_1(class)?;
        Ok(self)
    }
    
    /// Establecer ID
    pub fn id(mut self, id: &str) -> Result<Self, JsValue> {
        set_attribute(&self.element, "id", id)?;
        Ok(self)
    }
    
    /// Establecer text content
    pub fn text(mut self, text: &str) -> Self {
        set_text_content(&self.element, text);
        self
    }
    
    /// Establecer inner HTML
    pub fn html(mut self, html: &str) -> Self {
        self.element.set_inner_html(html);
        self
    }
    
    /// Agregar hijo
    pub fn child(mut self, child: Element) -> Result<Self, JsValue> {
        append_child(&self.element, &child)?;
        Ok(self)
    }
    
    /// Establecer atributo
    pub fn attr(mut self, name: &str, value: &str) -> Result<Self, JsValue> {
        set_attribute(&self.element, name, value)?;
        Ok(self)
    }
    
    /// Construir y retornar elemento
    pub fn build(self) -> Element {
        self.element
    }
}

