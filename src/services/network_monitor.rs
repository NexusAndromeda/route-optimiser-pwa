// ============================================================================
// MONITOR DE ESTADO DE RED
// ============================================================================
// Detecta cambios en la conectividad de red (online/offline)
// para pausar/reanudar sincronización automática
// ============================================================================
// ✅ COPIADO EXACTO DEL ORIGINAL (app/src/services/network_monitor.rs)

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Event};
use js_sys;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NetworkStatus {
    Online,
    Offline,
    Unknown,
}

/// Monitor de estado de red con listeners de eventos
/// Mejorado para prevenir memory leaks: previene múltiples registros de listeners
pub struct NetworkMonitor {
    status: Arc<Mutex<NetworkStatus>>,
    // Flag para prevenir múltiples registros de listeners
    monitoring_started: Arc<Mutex<bool>>,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        let status = Arc::new(Mutex::new(NetworkStatus::Unknown));
        
        // Verificar estado inicial usando js_sys
        if let Some(window) = window() {
            // Acceder a navigator.onLine directamente
            let navigator_obj = js_sys::Reflect::get(&window, &JsValue::from_str("navigator")).ok();
            
            if let Some(nav) = navigator_obj {
                let on_line = js_sys::Reflect::get(&nav, &JsValue::from_str("onLine"))
                    .ok()
                    .and_then(|v| v.as_bool());
                
                if let Some(is_online) = on_line {
                    *status.lock().unwrap() = if is_online {
                        NetworkStatus::Online
                    } else {
                        NetworkStatus::Offline
                    };
                }
            }
        }
        
        Self {
            status,
            monitoring_started: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Iniciar monitoreo de eventos de red
    /// Previene múltiples registros: solo se registra una vez
    pub fn start_monitoring<F>(&mut self, callback: F)
    where
        F: Fn(NetworkStatus) + 'static,
    {
        // Verificar si ya se inició el monitoreo
        {
            let mut started = self.monitoring_started.lock().unwrap();
            if *started {
                log::warn!("⚠️ NetworkMonitor: start_monitoring ya fue llamado, ignorando llamada duplicada");
                return;
            }
            *started = true;
        }
        
        let window = match window() {
            Some(w) => w,
            None => return,
        };
        
        let status = self.status.clone();
        let callback_arc = Arc::new(Mutex::new(callback));
        
        // Listener para evento "online"
        let online_closure = Closure::wrap(Box::new({
            let status = status.clone();
            let callback = callback_arc.clone();
            move |_event: Event| {
                log::info!("🌐 Network: ONLINE");
                *status.lock().unwrap() = NetworkStatus::Online;
                callback.lock().unwrap()(NetworkStatus::Online);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // Listener para evento "offline"
        let offline_closure = Closure::wrap(Box::new({
            let status = status.clone();
            let callback = callback_arc.clone();
            move |_event: Event| {
                log::warn!("📴 Network: OFFLINE");
                *status.lock().unwrap() = NetworkStatus::Offline;
                callback.lock().unwrap()(NetworkStatus::Offline);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // Registrar listeners
        let _ = window.add_event_listener_with_callback(
            "online",
            online_closure.as_ref().unchecked_ref(),
        );
        
        let _ = window.add_event_listener_with_callback(
            "offline",
            offline_closure.as_ref().unchecked_ref(),
        );
        
        // Mantener closures vivos
        // Nota: En Rust WASM, closure.forget() es necesario para mantener el closure vivo.
        // Los listeners globales (window) persisten durante toda la vida de la app,
        // lo cual es el comportamiento deseado.
        online_closure.forget();
        offline_closure.forget();
        
        log::info!("✅ NetworkMonitor: listeners registrados (solo una vez)");
    }
    
    /// Obtener estado actual de red
    pub fn current_status(&self) -> NetworkStatus {
        *self.status.lock().unwrap()
    }
    
    /// Verificar si está online
    pub fn is_online(&self) -> bool {
        matches!(self.current_status(), NetworkStatus::Online)
    }
    
    /// Verificar si está offline
    pub fn is_offline(&self) -> bool {
        matches!(self.current_status(), NetworkStatus::Offline)
    }
    
    /// Registrar callback cuando vuelve la conexión (método simplificado)
    /// Nota: Este método puede llamarse múltiples veces, registrando múltiples listeners.
    /// Para uso único, considera usar start_monitoring en su lugar.
    /// Para prevenir acumulación, verifica si ya existe un listener antes de llamar.
    pub fn on_online<F>(&mut self, callback: F)
    where
        F: Fn() + 'static,
    {
        let window = match window() {
            Some(w) => w,
            None => return,
        };
        
        let callback_arc = Arc::new(Mutex::new(callback));
        let status = self.status.clone();
        
        // Listener para evento "online"
        let online_closure = Closure::wrap(Box::new({
            let status = status.clone();
            let callback = callback_arc.clone();
            move |_event: Event| {
                log::info!("🌐 Network: ONLINE - Ejecutando callback");
                *status.lock().unwrap() = NetworkStatus::Online;
                callback.lock().unwrap()();
            }
        }) as Box<dyn FnMut(Event)>);
        
        // Registrar listener
        let _ = window.add_event_listener_with_callback(
            "online",
            online_closure.as_ref().unchecked_ref(),
        );
        
        // Mantener closure vivo
        // Nota: Para este método simplificado, usamos forget() porque normalmente
        // solo se llama una vez al inicio de la app
        online_closure.forget();
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NetworkMonitor {
    fn drop(&mut self) {
        // Closures se mantienen vivas hasta que se drop el monitor
        log::info!("🔌 Network monitor dropped");
    }
}
