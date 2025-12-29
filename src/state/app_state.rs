// ============================================================================
// APP STATE - Estado global de la aplicación
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;
use crate::state::{SessionState, AuthState, SyncStateWrapper};
use crate::views::PackageGroup;
use crate::dom::incremental::ModalType;
use web_sys;

/// Tipo de actualización del DOM
#[derive(Clone, Debug)]
pub enum UpdateType {
    /// Actualización incremental (solo elementos específicos)
    Incremental(IncrementalUpdate),
    /// Re-render completo (casos especiales: login/logout, cambio de sesión)
    FullRender,
}

/// Tipo de actualización incremental específica
#[derive(Clone, Debug)]
pub enum IncrementalUpdate {
    /// Actualizar bottom sheet (clases CSS y altura del mapa)
    BottomSheet,
    /// Actualizar selección de paquete (clases de cards)
    PackageSelection,
    /// Actualizar visibilidad de modal
    Modal(ModalType),
    /// Actualizar header (botones disabled/enabled)
    Header,
    /// Actualizar package list (re-renderizar solo la lista)
    PackageList,
    /// Actualizar sync indicator
    SyncIndicator,
    /// Actualizar paquetes en el mapa (sin destruir el mapa)
    MapPackages,
}

/// Estado global de la aplicación
#[derive(Clone)]
pub struct AppState {
    pub session: SessionState,
    pub auth: AuthState,
    pub sync: SyncStateWrapper,
    
    // UI State
    pub selected_package_index: Rc<RefCell<Option<usize>>>,
    pub map_enabled: Rc<RefCell<bool>>,
    pub edit_mode: Rc<RefCell<bool>>,
    pub filter_mode: Rc<RefCell<bool>>,
    pub language: Rc<RefCell<String>>,
    
    // Bottom Sheet State
    pub sheet_state: Rc<RefCell<String>>, // "collapsed" | "half" | "full"
    pub package_list_scroll_position: Rc<RefCell<Option<f64>>>, // Posición de scroll del contenedor .package-list
    
    // Edit Mode State
    pub edit_origin_idx: Rc<RefCell<Option<usize>>>, // Índice del paquete/grupo seleccionado como origen
    
    // Expanded Groups State
    pub expanded_groups: Rc<RefCell<HashSet<usize>>>, // Índices de grupos expandidos
    
    // Memoized Groups (cached para evitar re-agrupar)
    pub groups_memo: Rc<RefCell<Option<Vec<PackageGroup>>>>,
    
    // UI Visibility
    pub show_settings: Rc<RefCell<bool>>,
    pub show_scanner: Rc<RefCell<bool>>,
    pub show_details: Rc<RefCell<bool>>,
    pub show_tracking_modal: Rc<RefCell<bool>>,
    
    // Details Modal State (paquete y dirección para el modal de detalles)
    pub details_package: Rc<RefCell<Option<(crate::models::package::Package, crate::models::address::Address)>>>,
    
    // Estados de edición para Details Modal
    pub editing_address: Rc<RefCell<bool>>,
    pub editing_door_code: Rc<RefCell<bool>>,
    pub editing_driver_notes: Rc<RefCell<bool>>,
    pub address_input_value: Rc<RefCell<String>>,
    pub door_code_input_value: Rc<RefCell<String>>,
    pub driver_notes_input_value: Rc<RefCell<String>>,
    pub saving_address: Rc<RefCell<bool>>,
    pub saving_door_code: Rc<RefCell<bool>>,
    pub saving_mailbox: Rc<RefCell<bool>>,
    pub saving_driver_notes: Rc<RefCell<bool>>,
    pub edit_error_message: Rc<RefCell<Option<String>>>,
    
    // Reactivity: Callbacks para notificar cambios (usamos Rc para poder compartir)
    pub change_subscribers: Rc<RefCell<Vec<Rc<dyn Fn()>>>>,
}

