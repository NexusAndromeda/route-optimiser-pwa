// ============================================================================
// INCREMENTAL DOM UPDATES - Actualización incremental del DOM (estilo vanilla JS)
// ============================================================================
// Solo actualiza elementos específicos que cambiaron, sin re-renderizar todo
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlElement};
use wasm_bindgen::JsCast;
use crate::dom::{get_element_by_id, add_class, remove_class, set_attribute};
use crate::state::app_state::AppState;
use crate::views::PackageGroup;
use crate::utils::mapbox_ffi;
use gloo_timers::callback::Timeout;

/// Tipo de modal
#[derive(Clone, Copy, Debug)]
pub enum ModalType {
    Settings,
    Scanner,
    Details,
    Tracking,
}

/// Actualizar bottom sheet incrementalmente (clases CSS y altura del mapa)
/// Funciona tanto para chofer como para admin
pub fn update_bottom_sheet_incremental(state: &AppState) -> Result<(), JsValue> {
    use web_sys::HtmlElement;
    use wasm_bindgen::JsValue;
    
    // Detectar si estamos en modo admin o chofer
    let is_admin = *state.admin_mode.borrow();
    let sheet_state = if is_admin {
        state.admin_sheet_state.borrow().clone()
    } else {
        state.sheet_state.borrow().clone()
    };
    
    web_sys::console::log_1(&JsValue::from_str(&format!("🔵 [BOTTOM-SHEET-UPDATE] ========== INICIANDO ACTUALIZACIÓN ==========")));
    web_sys::console::log_1(&JsValue::from_str(&format!("🔵 [BOTTOM-SHEET-UPDATE] Modo: {} | Estado actual del sheet: '{}'", 
        if is_admin { "ADMIN" } else { "CHOFER" }, sheet_state)));
    
    // 1. Actualizar clases del bottom-sheet
    if let Some(bottom_sheet) = get_element_by_id("bottom-sheet") {
        web_sys::console::log_1(&JsValue::from_str("✅ [BOTTOM-SHEET-UPDATE] Elemento #bottom-sheet encontrado"));
        let _ = bottom_sheet.class_list().remove_1("collapsed");
        let _ = bottom_sheet.class_list().remove_1("half");
        let _ = bottom_sheet.class_list().remove_1("full");
        let result = bottom_sheet.class_list().add_1(&sheet_state);
        match result {
            Ok(_) => {
                web_sys::console::log_1(&JsValue::from_str(&format!("✅ [BOTTOM-SHEET-UPDATE] Clase '{}' agregada al bottom-sheet", sheet_state)));
            }
            Err(e) => {
                web_sys::console::error_1(&JsValue::from_str(&format!("❌ [BOTTOM-SHEET-UPDATE] Error agregando clase: {:?}", e)));
            }
        }
    } else {
        web_sys::console::warn_1(&JsValue::from_str("⚠️⚠️⚠️ [BOTTOM-SHEET-UPDATE] Elemento #bottom-sheet NO encontrado en el DOM"));
    }
    
    // 2. Actualizar backdrop (solo activo cuando no está collapsed)
    // El blur solo debe verse cuando el bottom sheet está abierto (half o full)
    if let Some(backdrop) = get_element_by_id("backdrop") {
        web_sys::console::log_1(&JsValue::from_str("✅ [BOTTOM-SHEET-UPDATE] Elemento #backdrop encontrado"));
        if sheet_state != "collapsed" {
            // Sheet abierto: activar backdrop con blur
            let _ = backdrop.class_list().add_1("active");
            web_sys::console::log_1(&JsValue::from_str("✅ [BOTTOM-SHEET-UPDATE] Backdrop activado (blur visible)"));
        } else {
            // Sheet collapsed: desactivar backdrop inmediatamente (sin blur)
            let _ = backdrop.class_list().remove_1("active");
            web_sys::console::log_1(&JsValue::from_str("✅ [BOTTOM-SHEET-UPDATE] Backdrop desactivado (sin blur, oculto)"));
        }
    } else {
        web_sys::console::warn_1(&JsValue::from_str("⚠️⚠️⚠️ [BOTTOM-SHEET-UPDATE] Elemento #backdrop NO encontrado en el DOM"));
    }
    
    // 3. Actualizar variable CSS --bottom-sheet-height en el elemento raíz (html)
    // Esto actualiza automáticamente la altura del mapa vía CSS
    let bottom_sheet_height = match sheet_state.as_str() {
        "collapsed" => "80px",  // Solo el handle visible
        "half" => "50vh",
        "full" => "85vh",
        _ => "50vh",
    };
    
    web_sys::console::log_1(&JsValue::from_str(&format!("🔵 [BOTTOM-SHEET-UPDATE] Altura calculada para CSS: '{}'", bottom_sheet_height)));
    
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(document_element) = document.document_element() {
                web_sys::console::log_1(&JsValue::from_str("✅ [BOTTOM-SHEET-UPDATE] Elemento raíz <html> encontrado"));
                
                // Obtener estilo actual
                let style_attr = document_element.get_attribute("style").unwrap_or_default();
                web_sys::console::log_1(&JsValue::from_str(&format!("📝 [BOTTOM-SHEET-UPDATE] Estilo actual en <html>: '{}'", style_attr)));
                
                // Limpiar variable CSS antigua y agregar nueva
                let mut style_parts: Vec<String> = style_attr.split(';')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && !s.starts_with("--bottom-sheet-height:"))
                    .collect();
                style_parts.push(format!("--bottom-sheet-height: {}", bottom_sheet_height));
                let new_style = style_parts.join("; ");
                
                web_sys::console::log_1(&JsValue::from_str(&format!("📝 [BOTTOM-SHEET-UPDATE] Nuevo estilo a establecer: '{}'", new_style)));
                
                // Establecer el nuevo estilo
                match document_element.set_attribute("style", &new_style) {
                    Ok(_) => {
                        web_sys::console::log_1(&JsValue::from_str(&format!("✅ [BOTTOM-SHEET-UPDATE] set_attribute('style', ...) exitoso")));
                        
                        // Verificar que se estableció correctamente
                        if let Some(verify_style) = document_element.get_attribute("style") {
                            web_sys::console::log_1(&JsValue::from_str(&format!("📝 [BOTTOM-SHEET-UPDATE] Verificación - Estilo después de establecer: '{}'", verify_style)));
                            
                            if verify_style.contains(&format!("--bottom-sheet-height: {}", bottom_sheet_height)) {
                                web_sys::console::log_1(&JsValue::from_str("✅✅✅ [BOTTOM-SHEET-UPDATE] Variable CSS confirmada en el atributo style"));
                            } else {
                                web_sys::console::warn_1(&JsValue::from_str(&format!("⚠️⚠️⚠️ [BOTTOM-SHEET-UPDATE] Variable CSS NO encontrada. Esperado: '--bottom-sheet-height: {}'", bottom_sheet_height)));
                            }
                        } else {
                            web_sys::console::warn_1(&JsValue::from_str("⚠️⚠️⚠️ [BOTTOM-SHEET-UPDATE] No se pudo verificar el atributo style después de establecerlo"));
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(&JsValue::from_str(&format!("❌❌❌ [BOTTOM-SHEET-UPDATE] Error estableciendo atributo style: {:?}", e)));
                        return Err(e);
                    }
                }
            } else {
                web_sys::console::error_1(&JsValue::from_str("❌❌❌ [BOTTOM-SHEET-UPDATE] No se pudo obtener document_element (document.documentElement)"));
            }
        } else {
            web_sys::console::error_1(&JsValue::from_str("❌❌❌ [BOTTOM-SHEET-UPDATE] No se pudo obtener document (window.document())"));
        }
    } else {
        web_sys::console::error_1(&JsValue::from_str("❌❌❌ [BOTTOM-SHEET-UPDATE] No se pudo obtener window"));
    }
    
    // 4. Forzar resize del mapa después de actualizar la variable CSS
    // El contenedor tiene una transición CSS de 0.3s, así que necesitamos esperar un poco
    // para que el CSS se aplique antes de redimensionar el mapa de Mapbox
    // Nota: La función JavaScript updateMapSizeForBottomSheet ya maneja el timing correctamente
    // (espera a que la transición termine), así que solo necesitamos llamarla después de un pequeño delay
    web_sys::console::log_1(&JsValue::from_str("🔄 [BOTTOM-SHEET-UPDATE] Programando resize del mapa después de actualizar CSS..."));
    
    // Llamar a la función JavaScript que redimensiona el mapa
    // La función JavaScript ya espera adecuadamente a que el CSS se aplique
    if let Some(window) = web_sys::window() {
        let function = js_sys::Function::new_no_args(
            "if (window.updateMapSizeForBottomSheet) { window.updateMapSizeForBottomSheet(); } else { console.warn('⚠️ updateMapSizeForBottomSheet no está disponible'); }"
        );
        match function.call0(&window.into()) {
            Ok(_) => {
                web_sys::console::log_1(&JsValue::from_str("✅ [BOTTOM-SHEET-UPDATE] Llamada a resize del mapa programada"));
            }
            Err(e) => {
                web_sys::console::error_1(&JsValue::from_str(&format!("❌ [BOTTOM-SHEET-UPDATE] Error programando resize: {:?}", e)));
            }
        }
    }
    
    web_sys::console::log_1(&JsValue::from_str("🔵 [BOTTOM-SHEET-UPDATE] ========== ACTUALIZACIÓN COMPLETADA =========="));
    Ok(())
}

