// ============================================================================
// APP VIEW - Vista principal de la aplicación (convertida de Yew a Rust puro)
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::{Element, console};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use js_sys;
use std::rc::Rc;
use std::collections::HashMap;
use gloo_timers::callback::Timeout;
use crate::dom::{ElementBuilder, append_child, set_attribute, set_class_name, get_element_by_id};
use crate::state::app_state::AppState;
use crate::views::{
    render_login,
    render_package_list,
    group_packages_by_address,
    PackageGroup,
    render_details_modal,
    render_sync_indicator,
    render_settings_popup,
    render_scanner,
    render_bottom_sheet,
    render_tracking_modal,
    render_admin_dashboard,
    render_status_change_modal,
};
use crate::viewmodels::map_viewmodel::MapViewModel;
use crate::models::package::Package;
use crate::utils::mapbox_ffi;
use serde_json;

/// Renderizar vista principal de la aplicación
pub fn render_app(state: &AppState) -> Result<Element, JsValue> {
    console::log_1(&JsValue::from_str("🎬 [APP] render_app() llamado"));
    
    // Verificar admin_mode primero
    let is_admin_mode = *state.admin_mode.borrow();
    if is_admin_mode {
        console::log_1(&JsValue::from_str("👑 [APP] Modo admin activo, renderizando dashboard admin"));
        return render_admin_dashboard(state);
    }
    
    let is_logged_in = state.auth.get_logged_in();
    let msg = format!("🔐 [APP] Usuario logged in: {}", is_logged_in);
    console::log_1(&JsValue::from_str(&msg));
    
    // Container principal
    let app_container = ElementBuilder::new("div")?
        .class("app-container")
        .build();
    
    if is_logged_in {
        if let Some(session) = state.session.get_session() {
            console::log_1(&JsValue::from_str("✅ [APP] Sesión disponible, renderizando main app"));
            // Renderizar app principal
            let main_app = render_main_app_view(state, &session)?;
            append_child(&app_container, &main_app)?;
                            } else {
            console::log_1(&JsValue::from_str("⏳ [APP] Sesión no disponible, mostrando mensaje de carga"));
            // Sesión no disponible
            let message = ElementBuilder::new("div")?
                .text(&crate::utils::i18n::t("cargando_sesion", state.language.borrow().as_str()))
                .build();
            append_child(&app_container, &message)?;
                            }
                        } else {
        console::log_1(&JsValue::from_str("🔑 [APP] Usuario no logueado, renderizando login"));
        // Renderizar login
        let login_view = render_login(state)?;
        append_child(&app_container, &login_view)?;
        console::log_1(&JsValue::from_str("✅ [APP] Login renderizado"));
    }
    
    Ok(app_container)
}