impl AppState {
    /// Crear nuevo estado de aplicación
    pub fn new() -> Self {
        // Cargar preferencias desde localStorage
        let map_enabled = Self::load_bool_pref("map_enabled", true);
        let language = Self::load_string_pref("language", "FR".to_string());
        
        Self {
            session: SessionState::new(),
            auth: AuthState::new(),
            sync: SyncStateWrapper::new(),
            
            selected_package_index: Rc::new(RefCell::new(None)),
            map_enabled: Rc::new(RefCell::new(map_enabled)),
            edit_mode: Rc::new(RefCell::new(false)),
            filter_mode: Rc::new(RefCell::new(false)),
            language: Rc::new(RefCell::new(language)),
            
            sheet_state: Rc::new(RefCell::new("half".to_string())), // Por defecto "half"
            package_list_scroll_position: Rc::new(RefCell::new(None)),
            edit_origin_idx: Rc::new(RefCell::new(None)),
            expanded_groups: Rc::new(RefCell::new(HashSet::new())),
            groups_memo: Rc::new(RefCell::new(None)),
            
            show_settings: Rc::new(RefCell::new(false)),
            show_scanner: Rc::new(RefCell::new(false)),
            show_details: Rc::new(RefCell::new(false)),
            show_tracking_modal: Rc::new(RefCell::new(false)),
            
            details_package: Rc::new(RefCell::new(None)),
            
            // Estados de edición para Details Modal (inicializados en false/vacío)
            editing_address: Rc::new(RefCell::new(false)),
            editing_door_code: Rc::new(RefCell::new(false)),
            editing_driver_notes: Rc::new(RefCell::new(false)),
            address_input_value: Rc::new(RefCell::new(String::new())),
            door_code_input_value: Rc::new(RefCell::new(String::new())),
            driver_notes_input_value: Rc::new(RefCell::new(String::new())),
            saving_address: Rc::new(RefCell::new(false)),
            saving_door_code: Rc::new(RefCell::new(false)),
            saving_mailbox: Rc::new(RefCell::new(false)),
            saving_driver_notes: Rc::new(RefCell::new(false)),
            edit_error_message: Rc::new(RefCell::new(None)),
            
            change_subscribers: Rc::new(RefCell::new(Vec::new())),
        }
    }
    