/// Actualizar selección de paquete (clases de cards seleccionados)
pub fn update_package_selection(state: &AppState) -> Result<(), JsValue> {
    let selected_idx = *state.selected_package_index.borrow();
    
    // Calcular grupos una sola vez (para verificar si es grupo y para actualizar lista si es necesario)
    let (groups_opt, is_group) = if let Some(idx) = selected_idx {
        if let Some(session) = state.session.get_session() {
            use crate::views::group_packages_by_address;
            use crate::models::package::Package;
            let mut packages: Vec<Package> = session.packages.values().cloned().collect();
            if *state.filter_mode.borrow() {
                packages.retain(|p| p.status.starts_with("STATUT_CHARGER"));
            }
            let groups = group_packages_by_address(packages);
            let is_group = groups.get(idx).map(|g| g.packages.len() > 1).unwrap_or(false);
            (Some((groups, session)), is_group)
        } else {
            (None, false)
        }
    } else {
        (None, false)
    };
    
    let document = crate::dom::document().ok_or_else(|| JsValue::from_str("No document"))?;
    
    // Remover "selected" de todos los cards usando JavaScript
    let remove_selected_js = r#"
        document.querySelectorAll('.package-card').forEach(function(card) {
            card.classList.remove('selected');
        });
    "#;
    let _ = js_sys::eval(remove_selected_js);
    
    // Remover expand handles de cards no seleccionados usando JavaScript
    // Esto evita tener que re-renderizar toda la lista solo para remover expand handles
    let remove_expand_handles_js = r#"
        document.querySelectorAll('.package-card').forEach(function(card) {
            const expandHandle = card.querySelector('.expand-handle');
            if (expandHandle && !card.classList.contains('selected')) {
                expandHandle.remove();
            }
        });
    "#;
    let _ = js_sys::eval(remove_expand_handles_js);
    
    // Actualizar botones "Go" - solo el card seleccionado debe tenerlo
    let update_go_buttons_js = if let Some(idx) = selected_idx {
        format!(r#"
            (function() {{
                const selectedIdx = {};
                // Remover botones "Go" de todos los cards excepto el seleccionado
                document.querySelectorAll('.package-card').forEach(function(card) {{
                    const dataIndex = parseInt(card.getAttribute('data-index') || '-1');
                    const goButton = card.querySelector('.btn-navigate');
                    
                    if (dataIndex === selectedIdx) {{
                        // Este es el card seleccionado, asegurarse de que tenga el botón "Go"
                        if (!goButton) {{
                            const recipientRow = card.querySelector('.package-recipient-row');
                            if (recipientRow) {{
                                const navBtn = document.createElement('button');
                                navBtn.className = 'btn-navigate';
                                navBtn.textContent = 'Go';
                                recipientRow.appendChild(navBtn);
                            }}
                        }}
                    }} else {{
                        // Este NO es el card seleccionado, remover el botón "Go" si existe
                        if (goButton) {{
                            goButton.remove();
                        }}
                    }}
                }});
            }})();
        "#, idx)
    } else {
        r#"
            (function() {
                // No hay selección, remover todos los botones "Go"
                document.querySelectorAll('.btn-navigate').forEach(function(btn) {
                    btn.remove();
                });
            })();
        "#.to_string()
    };
    let _ = js_sys::eval(&update_go_buttons_js);
    
    // Agregar "selected" al card actual usando data-index
    if let Some(idx) = selected_idx {
        let selector = format!("[data-index=\"{}\"]", idx);
        if let Ok(Some(selected_card)) = document.query_selector(&selector) {
            let _ = selected_card.class_list().add_1("selected");
            
            // Solo hacer scroll si NO hay una posición guardada que necesita restaurarse
            // Esto evita que el scroll interfiera con la restauración después de cerrar el modal
            let has_saved_scroll = state.package_list_scroll_position.borrow().is_some();
            if has_saved_scroll {
                web_sys::console::log_1(&JsValue::from_str(&format!("⏸️ [SCROLL] Scroll automático evitado para card {} - hay posición guardada que restaurar", idx)));
            } else {
                web_sys::console::log_1(&JsValue::from_str(&format!("📍 [SCROLL] Haciendo scroll automático al card {}", idx)));
                // Scroll mejorado: hacer scroll dentro del contenedor .package-list
                scroll_to_card_in_container(&selected_card)?;
            }
        }
    }
    
    // Actualizar mapa con selección
    if let Some(idx) = selected_idx {
        mapbox_ffi::update_selected_package(idx as i32);
    } else {
        mapbox_ffi::update_selected_package(-1);
    }
    
    // Si el seleccionado es un grupo, agregar el expand handle directamente con JavaScript
    // Esto evita re-renderizar toda la lista y causa el "aplastamiento"
    if is_group {
        if let Some(idx) = selected_idx {
            // Agregar expand handle directamente al card seleccionado usando JavaScript
            // Esto evita tener que re-renderizar toda la lista
            let add_expand_handle_js = format!(r#"
                (function() {{
                    const card = document.querySelector('[data-index="{}"]');
                    if (card && !card.querySelector('.expand-handle')) {{
                        // Verificar que es un grupo (tiene texto "paquetes" en el nombre)
                        const recipient = card.querySelector('.package-recipient');
                        const isGroup = recipient && recipient.textContent.includes('paquetes');
                        
                        if (isGroup) {{
                            const expandHandle = document.createElement('div');
                            expandHandle.className = 'expand-handle pulse';
                            
                            const expandIndicator = document.createElement('div');
                            expandIndicator.className = 'expand-indicator';
                            expandHandle.appendChild(expandIndicator);
                            
                            // Event listener para toggle - usar evento personalizado
                            expandHandle.addEventListener('click', function(e) {{
                                e.stopPropagation();
                                e.preventDefault();
                                console.log('🖱️ [JS] Click en expand handle, index:', {});
                                // Disparar evento personalizado que Rust puede escuchar
                                window.dispatchEvent(new CustomEvent('toggle-expand-group', {{
                                    detail: {{ index: {} }}
                                }}));
                            }});
                            
                            card.appendChild(expandHandle);
                        }}
                    }}
                }})();
            "#, idx, idx, idx);
            let _ = js_sys::eval(&add_expand_handle_js);
            
            // Registrar listener para el evento toggle-expand-group si no existe
            // Esto se hace una sola vez, no cada vez que se selecciona
            let setup_listener_js = r#"
                (function() {
                    if (!window._expandToggleListenerSetup) {
                        window.addEventListener('toggle-expand-group', function(e) {
                            const index = e.detail.index;
                            console.log('🔄 [JS] Evento toggle-expand-group recibido, index:', index);
                            // Llamar a función global de Rust para manejar el toggle
                            if (window.handle_toggle_expand_group) {
                                console.log('✅ [JS] Llamando a handle_toggle_expand_group');
                                window.handle_toggle_expand_group(index);
                            } else {
                                console.error('❌ [JS] handle_toggle_expand_group no está disponible en window');
                            }
                        });
                        window._expandToggleListenerSetup = true;
                        console.log('✅ [JS] Listener para toggle-expand-group configurado');
                    }
                })();
            "#;
            let _ = js_sys::eval(setup_listener_js);
        }
    }
    
    Ok(())
}

/// Actualizar visibilidad de modales
/// Nota: Los modales deben existir en el DOM (se crean en render inicial)
/// Esta función solo muestra/oculta cambiando clases CSS
pub fn update_modal_visibility(modal_type: ModalType, show: bool) -> Result<(), JsValue> {
    let modal_id = match modal_type {
        ModalType::Settings => "settings-popup",
        ModalType::Scanner => "scanner-modal",
        ModalType::Details => "details-modal",
        ModalType::Tracking => "tracking-modal",
    };
    
    if let Some(modal) = get_element_by_id(modal_id) {
        if show {
            let _ = modal.class_list().add_1("show");
            let _ = modal.class_list().add_1("active");
            // Nota: Quagga se inicializa en render_scanner cuando el modal se renderiza
            // (solo se renderiza cuando show_scanner es true)
        } else {
            let _ = modal.class_list().remove_1("show");
            let _ = modal.class_list().remove_1("active");
            
            // Si es el scanner, detener Quagga cuando se oculta
            if matches!(modal_type, ModalType::Scanner) {
                use crate::utils::barcode_ffi;
                log::info!("📷 [INCREMENTAL] Deteniendo QuaggaJS cuando se oculta el modal");
                barcode_ffi::stop_barcode_scanner();
            }
        }
    } else if show {
        // Si el modal no existe y queremos mostrarlo, necesitamos re-render completo
        // Esto solo pasa si el modal nunca se creó (caso edge)
        log::warn!("⚠️ Modal {} no existe en DOM, necesita re-render completo", modal_id);
        return Err(JsValue::from_str(&format!("Modal {} not found", modal_id)));
    }
    
    Ok(())
}

/// Actualizar modal de detalles usando manipulación directa del DOM (Rust puro)
/// Crea el modal si no existe, actualiza contenido si cambió, muestra/oculta sin re-render completo
pub fn update_details_modal_direct(state: &AppState) -> Result<(), JsValue> {
    use crate::dom::{document, query_selector, append_child, remove_child, set_inner_html};
    use crate::views::details_modal::render_details_modal;
    use std::rc::Rc;
    
    let show = *state.show_details.borrow();
    let modal_id = "details-modal";
    
    // Buscar modal existente
    let doc = document().ok_or_else(|| JsValue::from_str("No document"))?;
    let existing_modal = doc.query_selector(&format!("#{}", modal_id))?;
    
    if show {
        // Necesitamos mostrar el modal
        if let Some(details_package) = state.details_package.borrow().as_ref() {
            let (pkg, addr) = details_package;
            
            if existing_modal.is_none() {
                // Modal no existe, crearlo directamente en el DOM
                web_sys::console::log_1(&JsValue::from_str("🔨 [MODAL] Creando modal de detalles directamente en DOM (sin re-render completo)"));
                
                // Buscar el contenedor principal (donde normalmente se renderiza el modal)
                let main_app = doc.query_selector("main")?
                    .or_else(|| doc.query_selector("#app").ok().flatten())
                    .or_else(|| doc.body().map(|b| b.dyn_into::<Element>().ok()).flatten())
                    .ok_or_else(|| JsValue::from_str("No main container found"))?;
                
                // Crear callbacks
                let on_close_details = {
                    let state_clone = state.clone();
                    Rc::new(move || {
                        let state_for_restore = state_clone.clone();
                        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("🔄 [SCROLL] Cerrando modal de detalles, programando restauración de scroll"));
                        state_clone.set_show_details(false);
                        use gloo_timers::callback::Timeout;
                        Timeout::new(200, move || {
                            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("⏰ [SCROLL] Timeout completado, restaurando scroll ahora (y limpiando posición guardada)"));
                            state_for_restore.restore_package_list_scroll_position(true);
                        }).forget();
                    })
                };
                
                // Crear callbacks de edición REALES que envían al backend
                // Obtener session_id de la sesión actual
                let session_id = if *state.admin_mode.borrow() {
                    // En modo admin, usar sesión seleccionada
                    if let Some(session) = state.admin_selected_tournee_session.borrow().as_ref() {
                        session.session_id.clone()
                    } else {
                        return Err(JsValue::from_str("No admin session selected"));
                    }
                } else {
                    // En modo chofer, usar sesión normal
                    if let Some(session) = state.session.get_session() {
                    session.session_id.clone()
                } else {
                    return Err(JsValue::from_str("No session available"));
                    }
                };
                let addr_id = addr.address_id.clone();
                let pkg_tracking = pkg.tracking.clone();
                
                // Callback para editar dirección completa
                let on_edit_address = {
                    let state_clone = state.clone();
                    let session_id_clone = session_id.clone();
                    let addr_id_clone = addr_id.clone();
                    Some(Rc::new(move |new_label: String| {
                        let state_clone = state_clone.clone();
                        let session_id_clone = session_id_clone.clone();
                        let addr_id_clone = addr_id_clone.clone();
                        let new_label = new_label.trim().to_string();
                        
                        if new_label.is_empty() {
                            update_modal_error_message(&state_clone, Some("La dirección no puede estar vacía".to_string()));
                            return;
                        }
                        
                        *state_clone.saving_address.borrow_mut() = true;
                        *state_clone.edit_error_message.borrow_mut() = None;
                        update_modal_error_message(&state_clone, None);
                        
                        wasm_bindgen_futures::spawn_local(async move {
                            use crate::viewmodels::session_viewmodel::SessionViewModel;
                            let vm = SessionViewModel::new();
                            
                            match vm.update_address(&session_id_clone, &addr_id_clone, new_label.clone()).await {
                                Ok(updated_session) => {
                                    // Actualizar sesión
                                    state_clone.session.set_session(Some(updated_session.clone()));
                                    
                                    // Actualizar details_package
                                    let pkg_opt = {
                                        let borrow = state_clone.details_package.borrow();
                                        borrow.clone()
                                    };
                                    if let Some((pkg, _)) = pkg_opt {
                                        if let Some(updated_addr) = updated_session.addresses.get(&addr_id_clone) {
                                            *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                            // Actualizar solo la sección de dirección en el modal
                                            update_modal_address_section(&state_clone, updated_addr);
                                        }
                                    }
                                    
                                    *state_clone.saving_address.borrow_mut() = false;
                                    *state_clone.editing_address.borrow_mut() = false;
                                }
                                Err(e) => {
                                    log::error!("❌ Error actualizando dirección: {}", e);
                                    *state_clone.edit_error_message.borrow_mut() = Some(e.clone());
                                    update_modal_error_message(&state_clone, Some(e));
                                    *state_clone.saving_address.borrow_mut() = false;
                                }
                            }
                        });
                    }) as Rc<dyn Fn(String)>)
                };
                
                // Callback para editar código de puerta
                let on_edit_door_code = {
                    let state_clone = state.clone();
                    let session_id_clone = session_id.clone();
                    let addr_id_clone = addr_id.clone();
                    Some(Rc::new(move |new_code: String| {
                        let state_clone = state_clone.clone();
                        let session_id_clone = session_id_clone.clone();
                        let addr_id_clone = addr_id_clone.clone();
                        let door_code = Some(new_code.trim().to_string());
                        
                        *state_clone.saving_door_code.borrow_mut() = true;
                        *state_clone.edit_error_message.borrow_mut() = None;
                        update_modal_error_message(&state_clone, None);
                        
                        wasm_bindgen_futures::spawn_local(async move {
                            use crate::viewmodels::session_viewmodel::SessionViewModel;
                            let vm = SessionViewModel::new();
                            
                            match vm.update_address_fields(&session_id_clone, &addr_id_clone, door_code, None, None).await {
                                Ok(updated_session) => {
                                    state_clone.session.set_session(Some(updated_session.clone()));
                                    
                                    let pkg_opt = {
                                        let borrow = state_clone.details_package.borrow();
                                        borrow.clone()
                                    };
                                    if let Some((pkg, _)) = pkg_opt {
                                        if let Some(updated_addr) = updated_session.addresses.get(&addr_id_clone) {
                                            *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                            update_modal_door_code_section(&state_clone, updated_addr);
                                        }
                                    }
                                    
                                    *state_clone.saving_door_code.borrow_mut() = false;
                                    *state_clone.editing_door_code.borrow_mut() = false;
                                }
                                Err(e) => {
                                    log::error!("❌ Error actualizando código de puerta: {}", e);
                                    *state_clone.edit_error_message.borrow_mut() = Some(e.clone());
                                    update_modal_error_message(&state_clone, Some(e));
                                    *state_clone.saving_door_code.borrow_mut() = false;
                                }
                            }
                        });
                    }) as Rc<dyn Fn(String)>)
                };
                
                // Callback para editar acceso BAL (mailbox)
                let on_edit_mailbox = {
                    let state_clone = state.clone();
                    let session_id_clone = session_id.clone();
                    let addr_id_clone = addr_id.clone();
                    Some(Rc::new(move |new_value: bool| {
                        let state_clone = state_clone.clone();
                        let session_id_clone = session_id_clone.clone();
                        let addr_id_clone = addr_id_clone.clone();
                        
                        *state_clone.saving_mailbox.borrow_mut() = true;
                        *state_clone.edit_error_message.borrow_mut() = None;
                        update_modal_error_message(&state_clone, None);
                        update_modal_mailbox_section(&state_clone, new_value, true); // Actualizar UI inmediatamente
                        
                        wasm_bindgen_futures::spawn_local(async move {
                            use crate::viewmodels::session_viewmodel::SessionViewModel;
                            let vm = SessionViewModel::new();
                            
                            match vm.update_address_fields(&session_id_clone, &addr_id_clone, None, Some(new_value), None).await {
                                Ok(updated_session) => {
                                    state_clone.session.set_session(Some(updated_session.clone()));
                                    
                                    let pkg_opt = {
                                        let borrow = state_clone.details_package.borrow();
                                        borrow.clone()
                                    };
                                    if let Some((pkg, _)) = pkg_opt {
                                        if let Some(updated_addr) = updated_session.addresses.get(&addr_id_clone) {
                                            *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                            let has_mailbox = updated_addr.mailbox_access.is_some() && updated_addr.mailbox_access.as_ref().unwrap() == "true";
                                            update_modal_mailbox_section(&state_clone, has_mailbox, false);
                                        }
                                    }
                                    
                                    *state_clone.saving_mailbox.borrow_mut() = false;
                                }
                                Err(e) => {
                                    log::error!("❌ Error actualizando acceso BAL: {}", e);
                                    *state_clone.edit_error_message.borrow_mut() = Some(e.clone());
                                    update_modal_error_message(&state_clone, Some(e));
                                    *state_clone.saving_mailbox.borrow_mut() = false;
                                    // Revertir toggle en caso de error
                                    let pkg_opt = {
                                        let borrow = state_clone.details_package.borrow();
                                        borrow.clone()
                                    };
                                    if let Some((_, addr)) = pkg_opt {
                                        let has_mailbox = addr.mailbox_access.is_some() && addr.mailbox_access.as_ref().unwrap() == "true";
                                        update_modal_mailbox_section(&state_clone, has_mailbox, false);
                                    }
                                }
                            }
                        });
                    }) as Rc<dyn Fn(bool)>)
                };
                
                // Callback para editar notas chofer
                let on_edit_driver_notes = {
                    let state_clone = state.clone();
                    let session_id_clone = session_id.clone();
                    let addr_id_clone = addr_id.clone();
                    Some(Rc::new(move |new_notes: String| {
                        let state_clone = state_clone.clone();
                        let session_id_clone = session_id_clone.clone();
                        let addr_id_clone = addr_id_clone.clone();
                        let driver_notes = Some(new_notes.trim().to_string());
                        
                        *state_clone.saving_driver_notes.borrow_mut() = true;
                        *state_clone.edit_error_message.borrow_mut() = None;
                        update_modal_error_message(&state_clone, None);
                        
                        wasm_bindgen_futures::spawn_local(async move {
                            use crate::viewmodels::session_viewmodel::SessionViewModel;
                            let vm = SessionViewModel::new();
                            
                            match vm.update_address_fields(&session_id_clone, &addr_id_clone, None, None, driver_notes).await {
                                Ok(updated_session) => {
                                    state_clone.session.set_session(Some(updated_session.clone()));
                                    
                                    let pkg_opt = {
                                        let borrow = state_clone.details_package.borrow();
                                        borrow.clone()
                                    };
                                    if let Some((pkg, _)) = pkg_opt {
                                        if let Some(updated_addr) = updated_session.addresses.get(&addr_id_clone) {
                                            *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                            update_modal_driver_notes_section(&state_clone, updated_addr);
                                        }
                                    }
                                    
                                    *state_clone.saving_driver_notes.borrow_mut() = false;
                                    *state_clone.editing_driver_notes.borrow_mut() = false;
                                }
                                Err(e) => {
                                    log::error!("❌ Error actualizando notas chofer: {}", e);
                                    *state_clone.edit_error_message.borrow_mut() = Some(e.clone());
                                    update_modal_error_message(&state_clone, Some(e));
                                    *state_clone.saving_driver_notes.borrow_mut() = false;
                                }
                            }
                        });
                    }) as Rc<dyn Fn(String)>)
                };
                
                // Callback para marcar como problemático
                let on_mark_problematic = {
                    let state_clone = state.clone();
                    let session_id_clone = session_id.clone();
                    let addr_id_clone = addr_id.clone();
                    let pkg_tracking_clone = pkg_tracking.clone();
                    Some(Rc::new(move || {
                        let state_clone = state_clone.clone();
                        let session_id_clone = session_id_clone.clone();
                        let addr_id_clone = addr_id_clone.clone();
                        let pkg_tracking_clone = pkg_tracking_clone.clone();
                        
                        state_clone.session.set_loading(true);
                        *state_clone.edit_error_message.borrow_mut() = None;
                        update_modal_error_message(&state_clone, None);
                        
                        wasm_bindgen_futures::spawn_local(async move {
                            use crate::viewmodels::session_viewmodel::SessionViewModel;
                            let vm = SessionViewModel::new();
                            
                            match vm.mark_as_problematic(&session_id_clone, &addr_id_clone).await {
                                Ok(updated_session) => {
                                    state_clone.session.set_session(Some(updated_session.clone()));
                                    
                                    // Buscar el paquete actualizado
                                    if let Some(updated_pkg) = updated_session.packages.get(&pkg_tracking_clone) {
                                        if let Some(updated_addr) = updated_session.addresses.get(&addr_id_clone) {
                                            *state_clone.details_package.borrow_mut() = Some((updated_pkg.clone(), updated_addr.clone()));
                                            update_modal_problematic_section(&state_clone, updated_pkg.is_problematic);
                                        }
                                    }
                                    
                                    state_clone.session.set_loading(false);
                                }
                                Err(e) => {
                                    log::error!("❌ Error marcando como problemático: {}", e);
                                    *state_clone.edit_error_message.borrow_mut() = Some(e.clone());
                                    update_modal_error_message(&state_clone, Some(e));
                                    state_clone.session.set_loading(false);
                                }
                            }
                        });
                    }) as Rc<dyn Fn()>)
                };
                
                // Renderizar modal
                let details_modal = render_details_modal(
                    pkg,
                    addr,
                    state,
                    on_close_details,
                    on_edit_address,
                    on_edit_door_code,
                    on_edit_mailbox,
                    on_edit_driver_notes,
                    on_mark_problematic,
                )?;
                
                // Agregar al DOM
                append_child(&main_app, &details_modal)?;
                web_sys::console::log_1(&JsValue::from_str("✅ [MODAL] Modal creado directamente en DOM"));
            } else {
                // Modal existe - verificar si cambió el paquete/dirección y actualizar contenido si es necesario
                web_sys::console::log_1(&JsValue::from_str("👁️ [MODAL] Modal existe, verificando si necesita actualización"));
                
                // Comparar tracking/address_id para detectar cambios
                if let Some(modal) = existing_modal {
                    let _ = modal.class_list().add_1("show");
                    let _ = modal.class_list().add_1("active");
                    
                    // Actualizar todas las secciones del modal con los nuevos datos
                    // Esto asegura que el contenido esté sincronizado sin recrear el modal completo
                    update_modal_address_section(state, addr);
                    update_modal_door_code_section(state, addr);
                    let has_mailbox = addr.mailbox_access.is_some() && addr.mailbox_access.as_ref().unwrap() == "true";
                    update_modal_mailbox_section(state, has_mailbox, false);
                    update_modal_driver_notes_section(state, addr);
                    update_modal_problematic_section(state, pkg.is_problematic);
                    
                    // Limpiar mensajes de error si existen
                    update_modal_error_message(state, None);
                }
            }
        }
    } else {
        // Ocultar modal
        if let Some(modal) = existing_modal {
            let _ = modal.class_list().remove_1("show");
            let _ = modal.class_list().remove_1("active");
            web_sys::console::log_1(&JsValue::from_str("👁️ [MODAL] Modal ocultado"));
        }
    }
    
    Ok(())
}

