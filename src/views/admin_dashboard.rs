// ============================================================================
// ADMIN DASHBOARD - Dashboard administrativo completo
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::{Element, console};
use wasm_bindgen::closure::Closure;
use crate::dom::{ElementBuilder, append_child, add_class, remove_class, set_attribute};
use crate::dom::events::on_click;
use crate::state::app_state::AppState;
use crate::models::admin::{AdminDistrict, AdminTournee, StatusChangeRequest};
use crate::models::session::DeliverySession;
use crate::services::api_client::ApiClient;
use crate::views::bottom_sheet::{render_reusable_bottom_sheet, create_simple_header};
use crate::views::sync_indicator::render_sync_indicator;
use crate::utils::i18n::t;
use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;

/// Renderizar dashboard admin completo
pub fn render_admin_dashboard(state: &AppState) -> Result<Element, JsValue> {
    use crate::views::{render_settings_popup, render_details_modal};
    
    console::log_1(&JsValue::from_str("👑 [ADMIN] render_admin_dashboard() llamado"));
    
    let container = ElementBuilder::new("div")?
        .class("admin-container")
        .build();
    
    // Header
    let header = create_admin_header(state)?;
    append_child(&container, &header)?;
    
    // Dashboard stats (siempre visible)
    let dashboard = create_admin_dashboard(state)?;
    append_child(&container, &dashboard)?;
    
    // Si hay sesión seleccionada, mostrar grid
    if state.admin_selected_tournee_session.borrow().is_some() {
        let packages_view = create_packages_grid_view(state)?;
        append_child(&container, &packages_view)?;
    }
    
    // Bottom sheet SIEMPRE visible (para cambiar entre sesiones rápidamente)
    let bottom_sheet = create_districts_bottom_sheet(state)?;
    append_child(&container, &bottom_sheet)?;
    
    // Settings popup - siempre renderizar, mostrar/ocultar con CSS (como tracking modal)
    let on_close_settings = {
        let state_clone = state.clone();
        Rc::new(move || {
            state_clone.set_show_settings(false);
            crate::rerender_app();
        })
    };
    
    let on_logout = {
        let state_clone = state.clone();
        Rc::new(move || {
            log::info!("🚪 [ADMIN] Logout");
            // Limpiar auth state
            state_clone.auth.set_logged_in(false);
            *state_clone.admin_mode.borrow_mut() = false;
            *state_clone.admin_districts.borrow_mut() = Vec::new();
            *state_clone.admin_total_packages.borrow_mut() = 0;
            *state_clone.admin_username.borrow_mut() = None;
            *state_clone.admin_password.borrow_mut() = None;
            *state_clone.admin_societe.borrow_mut() = None;
            *state_clone.admin_selected_tournee.borrow_mut() = None;
            *state_clone.admin_selected_tournee_session.borrow_mut() = None;
            // Limpiar credenciales de localStorage
            use crate::services::OfflineService;
            let offline_service = OfflineService::new();
            if let Err(e) = offline_service.clear_admin_credentials() {
                log::error!("❌ Error limpiando credenciales admin: {}", e);
            }
            crate::rerender_app();
        })
    };
    
    let settings_popup = render_settings_popup(state, on_close_settings, on_logout)?;
    append_child(&container, &settings_popup)?;
    
    // Details modal (para traçabilité) - siempre renderizar, mostrar/ocultar con CSS
    if state.show_details.borrow().clone() {
        if let Some((pkg, addr)) = state.details_package.borrow().as_ref() {
            let on_close_details = {
                let state_clone = state.clone();
                Rc::new(move || {
                    state_clone.set_show_details(false);
                    crate::rerender_app();
                })
            };
            
            // Para admin, no necesitamos callbacks de edición
            let details_modal = render_details_modal(
                pkg,
                addr,
                state,
                on_close_details,
                None, // on_edit_address
                None, // on_edit_door_code
                None, // on_edit_mailbox
                None, // on_edit_driver_notes
                None, // on_mark_problematic
            )?;
            append_child(&container, &details_modal)?;
        }
    }
    
    // Setup polling solo si no hay sesión seleccionada
    if state.admin_selected_tournee_session.borrow().is_none() {
        setup_dashboard_polling(state);
    }
    
    Ok(container)
}