/// Renderizar vista principal cuando hay sesión
fn render_main_app_view(
    state: &AppState,
    session: &crate::models::session::DeliverySession,
) -> Result<Element, JsValue> {
    // Container principal
    let main_app = ElementBuilder::new("div")?
        .class("main-app")
        .build();
    
    // Header
    let header = create_header(state, Some(session))?;
    append_child(&main_app, &header)?;
    
    // Content area (con padding-top para el header fijo)
    let content = ElementBuilder::new("div")?
        .class("app-content")
        .attr("style", &format!("padding-top: {}px;", get_header_height()))?
        .build();
    
    // Mapa (condicional)
    let map_enabled = *state.map_enabled.borrow();
    if map_enabled {
        let map_container = create_map_container(state, session)?;
        append_child(&content, &map_container)?;
    }
    
    // Bottom Sheet (reemplaza package list simple)
    // Calcular grupos de paquetes (usar memo si existe, sino calcular)
    let groups = {
        let filter_mode = *state.filter_mode.borrow();
        let mut groups_memo_ref = state.groups_memo.borrow_mut();
        
        if let Some(ref memo_groups) = *groups_memo_ref {
            // Usar grupos memoizados
            if filter_mode {
                // Aplicar filtro a los grupos memoizados
                let mut filtered_groups = Vec::new();
                for group in memo_groups.iter() {
                    let filtered_packages: Vec<_> = group.packages.iter()
                        .filter(|p| p.status.starts_with("STATUT_CHARGER"))
                        .cloned()
                        .collect();
                    
                    if !filtered_packages.is_empty() {
                        filtered_groups.push(PackageGroup {
                            title: group.title.clone(),
                            count: filtered_packages.len(),
                            packages: filtered_packages,
                        });
                    }
                }
                filtered_groups
            } else {
                memo_groups.clone()
            }
        } else {
            // Calcular grupos (no están memoizados)
            let mut packages: Vec<Package> = session.packages.values().cloned().collect();
            
            // Aplicar filtro si está activo
            if filter_mode {
                packages.retain(|p| p.status.starts_with("STATUT_CHARGER"));
                log::info!("🔍 [APP] Filtro activo: {} paquetes después de filtrar", packages.len());
            }
            
            let computed_groups = group_packages_by_address(packages);
            
            // Guardar en memo (solo si no hay filtro activo, para reutilizar)
            if !filter_mode {
                *groups_memo_ref = Some(computed_groups.clone());
                log::info!("💾 [APP] Grupos memoizados para reutilización");
            }
            
            computed_groups
        }
    };
    
    // Registrar listener para eventos del mapa (packageSelected)
    setup_map_selection_listener(state.clone(), groups.len());
    
    // Callbacks para bottom sheet
    let on_toggle_sheet = {
        let state_clone = state.clone();
        Rc::new(move || {
            let current_state = state_clone.sheet_state.borrow().clone();
            let next_state = if current_state == "collapsed" {
                "half".to_string()
            } else if current_state == "half" {
                "full".to_string()
            } else {
                "collapsed".to_string()
            };
            state_clone.set_sheet_state(next_state);
        })
    };
    
    let on_close_sheet = {
        let state_clone = state.clone();
        Rc::new(move || {
            state_clone.set_sheet_state("collapsed".to_string());
        })
    };
    
    let on_package_selected = {
        let state_clone = state.clone();
        let groups_clone = groups.clone();
        let session_id = session.session_id.clone();
        Rc::new(move |index: usize| {
            log::info!("📦 [APP] Paquete seleccionado en bottom sheet: index={}", index);
            
            let edit_mode = *state_clone.edit_mode.borrow();
            
            // Si está en modo edición, manejar reordenamiento
            if edit_mode {
                let edit_origin = *state_clone.edit_origin_idx.borrow();
                
                if let Some(origin_idx) = edit_origin {
                    // Ya tenemos origen, este es el destino - ejecutar reordenamiento
                    if origin_idx != index {
                        log::info!("🔄 Reordenando desde bottom sheet: origen {} → destino {}", origin_idx, index);
                        
                        // Reordenar grupos
                        let mut groups_reordered = groups_clone.clone();
                        if origin_idx < groups_reordered.len() && index < groups_reordered.len() {
                            let group_to_move = groups_reordered.remove(origin_idx);
                            let dest_idx = if index > origin_idx { index - 1 } else { index };
                            groups_reordered.insert(dest_idx.min(groups_reordered.len()), group_to_move);
                            
                            // Actualizar route_order de todos los paquetes
                            let mut new_order = 0;
                            let mut trackings_order: Vec<(String, usize)> = Vec::new();
                            for group in &groups_reordered {
                                for pkg in &group.packages {
                                    trackings_order.push((pkg.tracking.clone(), new_order));
                                }
                                new_order += 1;
                            }
                            
                            // Obtener sesión actual para actualizarla
                            if let Some(mut updated_session) = state_clone.session.get_session() {
                                // Guardar posiciones anteriores ANTES de actualizar
                                let mut old_positions: Vec<(String, usize)> = Vec::new();
                                for (tracking, _) in &trackings_order {
                                    if let Some(pkg) = updated_session.packages.get(tracking) {
                                        let old_pos = pkg.route_order.unwrap_or(pkg.original_order);
                                        old_positions.push((tracking.clone(), old_pos));
                                    }
                                }
                                
                                // Actualizar route_order
                                for (tracking, order) in &trackings_order {
                                    if let Some(pkg) = updated_session.packages.get_mut(tracking) {
                                        pkg.route_order = Some(*order);
                                    }
                                }
                                
                                // Guardar sesión actualizada
                                state_clone.session.set_session(Some(updated_session.clone()));
                                
                                // Crear cambios de sincronización
                                let trackings_order_for_sync = trackings_order.clone();
                                let old_positions_for_sync = old_positions.clone();
                                let updated_session_for_sync = updated_session.clone();
                                let state_for_sync = state_clone.clone();
                                
                                wasm_bindgen_futures::spawn_local(async move {
                                    use crate::services::SyncService;
                                    use crate::models::sync::Change;
                                    use chrono::Utc;
                                    
                                    let sync_service = SyncService::new();
                                    
                                    let mut changes = Vec::new();
                                    for (tracking, new_pos) in trackings_order_for_sync {
                                        // Buscar la posición anterior
                                        let old_pos = old_positions_for_sync.iter()
                                            .find(|(t, _)| t == &tracking)
                                            .map(|(_, pos)| *pos)
                                            .unwrap_or(origin_idx);
                                        
                                        changes.push(Change::OrderChanged {
                                            package_internal_id: tracking.clone(),
                                            old_position: old_pos,
                                            new_position: new_pos,
                                            timestamp: Utc::now().timestamp(),
                                        });
                                    }
                                    
                                    // Sincronizar los cambios
                                    match sync_service.sync_session(&updated_session_for_sync, changes).await {
                                        crate::models::sync::SyncResult::Success { session: synced_session, .. } => {
                                            log::info!("✅ Cambios sincronizados exitosamente");
                                            state_for_sync.session.set_session(Some(synced_session));
                                        }
                                        crate::models::sync::SyncResult::Error { pending_changes, .. } => {
                                            log::warn!("⚠️ Error sincronizando, cambios guardados para reintentar: {} pendientes", pending_changes.len());
                                        }
                                        _ => {}
                                    }
                                    
                                    // Invalidar grupos memo y actualizar package list y mapa
                                    state_for_sync.invalidate_groups_memo();
                                    crate::rerender_app_with_type(crate::state::app_state::UpdateType::Incremental(crate::state::app_state::IncrementalUpdate::PackageList));
                                    crate::rerender_app_with_type(crate::state::app_state::UpdateType::Incremental(crate::state::app_state::IncrementalUpdate::MapPackages));
                                });
                                
                                log::info!("✅ Reordenamiento completado y sincronizado");
                            }
                        }
                        
                        // Limpiar origen
                        *state_clone.edit_origin_idx.borrow_mut() = None;
                    } else {
                        // Mismo índice, cancelar
                        *state_clone.edit_origin_idx.borrow_mut() = None;
                    }
                } else {
                    // Primer click - establecer como origen
                    *state_clone.edit_origin_idx.borrow_mut() = Some(index);
                    state_clone.set_selected_package_index(Some(index));
                    log::info!("📍 Origen establecido: {}", index);
                }
            } else {
                // Modo normal
                state_clone.set_selected_package_index(Some(index));
                
                // Sincronizar con mapa
                crate::utils::mapbox_ffi::update_selected_package(index as i32);
                
                // Centrar mapa en el paquete
                crate::utils::mapbox_ffi::center_map_on_package(index);
                
                // Hacer scroll al card (con delay para que el mapa se centre primero)
                use gloo_timers::callback::Timeout;
                Timeout::new(300, move || {
                    crate::utils::mapbox_ffi::scroll_to_selected_package(index);
                }).forget();
            }
        })
    };
    
    let bottom_sheet = render_bottom_sheet(
        state,
        session,
        &groups,
        on_toggle_sheet,
        on_close_sheet,
        on_package_selected,
    )?;
    
    append_child(&content, &bottom_sheet)?;
    
    append_child(&main_app, &content)?;
    
    // Asegurar que la barra de progreso esté en el DOM tras el render (evita que no aparezca hasta refrescar)
    {
        let state_progress = state.clone();
        let session_progress = session.clone();
        Timeout::new(0, move || {
            let _ = crate::dom::incremental::update_progress_bar(&state_progress, &session_progress);
        }).forget();
    }
    
    // Details modal - renderizar solo si hay details_package y show_details es true (como en Yew)
    let show_details = *state.show_details.borrow();
    if show_details {
        let details_package_opt = state.details_package.borrow().clone();
        if let Some((pkg, addr)) = details_package_opt.as_ref() {
            let on_close_details = {
                let state_clone = state.clone();
                Rc::new(move || {
                    let state_for_restore = state_clone.clone();
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("🔄 [SCROLL] Cerrando modal de detalles, programando restauración de scroll"));
                    state_clone.set_show_details(false);
                    // Restaurar posición de scroll después de cerrar el modal
                    // Usar delay más largo para asegurar que el modal se haya cerrado completamente
                    // Y limpiar la posición guardada después de restaurar
                    use gloo_timers::callback::Timeout;
                    Timeout::new(200, move || {
                        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("⏰ [SCROLL] Timeout completado, restaurando scroll ahora (y limpiando posición guardada)"));
                        state_for_restore.restore_package_list_scroll_position(true); // true = limpiar después de restaurar
                    }).forget();
                })
            };
            
            // Callbacks de edición
            let session_clone = session.clone();
            let state_clone = state.clone();
            let pkg_clone = pkg.clone();
            let addr_id = addr.address_id.clone();
            let session_id = session.session_id.clone();
            
            let on_edit_address = {
                let session_clone = session_clone.clone();
                let state_clone = state_clone.clone();
                let addr_id = addr_id.clone();
                let session_id = session_id.clone();
                Some(Rc::new(move |new_label: String| {
                    let session_clone = session_clone.clone();
                    let state_clone = state_clone.clone();
                    let addr_id = addr_id.clone();
                    let session_id = session_id.clone();
                    let new_label = new_label.clone();
                    
                    state_clone.session.set_loading(true);
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        use crate::viewmodels::session_viewmodel::SessionViewModel;
                        let vm = SessionViewModel::new();
                        match vm.update_address(&session_id, &addr_id, new_label).await {
                            Ok(updated_session) => {
                                state_clone.session.set_session(Some(updated_session.clone()));
                                
                                // Actualizar details_package si está abierto
                                let pkg_opt = {
                                    let borrow = state_clone.details_package.borrow();
                                    borrow.clone()
                                }; // El borrow se libera aquí explícitamente
                                if let Some((pkg, _)) = pkg_opt {
                                    if let Some(updated_addr) = updated_session.addresses.get(&addr_id) {
                                        *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                    }
                                }
                                
                                state_clone.session.set_loading(false);
                                crate::rerender_app();
                            }
                            Err(e) => {
                                log::error!("❌ Error actualizando dirección: {}", e);
                                *state_clone.edit_error_message.borrow_mut() = Some(e);
                                state_clone.session.set_loading(false);
                                crate::rerender_app();
                            }
                        }
                    });
                }) as Rc<dyn Fn(String)>)
            };
            
            let on_edit_door_code = {
                let session_clone = session_clone.clone();
                let state_clone = state_clone.clone();
                let addr_id = addr_id.clone();
                let session_id = session_id.clone();
                Some(Rc::new(move |new_code: String| {
                    let session_clone = session_clone.clone();
                    let state_clone = state_clone.clone();
                    let addr_id = addr_id.clone();
                    let session_id = session_id.clone();
                    let door_code = Some(new_code.trim().to_string());
                    
                    state_clone.session.set_loading(true);
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        use crate::viewmodels::session_viewmodel::SessionViewModel;
                        let vm = SessionViewModel::new();
                        match vm.update_address_fields(&session_id, &addr_id, door_code, None, None).await {
                            Ok(updated_session) => {
                                state_clone.session.set_session(Some(updated_session.clone()));
                                
                                // Actualizar details_package
                                let pkg_opt = {
                                    let borrow = state_clone.details_package.borrow();
                                    borrow.clone()
                                }; // El borrow se libera aquí explícitamente
                                if let Some((pkg, _)) = pkg_opt {
                                    if let Some(updated_addr) = updated_session.addresses.get(&addr_id) {
                                        *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                    }
                                }
                                
                                state_clone.session.set_loading(false);
                                crate::rerender_app();
                            }
                            Err(e) => {
                                log::error!("❌ Error actualizando código de puerta: {}", e);
                                *state_clone.edit_error_message.borrow_mut() = Some(e);
                                state_clone.session.set_loading(false);
                                crate::rerender_app();
                            }
                        }
                    });
                }) as Rc<dyn Fn(String)>)
            };
            
            let on_edit_mailbox = {
                let session_clone = session_clone.clone();
                let state_clone = state_clone.clone();
                let addr_id = addr_id.clone();
                let session_id = session_id.clone();
                Some(Rc::new(move |new_value: bool| {
                    let session_clone = session_clone.clone();
                    let state_clone = state_clone.clone();
                    let addr_id = addr_id.clone();
                    let session_id = session_id.clone();
                    
                    // Marcar como guardando
                    *state_clone.saving_mailbox.borrow_mut() = true;
                    state_clone.session.set_loading(true);
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        use crate::viewmodels::session_viewmodel::SessionViewModel;
                        log::info!("📬 [MAILBOX] Iniciando actualización de mailbox: address_id={}, nuevo_valor={}", addr_id, new_value);
                        
                        let vm = SessionViewModel::new();
                        match vm.update_address_fields(&session_id, &addr_id, None, Some(new_value), None).await {
                            Ok(updated_session) => {
                                log::info!("✅ [MAILBOX] Actualización exitosa, verificando dirección actualizada");
                                
                                // Verificar que la dirección se actualizó correctamente
                                if let Some(updated_addr) = updated_session.addresses.get(&addr_id) {
                                    log::info!("📬 [MAILBOX] Dirección actualizada - mailbox_access={:?}", updated_addr.mailbox_access);
                                } else {
                                    log::warn!("⚠️ [MAILBOX] Dirección no encontrada después de actualizar: {}", addr_id);
                                }
                                
                                // Primero actualizar la sesión
                                state_clone.session.set_session(Some(updated_session.clone()));
                                
                                // Luego actualizar details_package (después de que set_session haya liberado sus borrows)
                                let pkg_opt = {
                                    let borrow = state_clone.details_package.borrow();
                                    borrow.clone()
                                }; // El borrow se libera aquí explícitamente
                                
                                if let Some((pkg, _)) = pkg_opt {
                                    if let Some(updated_addr) = updated_session.addresses.get(&addr_id) {
                                        log::info!("📬 [MAILBOX] Actualizando details_package con mailbox_access={:?}", updated_addr.mailbox_access);
                                        *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                    }
                                }
                                
                                *state_clone.saving_mailbox.borrow_mut() = false;
                                state_clone.session.set_loading(false);
                                crate::rerender_app();
                            }
                            Err(e) => {
                                log::error!("❌ [MAILBOX] Error actualizando acceso BAL: {}", e);
                                *state_clone.edit_error_message.borrow_mut() = Some(e);
                                *state_clone.saving_mailbox.borrow_mut() = false;
                                state_clone.session.set_loading(false);
                                crate::rerender_app();
                            }
                        }
                    });
                }) as Rc<dyn Fn(bool)>)
            };
            
            let on_edit_driver_notes = {
                let session_clone = session_clone.clone();
                let state_clone = state_clone.clone();
                let addr_id = addr_id.clone();
                let session_id = session_id.clone();
                Some(Rc::new(move |new_notes: String| {
                    let session_clone = session_clone.clone();
                    let state_clone = state_clone.clone();
                    let addr_id = addr_id.clone();
                    let session_id = session_id.clone();
                    let driver_notes = Some(new_notes.trim().to_string());
                    
                    state_clone.session.set_loading(true);
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        use crate::viewmodels::session_viewmodel::SessionViewModel;
                        let vm = SessionViewModel::new();
                        match vm.update_address_fields(&session_id, &addr_id, None, None, driver_notes).await {
                            Ok(updated_session) => {
                                state_clone.session.set_session(Some(updated_session.clone()));
                                
                                // Actualizar details_package
                                let pkg_opt = {
                                    let borrow = state_clone.details_package.borrow();
                                    borrow.clone()
                                }; // El borrow se libera aquí explícitamente
                                if let Some((pkg, _)) = pkg_opt {
                                    if let Some(updated_addr) = updated_session.addresses.get(&addr_id) {
                                        *state_clone.details_package.borrow_mut() = Some((pkg, updated_addr.clone()));
                                    }
                                }
                                
                                state_clone.session.set_loading(false);
                                crate::rerender_app();
                            }
                            Err(e) => {
                                log::error!("❌ Error actualizando notas chofer: {}", e);
                                *state_clone.edit_error_message.borrow_mut() = Some(e);
                                state_clone.session.set_loading(false);
                                crate::rerender_app();
                            }
                        }
                    });
                }) as Rc<dyn Fn(String)>)
            };
            
            let on_mark_problematic = {
                let session_clone = session_clone.clone();
                let state_clone = state_clone.clone();
                let addr_id = addr_id.clone();
                let session_id = session_id.clone();
                let pkg_clone = pkg_clone.clone();
                Some(Rc::new(move || {
                    let session_clone = session_clone.clone();
                    let state_clone = state_clone.clone();
                    let addr_id = addr_id.clone();
                    let session_id = session_id.clone();
                    let pkg_clone = pkg_clone.clone();
                    
                    log::info!("⚠️ Marcando dirección como problemática: {}", addr_id);
                    state_clone.session.set_loading(true);
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        use crate::viewmodels::session_viewmodel::SessionViewModel;
                        let vm = SessionViewModel::new();
                        match vm.mark_as_problematic(&session_id, &addr_id).await {
                            Ok(updated_session) => {
                                log::info!("✅ Dirección marcada como problemática exitosamente");
                                
                                state_clone.session.set_session(Some(updated_session.clone()));
                                
                                // Actualizar details_package con la dirección actualizada
                                // El backend marca los paquetes como problemáticos automáticamente cuando la dirección tiene coordenadas 0.0, 0.0
                                if let Some(updated_addr) = updated_session.addresses.get(&addr_id) {
                                    // Buscar el paquete actualizado en la sesión
                                    if let Some(updated_pkg) = updated_session.packages.values().find(|p| p.address_id == addr_id) {
                                        *state_clone.details_package.borrow_mut() = Some((updated_pkg.clone(), updated_addr.clone()));
                                    } else {
                                        // Si no encontramos el paquete actualizado, usar el original pero actualizar la dirección
                                        *state_clone.details_package.borrow_mut() = Some((pkg_clone.clone(), updated_addr.clone()));
                                    }
                                }
                                
                                state_clone.session.set_loading(false);
                                crate::rerender_app();
                            }
                            Err(e) => {
                                log::error!("❌ Error marcando como problemático: {}", e);
                                *state_clone.edit_error_message.borrow_mut() = Some(e);
                                state_clone.session.set_loading(false);
                                crate::rerender_app();
                            }
                        }
                    });
                }) as Rc<dyn Fn()>)
            };
            
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
            append_child(&main_app, &details_modal)?;
        }
    }
    
    // Status change modal
    if let Ok(Some(status_modal)) = render_status_change_modal(state) {
        append_child(&main_app, &status_modal)?;
    }
    
    let show_scanner = *state.show_scanner.borrow();
    if show_scanner {
                let on_close_scanner = {
                    let state_clone = state.clone();
                    Rc::new(move || {
                        state_clone.set_show_scanner(false);
                    })
                };
        
        let on_barcode_detected = {
            let state_clone = state.clone();
            let groups_clone = groups.clone();
            Rc::new(move |barcode: String| {
                log::info!("📱 [APP] Código escaneado: {}", barcode);
                
                // Buscar group_idx del paquete
                let group_idx_opt = find_group_idx_by_tracking(&barcode, &groups_clone);
                
                if let Some(group_idx) = group_idx_opt {
                    log::info!("✅ [APP] group_idx encontrado: {}", group_idx);
                    
                    // Actualizar índice seleccionado
                    state_clone.set_selected_package_index(Some(group_idx));
                    
                    // Abrir bottom sheet si está collapsed
                    let current_state = state_clone.sheet_state.borrow().clone();
                    if current_state == "collapsed" {
                        state_clone.set_sheet_state("half".to_string());
                    }
                    
                    // Sincronizar con mapa
                    crate::utils::mapbox_ffi::update_selected_package(group_idx as i32);
                    crate::utils::mapbox_ffi::center_map_on_package(group_idx);
                    
                    // Hacer scroll al card
                    use gloo_timers::callback::Timeout;
                    Timeout::new(300, move || {
                        crate::utils::mapbox_ffi::scroll_to_selected_package(group_idx);
                    }).forget();
                } else {
                    log::warn!("⚠️ [APP] No se encontró group_idx para tracking: {}", barcode);
                }
                
                // Cerrar scanner
                state_clone.set_show_scanner(false);
            })
        };
        
        let scanner_modal = render_scanner(on_close_scanner, on_barcode_detected, state.language.borrow().as_str())?;
        append_child(&main_app, &scanner_modal)?;
    }
    
    // Modal de tracking - siempre renderizar, mostrar/ocultar con CSS (como en Yew)
    let on_close_tracking = {
        let state_clone = state.clone();
        Rc::new(move || {
            state_clone.set_show_tracking_modal(false);
        })
    };

    let on_tracking_selected = {
        let state_clone = state.clone();
        let groups_clone = groups.clone();
        Rc::new(move |tracking: String| {
            log::info!("🔍 [APP] Tracking seleccionado desde modal: {}", tracking);
            
            // Buscar group_idx
            let group_idx_opt = find_group_idx_by_tracking(&tracking, &groups_clone);
            
            if let Some(group_idx) = group_idx_opt {
                log::info!("✅ [APP] group_idx encontrado: {}", group_idx);
                
                // Actualizar índice seleccionado
                state_clone.set_selected_package_index(Some(group_idx));
                
                // Abrir bottom sheet si está collapsed
                let current_state = state_clone.sheet_state.borrow().clone();
                if current_state == "collapsed" {
                    state_clone.set_sheet_state("half".to_string());
                }
                
                // Sincronizar con mapa
                crate::utils::mapbox_ffi::update_selected_package(group_idx as i32);
                crate::utils::mapbox_ffi::center_map_on_package(group_idx);
                
                // Hacer scroll al card
                use gloo_timers::callback::Timeout;
                Timeout::new(300, move || {
                    crate::utils::mapbox_ffi::scroll_to_selected_package(group_idx);
                }).forget();
            } else {
                log::warn!("⚠️ [APP] No se encontró group_idx para tracking: {}", tracking);
            }
            
            // Cerrar modal
            state_clone.set_show_tracking_modal(false);
        })
    };
    
    let tracking_modal = render_tracking_modal(session, on_tracking_selected, on_close_tracking, state.language.borrow().as_str())?;
    append_child(&main_app, &tracking_modal)?;
    
    // Settings popup - siempre renderizar, mostrar/ocultar con CSS (como tracking modal)
    let on_close_settings = {
        let state_clone = state.clone();
        Rc::new(move || {
            state_clone.set_show_settings(false);
        })
    };
    
    let on_logout = {
        let state_clone = state.clone();
        Rc::new(move || {
            log::info!("🚪 [APP] Logout");
            // Limpiar auth state
            state_clone.auth.set_logged_in(false);
            state_clone.auth.set_username(None);
            state_clone.auth.set_token(None);
            state_clone.auth.set_company_id(None);
            // Limpiar sesión
            state_clone.session.set_session(None);
            state_clone.notify_subscribers();
        })
    };
    
    let settings_popup = render_settings_popup(state, on_close_settings, on_logout)?;
    append_child(&main_app, &settings_popup)?;
    
    Ok(main_app)
}