/// Actualizar mensaje de error en el modal usando manipulación directa del DOM
pub fn update_modal_error_message(state: &AppState, error_msg: Option<String>) {
    use crate::dom::{get_element_by_id, query_selector, append_child, remove_child, set_text_content, set_attribute};
    use crate::dom::ElementBuilder;
    
    if let Some(modal_body) = query_selector(".modal-body").ok().flatten() {
        // Buscar mensaje de error existente
        let existing_error = query_selector(".error-message").ok().flatten();
        
        if let Some(msg) = error_msg {
            // Mostrar/actualizar mensaje de error
            if let Some(error_div) = existing_error {
                set_text_content(&error_div, &msg);
            } else {
                // Crear nuevo mensaje de error
                if let Ok(error_div) = ElementBuilder::new("div")
                    .map(|b| b.class("error-message").build())
                {
                    let _ = set_attribute(&error_div, "style", "color: red; padding: 10px; margin-bottom: 10px; background: #ffe6e6; border-radius: 4px;");
                    set_text_content(&error_div, &msg);
                    // Insertar al principio del body
                    if let Some(first_child) = modal_body.first_child().and_then(|n| n.dyn_into::<Element>().ok()) {
                        let _ = modal_body.insert_before(&error_div, Some(&first_child));
                    } else {
                        let _ = append_child(&modal_body, &error_div);
                    }
                }
            }
        } else {
            // Ocultar mensaje de error
            if let Some(error_div) = existing_error {
                let _ = remove_child(&modal_body, &error_div);
            }
        }
    }
}

