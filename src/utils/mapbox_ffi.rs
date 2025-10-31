// ============================================================================
// MAPBOX FFI - Foreign Function Interface para JavaScript
// ============================================================================
// Solo wrappers para funciones JS - Sin estado, sin lógica
// ============================================================================

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = initMapbox)]
    pub fn init_mapbox(container_id: &str, is_dark: bool);
    
    #[wasm_bindgen(js_name = addPackagesToMap)]
    pub fn add_packages_to_map(packages_json: &str);
    
    #[wasm_bindgen(js_name = updateSelectedPackage)]
    pub fn update_selected_package(selected_index: i32);
}

/// Helper: Centrar mapa en paquete
pub fn center_map_on_package(index: usize) {
    if let Some(window) = web_sys::window() {
        let function = js_sys::Function::new_no_args(&format!(
            "if (window.centerMapOnPackage) window.centerMapOnPackage({});",
            index
        ));
        let _ = function.call0(&window.into());
    }
}

/// Helper: Scroll a paquete seleccionado
pub fn scroll_to_selected_package(index: usize) {
    if let Some(window) = web_sys::window() {
        let function = js_sys::Function::new_no_args(&format!(
            "if (window.scrollToSelectedPackage) window.scrollToSelectedPackage({});",
            index
        ));
        let _ = function.call0(&window.into());
    }
}