/// Helper: Buscar group_idx por tracking
fn find_group_idx_by_tracking(
    tracking: &str,
    groups: &[PackageGroup],
) -> Option<usize> {
    groups.iter()
        .enumerate()
        .find(|(_, group)| {
            group.packages.iter().any(|p| p.tracking == tracking)
        })
        .map(|(idx, _)| idx)
}

/// Crear header con título, botones y sync indicator
fn create_header(state: &AppState, session: Option<&crate::models::session::DeliverySession>) -> Result<Element, JsValue> {
    use crate::utils::i18n::t;
    
    let header = ElementBuilder::new("div")?
        .class("app-header")
        .build();
    
    // Título
    let title = ElementBuilder::new("h1")?
        .text("Route Optimizer")
        .build();
    append_child(&header, &title)?;
    
    // Actions container
    let actions = ElementBuilder::new("div")?
        .class("header-actions")
        .build();
    
    // Sync indicator (solo si no está Synced)
    if let Some(sync_indicator) = render_sync_indicator(state)? {
        append_child(&actions, &sync_indicator)?;
    }
    
    let language = state.language.borrow().clone();
    let loading = state.session.get_loading();
    let has_session = session.is_some();
    
    // Optimize route button (🎯)
    let optimize_btn = ElementBuilder::new("button")?
        .class("btn-icon-header btn-optimize-mini")
        .attr("title", &t("optimiser", &language))?
        .text("🎯")
        .build();
    
    if loading || !has_session {
        set_attribute(&optimize_btn, "disabled", "true")?;
    }
    
    {
        let state_clone = state.clone();
        let session_opt = session.map(|s| s.session_id.clone());
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            if let Some(session_id) = &session_opt {
                let state_clone = state_clone.clone();
                let session_id_clone = session_id.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    use crate::viewmodels::SessionViewModel;
                    let vm = SessionViewModel::new();
                    
                    *state_clone.session.loading.borrow_mut() = true;
                    // Actualizar header inmediatamente para mostrar loading state
                    let has_session = state_clone.session.get_session().is_some();
                    if let Err(e) = crate::dom::incremental::update_header(&state_clone, has_session) {
                        log::error!("❌ Error actualizando header: {:?}", e);
                    }
                    
                    match vm.optimize_route(&session_id_clone).await {
                        Ok(updated_session) => {
                            log::info!("✅ Optimización completada: {} paquetes", updated_session.stats.total_packages);
                            state_clone.session.set_session(Some(updated_session));
                            state_clone.session.set_loading(false);
                            state_clone.invalidate_groups_memo();
                            
                            // Actualizar header después de completar
                            let has_session_after = state_clone.session.get_session().is_some();
                            if let Err(e) = crate::dom::incremental::update_header(&state_clone, has_session_after) {
                                log::error!("❌ Error actualizando header: {:?}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("❌ Error optimizando ruta: {}", e);
                            state_clone.session.set_loading(false);
                            
                            // Actualizar header después del error
                            let has_session_after = state_clone.session.get_session().is_some();
                            if let Err(e) = crate::dom::incremental::update_header(&state_clone, has_session_after) {
                                log::error!("❌ Error actualizando header: {:?}", e);
                            }
                            
                            // Mostrar alert si el error es por falta de localización
                            if e.to_lowercase().contains("geolocalización") || 
                               e.to_lowercase().contains("ubicación") ||
                               e.to_lowercase().contains("location") {
                                if let Some(window) = web_sys::window() {
                                    let _ = window.alert_with_message(
                                        "⚠️ Debes activar tu localización primero.\n\nPor favor, haz clic en el botón de geolocalización (📍) en el mapa para activar tu ubicación antes de optimizar la ruta."
                                    );
                                }
                            }
                        }
                    }
                });
            }
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        optimize_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&actions, &optimize_btn)?;
    
    // Search tracking button (🔍)
    let search_btn = ElementBuilder::new("button")?
        .class("btn-icon-header btn-tracking-search")
        .attr("title", &t("buscar_tracking", &language))?
        .text("🔍")
        .build();
    
    if loading || !has_session {
        set_attribute(&search_btn, "disabled", "true")?;
    }
    
    {
                let state_clone = state.clone();
                let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                    state_clone.set_show_tracking_modal(true);
                }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        search_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&actions, &search_btn)?;
    
    // Scanner button (📷)
    let scanner_btn = ElementBuilder::new("button")?
        .class("btn-icon-header btn-scanner")
        .attr("title", &t("scanner", &language))?
        .text("📷")
        .build();
    
    if loading {
        set_attribute(&scanner_btn, "disabled", "true")?;
    }
    
    {
                let state_clone = state.clone();
                let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                    state_clone.set_show_scanner(true);
                }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        scanner_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&actions, &scanner_btn)?;
    
    // Refresh button (🔄)
    let refresh_btn = ElementBuilder::new("button")?
        .class("btn-icon-header btn-refresh")
        .attr("title", &t("rafraichir", &language))?
        .text(if loading { "⏳" } else { "🔄" })
        .build();
    
    if loading || !has_session {
        set_attribute(&refresh_btn, "disabled", "true")?;
    }
    
    {
        let state_clone = state.clone();
        let session_opt = session.map(|s| (s.session_id.clone(), s.driver.driver_id.clone(), s.driver.company_id.clone()));
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            if let Some((session_id, username, societe)) = &session_opt {
                let state_clone = state_clone.clone();
                let session_id_clone = session_id.clone();
                let username_clone = username.clone();
                let societe_clone = societe.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    use crate::viewmodels::SessionViewModel;
                    use crate::services::SyncService;
                    
                    let vm = SessionViewModel::new();
                    let sync_service = SyncService::new();
                    
                    *state_clone.session.loading.borrow_mut() = true;
                    // Actualizar header inmediatamente para mostrar loading state
                    let has_session = state_clone.session.get_session().is_some();
                    if let Err(e) = crate::dom::incremental::update_header(&state_clone, has_session) {
                        log::error!("❌ Error actualizando header: {:?}", e);
                    }
                    
                    // 1. Procesar cambios pendientes
                    log::info!("🔄 Procesando cambios pendientes antes de refrescar...");
                    if let Err(e) = sync_service.process_pending_queue().await {
                        log::warn!("⚠️ Error procesando cambios pendientes: {}", e);
                    }
                    
                    // 2. Sync incremental
                    match vm.sync_incremental(&session_id_clone, &username_clone, &societe_clone, None).await {
                        Ok(updated_session) => {
                            log::info!("✅ Sincronización incremental completada: {} paquetes", updated_session.stats.total_packages);
                            
                            // Actualizar sesión en el estado
                            state_clone.session.set_session(Some(updated_session.clone()));
                            state_clone.session.set_loading(false);
                            state_clone.invalidate_groups_memo();
                            
                            // Reactivar botones del header después del refresh
                            let has_session = state_clone.session.get_session().is_some();
                            if let Err(e) = crate::dom::incremental::update_header(&state_clone, has_session) {
                                log::error!("❌ Error actualizando header: {:?}", e);
                            }
                            
                            // ACTUALIZAR LISTA DE PAQUETES (esto es lo que faltaba)
                            log::info!("🔄 [REFRESH] Actualizando lista de paquetes después del sync...");
                            
                            // Actualizar progress bar primero (usando manipulación directa del DOM)
                            // Asegurar que el header del bottom sheet existe antes de actualizar
                            if get_element_by_id("drag-handle-container").is_none() {
                                log::warn!("⚠️ [REFRESH] drag-handle-container no encontrado, el header del bottom sheet no existe");
                            } else {
                                if let Err(e) = crate::dom::incremental::update_progress_bar(&state_clone, &updated_session) {
                                    log::warn!("⚠️ [REFRESH] Error actualizando progress bar: {:?}", e);
                                }
                            }
                            
                            // Luego actualizar la lista de paquetes (solo si el header existe)
                            if get_element_by_id("drag-handle-container").is_some() {
                                crate::rerender_app_with_type(crate::state::app_state::UpdateType::Incremental(
                                    crate::state::app_state::IncrementalUpdate::PackageList
                                ));
                            } else {
                                log::warn!("⚠️ [REFRESH] No se puede actualizar lista de paquetes: header del bottom sheet no existe");
                            }
                            
                            // Actualizar mapa (con delay para que el mapa esté listo)
                            use gloo_timers::callback::Timeout;
                            use crate::viewmodels::map_viewmodel::MapViewModel;
                            use crate::views::group_packages_by_address;
                            
                            let filter_mode = *state_clone.filter_mode.borrow();
                            let mut packages_vec: Vec<_> = updated_session.packages.values().cloned().collect();
                            if filter_mode {
                                packages_vec.retain(|p| p.status.starts_with("STATUT_CHARGER"));
                            }
                            
                            let groups = group_packages_by_address(packages_vec);
                            let packages_for_map = MapViewModel::prepare_packages_for_map(&groups, &updated_session);
                            let packages_json = serde_json::to_string(&packages_for_map)
                                .unwrap_or_else(|_| "[]".to_string());
                            
                            // Actualizar mapa también
                            Timeout::new(300, move || {
                                log::info!("🗺️ [REFRESH] Actualizando mapa con {} paquetes", packages_for_map.len());
                                crate::utils::mapbox_ffi::add_packages_to_map(&packages_json);
                            }).forget();
                            
                            log::info!("✅ [REFRESH] Actualización completa: lista y mapa actualizados");
                        }
                        Err(e) => {
                            log::error!("❌ Error en sincronización incremental: {}", e);
                            state_clone.session.set_loading(false);
                            state_clone.session.set_error(Some(e));
                            
                            // Reactivar botones del header incluso si hay error
                            let has_session = state_clone.session.get_session().is_some();
                            if let Err(e) = crate::dom::incremental::update_header(&state_clone, has_session) {
                                log::error!("❌ Error actualizando header: {:?}", e);
                            }
                        }
                    }
                });
            }
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        refresh_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&actions, &refresh_btn)?;
    
    // Settings button (⚙️)
    let settings_btn = ElementBuilder::new("button")?
        .class("btn-icon-header btn-settings")
        .attr("title", &t("parametres", &language))?
        .text("⚙️")
        .build();
    
    {
        let state_clone = state.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            state_clone.set_show_settings(true);
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        settings_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&actions, &settings_btn)?;
    append_child(&header, &actions)?;
    
    Ok(header)
}