/// Actualizar sección de dirección en el modal usando manipulación directa del DOM
pub fn update_modal_address_section(state: &AppState, addr: &crate::models::address::Address) {
    use crate::dom::{query_selector, set_text_content};
    use crate::utils::i18n::t;
    
    let lang = state.language.borrow().clone();
    let editing = *state.editing_address.borrow();
    
    if !editing {
        // Solo actualizar si no está en modo edición
        // Buscar la sección de dirección específicamente
        if let Ok(Some(_modal_body)) = query_selector(".modal-body") {
            if let Ok(sections) = crate::dom::query_selector_all(".modal-body .detail-section.editable") {
                for i in 0..sections.length() {
                    let section_js = sections.get(i as u32);
                    if !section_js.is_undefined() && !section_js.is_null() {
                        if let Ok(section) = section_js.dyn_into::<Element>() {
                            if let Ok(Some(label)) = section.query_selector(".detail-label") {
                                let label_text = label.text_content().unwrap_or_default();
                                if label_text.contains(&t("adresse", &lang)) || label_text.contains("Dirección") || label_text.contains("Adresse") {
                                    // Esta es la sección de dirección
                                    if let Ok(Some(value_container)) = section.query_selector(".detail-value-with-action") {
                                        if let Ok(Some(span)) = value_container.query_selector("span:not(.empty-value)") {
                                            set_text_content(&span, &addr.label);
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Actualizar sección de código de puerta en el modal usando manipulación directa del DOM
pub fn update_modal_door_code_section(state: &AppState, addr: &crate::models::address::Address) {
    use crate::dom::{query_selector, set_text_content, add_class, remove_class};
    use crate::utils::i18n::t;
    
    let lang = state.language.borrow().clone();
    let editing = *state.editing_door_code.borrow();
    
    if !editing {
        // Buscar todas las secciones editables y encontrar la de código de puerta
        // (la que tiene el label "codes_porte" o "Códigos de puerta")
        if let Ok(Some(_modal_body)) = query_selector(".modal-body") {
            if let Ok(sections) = crate::dom::query_selector_all(".modal-body .detail-section.editable") {
                for i in 0..sections.length() {
                    let section_js = sections.get(i as u32);
                    if !section_js.is_undefined() && !section_js.is_null() {
                        if let Ok(section) = section_js.dyn_into::<Element>() {
                            if let Ok(Some(label)) = section.query_selector(".detail-label") {
                                let label_text = label.text_content().unwrap_or_default();
                                if label_text.contains(&t("codes_porte", &lang)) || label_text.contains("Código") {
                                    // Esta es la sección de código de puerta
                                    if let Ok(Some(value_container)) = section.query_selector(".detail-value-with-action") {
                                        if let Ok(Some(span)) = value_container.query_selector("span") {
                                            if let Some(code) = &addr.door_code {
                                                if !code.is_empty() {
                                                    set_text_content(&span, code);
                                                    remove_class(&span, "empty-value");
                                                } else {
                                                    set_text_content(&span, &t("non_renseigne", &lang));
                                                    add_class(&span, "empty-value");
                                                }
                                            } else {
                                                set_text_content(&span, &t("non_renseigne", &lang));
                                                add_class(&span, "empty-value");
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Actualizar sección de acceso BAL (mailbox) en el modal usando manipulación directa del DOM
pub fn update_modal_mailbox_section(state: &AppState, has_mailbox: bool, saving: bool) {
    use crate::dom::{query_selector, set_text_content, set_attribute, remove_attribute};
    use crate::utils::i18n::t;
    
    let lang = state.language.borrow().clone();
    
    // Buscar la sección de acceso BAL
    if let Ok(Some(_modal_body)) = query_selector(".modal-body") {
        if let Ok(sections) = crate::dom::query_selector_all(".modal-body .detail-section.editable") {
            for i in 0..sections.length() {
                let section_js = sections.get(i as u32);
                if !section_js.is_undefined() && !section_js.is_null() {
                    if let Ok(section) = section_js.dyn_into::<Element>() {
                        if let Ok(Some(label)) = section.query_selector(".detail-label") {
                            let label_text = label.text_content().unwrap_or_default();
                            if label_text.contains(&t("acces_bal", &lang)) || label_text.contains("BAL") {
                                // Esta es la sección de acceso BAL
                                if let Ok(Some(value_container)) = section.query_selector(".detail-value-with-action") {
                                    // Actualizar texto
                                    if let Ok(Some(span)) = value_container.query_selector("span") {
                                        let bal_text = if has_mailbox {
                                            format!("✅ {}", t("oui_capital", &lang))
                                        } else {
                                            format!("❌ {}", t("non_capital", &lang))
                                        };
                                        set_text_content(&span, &bal_text);
                                    }
                                    
                                    // Actualizar toggle
                                    if let Ok(Some(toggle_input)) = value_container.query_selector("input[type='checkbox']") {
                                        if has_mailbox {
                                            let _ = set_attribute(&toggle_input, "checked", "checked");
                                        } else {
                                            let _ = remove_attribute(&toggle_input, "checked");
                                        }
                                        
                                        // Actualizar estado disabled durante guardado
                                        if saving {
                                            let _ = set_attribute(&toggle_input, "disabled", "true");
                                        } else {
                                            let _ = remove_attribute(&toggle_input, "disabled");
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Actualizar sección de notas chofer en el modal usando manipulación directa del DOM
pub fn update_modal_driver_notes_section(state: &AppState, addr: &crate::models::address::Address) {
    use crate::dom::{query_selector, set_text_content, add_class, remove_class};
    use crate::utils::i18n::t;
    
    let lang = state.language.borrow().clone();
    let editing = *state.editing_driver_notes.borrow();
    
    if !editing {
        // Buscar la sección de notas chofer
        if let Ok(Some(modal_body)) = query_selector(".modal-body") {
            if let Ok(sections) = crate::dom::query_selector_all(".modal-body .detail-section.editable") {
                for i in 0..sections.length() {
                    let section_js = sections.get(i as u32);
                    if !section_js.is_undefined() && !section_js.is_null() {
                        if let Ok(section) = section_js.dyn_into::<Element>() {
                            if let Ok(Some(label)) = section.query_selector(".detail-label") {
                                let label_text = label.text_content().unwrap_or_default();
                                if label_text.contains(&t("notes_chauffeur", &lang)) || label_text.contains("chofer") || label_text.contains("chauffeur") {
                                    // Esta es la sección de notas chofer
                                    if let Ok(Some(value_container)) = section.query_selector(".detail-value-with-action") {
                                        if let Ok(Some(span)) = value_container.query_selector("span") {
                                            if let Some(notes) = &addr.driver_notes {
                                                if !notes.is_empty() {
                                                    set_text_content(&span, &format!("\"{}\"", notes));
                                                    remove_class(&span, "empty-value");
                                                } else {
                                                    set_text_content(&span, &t("ajouter_note", &lang));
                                                    add_class(&span, "empty-value");
                                                }
                                            } else {
                                                set_text_content(&span, &t("ajouter_note", &lang));
                                                add_class(&span, "empty-value");
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Actualizar sección de problemático en el modal usando manipulación directa del DOM
pub fn update_modal_problematic_section(state: &AppState, is_problematic: bool) {
    use crate::dom::{query_selector, set_inner_html, remove_child, append_child};
    use crate::dom::ElementBuilder;
    use crate::utils::i18n::t;
    use wasm_bindgen::closure::Closure;
    
    let lang = state.language.borrow().clone();
    
    // Buscar la sección de problemático (última sección del modal)
    if let Ok(Some(_modal_body)) = query_selector(".modal-body") {
        if let Ok(sections) = crate::dom::query_selector_all(".modal-body .detail-section") {
            // La última sección suele ser la de problemático
            if sections.length() > 0 {
                let section_js = sections.get(sections.length() - 1);
                if !section_js.is_undefined() && !section_js.is_null() {
                    if let Ok(section) = section_js.dyn_into::<Element>() {
                        if let Ok(Some(detail_row)) = section.query_selector(".detail-row") {
                        if let Ok(Some(value_el)) = detail_row.query_selector(".detail-value") {
                            // Limpiar contenido anterior
                            crate::dom::set_inner_html(&value_el, "");
                            
                            if is_problematic {
                                // Mostrar badge
                                if let Ok(badge) = ElementBuilder::new("span")
                                    .map(|b| b.class("problematic-badge").text(&t("problematique", &lang)).build())
                                {
                                    let _ = append_child(&value_el, &badge);
                                }
                            } else {
                                // Mostrar botón
                                if let Ok(btn) = ElementBuilder::new("button")
                                    .and_then(|b| b.class("btn-problematic").attr("title", &t("marquer_problematique", &lang)))
                                    .map(|b| b.build())
                                {
                                    let btn_text = format!("⚠️ {}", t("marquer_problematique", &lang));
                                    crate::dom::set_text_content(&btn, &btn_text);
                                    
                                    // Event listener para marcar como problemático
                                    {
                                        let state_clone = state.clone();
                                        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                                            // Obtener session_id y address_id desde details_package
                                            let state_for_async = state_clone.clone();
                                            if let Some((pkg, addr)) = state_for_async.details_package.borrow().as_ref() {
                                                let session_id = if let Some(session) = state_for_async.session.get_session() {
                                                    session.session_id.clone()
                                                } else {
                                                    return;
                                                };
                                                let addr_id = addr.address_id.clone();
                                                let pkg_tracking = pkg.tracking.clone();
                                                
                                                state_for_async.session.set_loading(true);
                                                
                                                let state_for_async_inner = state_for_async.clone();
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    use crate::viewmodels::session_viewmodel::SessionViewModel;
                                                    let vm = SessionViewModel::new();
                                                    
                                                    match vm.mark_as_problematic(&session_id, &addr_id).await {
                                                        Ok(updated_session) => {
                                                            state_for_async_inner.session.set_session(Some(updated_session.clone()));
                                                            
                                                            if let Some(updated_pkg) = updated_session.packages.get(&pkg_tracking) {
                                                                if let Some(updated_addr) = updated_session.addresses.get(&addr_id) {
                                                                    *state_for_async_inner.details_package.borrow_mut() = Some((updated_pkg.clone(), updated_addr.clone()));
                                                                    update_modal_problematic_section(&state_for_async_inner, updated_pkg.is_problematic);
                                                                }
                                                            }
                                                            
                                                            state_for_async_inner.session.set_loading(false);
                                                        }
                                                        Err(e) => {
                                                            log::error!("❌ Error marcando como problemático: {}", e);
                                                            *state_for_async_inner.edit_error_message.borrow_mut() = Some(e.clone());
                                                            update_modal_error_message(&state_for_async_inner, Some(e));
                                                            state_for_async_inner.session.set_loading(false);
                                                        }
                                                    }
                                                });
                                            }
                                        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
                                        
                                        btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).ok();
                                        closure.forget();
                                    }
                                    
                                    let _ = append_child(&value_el, &btn);
                                }
                            }
                        }
                    }
                    }
                }
            }
        }
    }
}

/// Toggle modo edición para dirección usando manipulación directa del DOM
pub fn toggle_edit_mode_address(state: &AppState, enable: bool) {
    use crate::dom::{query_selector, remove_child, append_child, set_attribute, set_text_content};
    use crate::dom::ElementBuilder;
    use crate::utils::i18n::t;
    use wasm_bindgen::closure::Closure;
    
    let lang = state.language.borrow().clone();
    
    // Buscar la sección de dirección
    if let Ok(Some(_modal_body)) = query_selector(".modal-body") {
        if let Ok(sections) = crate::dom::query_selector_all(".modal-body .detail-section.editable") {
            for i in 0..sections.length() {
                let section_js = sections.get(i as u32);
                if !section_js.is_undefined() && !section_js.is_null() {
                    if let Ok(section) = section_js.dyn_into::<Element>() {
                        if let Ok(Some(label)) = section.query_selector(".detail-label") {
                            let label_text = label.text_content().unwrap_or_default();
                            if label_text.contains(&t("adresse", &lang)) || label_text.contains("Dirección") {
                                if let Ok(Some(value_container)) = section.query_selector(".detail-value-with-action") {
                                    if enable {
                                        // Modo edición: crear input
                                        crate::dom::set_inner_html(&value_container, "");
                                        
                                        if let Ok(input_group) = ElementBuilder::new("div")
                                            .map(|b| b.class("edit-input-group").build())
                                        {
                                            if let Ok(input) = crate::dom::create_element("input") {
                                                let _ = set_attribute(&input, "type", "text");
                                                let _ = set_attribute(&input, "class", "edit-input");
                                                let _ = set_attribute(&input, "placeholder", &t("nouvelle_adresse", &lang));
                                                
                                                let current_value = state.address_input_value.borrow().clone();
                                                let _ = set_attribute(&input, "value", &current_value);
                                                
                                                // Event listeners
                                                {
                                                    let state_clone = state.clone();
                                                    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                                                        if let Some(input_el) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                                                            *state_clone.address_input_value.borrow_mut() = input_el.value();
                                                        }
                                                    }) as Box<dyn FnMut(web_sys::Event)>);
                                                    let _ = input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref());
                                                    closure.forget();
                                                }
                                                
                                                {
                                                    let state_clone = state.clone();
                                                    let addr_label = {
                                                        if let Some((_, addr)) = state_clone.details_package.borrow().as_ref() {
                                                            addr.label.clone()
                                                        } else {
                                                            String::new()
                                                        }
                                                    };
                                                    
                                                    // Obtener callback de edición desde el estado (necesitamos acceso a session_id y address_id)
                                                    let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
                                                        if e.key() == "Enter" {
                                                            e.prevent_default();
                                                            let value = state_clone.address_input_value.borrow().trim().to_string();
                                                            if !value.is_empty() {
                                                                // Llamar al callback de edición si existe
                                                                // Por ahora, actualizar directamente
                                                                if let Some((pkg, addr)) = state_clone.details_package.borrow().as_ref() {
                                                                    let session_id = if let Some(session) = state_clone.session.get_session() {
                                                                        session.session_id.clone()
                                                                    } else {
                                                                        return;
                                                                    };
                                                                    let addr_id = addr.address_id.clone();
                                                                    let state_clone = state_clone.clone();
                                                                    
                                                                    *state_clone.saving_address.borrow_mut() = true;
                                                                    
                                                                    wasm_bindgen_futures::spawn_local(async move {
                                                                        use crate::viewmodels::session_viewmodel::SessionViewModel;
                                                                        let vm = SessionViewModel::new();
                                                                        
                                                                        match vm.update_address(&session_id, &addr_id, value.clone()).await {
                                                                            Ok(updated_session) => {
                                                                                state_clone.session.set_session(Some(updated_session.clone()));
                                                                                
                                                                                let pkg_opt = {
                                                                                    let borrow = state_clone.details_package.borrow();
                                                                                    borrow.clone()
                                                                                };
                                                                                if let Some((pkg, _)) = pkg_opt {
                                                                                    if let Some(updated_addr) = updated_session.addresses.get(&addr_id) {
                                                                                        *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                                                                        update_modal_address_section(&state_clone, updated_addr);
                                                                                        toggle_edit_mode_address(&state_clone, false);
                                                                                    }
                                                                                }
                                                                                
                                                                                *state_clone.saving_address.borrow_mut() = false;
                                                                            }
                                                                            Err(e) => {
                                                                                *state_clone.edit_error_message.borrow_mut() = Some(e.clone());
                                                                                update_modal_error_message(&state_clone, Some(e));
                                                                                *state_clone.saving_address.borrow_mut() = false;
                                                                            }
                                                                        }
                                                                    });
                                                                }
                                                            }
                                                        } else if e.key() == "Escape" {
                                                            e.prevent_default();
                                                            *state_clone.address_input_value.borrow_mut() = addr_label.clone();
                                                            toggle_edit_mode_address(&state_clone, false);
                                                        }
                                                    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
                                                    
                                                    let _ = input.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
                                                    closure.forget();
                                                }
                                                
                                                let _ = append_child(&input_group, &input);
                                                let _ = append_child(&value_container, &input_group);
                                                
                                                // Focus en el input
                                                if let Ok(html_input) = input.dyn_into::<web_sys::HtmlInputElement>() {
                                                    let _ = html_input.focus();
                                                }
                                            }
                                        }
                                    } else {
                                        // Modo visualización: mostrar span + botón editar
                                        crate::dom::set_inner_html(&value_container, "");
                                        
                                        let addr_label = {
                                            if let Some((_, addr)) = state.details_package.borrow().as_ref() {
                                                addr.label.clone()
                                            } else {
                                                String::new()
                                            }
                                        };
                                        
                                        if let Ok(span) = ElementBuilder::new("span")
                                            .map(|b| b.text(&addr_label).build())
                                        {
                                            let _ = append_child(&value_container, &span);
                                        }
                                        
                                        if let Ok(edit_btn) = ElementBuilder::new("button")
                                            .and_then(|b| b.class("btn-icon").attr("title", &t("modifier", &lang)))
                                            .map(|b| b.text("⚙️").build())
                                        {
                                            {
                                                let state_clone = state.clone();
                                                let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                                                    let addr_label = {
                                                        if let Some((_, addr)) = state_clone.details_package.borrow().as_ref() {
                                                            addr.label.clone()
                                                        } else {
                                                            String::new()
                                                        }
                                                    };
                                                    *state_clone.address_input_value.borrow_mut() = addr_label.clone();
                                                    toggle_edit_mode_address(&state_clone, true);
                                                }) as Box<dyn FnMut(web_sys::MouseEvent)>);
                                                
                                                let _ = edit_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
                                                closure.forget();
                                            }
                                            
                                            let _ = append_child(&value_container, &edit_btn);
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Toggle modo edición para código de puerta usando manipulación directa del DOM
pub fn toggle_edit_mode_door_code(state: &AppState, enable: bool) {
    use crate::dom::{query_selector, set_inner_html, append_child, set_attribute, set_text_content, add_class, remove_class};
    use crate::dom::ElementBuilder;
    use crate::utils::i18n::t;
    use wasm_bindgen::closure::Closure;
    
    let lang = state.language.borrow().clone();
    
    // Buscar la sección de código de puerta
    if let Ok(Some(_modal_body)) = query_selector(".modal-body") {
        if let Ok(sections) = crate::dom::query_selector_all(".modal-body .detail-section.editable") {
            for i in 0..sections.length() {
                let section_js = sections.get(i as u32);
                if !section_js.is_undefined() && !section_js.is_null() {
                    if let Ok(section) = section_js.dyn_into::<Element>() {
                        if let Ok(Some(label)) = section.query_selector(".detail-label") {
                            let label_text = label.text_content().unwrap_or_default();
                            if label_text.contains(&t("codes_porte", &lang)) || label_text.contains("Código") || label_text.contains("code") {
                                if let Ok(Some(value_container)) = section.query_selector(".detail-value-with-action") {
                                    if enable {
                                        // Modo edición: crear input
                                        set_inner_html(&value_container, "");
                                        
                                        if let Ok(input_group) = ElementBuilder::new("div")
                                            .map(|b| b.class("edit-input-group").build())
                                        {
                                            if let Ok(input) = crate::dom::create_element("input") {
                                                let _ = set_attribute(&input, "type", "text");
                                                let _ = set_attribute(&input, "class", "edit-input");
                                                let _ = set_attribute(&input, "placeholder", &t("code_de_porte", &lang));
                                                
                                                let current_value = state.door_code_input_value.borrow().clone();
                                                let _ = set_attribute(&input, "value", &current_value);
                                                
                                                // Event listener para input
                                                {
                                                    let state_clone = state.clone();
                                                    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                                                        if let Some(input_el) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                                                            *state_clone.door_code_input_value.borrow_mut() = input_el.value();
                                                        }
                                                    }) as Box<dyn FnMut(web_sys::Event)>);
                                                    let _ = input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref());
                                                    closure.forget();
                                                }
                                                
                                                // Event listener para Enter/Escape
                                                {
                                                    let state_clone = state.clone();
                                                    let door_code_clone = {
                                                        if let Some((_, addr)) = state_clone.details_package.borrow().as_ref() {
                                                            addr.door_code.clone().unwrap_or_default()
                                                        } else {
                                                            String::new()
                                                        }
                                                    };
                                                    
                                                    let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
                                                        if e.key() == "Enter" {
                                                            e.prevent_default();
                                                            let value = state_clone.door_code_input_value.borrow().trim().to_string();
                                                            if !value.is_empty() {
                                                                if let Some((pkg, addr)) = state_clone.details_package.borrow().as_ref() {
                                                                    let session_id = if let Some(session) = state_clone.session.get_session() {
                                                                        session.session_id.clone()
                                                                    } else {
                                                                        return;
                                                                    };
                                                                    let addr_id = addr.address_id.clone();
                                                                    let state_clone = state_clone.clone();
                                                                    
                                                                    *state_clone.saving_door_code.borrow_mut() = true;
                                                                    
                                                                    wasm_bindgen_futures::spawn_local(async move {
                                                                        use crate::viewmodels::session_viewmodel::SessionViewModel;
                                                                        let vm = SessionViewModel::new();
                                                                        
                                                                        match vm.update_address_fields(&session_id, &addr_id, Some(value.clone()), None, None).await {
                                                                            Ok(updated_session) => {
                                                                                state_clone.session.set_session(Some(updated_session.clone()));
                                                                                
                                                                                let pkg_opt = {
                                                                                    let borrow = state_clone.details_package.borrow();
                                                                                    borrow.clone()
                                                                                };
                                                                                if let Some((pkg, _)) = pkg_opt {
                                                                                    if let Some(updated_addr) = updated_session.addresses.get(&addr_id) {
                                                                                        *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                                                                        update_modal_door_code_section(&state_clone, updated_addr);
                                                                                        toggle_edit_mode_door_code(&state_clone, false);
                                                                                    }
                                                                                }
                                                                                
                                                                                *state_clone.saving_door_code.borrow_mut() = false;
                                                                            }
                                                                            Err(e) => {
                                                                                *state_clone.edit_error_message.borrow_mut() = Some(e.clone());
                                                                                update_modal_error_message(&state_clone, Some(e));
                                                                                *state_clone.saving_door_code.borrow_mut() = false;
                                                                            }
                                                                        }
                                                                    });
                                                                }
                                                            }
                                                        } else if e.key() == "Escape" {
                                                            e.prevent_default();
                                                            *state_clone.door_code_input_value.borrow_mut() = door_code_clone.clone();
                                                            toggle_edit_mode_door_code(&state_clone, false);
                                                        }
                                                    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
                                                    
                                                    let _ = input.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
                                                    closure.forget();
                                                }
                                                
                                                let _ = append_child(&input_group, &input);
                                                let _ = append_child(&value_container, &input_group);
                                                
                                                // Focus en el input
                                                if let Ok(html_input) = input.dyn_into::<web_sys::HtmlInputElement>() {
                                                    let _ = html_input.focus();
                                                }
                                            }
                                        }
                                    } else {
                                        // Modo visualización: mostrar span + botón editar
                                        set_inner_html(&value_container, "");
                                        
                                        let door_code = {
                                            if let Some((_, addr)) = state.details_package.borrow().as_ref() {
                                                addr.door_code.clone().unwrap_or_default()
                                            } else {
                                                String::new()
                                            }
                                        };
                                        
                                        let span = if !door_code.is_empty() {
                                            ElementBuilder::new("span")
                                                .map(|b| b.text(&door_code).build())
                                        } else {
                                            ElementBuilder::new("span")
                                                .map(|b| b.class("empty-value").text(&t("non_renseigne", &lang)).build())
                                        };
                                        
                                        if let Ok(span_el) = span {
                                            let _ = append_child(&value_container, &span_el);
                                        }
                                        
                                        if let Ok(edit_btn) = ElementBuilder::new("button")
                                            .and_then(|b| b.class("btn-icon").attr("title", &t("modifier", &lang)))
                                            .map(|b| b.text("⚙️").build())
                                        {
                                            {
                                                let state_clone = state.clone();
                                                let door_code_clone = door_code.clone();
                                                let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                                                    *state_clone.door_code_input_value.borrow_mut() = door_code_clone.clone();
                                                    toggle_edit_mode_door_code(&state_clone, true);
                                                }) as Box<dyn FnMut(web_sys::MouseEvent)>);
                                                
                                                let _ = edit_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
                                                closure.forget();
                                            }
                                            
                                            let _ = append_child(&value_container, &edit_btn);
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Toggle modo edición para notas chofer usando manipulación directa del DOM
pub fn toggle_edit_mode_driver_notes(state: &AppState, enable: bool) {
    use crate::dom::{query_selector, set_inner_html, append_child, set_attribute, set_text_content, add_class, remove_class};
    use crate::dom::ElementBuilder;
    use crate::utils::i18n::t;
    use wasm_bindgen::closure::Closure;
    
    let lang = state.language.borrow().clone();
    
    // Buscar la sección de notas chofer
    if let Ok(Some(_modal_body)) = query_selector(".modal-body") {
        if let Ok(sections) = crate::dom::query_selector_all(".modal-body .detail-section.editable") {
            for i in 0..sections.length() {
                let section_js = sections.get(i as u32);
                if !section_js.is_undefined() && !section_js.is_null() {
                    if let Ok(section) = section_js.dyn_into::<Element>() {
                        if let Ok(Some(label)) = section.query_selector(".detail-label") {
                            let label_text = label.text_content().unwrap_or_default();
                            if label_text.contains(&t("notes_chauffeur", &lang)) || label_text.contains("chofer") || label_text.contains("chauffeur") {
                                if let Ok(Some(value_container)) = section.query_selector(".detail-value-with-action") {
                                    if enable {
                                        // Modo edición: crear input
                                        set_inner_html(&value_container, "");
                                        
                                        if let Ok(input_group) = ElementBuilder::new("div")
                                            .map(|b| b.class("edit-input-group").build())
                                        {
                                            if let Ok(input) = crate::dom::create_element("input") {
                                                let _ = set_attribute(&input, "type", "text");
                                                let _ = set_attribute(&input, "class", "edit-input");
                                                let _ = set_attribute(&input, "placeholder", &t("ajouter_note", &lang));
                                                
                                                let current_value = state.driver_notes_input_value.borrow().clone();
                                                let _ = set_attribute(&input, "value", &current_value);
                                                
                                                // Event listener para input
                                                {
                                                    let state_clone = state.clone();
                                                    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                                                        if let Some(input_el) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                                                            *state_clone.driver_notes_input_value.borrow_mut() = input_el.value();
                                                        }
                                                    }) as Box<dyn FnMut(web_sys::Event)>);
                                                    let _ = input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref());
                                                    closure.forget();
                                                }
                                                
                                                // Event listener para Enter/Escape
                                                {
                                                    let state_clone = state.clone();
                                                    let driver_notes_clone = {
                                                        if let Some((_, addr)) = state_clone.details_package.borrow().as_ref() {
                                                            addr.driver_notes.clone().unwrap_or_default()
                                                        } else {
                                                            String::new()
                                                        }
                                                    };
                                                    
                                                    let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
                                                        if e.key() == "Enter" {
                                                            e.prevent_default();
                                                            let value = state_clone.driver_notes_input_value.borrow().trim().to_string();
                                                            if let Some((pkg, addr)) = state_clone.details_package.borrow().as_ref() {
                                                                let session_id = if let Some(session) = state_clone.session.get_session() {
                                                                    session.session_id.clone()
                                                                } else {
                                                                    return;
                                                                };
                                                                let addr_id = addr.address_id.clone();
                                                                let state_clone = state_clone.clone();
                                                                
                                                                *state_clone.saving_driver_notes.borrow_mut() = true;
                                                                
                                                                wasm_bindgen_futures::spawn_local(async move {
                                                                    use crate::viewmodels::session_viewmodel::SessionViewModel;
                                                                    let vm = SessionViewModel::new();
                                                                    
                                                                    match vm.update_address_fields(&session_id, &addr_id, None, None, Some(value.clone())).await {
                                                                        Ok(updated_session) => {
                                                                            state_clone.session.set_session(Some(updated_session.clone()));
                                                                            
                                                                            let pkg_opt = {
                                                                                let borrow = state_clone.details_package.borrow();
                                                                                borrow.clone()
                                                                            };
                                                                            if let Some((pkg, _)) = pkg_opt {
                                                                                if let Some(updated_addr) = updated_session.addresses.get(&addr_id) {
                                                                                    *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                                                                    update_modal_driver_notes_section(&state_clone, updated_addr);
                                                                                    toggle_edit_mode_driver_notes(&state_clone, false);
                                                                                }
                                                                            }
                                                                            
                                                                            *state_clone.saving_driver_notes.borrow_mut() = false;
                                                                        }
                                                                        Err(e) => {
                                                                            *state_clone.edit_error_message.borrow_mut() = Some(e.clone());
                                                                            update_modal_error_message(&state_clone, Some(e));
                                                                            *state_clone.saving_driver_notes.borrow_mut() = false;
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        } else if e.key() == "Escape" {
                                                            e.prevent_default();
                                                            *state_clone.driver_notes_input_value.borrow_mut() = driver_notes_clone.clone();
                                                            toggle_edit_mode_driver_notes(&state_clone, false);
                                                        }
                                                    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
                                                    
                                                    let _ = input.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
                                                    closure.forget();
                                                }
                                                
                                                let _ = append_child(&input_group, &input);
                                                let _ = append_child(&value_container, &input_group);
                                                
                                                // Focus en el input
                                                if let Ok(html_input) = input.dyn_into::<web_sys::HtmlInputElement>() {
                                                    let _ = html_input.focus();
                                                }
                                            }
                                        }
                                    } else {
                                        // Modo visualización: mostrar span + botón editar
                                        set_inner_html(&value_container, "");
                                        
                                        let driver_notes = {
                                            if let Some((_, addr)) = state.details_package.borrow().as_ref() {
                                                addr.driver_notes.clone().unwrap_or_default()
                                            } else {
                                                String::new()
                                            }
                                        };
                                        
                                        let span = if !driver_notes.is_empty() {
                                            ElementBuilder::new("span")
                                                .map(|b| b.text(&format!("\"{}\"", driver_notes)).build())
                                        } else {
                                            ElementBuilder::new("span")
                                                .map(|b| b.class("empty-value").text(&t("ajouter_note", &lang)).build())
                                        };
                                        
                                        if let Ok(span_el) = span {
                                            let _ = append_child(&value_container, &span_el);
                                        }
                                        
                                        if let Ok(edit_btn) = ElementBuilder::new("button")
                                            .and_then(|b| b.class("btn-icon").attr("title", &t("modifier", &lang)))
                                            .map(|b| b.text("⚙️").build())
                                        {
                                            {
                                                let state_clone = state.clone();
                                                let driver_notes_clone = driver_notes.clone();
                                                let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                                                    *state_clone.driver_notes_input_value.borrow_mut() = driver_notes_clone.clone();
                                                    toggle_edit_mode_driver_notes(&state_clone, true);
                                                }) as Box<dyn FnMut(web_sys::MouseEvent)>);
                                                
                                                let _ = edit_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
                                                closure.forget();
                                            }
                                            
                                            let _ = append_child(&value_container, &edit_btn);
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Actualizar header (botones disabled/enabled y texto del botón refresh)
pub fn update_header(state: &AppState, has_session: bool) -> Result<(), JsValue> {
    use crate::dom::document;
    use crate::dom::set_text_content;
    
    let loading = state.session.get_loading();
    
    // Lista de botones a actualizar (buscar por clase)
    let buttons = vec![
        ("btn-optimize-mini", loading || !has_session),
        ("btn-tracking-search", loading || !has_session),
        ("btn-scanner", loading),
        ("btn-refresh", loading || !has_session),
    ];
    
    // Buscar botones usando Document.query_selector (solo dentro del header para evitar conflictos)
    if let Some(doc) = document() {
        // Buscar primero el header para asegurar que solo actualizamos botones del header
        if let Ok(Some(header)) = doc.query_selector(".app-header") {
            for (class_name, should_disable) in buttons {
                if let Ok(Some(btn)) = header.query_selector(&format!(".{}", class_name)) {
                    if let Some(btn_elem) = btn.dyn_ref::<Element>() {
                        if should_disable {
                            let _ = btn_elem.set_attribute("disabled", "true");
                        } else {
                            let _ = btn_elem.remove_attribute("disabled");
                        }
                        
                        // Actualizar texto del botón refresh según estado de loading
                        if class_name == "btn-refresh" {
                            let text = if loading { "⏳" } else { "🔄" };
                            set_text_content(&btn_elem, &text);
                        }
                    }
                }
            }
        } else {
            log::warn!("⚠️ [UPDATE_HEADER] Header no encontrado en el DOM");
        }
    }
    
    Ok(())
}

/// Actualizar solo un card específico (para expand/collapse sin re-renderizar toda la lista)
pub fn update_single_package_card(
    state: &AppState,
    group_idx: usize,
    group: &PackageGroup,
    addresses: &std::collections::HashMap<String, String>,
    session: &crate::models::session::DeliverySession,
) -> Result<(), JsValue> {
    use crate::views::package_card::render_package_card;
    use crate::dom::{get_element_by_id, remove_child, append_child};
    use std::collections::HashMap;
    use std::rc::Rc;
    use web_sys::Node;
    
    // Buscar el card existente por data-index usando JavaScript
    let card_selector = format!(r#"[data-index="{}"]"#, group_idx);
    
    // Obtener el contenedor de la lista
    if let Some(package_list_container) = get_element_by_id("package-list") {
        // Obtener dirección
        let address = group.packages.first()
            .and_then(|p| addresses.get(&p.address_id))
            .map(|s| s.as_str());
        
        let selected_index = *state.selected_package_index.borrow();
        let is_selected = selected_index == Some(group_idx);
        let is_expanded = state.expanded_groups.borrow().contains(&group_idx);
        
        // Crear callbacks
        let on_package_selected = {
            let state_clone = state.clone();
            Rc::new(move |index: usize| {
                state_clone.set_selected_package_index(Some(index));
                crate::utils::mapbox_ffi::update_selected_package(index as i32);
                crate::utils::mapbox_ffi::center_map_on_package(index);
                use gloo_timers::callback::Timeout;
                Timeout::new(300, move || {
                    crate::utils::mapbox_ffi::scroll_to_selected_package(index);
                }).forget();
            })
        };
        
        let on_info = {
            let state_clone = state.clone();
            let session_clone = session.clone();
            Rc::new(move |tracking: String| {
                if let Some(pkg) = session_clone.packages.get(&tracking) {
                    if let Some(addr) = session_clone.addresses.get(&pkg.address_id) {
                        {
                            let mut details = state_clone.details_package.borrow_mut();
                            *details = Some((pkg.clone(), addr.clone()));
                        }
                        state_clone.set_show_details(true);
                    }
                }
            })
        };
        
        // Crear paquete virtual para el grupo
        let mut group_package = group.packages.first().unwrap().clone();
        if group.packages.len() > 1 {
            group_package.customer_name = format!("{} paquetes", group.packages.len());
            group_package.is_group = true;
            group_package.group_packages = Some(group.packages.clone());
        }
        
        // Toggle expand callback
        let on_toggle_expand: Option<Rc<dyn Fn(usize)>> = if group.packages.len() > 1 {
            let state_clone = state.clone();
            let current_group_idx = group_idx;
            Some(Rc::new(move |_idx: usize| {
                {
                    let mut expanded = state_clone.expanded_groups.borrow_mut();
                    if expanded.contains(&current_group_idx) {
                        expanded.remove(&current_group_idx);
                    } else {
                        expanded.insert(current_group_idx);
                    }
                }
                crate::rerender_app_with_type(crate::state::app_state::UpdateType::Incremental(
                    crate::state::app_state::IncrementalUpdate::PackageList
                ));
            }) as Rc<dyn Fn(usize)>)
        } else {
            None
        };
        
        // Re-renderizar solo este card
        let new_card = render_package_card(
            &group_package,
            group_idx,
            address,
            is_selected,
            is_expanded,
            on_package_selected,
            on_info,
            on_toggle_expand,
        )?;
        
        // Reemplazar el card usando JavaScript directamente
        // Esto evita el "aplastamiento" porque no limpiamos toda la lista
        let replace_js = format!(r#"
            (function() {{
                const oldCard = document.querySelector('{}');
                if (oldCard && oldCard.parentNode) {{
                    // Crear nuevo card desde el elemento Rust
                    const newCard = arguments[0];
                    if (newCard) {{
                        // Reemplazar el card antiguo con el nuevo
                        oldCard.parentNode.replaceChild(newCard, oldCard);
                        console.log('✅ [JS] Card reemplazado sin re-renderizar toda la lista');
                    }} else {{
                        console.error('❌ [JS] No se pudo obtener el nuevo card');
                    }}
                }} else {{
                    console.error('❌ [JS] No se encontró el card antiguo o su parent');
                }}
            }})();
        "#, card_selector);
        
        // Ejecutar el reemplazo pasando el nuevo card como argumento
        // Necesitamos usar eval con el elemento como variable global temporal
        use wasm_bindgen::JsCast;
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        js_sys::Reflect::set(&window, &JsValue::from_str("_tempNewCard"), &new_card.clone().into())?;
        
        // Reemplazar card preservando altura para evitar "aplastamiento" al colapsar
        // Técnica: medir nuevo card, preservar altura del antiguo durante transición
        let replace_with_temp = format!(r#"
            (function() {{
                const oldCard = document.querySelector('{}');
                const newCard = window._tempNewCard;
                const container = document.getElementById('package-list');
                
                if (oldCard && oldCard.parentNode && newCard && container) {{
                    // Preservar estado antes del reemplazo
                    const containerScrollTop = container.scrollTop;
                    const oldCardHeight = oldCard.offsetHeight;
                    
                    // Medir nuevo card antes de insertarlo (usando clone invisible)
                    const newCardClone = newCard.cloneNode(true);
                    newCardClone.style.visibility = 'hidden';
                    newCardClone.style.position = 'absolute';
                    newCardClone.style.top = '-9999px';
                    document.body.appendChild(newCardClone);
                    void newCardClone.offsetHeight; // Forzar reflow
                    const newCardHeight = newCardClone.offsetHeight;
                    document.body.removeChild(newCardClone);
                    
                    // Si el nuevo card es más pequeño, usar transición suave
                    if (newCardHeight < oldCardHeight) {{
                        // NO tocar el contenedor para evitar reflows que afecten el ancho
                        // Solo animar el card individual
                        
                        // Establecer altura fija en el card antiguo antes de reemplazar
                        oldCard.style.height = oldCardHeight + 'px';
                        oldCard.style.overflow = 'hidden';
                        oldCard.style.boxSizing = 'border-box';
                        
                        // Reemplazar el card
                        oldCard.parentNode.replaceChild(newCard, oldCard);
                        
                        // Establecer altura inicial en el nuevo card (igual a la antigua)
                        // Usar box-sizing para evitar que padding afecte el cálculo
                        newCard.style.height = oldCardHeight + 'px';
                        newCard.style.overflow = 'hidden';
                        newCard.style.boxSizing = 'border-box';
                        newCard.style.transition = 'height 0.3s cubic-bezier(0.4, 0, 0.2, 1)';
                        
                        // Forzar reflow para que el navegador registre la altura inicial
                        void newCard.offsetHeight;
                        
                        // Animar a la altura final en el siguiente frame
                        requestAnimationFrame(function() {{
                            requestAnimationFrame(function() {{
                                // Animar el card a su nueva altura
                                newCard.style.height = newCardHeight + 'px';
                                
                                // Remover estilos después de la transición
                                setTimeout(function() {{
                                    newCard.style.height = '';
                                    newCard.style.overflow = '';
                                    newCard.style.boxSizing = '';
                                    newCard.style.transition = '';
                                    
                                    // Preservar scroll position después de la transición
                                    if (Math.abs(container.scrollTop - containerScrollTop) > 5) {{
                                        container.scrollTop = containerScrollTop;
                                    }}
                                    
                                    console.log('✅ [JS] Card reemplazado con transición suave');
                                    delete window._tempNewCard;
                                }}, 350); // Tiempo más largo que la transición (300ms + margen)
                            }});
                        }});
                    }} else {{
                        // El nuevo card es igual o más grande, reemplazo directo está bien
                        oldCard.parentNode.replaceChild(newCard, oldCard);
                        if (Math.abs(container.scrollTop - containerScrollTop) > 5) {{
                            container.scrollTop = containerScrollTop;
                        }}
                        console.log('✅ [JS] Card reemplazado sin re-renderizar toda la lista');
                        delete window._tempNewCard;
                    }}
                }} else {{
                    console.error('❌ [JS] Error reemplazando card');
                    delete window._tempNewCard;
                }}
            }})();
        "#, card_selector);
        
        let _ = js_sys::eval(&replace_with_temp);
        
        // IMPORTANTE: Actualizar botones "Go" de otros cards para mantener consistencia
        // Cuando se actualiza un card, necesitamos asegurarnos de que solo el card seleccionado
        // tenga el botón "Go" visible
        let selected_index = *state.selected_package_index.borrow();
        let update_go_buttons_js = format!(r#"
            (function() {{
                const selectedIdx = {};
                // Remover botones "Go" de todos los cards excepto el seleccionado
                document.querySelectorAll('.package-card').forEach(function(card, idx) {{
                    const dataIndex = parseInt(card.getAttribute('data-index') || '-1');
                    const goButton = card.querySelector('.btn-navigate');
                    
                    if (dataIndex === selectedIdx) {{
                        // Este es el card seleccionado, asegurarse de que tenga el botón "Go"
                        if (!goButton) {{
                            const recipientRow = card.querySelector('.package-recipient-row');
                            if (recipientRow) {{
                                const navBtn = document.createElement('button');
                                navBtn.className = 'btn-navigate';
                                navBtn.textContent = 'Go';
                                recipientRow.appendChild(navBtn);
                            }}
                        }}
                    }} else {{
                        // Este NO es el card seleccionado, remover el botón "Go" si existe
                        if (goButton) {{
                            goButton.remove();
                        }}
                    }}
                }});
            }})();
        "#, selected_index.map(|i| i as i32).unwrap_or(-1));
        let _ = js_sys::eval(&update_go_buttons_js);
    }
    
    Ok(())
}

/// Actualizar progress bar del bottom sheet (direcciones tratadas y paquetes entregados/fallidos)
/// Asegura que el header del bottom sheet (drag-handle-container) siempre exista
pub fn update_progress_bar(state: &AppState, session: &crate::models::session::DeliverySession) -> Result<(), JsValue> {
    use crate::dom::{get_element_by_id, append_child, remove_child};
    use crate::views::bottom_sheet::render_progress_info;
    use wasm_bindgen::closure::Closure;
    
    // Asegurar que drag-handle-container existe - si no existe, no podemos actualizar
    let drag_handle_container = match get_element_by_id("drag-handle-container") {
        Some(container) => container,
        None => {
            log::warn!("⚠️ [PROGRESS_BAR] drag-handle-container no encontrado, el header del bottom sheet no existe");
            return Ok(());
        }
    };
    
    // Buscar y remover progress-info y progress-bar-container existentes
    if let Some(progress_info) = get_element_by_id("progress-info") {
        let _ = remove_child(&drag_handle_container, &progress_info);
    }
    if let Some(progress_bar_container) = get_element_by_id("progress-bar-container") {
        let _ = remove_child(&drag_handle_container, &progress_bar_container);
    }
    
    // Solo actualizar si el sheet no está collapsed
    let sheet_state = state.sheet_state.borrow().clone();
    if sheet_state != "collapsed" {
        // Renderizar nuevo progress info y progress bar (ya tienen IDs)
        let (progress_info, progress_bar_container) = render_progress_info(session, state)?;
        
        // Agregar después del drag-handle (que es el primer hijo)
        append_child(&drag_handle_container, &progress_info)?;
        append_child(&drag_handle_container, &progress_bar_container)?;
    }
    
    Ok(())
}

/// Actualizar package list incrementalmente (solo cards que cambiaron, sin limpiar toda la lista)
pub fn update_package_list(state: &AppState, groups: &[PackageGroup], session: &crate::models::session::DeliverySession) -> Result<(), JsValue> {
    use crate::views::package_card::render_package_card;
    use crate::dom::{append_child, remove_child, query_selector};
    use crate::dom::document;
    use std::collections::HashMap;
    use std::rc::Rc;
    use wasm_bindgen::JsCast;
    
    if let Some(package_list_container) = get_element_by_id("package-list") {
        // PRESERVAR ESTADO EXPANDIDO DE GRUPOS (para evitar "aplastamiento")
        let expanded_groups_before: Vec<usize> = state.expanded_groups.borrow().iter().cloned().collect();
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("💾 [UPDATE] Preservando {} grupos expandidos antes de actualizar", expanded_groups_before.len())));
        
        // Preservar scroll position
        let preserve_state_js = r#"
            (function() {
                const container = document.getElementById('package-list');
                if (container) {
                    window._packageListState = {
                        scrollTop: container.scrollTop,
                        scrollHeight: container.scrollHeight,
                        clientHeight: container.clientHeight
                    };
                    console.log('💾 [JS] Estado preservado:', window._packageListState);
                }
            })();
        "#;
        let _ = js_sys::eval(preserve_state_js);
        
        // NO LIMPIAR LA LISTA - Actualizar incrementalmente
        // Obtener cards existentes del DOM
        let existing_cards_js = r#"
            (function() {
                const cards = document.querySelectorAll('.package-card[data-index]');
                const result = [];
                for (let i = 0; i < cards.length; i++) {
                    const card = cards[i];
                    const index = parseInt(card.getAttribute('data-index'));
                    const tracking = card.getAttribute('data-tracking') || '';
                    result.push({index: index, tracking: tracking, element: card});
                }
                return result;
            })();
        "#;
        let existing_cards_result = js_sys::eval(existing_cards_js)?;
        
        // Crear mapa de addresses
        let addresses_map: HashMap<String, String> = session.addresses
            .iter()
            .map(|(k, v)| (k.clone(), v.label.clone()))
            .collect();
        
        let selected_index = *state.selected_package_index.borrow();
        
        // Crear callbacks
        let on_package_selected = {
            let state_clone = state.clone();
            Rc::new(move |index: usize| {
                state_clone.set_selected_package_index(Some(index));
                crate::utils::mapbox_ffi::update_selected_package(index as i32);
                crate::utils::mapbox_ffi::center_map_on_package(index);
                use gloo_timers::callback::Timeout;
                Timeout::new(300, move || {
                    crate::utils::mapbox_ffi::scroll_to_selected_package(index);
                }).forget();
            })
        };
        
        let on_info = {
            let state_clone = state.clone();
            let session_clone = session.clone();
            Rc::new(move |tracking: String| {
                if let Some(pkg) = session_clone.packages.get(&tracking) {
                    if let Some(addr) = session_clone.addresses.get(&pkg.address_id) {
                        {
                            let mut details = state_clone.details_package.borrow_mut();
                            *details = Some((pkg.clone(), addr.clone()));
                        }
                        state_clone.set_show_details(true);
                    }
                }
            })
        };
        
        // Comparar grupos existentes con nuevos y actualizar solo los que cambiaron
        // Crear un mapa de tracking -> grupo para comparación rápida
        let mut new_groups_map: HashMap<String, (usize, &PackageGroup)> = HashMap::new();
        for (idx, group) in groups.iter().enumerate() {
            if let Some(first_pkg) = group.packages.first() {
                new_groups_map.insert(first_pkg.tracking.clone(), (idx, group));
            }
        }
        
        // Obtener cards existentes del DOM usando query_selector_all
        let existing_cards = crate::dom::query_selector_all(".package-card[data-index]")?;
        let mut existing_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();
        
        // Iterar sobre cards existentes y actualizar solo los que cambiaron
        for i in 0..existing_cards.length() {
            let card_js_value = existing_cards.get(i as u32);
            if let Ok(card_elem) = card_js_value.dyn_into::<Element>() {
                if let Some(data_index) = card_elem.get_attribute("data-index") {
                    if let Ok(index) = data_index.parse::<usize>() {
                        existing_indices.insert(index);
                        
                        // Verificar si este grupo cambió
                        if index < groups.len() {
                            let group = &groups[index];
                            if let Some(first_pkg) = group.packages.first() {
                                // Verificar si el status o algún dato cambió comparando con el DOM
                                let card_status = card_elem.get_attribute("data-status").unwrap_or_default();
                                let new_status = &first_pkg.status;
                                
                                // Si el status cambió, actualizar el card
                                if card_status != *new_status {
                                    log::info!("🔄 [UPDATE] Actualizando card {} (status cambió: {} → {})", index, card_status, new_status);
                                    if let Err(e) = update_single_package_card(
                                        state,
                                        index,
                                        group,
                                        &addresses_map,
                                        session,
                                    ) {
                                        log::warn!("⚠️ [UPDATE] Error actualizando card {}: {:?}", index, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Agregar nuevos cards que no existen
        for (idx, group) in groups.iter().enumerate() {
            if !existing_indices.contains(&idx) {
                log::info!("➕ [UPDATE] Agregando nuevo card para grupo {}", idx);
                // Usar update_single_package_card para agregar el nuevo card
                if let Err(e) = update_single_package_card(
                    state,
                    idx,
                    group,
                    &addresses_map,
                    session,
                ) {
                    log::warn!("⚠️ [UPDATE] Error agregando card {}: {:?}", idx, e);
                }
            }
        }
        
        // Remover cards que ya no existen (si hay menos grupos ahora)
        let max_existing_idx = existing_indices.iter().max().copied().unwrap_or(0);
        if max_existing_idx >= groups.len() {
            for idx in groups.len()..=max_existing_idx {
                let card_selector = format!(r#"[data-index="{}"]"#, idx);
                let remove_js = format!(r#"
                    (function() {{
                        const card = document.querySelector('{}');
                        if (card && card.parentNode) {{
                            card.parentNode.removeChild(card);
                            console.log('🗑️ [UPDATE] Card {} removido');
                        }}
                    }})();
                "#, card_selector, idx);
                let _ = js_sys::eval(&remove_js);
            }
        }
        
        // Preservar estado expandido - los grupos ya están expandidos en el DOM, no necesitamos restaurarlos
        
        // Restaurar scroll position usando JavaScript de forma más robusta
        // Usar requestAnimationFrame para asegurar que el DOM se haya actualizado
        let restore_state_js = r#"
            (function() {
                if (window._packageListState) {
                    const container = document.getElementById('package-list');
                    if (container) {
                        console.log('🔄 [JS] Restaurando scroll, estado guardado:', window._packageListState);
                        console.log('🔄 [JS] Estado actual del contenedor antes de restaurar:', {
                            scrollTop: container.scrollTop,
                            scrollHeight: container.scrollHeight,
                            clientHeight: container.clientHeight
                        });
                        // Usar requestAnimationFrame para restaurar después del reflow
                        requestAnimationFrame(function() {
                            console.log('📐 [JS] Primer RAF ejecutado');
                            requestAnimationFrame(function() {
                                console.log('📐 [JS] Segundo RAF ejecutado, restaurando scroll');
                                // Doble requestAnimationFrame para asegurar que el layout esté completo
                                if (window._packageListState) {
                                    container.scrollTop = window._packageListState.scrollTop;
                                    console.log('✅ [JS] Scroll restaurado a:', window._packageListState.scrollTop);
                                    console.log('✅ [JS] Estado final del contenedor:', {
                                        scrollTop: container.scrollTop,
                                        scrollHeight: container.scrollHeight,
                                        clientHeight: container.clientHeight
                                    });
                                    delete window._packageListState;
                                }
                            });
                        });
                    }
                }
            })();
        "#;
        let _ = js_sys::eval(restore_state_js);
        
        // #region agent log
        // Logs removidos temporalmente para compilación
        // #endregion
    }
    
    Ok(())
}

/// Actualizar sync indicator
pub fn update_sync_indicator(state: &AppState) -> Result<(), JsValue> {
    use crate::views::render_sync_indicator;
    use crate::dom::{set_inner_html, append_child};
    
    if let Some(sync_container) = get_element_by_id("sync-indicator") {
        // Re-renderizar solo el indicador (puede ser None si está Synced)
        set_inner_html(&sync_container, "");
        if let Some(new_indicator) = render_sync_indicator(state)? {
            append_child(&sync_container, &new_indicator)?;
        }
    }
    
    Ok(())
}

/// Actualizar paquetes en el mapa (sin destruir el mapa)
pub fn update_map_packages(state: &AppState, session: &crate::models::session::DeliverySession) -> Result<(), JsValue> {
    use crate::viewmodels::map_viewmodel::MapViewModel;
    use crate::views::group_packages_by_address;
    use crate::models::package::Package;
    use serde_json;
    
    // Preparar paquetes para el mapa (aplicar filtro si está activo)
    let mut packages: Vec<Package> = session.packages.values().cloned().collect();
    if *state.filter_mode.borrow() {
        packages.retain(|p| p.status.starts_with("STATUT_CHARGER"));
    }
    
    let groups = group_packages_by_address(packages);
    let map_packages = MapViewModel::prepare_packages_for_map(&groups, session);
    let packages_json = serde_json::to_string(&map_packages)
        .unwrap_or_else(|_| "[]".to_string());
    
    // Actualizar paquetes en el mapa sin recrearlo
    mapbox_ffi::add_packages_to_map(&packages_json);
    
    // Actualizar selección si hay una
    if let Some(selected_idx) = *state.selected_package_index.borrow() {
        mapbox_ffi::update_selected_package(selected_idx as i32);
    }
    
    Ok(())
}

/// Crear el contenedor packages-expanded en Rust puro
/// Extrae la lógica de construcción del contenedor expandido de package_card.rs
fn create_expanded_container_rust(
    group: &PackageGroup,
    addresses: &std::collections::HashMap<String, String>,
    session: &crate::models::session::DeliverySession,
    state: &AppState,
) -> Result<Element, JsValue> {
    use crate::dom::{ElementBuilder, append_child};
    use wasm_bindgen::closure::Closure;
    use std::rc::Rc;
    
    let expanded_container = ElementBuilder::new("div")?
        .class("packages-expanded")
        .build();
    
    // Iterar sobre los paquetes del grupo directamente
    for (idx, pkg_inner) in group.packages.iter().enumerate() {
                let pkg_status_color = match pkg_inner.status.as_str() {
                    s if s.contains("RECEPTIONNER") => "package-number-yellow",
                    s if s.contains("LIVRER") => "package-number-green",
                    s if s.contains("NONLIV") => "package-number-red",
                    _ => "package-number-normal",
                };
                
                let pkg_type_class = match pkg_inner.delivery_type {
                    crate::models::package::DeliveryType::PickupPoint => "type-relais",
                    crate::models::package::DeliveryType::Rcs => "type-rcs",
                    _ => "type-domicile",
                };
                
                let package_item = ElementBuilder::new("div")?
                    .class(&format!("package-item {}", pkg_type_class))
                    .build();
                
                let item_header = ElementBuilder::new("div")?
                    .class("package-item-header")
                    .build();
                
                let pkg_number = ElementBuilder::new("span")?
                    .class(&format!("package-number {}", pkg_status_color))
                    .text(&(idx + 1).to_string())
                    .build();
                append_child(&item_header, &pkg_number)?;
                
                match pkg_inner.delivery_type {
                    crate::models::package::DeliveryType::PickupPoint => {
                        let content = ElementBuilder::new("div")?
                            .class("package-item-content")
                            .build();
                        
                        let tracking_label = ElementBuilder::new("span")?
                            .class("tracking-label")
                            .text("📦 ")
                            .build();
                        let tracking_value = ElementBuilder::new("span")?
                            .class("tracking-value")
                            .text(&pkg_inner.tracking)
                            .build();
                        
                        append_child(&content, &tracking_label)?;
                        append_child(&content, &tracking_value)?;
                        append_child(&item_header, &content)?;
                    },
                    _ => {
                        let customer_name = ElementBuilder::new("strong")?
                            .class("package-customer")
                            .text(&pkg_inner.customer_name)
                            .build();
                        append_child(&item_header, &customer_name)?;
                        
                        let details_btn = ElementBuilder::new("button")?
                            .class("btn-package-details")
                            .text("i")
                            .build();
                        
                        {
                            let tracking = pkg_inner.tracking.clone();
                            let session_clone = session.clone();
                            let state_clone = state.clone();
                            let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                                e.stop_propagation();
                                e.prevent_default();
                                if let Some(pkg) = session_clone.packages.get(&tracking) {
                                    if let Some(addr) = session_clone.addresses.get(&pkg.address_id) {
                                        {
                                            let mut details = state_clone.details_package.borrow_mut();
                                            *details = Some((pkg.clone(), addr.clone()));
                                        }
                                        state_clone.set_show_details(true);
                                    }
                                }
                            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
                            
                            details_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
                            closure.forget();
                        }
                        
                        append_child(&item_header, &details_btn)?;
                    }
                }
                
                append_child(&package_item, &item_header)?;
                append_child(&expanded_container, &package_item)?;
    }
    
    Ok(expanded_container)
}

/// Toggle expand/collapse de un grupo manipulando directamente el DOM desde Rust
/// Esta es la solución más eficiente - todo en Rust, sin JavaScript intermedio
pub fn toggle_group_expand_direct_rust(
    state: &AppState,
    group_idx: usize,
    group: &PackageGroup,
    addresses: &std::collections::HashMap<String, String>,
    session: &crate::models::session::DeliverySession,
) -> Result<(), JsValue> {
    use crate::dom::{document, query_selector, append_child, remove_child};
    use crate::dom::ElementBuilder;
    use wasm_bindgen::JsCast;
    use web_sys::Node;
    
    let card_selector = format!(r#"[data-index="{}"]"#, group_idx);
    let is_expanded = state.expanded_groups.borrow().contains(&group_idx);
    
    // Buscar el card usando query_selector (100% Rust)
    let doc = document().ok_or_else(|| JsValue::from_str("No document"))?;
    let card = doc.query_selector(&card_selector)?
        .ok_or_else(|| JsValue::from_str("Card not found"))?;
    
    // Buscar si ya existe el contenedor expanded
    let expanded_container_opt = card.query_selector(".packages-expanded")?;
    
    if is_expanded {
        // EXPANDIR: Crear y agregar el contenedor si no existe
        if expanded_container_opt.is_none() {
            // Crear el contenedor expanded usando la misma lógica que render_package_card
            let expanded_el = create_expanded_container_rust(
                group,
                addresses,
                session,
                state,
            )?;
            
            // Buscar el expand-handle para insertar antes de él
            let expand_handle_opt = card.query_selector(".expand-handle")?;
            
            if let Some(handle) = expand_handle_opt {
                // Insertar antes del expand-handle
                if let Some(parent) = handle.parent_node() {
                    if let Some(parent_el) = parent.dyn_ref::<Element>() {
                        parent_el.insert_before(&expanded_el, Some(&handle))?;
                    }
                }
            } else {
                // Si no hay expand-handle, agregar al final del card
                append_child(&card, &expanded_el)?;
            }
            
            // Aplicar transición CSS suave usando set_attribute con estilos inline
            if let Ok(html_el) = expanded_el.clone().dyn_into::<web_sys::HtmlElement>() {
                // Establecer estilos iniciales para la animación
                expanded_el.set_attribute("style", "max-height: 0; overflow: hidden; transition: max-height 0.3s cubic-bezier(0.4, 0, 0.2, 1);")?;
                
                // Forzar reflow
                let _ = html_el.offset_height();
                
                // Usar requestAnimationFrame desde Rust (usando Timeout con 0ms)
                let expanded_el_clone = expanded_el.clone();
                Timeout::new(0, move || {
                    if let Ok(html_el) = expanded_el_clone.clone().dyn_into::<web_sys::HtmlElement>() {
                        let height = html_el.scroll_height();
                        let _ = expanded_el_clone.set_attribute("style", &format!("max-height: {}px; overflow: hidden; transition: max-height 0.3s cubic-bezier(0.4, 0, 0.2, 1);", height));
                        
                        // Remover estilos después de la transición
                        let expanded_el_final = expanded_el_clone.clone();
                        Timeout::new(300, move || {
                            let _ = expanded_el_final.remove_attribute("style");
                        }).forget();
                    }
                }).forget();
            }
        }
    } else {
        // COLAPSAR: Remover el contenedor con transición suave
        if let Some(expanded) = expanded_container_opt {
            if let Ok(html_el) = expanded.clone().dyn_into::<web_sys::HtmlElement>() {
                let current_height = html_el.scroll_height();
                // Establecer altura actual y transición
                expanded.set_attribute("style", &format!("max-height: {}px; overflow: hidden; transition: max-height 0.3s cubic-bezier(0.4, 0, 0.2, 1);", current_height))?;
                
                // Forzar reflow
                let _ = html_el.offset_height();
                
                // Animar hacia 0 y luego remover
                let expanded_clone = expanded.clone();
                Timeout::new(0, move || {
                    let _ = expanded_clone.set_attribute("style", "max-height: 0; overflow: hidden; transition: max-height 0.3s cubic-bezier(0.4, 0, 0.2, 1);");
                    
                    // Remover después de la transición
                    let expanded_final = expanded_clone.clone();
                    Timeout::new(300, move || {
                        if let Some(parent) = expanded_final.parent_node() {
                            if let Some(parent_el) = parent.dyn_ref::<Element>() {
                                let _ = parent_el.remove_child(&expanded_final);
                            }
                        }
                    }).forget();
                }).forget();
            }
        }
    }
    
    Ok(())
}

/// Scroll mejorado: hacer scroll dentro del contenedor .package-list en lugar del documento completo
/// Esto asegura que el scroll funcione correctamente dentro del bottom-sheet
fn scroll_to_card_in_container(card: &Element) -> Result<(), JsValue> {
    use crate::dom::{document, query_selector};
    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;
    
    // Buscar el contenedor .package-list (el que tiene overflow-y: auto)
    let package_list_selector = ".package-list";
    let package_list_opt = query_selector(package_list_selector)?;
    
    if let Some(package_list) = package_list_opt {
        web_sys::console::log_1(&JsValue::from_str("✅ [SCROLL] Contenedor .package-list encontrado"));
        // Convertir a HtmlElement para acceder a scrollTop y scrollHeight
        if let Ok(list_html) = package_list.clone().dyn_into::<HtmlElement>() {
            // Obtener posiciones usando JavaScript para mayor precisión
            let scroll_js = format!(r#"
                (function() {{
                    const card = arguments[0];
                    const container = arguments[1];
                    
                    console.log('📍 [JS-SCROLL] scroll_to_card_in_container ejecutándose');
                    
                    if (!card || !container) {{
                        console.warn('⚠️ [JS-SCROLL] Card o container no encontrado');
                        return;
                    }}
                    
                    // Obtener posiciones relativas
                    const cardRect = card.getBoundingClientRect();
                    const containerRect = container.getBoundingClientRect();
                    const currentScroll = container.scrollTop;
                    
                    console.log('📊 [JS-SCROLL] Estado inicial:', {{
                        currentScroll: currentScroll,
                        cardTop: cardRect.top,
                        containerTop: containerRect.top,
                        cardHeight: cardRect.height,
                        containerHeight: container.clientHeight
                    }});
                    
                    // Calcular posición del card dentro del contenedor
                    const cardTop = cardRect.top - containerRect.top + container.scrollTop;
                    const cardHeight = cardRect.height;
                    const containerHeight = container.clientHeight;
                    
                    // Calcular scrollTop para centrar el card
                    const targetScrollTop = cardTop - (containerHeight / 2) + (cardHeight / 2);
                    
                    console.log('🎯 [JS-SCROLL] Objetivo calculado:', {{
                        cardTop: cardTop,
                        targetScrollTop: targetScrollTop,
                        distance: targetScrollTop - currentScroll
                    }});
                    
                    // Hacer scroll suave usando requestAnimationFrame
                    const startScrollTop = container.scrollTop;
                    const distance = targetScrollTop - startScrollTop;
                    const duration = 300; // ms
                    const startTime = performance.now();
                    
                    function animateScroll(currentTime) {{
                        const elapsed = currentTime - startTime;
                        const progress = Math.min(elapsed / duration, 1);
                        
                        // Easing function: ease-out cubic
                        const easeOutCubic = 1 - Math.pow(1 - progress, 3);
                        
                        const newScrollTop = startScrollTop + (distance * easeOutCubic);
                        container.scrollTop = newScrollTop;
                        
                        if (progress < 1) {{
                            requestAnimationFrame(animateScroll);
                        }} else {{
                            console.log('✅ [JS-SCROLL] Animación completada, scroll final:', container.scrollTop);
                            // Agregar clase flash al card después del scroll
                            card.classList.add('flash');
                            setTimeout(() => {{
                                card.classList.remove('flash');
                            }}, 500);
                        }}
                    }}
                    
                    console.log('🚀 [JS-SCROLL] Iniciando animación de scroll');
                    requestAnimationFrame(animateScroll);
                }})();
            "#);
            
            // Ejecutar JavaScript pasando los elementos como argumentos
            if let Some(window) = web_sys::window() {
                let function = js_sys::Function::new_no_args(&scroll_js);
                
                // Pasar los elementos como argumentos usando Reflect
                let args = js_sys::Array::new();
                args.push(&card.clone().into());
                args.push(&list_html.clone().into());
                
                let _ = function.call1(&window.into(), &args.into());
            }
        }
    } else {
        web_sys::console::warn_1(&JsValue::from_str("⚠️ [SCROLL] Contenedor .package-list no encontrado, usando fallback scrollIntoView"));
        // Fallback: si no se encuentra .package-list, usar scrollIntoView normal
        if let Ok(card_html) = card.clone().dyn_into::<HtmlElement>() {
            // Usar scrollIntoView como fallback
            let scroll_into_view_js = r#"
                (function() {
                    const card = arguments[0];
                    if (card && card.scrollIntoView) {
                        card.scrollIntoView({
                            behavior: 'smooth',
                            block: 'center'
                        });
                        // Agregar clase flash
                        card.classList.add('flash');
                        setTimeout(() => {
                            card.classList.remove('flash');
                        }, 500);
                    }
                })();
            "#;
            
            if let Some(window) = web_sys::window() {
                let function = js_sys::Function::new_no_args(scroll_into_view_js);
                let args = js_sys::Array::new();
                args.push(&card_html.clone().into());
                let _ = function.call1(&window.into(), &args.into());
            }
        }
    }
    
    Ok(())
}

