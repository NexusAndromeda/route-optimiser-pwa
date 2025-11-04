// ============================================================================
// BARCODE SCANNER FFI - Foreign Function Interface para JavaScript
// ============================================================================
// Wrappers para funciones JS de QuaggaJS - Sin estado, sin lógica
// ============================================================================

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = initBarcodeScanner)]
    pub fn init_barcode_scanner(
        container_id: &str,
        on_barcode_detected: &js_sys::Function,
        on_error: &js_sys::Function,
    );
    
    #[wasm_bindgen(js_name = initBarcodeScannerWithReady)]
    pub fn init_barcode_scanner_with_ready(
        container_id: &str,
        on_barcode_detected: &js_sys::Function,
        on_error: &js_sys::Function,
        on_ready: &js_sys::Function,
    );
    
    #[wasm_bindgen(js_name = stopBarcodeScanner)]
    pub fn stop_barcode_scanner();
    
    #[wasm_bindgen(js_name = showScannerError)]
    pub fn show_scanner_error();
    
    #[wasm_bindgen(js_name = hideScannerError)]
    pub fn hide_scanner_error();
}

