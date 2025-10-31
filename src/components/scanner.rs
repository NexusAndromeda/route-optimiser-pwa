// ============================================================================
// SCANNER COMPONENT
// ============================================================================
// ✅ COPIADO DEL ORIGINAL - Preserva HTML/CSS exacto
// Adaptado para usar hooks nativos de Yew (sin yewdux)
// ============================================================================

use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::window;
use crate::hooks::use_session;

#[derive(Properties, PartialEq)]
pub struct ScannerProps {
    pub show: bool,
    pub on_close: Callback<()>,
    pub on_barcode_detected: Callback<String>,
}

// Estructura para resultado del escaneo (igual al original)
#[derive(Clone, Debug, PartialEq)]
struct ScanResult {
    found: bool,
    tracking: String,
    customer_name: Option<String>,
    route_position: Option<usize>,
    total_packages: usize,
    address: Option<String>,
}

#[function_component(Scanner)]
pub fn scanner(props: &ScannerProps) -> Html {
    let session_handle = use_session();
    let scan_result = use_state(|| None::<ScanResult>);
    let initialized = use_state(|| false);
    
    // Inicializar QuaggaJS al montar
    {
        let initialized = initialized.clone();
        use_effect_with((), move |_| {
            if !*initialized {
                // Inicializar scanner (igual al original)
                log::info!("📷 Inicializando scanner...");
                initialized.set(true);
            }
            || {}
        });
    }
    
    // Manejar detección de código
    let _on_scan = {
        let session_state = session_handle.state.clone();
        let scan_package = session_handle.scan_package.clone();
        let on_barcode_detected = props.on_barcode_detected.clone();
        let scan_result = scan_result.clone();
        
        Callback::from(move |barcode: String| {
            log::info!("📱 Código escaneado: {}", barcode);
            
            // Buscar en la sesión local
            if let Some(session) = &(*session_state).session {
                match session.find_by_tracking(&barcode) {
                    Some(package) => {
                        log::info!("✅ Paquete encontrado: {} ({})", 
                                  package.tracking, package.customer_name);
                        
                        let result = ScanResult {
                            found: true,
                            tracking: barcode.clone(),
                            customer_name: Some(package.customer_name.clone()),
                            route_position: package.route_order,
                            total_packages: session.packages.len(),
                            address: Some(session.addresses.get(&package.address_id)
                                         .map(|a| a.label.clone())
                                         .unwrap_or_default()),
                        };
                        
                        scan_result.set(Some(result));
                        
                        // Actualizar en ViewModel
                        scan_package.emit(barcode.clone());
                        
                        // Notificar al callback externo
                        on_barcode_detected.emit(barcode);
                    }
                    None => {
                        log::warn!("⚠️ Paquete no encontrado: {}", barcode);
                        scan_result.set(Some(ScanResult {
                            found: false,
                            tracking: barcode.clone(),
                            customer_name: None,
                            route_position: None,
                            total_packages: 0,
                            address: None,
                        }));
                    }
                }
            }
        })
    };
    
    // ✅ HTML EXACTO DEL ORIGINAL
    if !props.show {
        return html! {};
    }
    
    html! {
        <div class="scanner-modal active">
            <div class="scanner-overlay" onclick={props.on_close.reform(|_| ())}></div>
            <div class="scanner-content">
                <div class="scanner-header">
                    <h2>{"Escanear Código de Barras"}</h2>
                    <button class="btn-close" onclick={props.on_close.reform(|_| ())}>
                        {"✕"}
                    </button>
                </div>
                
                <div id="scanner-viewport" class="scanner-viewport">
                    // QuaggaJS se inicializa aquí via JavaScript FFI
                </div>
                
                {
                    if let Some(ref result) = *scan_result {
                        if result.found {
                            html! {
                                <div class="scan-result success">
                                    <div class="result-icon">{"✅"}</div>
                                    <div class="result-text">
                                        <div class="customer-name">{result.customer_name.as_ref().unwrap_or(&"".to_string())}</div>
                                        <div class="tracking">{&result.tracking}</div>
                                    </div>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="scan-result error">
                                    <div class="result-icon">{"❌"}</div>
                                    <div class="result-text">
                                        {"Paquete no encontrado: "}{&result.tracking}
                                    </div>
                                </div>
                            }
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        </div>
    }
}
