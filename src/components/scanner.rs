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
use crate::utils::barcode_ffi;

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
    let is_scanning = use_state(|| false);
    let is_initializing = use_state(|| false);
    
    // Inicializar/detener QuaggaJS cuando se muestra/oculta el scanner
    {
        let show = props.show;
        let scan_result = scan_result.clone();
        let session_state = session_handle.state.clone();
        let scan_package = session_handle.scan_package.clone();
        let on_barcode_detected = props.on_barcode_detected.clone();
        let is_scanning = is_scanning.clone();
        let is_initializing = is_initializing.clone();
        
        use_effect_with(show, move |is_showing| {
            if *is_showing {
                log::info!("📷 Inicializando scanner QuaggaJS...");
                is_initializing.set(true);
                is_scanning.set(false);
                
                // Clonar estados para usar en múltiples closures
                let is_scanning_detected = is_scanning.clone();
                let is_scanning_error = is_scanning.clone();
                let is_scanning_ready = is_scanning.clone();
                let is_initializing_error = is_initializing.clone();
                let is_initializing_ready = is_initializing.clone();
                
                // Callback cuando se detecta un código
                let on_detected = Closure::wrap(Box::new(move |barcode: JsValue| {
                    is_scanning_detected.set(false);
                    let barcode_str = barcode.as_string().unwrap_or_default();
                    
                    // ═══════════════════════════════════════════════════════════════
                    // LOGS DETALLADOS DE DEBUGGING
                    // ═══════════════════════════════════════════════════════════════
                    log::info!("📱 [SCANNER] Código escaneado: '{}'", barcode_str);
                    log::info!("📱 [SCANNER] Longitud: {} caracteres", barcode_str.len());
                    log::info!("📱 [SCANNER] Bytes: {:?}", barcode_str.as_bytes());
            
            // Buscar en la sesión local
            if let Some(session) = &(*session_state).session {
                        log::info!("📦 [SCANNER] Sesión activa con {} paquetes", session.packages.len());
                        
                        // Listar primeros trackings disponibles para debugging
                        let trackings_list: Vec<_> = session.packages.keys().take(10).collect();
                        log::info!("🔍 [SCANNER] Primeros {} trackings disponibles:", trackings_list.len());
                        for (idx, tracking) in trackings_list.iter().enumerate() {
                            log::info!("  [{}] '{}' (len: {}, bytes: {:?})", 
                                      idx + 1, tracking, tracking.len(), tracking.as_bytes());
                        }
                        
                        // Buscar trackings que contengan el substring escaneado
                        let matching_substring: Vec<_> = session.packages.keys()
                            .filter(|t| t.contains(&barcode_str) || barcode_str.contains(t.as_str()))
                            .take(5)
                            .collect();
                        if !matching_substring.is_empty() {
                            log::warn!("💡 [SCANNER] Trackings con substring coincidente: {:?}", matching_substring);
                        }
                        
                        // Comparación byte-by-byte si hay tracking similar
                        for tracking in session.packages.keys() {
                            if tracking.len() == barcode_str.len() {
                                let diff_bytes: Vec<_> = tracking.as_bytes().iter()
                                    .zip(barcode_str.as_bytes().iter())
                                    .enumerate()
                                    .filter(|(_, (a, b))| a != b)
                                    .collect();
                                if !diff_bytes.is_empty() && diff_bytes.len() <= 3 {
                                    log::warn!("🔍 [SCANNER] Tracking similar encontrado: '{}'", tracking);
                                    log::warn!("🔍 [SCANNER] Diferencias en bytes: {:?}", diff_bytes);
                                }
                            }
                        }
                        
                        match session.find_by_tracking(&barcode_str) {
                    Some(package) => {
                                log::info!("✅ [SCANNER] Paquete encontrado: {} ({})", 
                                  package.tracking, package.customer_name);
                                
                                // ⭐ DETENER scanner cuando encuentra paquete exitosamente
                                barcode_ffi::stop_barcode_scanner();
                        
                        let result = ScanResult {
                            found: true,
                                    tracking: barcode_str.clone(),
                            customer_name: Some(package.customer_name.clone()),
                            route_position: package.route_order,
                            total_packages: session.packages.len(),
                            address: Some(session.addresses.get(&package.address_id)
                                         .map(|a| a.label.clone())
                                         .unwrap_or_default()),
                        };
                        
                        scan_result.set(Some(result));
                        
                        // Actualizar en ViewModel
                                scan_package.emit(barcode_str.clone());
                        
                        // Notificar al callback externo
                                on_barcode_detected.emit(barcode_str);
                    }
                    None => {
                                log::warn!("⚠️ [SCANNER] Paquete no encontrado: '{}'", barcode_str);
                                
                                // ⭐ NO DETENER scanner - solo mostrar error visual
                                barcode_ffi::show_scanner_error();
                                
                                // NO establecer scan_result para que siga escaneando
                                // El error visual (borde rojo) se mostrará por 2.5 segundos automáticamente
                            }
                        }
                    } else {
                        log::warn!("⚠️ [SCANNER] No hay sesión activa");
                        
                        // ⭐ NO DETENER scanner - solo mostrar error visual
                        barcode_ffi::show_scanner_error();
                        
                        // NO establecer scan_result para que siga escaneando
                    }
                }) as Box<dyn FnMut(JsValue)>);
                
                // Callback para errores
                let on_error = Closure::wrap(Box::new(move |error: JsValue| {
                    log::error!("❌ Error en scanner QuaggaJS: {:?}", error);
                    is_initializing_error.set(false);
                    is_scanning_error.set(false);
                }) as Box<dyn FnMut(JsValue)>);
                
                // Callback cuando QuaggaJS está listo y escaneando
                let on_ready = Closure::wrap(Box::new(move |_ready: JsValue| {
                    log::info!("✅ Scanner QuaggaJS listo y escaneando");
                    is_initializing_ready.set(false);
                    is_scanning_ready.set(true);
                }) as Box<dyn FnMut(JsValue)>);
                
                // Inicializar QuaggaJS con delay para asegurar que el DOM esté listo
                wasm_bindgen_futures::spawn_local(async move {
                    // Pequeño delay para asegurar que el contenedor esté en el DOM
                    gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                    
                    barcode_ffi::init_barcode_scanner_with_ready(
                        "scanner-viewport",
                        on_detected.as_ref().unchecked_ref(),
                        on_error.as_ref().unchecked_ref(),
                        on_ready.as_ref().unchecked_ref(),
                    );
                    
                    // Mantener closures vivas
                    on_detected.forget();
                    on_error.forget();
                    on_ready.forget();
                });
            } else {
                log::info!("📷 Deteniendo scanner QuaggaJS...");
                barcode_ffi::stop_barcode_scanner();
                // Limpiar resultado cuando se cierra
                scan_result.set(None);
                is_scanning.set(false);
                is_initializing.set(false);
                    }
            
            || {}
        });
    }
    
    // ✅ HTML EXACTO DEL ORIGINAL
    if !props.show {
        return html! {};
    }
    
    html! {
        <div class="scanner-modal active">
            <div class="scanner-overlay" onclick={{
                let on_close = props.on_close.clone();
                Callback::from(move |_| {
                    // Asegurar que se detiene la cámara al cerrar
                    barcode_ffi::stop_barcode_scanner();
                    on_close.emit(());
                })
            }}></div>
            <div class="scanner-content">
                <div class="scanner-header">
                    <h2>{"Escanear Código de Barras"}</h2>
                    <button class="btn-close" onclick={{
                        let on_close = props.on_close.clone();
                        Callback::from(move |_| {
                            // Asegurar que se detiene la cámara al cerrar con X
                            barcode_ffi::stop_barcode_scanner();
                            on_close.emit(());
                        })
                    }}>
                        {"✕"}
                    </button>
                </div>
                
                <div id="scanner-viewport" class="scanner-viewport">
                    {
                        if *is_initializing {
                            html! {
                                <div class="scanner-loading">
                                    <div class="loading-spinner"></div>
                                    <div class="loading-text">{"Inicializando cámara..."}</div>
                                </div>
                            }
                        } else if *is_scanning {
                            html! {
                                <div class="scanner-overlay-indicator">
                                    <div class="scanning-indicator">
                                        <div class="scanning-dot"></div>
                                        <div class="scanning-text">{"Escaneando..."}</div>
                                    </div>
                                    <div class="scanning-frame"></div>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
                
                {
                    if let Some(ref result) = *scan_result {
                        if result.found {
                            html! {
                                <div class="scan-result success">
                                    <div class="result-icon">{"✅"}</div>
                                    <div class="result-text">
                                        <div class="tracking-large">{&result.tracking}</div>
                                        {
                                            if let Some(route_order) = result.route_position {
                                                html! {
                                                    <div class="route-order-large">{"Posición: "}{route_order}</div>
                                                }
                                            } else {
                                                html! {
                                                    <div class="route-order-large">{"No optimizado aún"}</div>
                                                }
                                            }
                                        }
                                        <div class="customer-name">{result.customer_name.as_ref().unwrap_or(&"".to_string())}</div>
                                    </div>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="scan-result error">
                                    <div class="result-icon">{"❌"}</div>
                                    <div class="result-text">
                                        <div class="tracking-large">{"Paquete no encontrado"}</div>
                                        <div class="tracking">{&result.tracking}</div>
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