/// Crear container para el mapa
fn create_map_container(
    state: &AppState,
    session: &crate::models::session::DeliverySession,
) -> Result<Element, JsValue> {
    // Crear container con ID "map" directamente (Mapbox espera este ID)
    let map_container = ElementBuilder::new("div")?
        .class("map-container")
        .attr("id", "map")?
        .build();
    
    // Detectar dark mode
    let is_dark = web_sys::window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
        .flatten()
        .map(|mq| mq.matches())
        .unwrap_or(false);
    
    // Preparar paquetes para el mapa
    let packages: Vec<Package> = session.packages.values().cloned().collect();
    let groups = group_packages_by_address(packages);
    let map_packages = MapViewModel::prepare_packages_for_map(&groups, session);
    let packages_json = serde_json::to_string(&map_packages)
        .unwrap_or_else(|_| "[]".to_string());
    
    // Clonar valores necesarios para el closure
    let packages_json_for_closure = packages_json;
    let selected_idx = *state.selected_package_index.borrow();
    let is_dark_for_closure = is_dark;
    
    // Usar setTimeout con gloo_timers para asegurar que el elemento esté en el DOM antes de inicializar
    // Aumentar delay a 200ms para asegurar que el DOM esté completamente renderizado
    // (especialmente importante después de login cuando se re-renderiza toda la app)
    Timeout::new(200, move || {
        console::log_1(&JsValue::from_str("🗺️ [MAP] Inicializando Mapbox después de que el elemento esté en el DOM"));
        
        // Verificar que el elemento existe y tiene dimensiones antes de inicializar
        if let Some(map_element) = crate::dom::get_element_by_id("map") {
            // Verificar que el elemento tiene dimensiones (está visible)
            if let Ok(html_element) = map_element.dyn_into::<web_sys::HtmlElement>() {
                let width = html_element.offset_width();
                let height = html_element.offset_height();
                
                if width > 0 && height > 0 {
                    console::log_1(&JsValue::from_str(&format!("✅ [MAP] Elemento encontrado con dimensiones: {}x{}", width, height)));
                    
                    // Inicializar mapa
                    mapbox_ffi::init_mapbox("map", is_dark_for_closure);
                    
                    // Agregar paquetes al mapa después de un pequeño delay adicional
                    // para asegurar que el mapa esté completamente inicializado
                    let packages_json_clone = packages_json_for_closure.clone();
                    let selected_idx_clone = selected_idx;
                    Timeout::new(100, move || {
                        mapbox_ffi::add_packages_to_map(&packages_json_clone);
                        
                        // Actualizar paquete seleccionado si hay uno
                        if let Some(idx) = selected_idx_clone {
                            mapbox_ffi::update_selected_package(idx as i32);
                        }
                        
                        console::log_1(&JsValue::from_str("✅ [MAP] Paquetes agregados al mapa"));
                    }).forget();
                    
                    console::log_1(&JsValue::from_str("✅ [MAP] Mapbox inicializado exitosamente"));
                } else {
                    console::warn_1(&JsValue::from_str(&format!("⚠️ [MAP] Elemento 'map' encontrado pero sin dimensiones: {}x{}, reintentando...", width, height)));
                    // Reintentar después de otro delay si no tiene dimensiones
                    let packages_json_retry = packages_json_for_closure.clone();
                    let selected_idx_retry = selected_idx;
                    let is_dark_retry = is_dark_for_closure;
                    Timeout::new(300, move || {
                        if let Some(map_el) = crate::dom::get_element_by_id("map") {
                            if let Ok(html_el) = map_el.dyn_into::<web_sys::HtmlElement>() {
                                if html_el.offset_width() > 0 && html_el.offset_height() > 0 {
                                    mapbox_ffi::init_mapbox("map", is_dark_retry);
                                    Timeout::new(100, move || {
                                        mapbox_ffi::add_packages_to_map(&packages_json_retry);
                                        if let Some(idx) = selected_idx_retry {
                                            mapbox_ffi::update_selected_package(idx as i32);
                                        }
                                    }).forget();
                                }
                            }
                        }
                    }).forget();
                }
            } else {
                console::warn_1(&JsValue::from_str("⚠️ [MAP] Elemento 'map' no es un HtmlElement"));
            }
        } else {
            console::warn_1(&JsValue::from_str("⚠️ [MAP] Elemento 'map' no encontrado, reintentando..."));
            // Reintentar después de otro delay si no se encuentra
            let packages_json_retry = packages_json_for_closure.clone();
            let selected_idx_retry = selected_idx;
            let is_dark_retry = is_dark_for_closure;
            Timeout::new(500, move || {
                if let Some(map_el) = crate::dom::get_element_by_id("map") {
                    if let Ok(html_el) = map_el.dyn_into::<web_sys::HtmlElement>() {
                        if html_el.offset_width() > 0 && html_el.offset_height() > 0 {
                            mapbox_ffi::init_mapbox("map", is_dark_retry);
                            Timeout::new(100, move || {
                                mapbox_ffi::add_packages_to_map(&packages_json_retry);
                                if let Some(idx) = selected_idx_retry {
                                    mapbox_ffi::update_selected_package(idx as i32);
                                }
                            }).forget();
                        }
                    }
                }
            }).forget();
        }
    }).forget();
    
    Ok(map_container)
}