/// Crear header del admin (con botones como chofer)
fn create_admin_header(state: &AppState) -> Result<Element, JsValue> {
    let header = ElementBuilder::new("div")?
        .class("app-header")
        .build();
    
    // Título
    let title = ElementBuilder::new("h1")?
        .text("👔 Admin Dashboard")
        .build();
    append_child(&header, &title)?;
    
    // Actions container
    let actions = ElementBuilder::new("div")?
        .class("header-actions")
        .build();
    
    // Sync indicator (si existe)
    if let Some(sync_indicator) = render_sync_indicator(state)? {
        append_child(&actions, &sync_indicator)?;
    }
    
    let language = state.language.borrow().clone();
    
    // Search tracking button (🔍) - para buscar en todas las sesiones
    let search_btn = ElementBuilder::new("button")?
        .class("btn-icon-header btn-tracking-search")
        .attr("title", "Buscar tracking")?
        .text("🔍")
        .build();
    
    {
        let state_clone = state.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            state_clone.set_show_tracking_modal(true);
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        search_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&actions, &search_btn)?;
    
    // Refresh button (🔄) - para actualizar dashboard
    let refresh_btn = ElementBuilder::new("button")?
        .class("btn-icon-header btn-refresh")
        .attr("title", &t("rafraichir", &language))?
        .text("🔄")
        .build();
    
    {
        let state_clone = state.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            // Refrescar dashboard admin
            let state_for_refresh = state_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                use crate::services::api_client::ApiClient;
                let api = ApiClient::new();
                
                let username_opt = state_for_refresh.admin_username.borrow().clone();
                let password_opt = state_for_refresh.admin_password.borrow().clone();
                let societe_opt = state_for_refresh.admin_societe.borrow().clone();
                
                if username_opt.is_none() || password_opt.is_none() || societe_opt.is_none() {
                    console::log_1(&JsValue::from_str("⚠️ [ADMIN] Credenciales no disponibles para refresh"));
                    return;
                }
                
                let username = username_opt.unwrap();
                let password = password_opt.unwrap();
                let societe = societe_opt.unwrap();
                
                // Formatear fecha de hoy
                let today = js_sys::Date::new_0();
                let date_debut = format!(
                    "{:04}-{:02}-{:02}T00:00:00.000Z",
                    today.get_full_year(),
                    today.get_month() + 1,
                    today.get_date()
                );
                
                match api.fetch_admin_dashboard(&username, &password, &societe, &date_debut).await {
                    Ok(response) => {
                        *state_for_refresh.admin_districts.borrow_mut() = response.districts;
                        *state_for_refresh.admin_total_packages.borrow_mut() = response.total_packages;
                        crate::rerender_app();
                    }
                    Err(e) => {
                        console::error_1(&JsValue::from_str(&format!("❌ [ADMIN] Error refrescando: {}", e)));
                    }
                }
            });
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        
        refresh_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&actions, &refresh_btn)?;
    
    // Badge de notificaciones
    let notifications_count = state.admin_status_requests.borrow().len();
    if notifications_count > 0 {
        let notif_badge = ElementBuilder::new("button")?
            .class("btn-icon-header notif-badge")
            .attr("title", &format!("{} demandes en attente", notifications_count))?
            .text(&format!("🔔 {}", notifications_count))
            .build();
        
        {
            let state_clone = state.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                *state_clone.admin_view.borrow_mut() = "status_requests".to_string();
                crate::rerender_app();
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            
            notif_badge.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        append_child(&actions, &notif_badge)?;
    }
    
    // Botón de parámetros (⚙️) - reemplaza el logout, abre settings popup
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


/// Crear dashboard (sin barra de progreso, está en bottom sheet)
fn create_admin_dashboard(state: &AppState) -> Result<Element, JsValue> {
    let dashboard = ElementBuilder::new("div")?
        .class("admin-dashboard")
        .build();
    
    let title = ElementBuilder::new("h2")?
        .text("📊 Tableau de bord")
        .build();
    append_child(&dashboard, &title)?;
    
    Ok(dashboard)
}

/// Renderizar progress info para admin (agregado de todas las tournées)
/// Similar a render_progress_info del chofer pero usando datos agregados
fn render_admin_progress_info(state: &AppState) -> Result<(Element, Element), JsValue> {
    let total_packages = state.admin_total_packages.borrow().clone();
    
    // Calcular entregados y fallidos desde todas las tournées
    let total_delivered: usize = state.admin_districts.borrow()
        .iter()
        .flat_map(|d| d.tournees.iter())
        .map(|t| t.delivered_count)
        .sum();
    
    // Para fallidos, necesitamos contar desde las sesiones (no tenemos ese dato en AdminTournee)
    // Por ahora, calculamos como: total - entregados (aproximado)
    // TODO: Agregar failed_count a AdminTournee si es necesario
    let total_failed = 0; // Por ahora 0, se puede mejorar después
    
    // Porcentajes para la barra de progreso
    let delivered_percent = if total_packages > 0 {
        (total_delivered * 100) / total_packages
    } else {
        0
    };
    
    let failed_percent = if total_packages > 0 {
        (total_failed * 100) / total_packages
    } else {
        0
    };
    
    // ===== PROGRESS INFO (primer hermano) =====
    let progress_info = ElementBuilder::new("div")?
        .attr("id", "admin-progress-info")?
        .class("progress-info")
        .build();
    
    // Texto de progreso (tournées activas)
    let progress_text = ElementBuilder::new("div")?
        .class("progress-text")
        .build();
    
    let total_tournees: usize = state.admin_districts.borrow()
        .iter()
        .map(|d| d.tournees.len())
        .sum();
    
    let language = state.language.borrow().clone();
    let tournees_text = if language == "ES" { "tournées" } else { "tournées" };
    let progress_count = ElementBuilder::new("span")?
        .class("progress-count")
        .text(&format!("✓ {} {}", total_tournees, tournees_text))
        .build();
    
    append_child(&progress_text, &progress_count)?;
    
    // Contador de paquetes
    let progress_packages = ElementBuilder::new("div")?
        .class("progress-packages")
        .build();
    
    let packages_count = ElementBuilder::new("span")?
        .class("packages-count")
        .text(&format!("{}/{} {}", total_delivered, total_packages, t("paquets", &language)))
        .build();
    
    append_child(&progress_packages, &packages_count)?;
    
    // Agregar texto y paquetes a progress-info
    append_child(&progress_info, &progress_text)?;
    append_child(&progress_info, &progress_packages)?;
    
    // ===== PROGRESS BAR CONTAINER (segundo hermano) =====
    let progress_bar_container = ElementBuilder::new("div")?
        .attr("id", "admin-progress-bar-container")?
        .class("progress-bar-container")
        .build();
    
    // Barra verde (entregados)
    let progress_bar_delivered = ElementBuilder::new("div")?
        .class("progress-bar progress-bar-delivered")
        .build();
    
    set_attribute(&progress_bar_delivered, "style", &format!("width: {}%", delivered_percent))?;
    
    // Barra roja (fallidos) - se superpone después de la verde
    let progress_bar_failed = ElementBuilder::new("div")?
        .class("progress-bar progress-bar-failed")
        .build();
    
    set_attribute(&progress_bar_failed, "style", &format!("width: {}%; left: {}%", failed_percent, delivered_percent))?;
    
    append_child(&progress_bar_container, &progress_bar_delivered)?;
    append_child(&progress_bar_container, &progress_bar_failed)?;
    
    Ok((progress_info, progress_bar_container))
}

/// Crear stat card
fn create_stat_card(icon: &str, value: &str, label: &str) -> Result<Element, JsValue> {
    let card = ElementBuilder::new("div")?
        .class("stat-card")
        .build();
    
    let value_el = ElementBuilder::new("div")?
        .class("stat-value")
        .text(&format!("{} {}", icon, value))
        .build();
    
    let label_el = ElementBuilder::new("div")?
        .class("stat-label")
        .text(label)
        .build();
    
    append_child(&card, &value_el)?;
    append_child(&card, &label_el)?;
    
    Ok(card)
}

/// Crear bottom sheet de districts con barra de progreso siempre visible (específico de admin)
fn create_districts_bottom_sheet(state: &AppState) -> Result<Element, JsValue> {
    let sheet_state = state.admin_sheet_state.borrow().clone();
    
    // La barra de progreso SIEMPRE debe estar visible (incluso cuando collapsed)
    let (progress_info, progress_bar_container) = render_admin_progress_info(state)?;
    
    // Container principal
    let container = ElementBuilder::new("div")?
        .attr("id", "package-container")?
        .class("package-container")
        .build();
    
    // Backdrop
    let backdrop = ElementBuilder::new("div")?
        .attr("id", "backdrop")?
        .class("backdrop")
        .build();
    
    if sheet_state != "collapsed" {
        add_class(&backdrop, "active")?;
    }
    
    {
        let state_clone = state.clone();
        on_click(&backdrop, move |_| {
            state_clone.set_admin_sheet_state("collapsed".to_string());
        })?;
    }
    
    append_child(&container, &backdrop)?;
    
    // Bottom Sheet
    let bottom_sheet = ElementBuilder::new("div")?
        .attr("id", "bottom-sheet")?
        .class("bottom-sheet")
        .build();
    
    // Agregar clase de estado
    add_class(&bottom_sheet, &sheet_state)?;
    
    // Drag Handle Container (header con progress)
    let drag_handle_container = ElementBuilder::new("div")?
        .attr("id", "drag-handle-container")?
        .class("drag-handle-container")
        .build();
    
    // Drag Handle
    let drag_handle = ElementBuilder::new("div")?
        .class("drag-handle")
        .build();
    
    {
        let state_clone = state.clone();
        on_click(&drag_handle_container, move |_| {
            let current = state_clone.admin_sheet_state.borrow().clone();
            let new_state = match current.as_str() {
                "collapsed" => "half",
                "half" => "full",
                "full" => "collapsed",
                _ => "half",
            };
            state_clone.set_admin_sheet_state(new_state.to_string());
        })?;
    }
    
    append_child(&drag_handle_container, &drag_handle)?;
    
    // AGREGAR SIEMPRE los header elements (incluso cuando collapsed) - específico de admin
    append_child(&drag_handle_container, &progress_info)?;
    append_child(&drag_handle_container, &progress_bar_container)?;
    
    append_child(&bottom_sheet, &drag_handle_container)?;
    
    // Crear body content (lista de districts)
    let body_content = ElementBuilder::new("div")?
        .build();
    
    for (idx, district) in state.admin_districts.borrow().iter().enumerate() {
        let district_card = create_district_card(state, district, idx)?;
        append_child(&body_content, &district_card)?;
    }
    
    // Agregar body content con clase package-list para scroll consistente
    add_class(&body_content, "package-list")?;
    append_child(&bottom_sheet, &body_content)?;
    
    append_child(&container, &bottom_sheet)?;
    
    Ok(container)
}

/// Crear card de distrito (estilo package-card)
fn create_district_card(state: &AppState, district: &AdminDistrict, index: usize) -> Result<Element, JsValue> {
    // Capturar is_expanded antes de crear el card para evitar borrow conflicts
    let code_postal = district.code_postal.clone();
    let is_expanded = state.admin_expanded_districts.borrow().contains(&code_postal);
    
    // Usar clases de package-card para mantener consistencia
    let mut classes = vec!["package-card", "district-card"];
    if is_expanded {
        classes.push("selected");
    }
    
    let card = ElementBuilder::new("div")?
        .class(&classes.join(" "))
        .build();
    
    // Header estilo package-header
    let header = ElementBuilder::new("div")?
        .class("package-header")
        .build();
    
    // Botón info (placeholder por ahora)
    let info_btn = ElementBuilder::new("button")?
        .class("btn-info")
        .text("i")
        .build();
    
    {
        let code_postal = district.code_postal.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            e.prevent_default();
            console::log_1(&JsValue::from_str(&format!("Info district: {}", code_postal)));
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        
        info_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&header, &info_btn)?;
    append_child(&card, &header)?;
    
    // Main content estilo package-main
    let main = ElementBuilder::new("div")?
        .class("package-main")
        .build();
    
    let info = ElementBuilder::new("div")?
        .class("package-info")
        .build();
    
    // Título del district (código postal - nombre ciudad)
    let recipient = ElementBuilder::new("div")?
        .class("package-recipient")
        .text(&format!("{} - {}", 
            district.code_postal, 
            district.nom_ville.as_deref().unwrap_or("")))
        .build();
    append_child(&info, &recipient)?;
    
    // Info resumida (tournées y colis)
    let address = ElementBuilder::new("div")?
        .class("package-address")
        .text(&format!("🚚 {} tournées • {} colis", 
            district.tournees.len(),
            district.tournees.iter().map(|t| t.nb_colis).sum::<usize>()))
        .build();
    append_child(&info, &address)?;
    
    append_child(&main, &info)?;
    append_child(&card, &main)?;
    
    // EXPAND HANDLE - solo cuando NO está expandido (para indicar que se puede expandir)
    if !is_expanded {
        let mut expand_handle_classes = vec!["expand-handle", "pulse"];
        let expand_handle = ElementBuilder::new("div")?
            .class(&expand_handle_classes.join(" "))
            .build();
        
        let expand_indicator = ElementBuilder::new("div")?
            .class("expand-indicator")
            .build();
        append_child(&expand_handle, &expand_indicator)?;
        
        {
            let state_clone = state.clone();
            let code_postal_clone = code_postal.clone();
            let expand_handle_clone = expand_handle.clone();
            let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                e.stop_propagation();
                e.prevent_default();
                
                // Liberar el borrow inmediatamente después de modificar
                {
                    let mut expanded = state_clone.admin_expanded_districts.borrow_mut();
                    expanded.insert(code_postal_clone.clone());
                } // Borrow liberado aquí
                
                // Ahora es seguro llamar a rerender_app()
                crate::rerender_app();
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            
            expand_handle_clone.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        append_child(&card, &expand_handle)?;
    }
    
    // Lista de tournées expandidas (estilo packages-expanded)
    if is_expanded {
        let expanded_container = ElementBuilder::new("div")?
            .class("packages-expanded")
            .build();
        
        // Agrupar tournées por letra
        let mut tournees_by_letter: std::collections::HashMap<String, Vec<&AdminTournee>> = std::collections::HashMap::new();
        for tournee in &district.tournees {
            tournees_by_letter.entry(tournee.letter.clone()).or_default().push(tournee);
        }
        
        for (letter, tournees) in tournees_by_letter {
            // Título de sección por letra
            let letter_title = ElementBuilder::new("div")?
                .class("tournee-letter-title")
                .text(&format!("🅰️ Tournée {}", letter))
                .build();
            append_child(&expanded_container, &letter_title)?;
            
            // Cards de tournées
            for (tournee_idx, tournee) in tournees.iter().enumerate() {
                let tournee_card = create_tournee_card(state, district, tournee, tournee_idx)?;
                append_child(&expanded_container, &tournee_card)?;
            }
        }
        
        append_child(&card, &expanded_container)?;
        
        // EXPAND HANDLE para cerrar - igual que el de abrir pero sin pulse
        let expand_handle = ElementBuilder::new("div")?
            .class("expand-handle")
            .build();
        
        let expand_indicator = ElementBuilder::new("div")?
            .class("expand-indicator")
            .build();
        append_child(&expand_handle, &expand_indicator)?;
        
        {
            let state_clone = state.clone();
            let code_postal_clone = code_postal.clone();
            let expand_handle_clone = expand_handle.clone();
            let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                e.stop_propagation();
                e.prevent_default();
                
                // Liberar el borrow inmediatamente después de modificar
                {
                    let mut expanded = state_clone.admin_expanded_districts.borrow_mut();
                    expanded.remove(&code_postal_clone);
                } // Borrow liberado aquí
                
                // Ahora es seguro llamar a rerender_app()
                crate::rerender_app();
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            
            expand_handle_clone.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        append_child(&card, &expand_handle)?;
    }
    
    Ok(card)
}

/// Crear card de tournée (estilo package-card, click selecciona)
fn create_tournee_card(state: &AppState, district: &AdminDistrict, tournee: &AdminTournee, index: usize) -> Result<Element, JsValue> {
    // Usar clases de package-card para mantener consistencia
    let mut classes = vec!["package-card", "tournee-card", "type-domicile"];
    
    let card = ElementBuilder::new("div")?
        .class(&classes.join(" "))
        .build();
    
    // Click en card completo para seleccionar tournée
    {
        let state_clone = state.clone();
        let code_tournee = tournee.code_tournee.clone();
        let card_clone = card.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            // Fetch y seleccionar tournée
            let state_for_fetch = state_clone.clone();
            let code_tournee_clone = code_tournee.clone();
            wasm_bindgen_futures::spawn_local(async move {
                use crate::services::api_client::ApiClient;
                let api = ApiClient::new();
                
                // Obtener credenciales del admin
                let username_opt = state_for_fetch.admin_username.borrow().clone();
                let password_opt = state_for_fetch.admin_password.borrow().clone();
                let societe_opt = state_for_fetch.admin_societe.borrow().clone();
                
                if username_opt.is_none() || password_opt.is_none() || societe_opt.is_none() {
                    console::log_1(&JsValue::from_str("⚠️ [TOURNEE] Credenciales no disponibles"));
                    return;
                }
                
                let username = username_opt.unwrap();
                let societe = societe_opt.unwrap();
                let sso_token = "".to_string(); // El backend obtendrá el token de la sesión existente
                
                // Formatear fecha de hoy
                let today = js_sys::Date::new_0();
                let date = format!(
                    "{:04}-{:02}-{:02}",
                    today.get_full_year(),
                    today.get_month() + 1,
                    today.get_date()
                );
                
                match api.fetch_tournee_packages(&code_tournee_clone, &sso_token, &username, &societe, &date).await {
                    Ok(session) => {
                        console::log_1(&JsValue::from_str(&format!("✅ [TOURNEE] {} paquetes obtenidos", session.packages.len())));
                        
                        // Guardar sesión y seleccionar tournée
                        *state_for_fetch.admin_selected_tournee_session.borrow_mut() = Some(session);
                        *state_for_fetch.admin_selected_tournee.borrow_mut() = Some(code_tournee_clone);
                        
                        crate::rerender_app();
                    }
                    Err(e) => {
                        console::error_1(&JsValue::from_str(&format!("❌ [TOURNEE] Error: {}", e)));
                    }
                }
            });
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        
        card_clone.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    // Header estilo package-header
    let header = ElementBuilder::new("div")?
        .class("package-header")
        .build();
    
    // Botón info
    let info_btn = ElementBuilder::new("button")?
        .class("btn-info")
        .text("i")
        .build();
    
    {
        let matricule = tournee.matricule.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            e.prevent_default();
            console::log_1(&JsValue::from_str(&format!("Info tournée: {}", matricule)));
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        
        info_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&header, &info_btn)?;
    append_child(&card, &header)?;
    
    // Main content estilo package-main
    let main = ElementBuilder::new("div")?
        .class("package-main")
        .build();
    
    let info = ElementBuilder::new("div")?
        .class("package-info")
        .build();
    
    // Matricule del chofer
    let recipient = ElementBuilder::new("div")?
        .class("package-recipient")
        .text(&format!("👨‍✈️ {}", tournee.matricule))
        .build();
    append_child(&info, &recipient)?;
    
    // Info de colis y livrés
    let address = ElementBuilder::new("div")?
        .class("package-address")
        .text(&format!("📦 {} colis • {} livrés • {}", 
            tournee.nb_colis,
            tournee.delivered_count,
            tournee.statut))
        .build();
    append_child(&info, &address)?;
    
    append_child(&main, &info)?;
    append_child(&card, &main)?;
    
    Ok(card)
}

/// Crear vista de grid de paquetes (2 columnas, cards cuadrados)
fn create_packages_grid_view(state: &AppState) -> Result<Element, JsValue> {
    let container = ElementBuilder::new("div")?
        .class("packages-grid-view")
        .build();
    
    // Header del grid
    let header = ElementBuilder::new("div")?
        .class("grid-header")
        .build();
    
    // Título con info de la tournée
    let title_text = if let Some(ref session) = *state.admin_selected_tournee_session.borrow() {
        format!("Tournée {} - {} paquets", 
            session.driver.driver_id,
            session.stats.total_packages)
    } else {
        "Paquets de la tournée".to_string()
    };
    
    let title = ElementBuilder::new("h2")?
        .text(&title_text)
        .build();
    
    append_child(&header, &title)?;
    append_child(&container, &header)?;
    
    // Contenedor responsive wrapper para el grid
    let grid_wrapper = ElementBuilder::new("div")?
        .class("packages-grid-wrapper")
        .build();
    
    // Grid de paquetes (2 columnas)
    let grid = ElementBuilder::new("div")?
        .class("packages-grid")
        .build();
    
    // Obtener sesión y renderizar paquetes
    if let Some(ref session) = *state.admin_selected_tournee_session.borrow() {
        // Ordenar paquetes por route_order o original_order
        let mut packages: Vec<_> = session.packages.values().collect();
        if session.is_optimized {
            packages.sort_by_key(|p| p.route_order.unwrap_or(usize::MAX));
        } else {
            packages.sort_by_key(|p| p.original_order);
        }
        
        // Renderizar cada paquete como card cuadrado
        for (idx, pkg) in packages.iter().enumerate() {
            let package_card = create_package_square_card(pkg, idx + 1, state, session)?;
            append_child(&grid, &package_card)?;
        }
    } else {
        // Placeholder si no hay sesión
        let placeholder = ElementBuilder::new("div")?
            .class("packages-placeholder")
            .text("Chargement des paquets...")
            .build();
        append_child(&grid, &placeholder)?;
    }
    
    append_child(&grid_wrapper, &grid)?;
    append_child(&container, &grid_wrapper)?;
    
    Ok(container)
}

/// Crear card cuadrado de paquete para grid
fn create_package_square_card(
    pkg: &crate::models::package::Package, 
    number: usize,
    state: &AppState,
    session: &crate::models::session::DeliverySession,
) -> Result<Element, JsValue> {
    // Determinar clase según status (mismo sistema que chofer)
    let status_class = match pkg.status.as_str() {
        s if s.contains("RECEPTIONNER") => "package-square-yellow",
        s if s.contains("LIVRER") || s.contains("LIVRE") => "package-square-green",
        s if s.contains("NONLIV") || s.contains("ECHEC") => "package-square-red",
        _ => "package-square-normal",
    };
    
    // Card principal
    let card = ElementBuilder::new("div")?
        .class(&format!("package-square-card {}", status_class))
        .build();
    
    // HEADER: Número + Botón detalles (sin tracking)
    let header = ElementBuilder::new("div")?
        .class("package-square-header")
        .build();
    
    // Número coloreado (círculo pequeño)
    let number_class = match pkg.status.as_str() {
        s if s.contains("RECEPTIONNER") => "package-square-number package-square-number-yellow",
        s if s.contains("LIVRER") || s.contains("LIVRE") => "package-square-number package-square-number-green",
        s if s.contains("NONLIV") || s.contains("ECHEC") => "package-square-number package-square-number-red",
        _ => "package-square-number package-square-number-normal",
    };
    
    let number_el = ElementBuilder::new("div")?
        .class(number_class)
        .text(&number.to_string())
        .build();
    
    // Botón detalles "i"
    let details_btn = ElementBuilder::new("button")?
        .class("btn-info")
        .text("i")
        .build();
    
    // Event listener para abrir modal de detalles con traçabilité
    {
        let tracking = pkg.tracking.clone();
        let state_clone = state.clone();
        let session_clone = session.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
            e.prevent_default();
            
            // Abrir modal de detalles y obtener traçabilité
            if let Some(pkg) = session_clone.packages.get(&tracking) {
                if let Some(addr) = session_clone.addresses.get(&pkg.address_id) {
                    {
                        let mut details = state_clone.details_package.borrow_mut();
                        *details = Some((pkg.clone(), addr.clone()));
                    }
                    state_clone.set_show_details(true);
                    
                    // Obtener traçabilité del paquete
                    let tracking_clone = tracking.clone();
                    let state_for_async = state_clone.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        // Obtener credenciales admin
                        if let (Some(username), Some(password), Some(societe)) = (
                            state_for_async.admin_username.borrow().as_ref(),
                            state_for_async.admin_password.borrow().as_ref(),
                            state_for_async.admin_societe.borrow().as_ref(),
                        ) {
                            // Primero autenticar para obtener el token
                            use crate::services::api_client::ApiClient;
                            let api = ApiClient::new();
                            
                            // Formatear fecha de hoy
                            let today = js_sys::Date::new_0();
                            let date_debut = format!(
                                "{:04}-{:02}-{:02}T00:00:00.000Z",
                                today.get_full_year(),
                                today.get_month() + 1,
                                today.get_date()
                            );
                            
                            // Obtener dashboard para obtener el token
                            match api.fetch_admin_dashboard(username, password, societe, &date_debut).await {
                                Ok(dashboard_response) => {
                                    // Ahora obtener traçabilité con el token
                                    match api.fetch_package_traceability(
                                        &tracking_clone,
                                        &dashboard_response.sso_token,
                                        username,
                                        societe,
                                    ).await {
                                        Ok(traceability) => {
                                            *state_for_async.package_traceability.borrow_mut() = Some(traceability);
                                            crate::rerender_app();
                                        }
                                        Err(e) => {
                                            log::error!("❌ Error obteniendo traçabilité: {}", e);
                                            console::error_1(&JsValue::from_str(&format!("Error traçabilité: {}", e)));
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("❌ Error autenticando para traçabilité: {}", e);
                                    console::error_1(&JsValue::from_str(&format!("Error auth: {}", e)));
                                }
                            }
                        } else {
                            log::warn!("⚠️ Credenciales admin no disponibles para traçabilité");
                        }
                    });
                }
            }
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        
        details_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    append_child(&header, &number_el)?;
    append_child(&header, &details_btn)?;
    append_child(&card, &header)?;
    
    // MAIN CONTENT: Tracking + Status (similar a cards de chofer)
    let main = ElementBuilder::new("div")?
        .class("package-main")
        .build();
    
    let info = ElementBuilder::new("div")?
        .class("package-info")
        .build();
    
    // Tracking (equivalente a nombre cliente en chofer)
    let tracking_el = ElementBuilder::new("div")?
        .class("package-recipient")
        .text(&pkg.tracking)
        .build();
    append_child(&info, &tracking_el)?;
    
    // Status (equivalente a dirección en chofer) - por ahora en texto crudo
    let status_el = ElementBuilder::new("div")?
        .class("package-address")
        .text(&pkg.status)
        .build();
    append_child(&info, &status_el)?;
    
    append_child(&main, &info)?;
    append_child(&card, &main)?;
    
    Ok(card)
}

/// Crear vista de status requests
fn create_status_requests_view(state: &AppState) -> Result<Element, JsValue> {
    let container = ElementBuilder::new("div")?
        .class("status-requests-view")
        .build();
    
    // Header
    let header = ElementBuilder::new("div")?
        .class("requests-header")
        .build();
    
    let title = ElementBuilder::new("h2")?
        .text("📋 Demandes de changement de statut")
        .build();
    
    // Auto-refresh toggle
    let auto_refresh = *state.admin_auto_refresh.borrow();
    let refresh_btn = ElementBuilder::new("button")?
        .class("btn-auto-refresh")
        .text(if auto_refresh { "🔄 Auto-refresh: ON" } else { "⏸️ Auto-refresh: OFF" })
        .build();
    
    {
        let state_clone = state.clone();
        on_click(&refresh_btn, move |_| {
            let current = *state_clone.admin_auto_refresh.borrow();
            *state_clone.admin_auto_refresh.borrow_mut() = !current;
            crate::rerender_app();
        })?;
    }
    
    append_child(&header, &title)?;
    append_child(&header, &refresh_btn)?;
    append_child(&container, &header)?;
    
    // Lista de requests
    let requests_list = ElementBuilder::new("div")?
        .class("requests-list")
        .build();
    
    for request in state.admin_status_requests.borrow().iter() {
        let request_card = create_status_request_card(state, request)?;
        append_child(&requests_list, &request_card)?;
    }
    
    append_child(&container, &requests_list)?;
    
    // Setup auto-refresh si está activo
    if auto_refresh {
        setup_auto_refresh(state);
    }
    
    Ok(container)
}

/// Crear card de status request
fn create_status_request_card(state: &AppState, request: &StatusChangeRequest) -> Result<Element, JsValue> {
    let card = ElementBuilder::new("div")?
        .class("status-request-card")
        .build();
    
    // Header
    let header = ElementBuilder::new("div")?
        .class("request-header")
        .build();
    
    let tracking = ElementBuilder::new("div")?
        .class("request-tracking")
        .text(&format!("📦 {}", request.tracking_code))
        .build();
    
    let badge = ElementBuilder::new("span")?
        .class("request-badge urgent")
        .text("🔴 URGENT")
        .build();
    
    append_child(&header, &tracking)?;
    append_child(&header, &badge)?;
    append_child(&card, &header)?;
    
    // Info
    let info = ElementBuilder::new("div")?
        .class("request-info")
        .build();
    
    let customer = ElementBuilder::new("div")?
        .text(&format!("👤 {}", request.customer_name))
        .build();
    
    let address = ElementBuilder::new("div")?
        .text(&format!("📍 {}", request.customer_address))
        .build();
    
    let driver = ElementBuilder::new("div")?
        .text(&format!("👨‍✈️ Signalé par: {}", request.driver_matricule))
        .build();
    
    append_child(&info, &customer)?;
    append_child(&info, &address)?;
    append_child(&info, &driver)?;
    
    if let Some(notes) = &request.notes {
        let notes_el = ElementBuilder::new("div")?
            .class("request-notes")
            .text(&format!("📝 Notes: {}", notes))
            .build();
        append_child(&info, &notes_el)?;
    }
    
    append_child(&card, &info)?;
    
    // Actions
    let actions = ElementBuilder::new("div")?
        .class("request-actions")
        .build();
    
    let confirm_btn = ElementBuilder::new("button")?
        .class("btn-confirm-send")
        .text("✅ Confirmer et envoyer email")
        .build();
    
    {
        let state_clone = state.clone();
        let request_id = request.id.clone().unwrap_or_default();
        on_click(&confirm_btn, move |_| {
            // TODO: Call API to confirm and send email
            console::log_1(&JsValue::from_str(&format!("Confirming request: {}", request_id)));
            // Refresh requests after
        })?;
    }
    
    append_child(&actions, &confirm_btn)?;
    append_child(&card, &actions)?;
    
    Ok(card)
}

/// Setup auto-refresh timer
fn setup_auto_refresh(state: &AppState) {
    use gloo_timers::callback::Interval;
    
    let state_clone = state.clone();
    
    // Limpiar interval anterior si existe (usando una key única)
    // Por simplicidad, creamos un nuevo interval cada vez que se llama
    // En producción, podrías querer guardar el handle del interval para poder cancelarlo
    
    Interval::new(30000, move || { // 30 segundos = 30000ms
        // Solo hacer refresh si auto_refresh está activo y estamos en vista status_requests
        let should_refresh = *state_clone.admin_auto_refresh.borrow() 
            && *state_clone.admin_view.borrow() == "status_requests";
        
        if should_refresh {
            console::log_1(&JsValue::from_str("🔄 [AUTO-REFRESH] Refrescando status requests..."));
            
            // Fetch status requests
            let state_for_fetch = state_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                use crate::services::api_client::ApiClient;
                let api = ApiClient::new();
                
                match api.fetch_status_requests().await {
                    Ok(requests) => {
                        console::log_1(&JsValue::from_str(&format!("✅ [AUTO-REFRESH] {} requests obtenidos", requests.len())));
                        *state_for_fetch.admin_status_requests.borrow_mut() = requests;
                        // Re-renderizar solo la vista de requests (no toda la app)
                        crate::rerender_app();
                    }
                    Err(e) => {
                        console::error_1(&JsValue::from_str(&format!("❌ [AUTO-REFRESH] Error: {}", e)));
                    }
                }
            });
        }
    }).forget(); // .forget() para que el interval no se cancele
}

/// Setup polling automático para dashboard (cada 60 segundos)
fn setup_dashboard_polling(state: &AppState) {
    use gloo_timers::callback::Interval;
    
    let state_clone = state.clone();
    
    Interval::new(60000, move || { // 60 segundos = 60000ms
        // Solo hacer refresh si estamos en modo admin y en vista districts
        let should_refresh = *state_clone.admin_mode.borrow() 
            && *state_clone.admin_view.borrow() == "districts";
        
        if should_refresh {
            // Verificar que tengamos credenciales guardadas
            let username_opt = state_clone.admin_username.borrow().clone();
            let password_opt = state_clone.admin_password.borrow().clone();
            let societe_opt = state_clone.admin_societe.borrow().clone();
            
            if username_opt.is_none() || password_opt.is_none() || societe_opt.is_none() {
                console::log_1(&JsValue::from_str("⚠️ [DASHBOARD-POLLING] Credenciales no disponibles"));
                return;
            }
            
            let username = username_opt.unwrap();
            let password = password_opt.unwrap();
            let societe = societe_opt.unwrap();
            
            console::log_1(&JsValue::from_str("🔄 [DASHBOARD-POLLING] Actualizando dashboard..."));
            
            // Fetch dashboard actualizado
            let state_for_fetch = state_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                use crate::services::api_client::ApiClient;
                let api = ApiClient::new();
                
                // Formatear fecha de hoy para date_debut
                let today = js_sys::Date::new_0();
                let date_debut = format!(
                    "{:04}-{:02}-{:02}T00:00:00.000Z",
                    today.get_full_year(),
                    today.get_month() + 1,
                    today.get_date()
                );
                
                match api.fetch_admin_dashboard(&username, &password, &societe, &date_debut).await {
                    Ok(response) => {
                        console::log_1(&JsValue::from_str(&format!("✅ [DASHBOARD-POLLING] {} districts, {} paquetes obtenidos", 
                            response.districts.len(), response.total_packages)));
                        
                        // Preservar districts expandidos antes de actualizar
                        let expanded_districts = state_for_fetch.admin_expanded_districts.borrow().clone();
                        
                        // Actualizar estado
                        *state_for_fetch.admin_districts.borrow_mut() = response.districts;
                        *state_for_fetch.admin_total_packages.borrow_mut() = response.total_packages;
                        
                        // Restaurar districts expandidos
                        *state_for_fetch.admin_expanded_districts.borrow_mut() = expanded_districts;
                        
                        // Re-renderizar la app para mostrar cambios
                        crate::rerender_app();
                    }
                    Err(e) => {
                        console::error_1(&JsValue::from_str(&format!("❌ [DASHBOARD-POLLING] Error: {}", e)));
                    }
                }
            });
        }
    }).forget(); // .forget() para que el interval no se cancele
}