    /// Cargar preferencia booleana desde localStorage
    fn load_bool_pref(key: &str, default: bool) -> bool {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(value)) = storage.get_item(key) {
                    return value == "true";
                }
            }
        }
        default
    }
    
    /// Cargar preferencia string desde localStorage
    fn load_string_pref(key: &str, default: String) -> String {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(value)) = storage.get_item(key) {
                    return value;
                }
            }
        }
        default
    }
    
    /// Guardar preferencia booleana en localStorage
    pub fn save_bool_pref(&self, key: &str, value: bool) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(key, &value.to_string());
            }
        }
    }
    
    /// Guardar preferencia string en localStorage
    pub fn save_string_pref(&self, key: &str, value: &str) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(key, value);
            }
        }
    }
    
    /// Establecer map_enabled y guardar en localStorage
    pub fn set_map_enabled(&self, enabled: bool) {
        *self.map_enabled.borrow_mut() = enabled;
        self.save_bool_pref("map_enabled", enabled);
    }
    
    /// Establecer language y guardar en localStorage
    pub fn set_language(&self, lang: String) {
        *self.language.borrow_mut() = lang.clone();
        self.save_string_pref("language", &lang);
    }
    
    /// Suscribirse a cambios de estado crítico
    pub fn subscribe_to_changes<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.change_subscribers.borrow_mut().push(Rc::new(callback));
    }
    
    /// Notificar a todos los subscribers de cambios (versión antigua, mantiene compatibilidad)
    pub fn notify_subscribers(&self) {
        self.notify_subscribers_with_type(UpdateType::FullRender);
    }
    
    /// Notificar a todos los subscribers de cambios con tipo específico
    pub fn notify_subscribers_with_type(&self, update_type: UpdateType) {
        // Enviar tipo de actualización a los subscribers
        // Los subscribers decidirán si hacer actualización incremental o completa
        for callback in self.change_subscribers.borrow().iter() {
            callback();
        }
        
        // También ejecutar actualización incremental directamente si corresponde
        match update_type {
            UpdateType::Incremental(inc_type) => {
                // La actualización incremental se manejará en app.rs
                // Aquí solo guardamos el tipo para que app.rs lo use
            }
            UpdateType::FullRender => {
                // Re-render completo se maneja en app.rs vía rerender_app()
            }
        }
    }
    
    /// Establecer sheet_state y actualizar incrementalmente
    pub fn set_sheet_state(&self, state: String) {
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("🔵 [SET-SHEET-STATE] Llamado con estado: {}", state)));
        *self.sheet_state.borrow_mut() = state.clone();
        self.save_string_pref("sheet_state", &state);
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("🔵 [SET-SHEET-STATE] Llamando a rerender_app_with_type(BottomSheet)"));
        // Actualización incremental del bottom sheet
        crate::rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::BottomSheet));
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("✅ [SET-SHEET-STATE] rerender_app_with_type completado"));
    }
    
    /// Establecer edit_mode y limpiar edit_origin_idx si se desactiva
    pub fn set_edit_mode(&self, enabled: bool) {
        *self.edit_mode.borrow_mut() = enabled;
        if !enabled {
            *self.edit_origin_idx.borrow_mut() = None;
        }
        // No necesita actualización visual inmediata, solo cambio de estado interno
    }
    
    /// Establecer filter_mode y invalidar groups_memo
    pub fn set_filter_mode(&self, enabled: bool) {
        *self.filter_mode.borrow_mut() = enabled;
        // Invalidar memo cuando cambia el filtro
        *self.groups_memo.borrow_mut() = None;
        // Actualización incremental: re-renderizar package list y mapa
        crate::rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::PackageList));
        if let Some(_session) = self.session.get_session() {
            crate::rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::MapPackages));
        }
    }
    
    /// Establecer selected_package_index y actualizar incrementalmente
    pub fn set_selected_package_index(&self, index: Option<usize>) {
        *self.selected_package_index.borrow_mut() = index;
        // Actualización incremental de selección
        crate::rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::PackageSelection));
    }
    
    /// Invalidar groups_memo (cuando cambia la sesión)
    pub fn invalidate_groups_memo(&self) {
        *self.groups_memo.borrow_mut() = None;
    }
    
    /// Establecer show_settings y actualizar modal incrementalmente
    pub fn set_show_settings(&self, show: bool) {
        *self.show_settings.borrow_mut() = show;
        crate::rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::Modal(ModalType::Settings)));
    }
    
    /// Establecer show_scanner y actualizar modal incrementalmente
    pub fn set_show_scanner(&self, show: bool) {
        *self.show_scanner.borrow_mut() = show;
        crate::rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::Modal(ModalType::Scanner)));
    }
    
    /// Establecer show_details y actualizar modal incrementalmente usando manipulación directa del DOM
    /// NO hace re-render completo, preserva el scroll del package-list
    pub fn set_show_details(&self, show: bool) {
        *self.show_details.borrow_mut() = show;
        // Si estamos cerrando el modal, resetear estados de edición
        if !show {
            self.reset_edit_states();
        }
        // SIEMPRE usar actualización incremental con manipulación directa del DOM
        // Esto preserva el scroll y evita el "salto" visual
        crate::rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::Modal(ModalType::Details)));
    }
    
    /// Guardar posición de scroll del contenedor .package-list
    pub fn save_package_list_scroll_position(&self) {
        use crate::dom::query_selector;
        use wasm_bindgen::JsCast;
        use web_sys::HtmlElement;
        use wasm_bindgen::JsValue;
        
        if let Ok(Some(package_list)) = query_selector(".package-list") {
            if let Ok(list_html) = package_list.dyn_into::<HtmlElement>() {
                let scroll_top = list_html.scroll_top() as f64;
                web_sys::console::log_1(&JsValue::from_str(&format!("💾 [SCROLL] Guardando posición de scroll: {}px", scroll_top)));
                *self.package_list_scroll_position.borrow_mut() = Some(scroll_top);
                web_sys::console::log_1(&JsValue::from_str(&format!("✅ [SCROLL] Posición guardada en estado: {:?}", self.package_list_scroll_position.borrow())));
            } else {
                web_sys::console::warn_1(&JsValue::from_str("⚠️ [SCROLL] No se pudo convertir a HtmlElement para guardar scroll"));
            }
        } else {
            web_sys::console::warn_1(&JsValue::from_str("⚠️ [SCROLL] Contenedor .package-list no encontrado para guardar scroll"));
        }
    }
    
    /// Restaurar posición de scroll del contenedor .package-list usando manipulación DOM directa en Rust
    /// Usa requestAnimationFrame doble para asegurar que el layout esté completo antes de restaurar
    /// Si clear_after_restore es true, limpia la posición guardada después de restaurar (por defecto false)
    pub fn restore_package_list_scroll_position(&self, clear_after_restore: bool) {
        use crate::dom::query_selector;
        use wasm_bindgen::JsCast;
        use web_sys::HtmlElement;
        use wasm_bindgen::JsValue;
        
        let scroll_pos_opt = *self.package_list_scroll_position.borrow();
        if let Some(scroll_pos) = scroll_pos_opt {
            web_sys::console::log_1(&JsValue::from_str(&format!("🔄 [SCROLL] Intentando restaurar posición: {}px (clear_after_restore: {})", scroll_pos, clear_after_restore)));
            
            // Verificar posición actual antes de restaurar
            if let Ok(Some(package_list)) = query_selector(".package-list") {
                if let Ok(list_html) = package_list.dyn_into::<HtmlElement>() {
                    let current_scroll = list_html.scroll_top();
                    web_sys::console::log_1(&JsValue::from_str(&format!("📊 [SCROLL] Posición actual antes de restaurar: {}px", current_scroll)));
                }
            }
            
            // Usar JavaScript con requestAnimationFrame doble para asegurar que el DOM esté completamente renderizado
            let restore_js = format!(r#"
                (function() {{
                    const container = document.querySelector('.package-list');
                    if (container) {{
                        const targetScroll = {};
                        const currentScroll = container.scrollTop;
                        console.log('🔄 [JS] Restaurando scroll - Actual: ' + currentScroll + 'px, Objetivo: ' + targetScroll + 'px');
                        
                        // Doble requestAnimationFrame para asegurar que el layout esté completo
                        requestAnimationFrame(function() {{
                            const scrollAfterFirstRAF = container.scrollTop;
                            console.log('📐 [JS] Primer RAF - Scroll actual: ' + scrollAfterFirstRAF + 'px');
                            requestAnimationFrame(function() {{
                                const scrollBeforeRestore = container.scrollTop;
                                console.log('📐 [JS] Segundo RAF - Scroll antes de restaurar: ' + scrollBeforeRestore + 'px');
                                container.scrollTop = targetScroll;
                                const scrollAfterRestore = container.scrollTop;
                                console.log('✅ [JS] Scroll restaurado - Antes: ' + scrollBeforeRestore + 'px, Después: ' + scrollAfterRestore + 'px, Objetivo: ' + targetScroll + 'px');
                            }});
                        }});
                    }} else {{
                        console.warn('⚠️ [JS] Contenedor .package-list no encontrado para restaurar scroll');
                    }}
                }})();
            "#, scroll_pos);
            
            if let Some(window) = web_sys::window() {
                let function = js_sys::Function::new_no_args(&restore_js);
                let _ = function.call0(&window.into());
                web_sys::console::log_1(&JsValue::from_str("✅ [SCROLL] Función de restauración programada en JavaScript"));
            }
            
            // Solo limpiar la posición guardada si se solicita explícitamente
            if clear_after_restore {
                *self.package_list_scroll_position.borrow_mut() = None;
                web_sys::console::log_1(&JsValue::from_str("🧹 [SCROLL] Posición guardada limpiada del estado"));
            } else {
                web_sys::console::log_1(&JsValue::from_str("💾 [SCROLL] Posición guardada mantenida en estado (para restauración después de cerrar modal)"));
            }
        } else {
            web_sys::console::log_1(&JsValue::from_str("ℹ️ [SCROLL] No hay posición guardada para restaurar"));
        }
    }
    
    /// Resetear todos los estados de edición del Details Modal
    pub fn reset_edit_states(&self) {
        *self.editing_address.borrow_mut() = false;
        *self.editing_door_code.borrow_mut() = false;
        *self.editing_driver_notes.borrow_mut() = false;
        *self.address_input_value.borrow_mut() = String::new();
        *self.door_code_input_value.borrow_mut() = String::new();
        *self.driver_notes_input_value.borrow_mut() = String::new();
        *self.saving_address.borrow_mut() = false;
        *self.saving_door_code.borrow_mut() = false;
        *self.saving_mailbox.borrow_mut() = false;
        *self.saving_driver_notes.borrow_mut() = false;
        *self.edit_error_message.borrow_mut() = None;
    }
    
    /// Inicializar estados de edición con valores actuales de la dirección
    pub fn init_edit_states(&self, address: &crate::models::address::Address) {
        *self.address_input_value.borrow_mut() = address.label.clone();
        *self.door_code_input_value.borrow_mut() = address.door_code.clone().unwrap_or_default();
        *self.driver_notes_input_value.borrow_mut() = address.driver_notes.clone().unwrap_or_default();
    }
    
    /// Establecer show_tracking_modal y actualizar modal incrementalmente
    pub fn set_show_tracking_modal(&self, show: bool) {
        *self.show_tracking_modal.borrow_mut() = show;
        crate::rerender_app_with_type(UpdateType::Incremental(IncrementalUpdate::Modal(ModalType::Tracking)));
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