/// Obtener altura del header (de CSS variable o default)
fn get_header_height() -> String {
    // Por defecto 60px, pero debería leerse de CSS variable
    "60".to_string()
}

/// Configurar listener para eventos de selección del mapa
fn setup_map_selection_listener(state: crate::state::app_state::AppState, groups_count: usize) {
    if let Some(win) = web_sys::window() {
        let state_clone = state.clone();
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: wasm_bindgen::JsValue| {
            log::info!("📡 [MAP] Evento 'packageSelected' recibido");
            
            // Obtener detail.index del evento custom
            if let Ok(detail) = js_sys::Reflect::get(&event, &wasm_bindgen::JsValue::from_str("detail")) {
                if let Ok(index_val) = js_sys::Reflect::get(&detail, &wasm_bindgen::JsValue::from_str("index")) {
                    if let Some(index_f64) = index_val.as_f64() {
                        let package_index = index_f64 as usize;
                        
                        log::info!("📍 [MAP] group_idx recibido del mapa: {}", package_index);
                        
                        // Validar índice
                        if package_index >= groups_count {
                            log::warn!("⚠️ [MAP] group_idx {} >= grupos disponibles {}, ignorando", 
                                      package_index, groups_count);
                            return;
                        }
                        
                        // Actualizar índice seleccionado
                        state_clone.set_selected_package_index(Some(package_index));
                        
                        // Abrir bottom sheet si está collapsed
                        let current_state = state_clone.sheet_state.borrow().clone();
                        if current_state == "collapsed" {
                            state_clone.set_sheet_state("half".to_string());
                            log::info!("📱 [MAP] Bottom sheet abierto desde colapsado → half");
                        }
                        
                        // Hacer scroll al card seleccionado (con delay para que el sheet se abra)
                        use gloo_timers::callback::Timeout;
                        Timeout::new(300, move || {
                            crate::utils::mapbox_ffi::scroll_to_selected_package(package_index);
                        }).forget();
                        
                        log::info!("✅ [MAP] Selección sincronizada con bottom sheet");
                    }
                }
            }
        }) as Box<dyn FnMut(wasm_bindgen::JsValue)>);
        
        // Registrar listener (se mantiene vivo con forget)
        if win.add_event_listener_with_callback("packageSelected", closure.as_ref().unchecked_ref()).is_ok() {
            log::info!("✅ [MAP] Listener 'packageSelected' registrado");
            closure.forget();
        } else {
            log::error!("❌ [MAP] Error registrando listener 'packageSelected'");
        }
    }
}
