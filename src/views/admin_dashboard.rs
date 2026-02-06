// ============================================================================
// ADMIN DASHBOARD - Dashboard administrativo completo
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::{Element, console};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use crate::dom::{ElementBuilder, append_child, add_class, remove_class, set_attribute, create_element, set_inner_html};
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
    
    let admin_view = state.admin_view.borrow().clone();
    if admin_view == "status_requests" {
        // Vista de colis en attente de confirmation
        let requests_view = create_status_requests_view(state)?;
        append_child(&container, &requests_view)?;
    } else if state.admin_selected_tournee_session.borrow().is_some() {
        // Si hay sesión seleccionada, mostrar grid de paquetes
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
            *state_clone.admin_sso_token.borrow_mut() = None;
            *state_clone.admin_selected_status_request.borrow_mut() = None;
            *state_clone.admin_traceability_loading.borrow_mut() = false;
            *state_clone.admin_status_requests.borrow_mut() = Vec::new();
            let _ = state_clone.admin_dashboard_polling_interval.borrow_mut().take();
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
    
    // Modal de détail d'une demande (historique + confirmer)
    if let Some(modal) = render_status_request_detail_modal(state)? {
        append_child(&container, &modal)?;
    }
    
    // Modal de búsqueda de tracking (buscar en todas las tournées)
    let admin_tracking_modal = create_admin_tracking_modal(state)?;
    append_child(&container, &admin_tracking_modal)?;
    
    // Polling siempre activo en modo admin para detectar nuevas sesiones/tournées
    // (antes se paraba al seleccionar una tournée, pero eso impedía ver nuevas tournées al reconectar un chofer)
    setup_dashboard_polling(state);
    
    Ok(container)
}

/// Modal de búsqueda de tracking (buscar en todas las tournées)
fn create_admin_tracking_modal(state: &AppState) -> Result<Element, JsValue> {
    let language = state.language.borrow().clone();
    use crate::models::admin::SearchTrackingRequest;

    let modal = ElementBuilder::new("div")?
        .id("tracking-modal")?
        .class("company-modal")
        .build();

    {
        let state_clone = state.clone();
        on_click(&modal, move |_e: web_sys::MouseEvent| {
            state_clone.set_show_tracking_modal(false);
        })?;
    }

    let modal_content = ElementBuilder::new("div")?
        .class("company-modal-content")
        .build();

    {
        let modal_content_clone = modal_content.clone();
        on_click(&modal_content, move |e: web_sys::MouseEvent| {
            e.stop_propagation();
        })?;
    }

    let modal_header = ElementBuilder::new("div")?
        .class("company-modal-header")
        .build();
    let header_title = ElementBuilder::new("h3")?
        .text(&t("rechercher_tracking", &language))
        .build();
    let close_btn = ElementBuilder::new("button")?
        .attr("type", "button")?
        .attr("class", "btn-close")?
        .text("✕")
        .build();
    {
        let state_clone = state.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            state_clone.set_show_tracking_modal(false);
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        close_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    append_child(&modal_header, &header_title)?;
    append_child(&modal_header, &close_btn)?;

    let search_container = ElementBuilder::new("div")?
        .class("company-search")
        .build();
    let search_input = crate::dom::create_element("input")?;
    set_attribute(&search_input, "type", "text")?;
    set_attribute(&search_input, "id", "admin-tracking-search")?;
    set_attribute(&search_input, "placeholder", &t("saisir_tracking_rechercher", &language))?;

    let search_btn = ElementBuilder::new("button")?
        .attr("type", "button")?
        .class("btn-preview-toggle")
        .text(&format!("🔍 {}", t("rechercher", &language)))
        .build();

    let results_container = ElementBuilder::new("div")?
        .id("admin-tracking-results")?
        .class("company-list")
        .build();
    let placeholder = ElementBuilder::new("div")?
        .class("company-empty")
        .text(&t("saisir_tracking_rechercher", &language))
        .build();
    append_child(&results_container, &placeholder)?;

    {
        let state_clone = state.clone();
        let results_clone = results_container.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            let input_el = crate::dom::get_element_by_id("admin-tracking-search");
            let query = input_el
                .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();
            if query.is_empty() {
                return;
            }
            let societe = state_clone.admin_societe.borrow().clone().unwrap_or_default();
            let today = js_sys::Date::new_0();
            let date = format!(
                "{:04}-{:02}-{:02}",
                today.get_full_year(),
                today.get_month() + 1,
                today.get_date()
            );
            let state_for_search = state_clone.clone();
            let results_for_async = results_clone.clone();
            spawn_local(async move {
                let api = ApiClient::new();
                let req = SearchTrackingRequest {
                    tracking: query.clone(),
                    societe,
                    date,
                };
                match api.search_tracking(&req).await {
                    Ok(res) => {
                        set_inner_html(&results_for_async, "");
                        if res.found {
                            if let (Some(code), Some(session)) = (res.code_tournee, res.session) {
                                let item = ElementBuilder::new("div").unwrap().class("company-item").build();
                                let text = ElementBuilder::new("div").unwrap()
                                    .class("company-name")
                                    .text(&format!("{} – Tournée {}", query, code))
                                    .build();
                                append_child(&item, &text).ok();
                                let state_open = state_for_search.clone();
                                let code_clone = code.clone();
                                let session_clone = session.clone();
                                let closure2 = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                                    *state_open.admin_selected_tournee_session.borrow_mut() = Some(session_clone.clone());
                                    *state_open.admin_selected_tournee.borrow_mut() = Some(code_clone.clone());
                                    *state_open.admin_view.borrow_mut() = "packages".to_string();
                                    state_open.set_show_tracking_modal(false);
                                    crate::rerender_app();
                                }) as Box<dyn FnMut(web_sys::MouseEvent)>);
                                item.add_event_listener_with_callback("click", closure2.as_ref().unchecked_ref()).ok();
                                closure2.forget();
                                append_child(&results_for_async, &item).ok();
                            }
                        } else {
                            let lang_empty = state_for_search.language.borrow().clone();
                            let empty = ElementBuilder::new("div").unwrap()
                                .class("company-empty")
                                .text(&t("aucun_colis_trouve", &lang_empty))
                                .build();
                            append_child(&results_for_async, &empty).ok();
                        }
                    }
                    Err(e) => {
                        set_inner_html(&results_for_async, "");
                        let lang_err = state_for_search.language.borrow().clone();
                        let err_el = ElementBuilder::new("div").unwrap()
                            .class("company-empty")
                            .text(&format!("{}: {}", t("erreur", &lang_err), e))
                            .build();
                        append_child(&results_for_async, &err_el).ok();
                    }
                }
            });
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        search_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    append_child(&search_container, &search_input)?;
    append_child(&search_container, &search_btn)?;
    append_child(&modal_content, &modal_header)?;
    append_child(&modal_content, &search_container)?;
    append_child(&modal_content, &results_container)?;
    append_child(&modal, &modal_content)?;

    Ok(modal)
}

/// Crear header del admin (con botones como chofer)
fn create_admin_header(state: &AppState) -> Result<Element, JsValue> {
    let header = ElementBuilder::new("div")?
        .class("app-header")
        .build();
    
    // Título (botón para volver al dashboard principal: demandes + Excel)
    let lang_header = state.language.borrow().clone();
    let title = ElementBuilder::new("button")?
        .class("btn-title-header")
        .text(&format!("👔 {}", t("admin_dashboard", &lang_header)))
        .build();
    {
        let state_clone = state.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            *state_clone.admin_selected_tournee_session.borrow_mut() = None;
            *state_clone.admin_selected_tournee.borrow_mut() = None;
            *state_clone.admin_view.borrow_mut() = "status_requests".to_string();
            crate::rerender_app();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        title.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
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
        .attr("title", &t("buscar_tracking", &language))?
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
                        *state_for_refresh.admin_districts.borrow_mut() = response.districts.clone();
                        *state_for_refresh.admin_total_packages.borrow_mut() = response.total_packages;
                        *state_for_refresh.admin_sso_token.borrow_mut() = Some(response.sso_token.clone());
                        let state_req = state_for_refresh.clone();
                        if let Ok(requests) = api.fetch_status_requests("refresh_button").await {
                            *state_req.admin_status_requests.borrow_mut() = requests;
                        }
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
    
    // Badge de notificaciones (siempre visible; muestra número cuando hay pendientes)
    let notifications_count = state.admin_status_requests.borrow().iter().filter(|r| r.status == "pending").count();
    let notif_text = if notifications_count > 0 {
        format!("🔔 {}", notifications_count)
    } else {
        "🔔".to_string()
    };
    let notif_title = if notifications_count > 0 {
        format!("{} {}", notifications_count, t("demandes_en_attente", &language))
    } else {
        t("demandes_en_attente", &language)
    };
    let notif_badge = ElementBuilder::new("button")?
        .class("btn-icon-header notif-badge")
        .attr("title", &notif_title)?
        .text(&notif_text)
        .build();
    {
        let state_clone = state.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            *state_clone.admin_view.borrow_mut() = "status_requests".to_string();
            let state_fetch = state_clone.clone();
            wasm_bindgen_futures::spawn_local(async move {
                use crate::services::api_client::ApiClient;
                let api = ApiClient::new();
                if let Ok(requests) = api.fetch_status_requests("notif_badge").await {
                    *state_fetch.admin_status_requests.borrow_mut() = requests;
                }
                crate::rerender_app();
            });
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        notif_badge.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    append_child(&actions, &notif_badge)?;
    
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
    let language = state.language.borrow().clone();
    let dashboard = ElementBuilder::new("div")?
        .class("admin-dashboard")
        .build();
    
    let title = ElementBuilder::new("h2")?
        .text(&format!("📊 {}", t("tableau_bord", &language)))
        .build();
    append_child(&dashboard, &title)?;
    
    Ok(dashboard)
}

/// Renderizar progress info para admin (agregado de todas las tournées)
/// Similar a render_progress_info del chofer pero usando datos agregados
fn render_admin_progress_info(state: &AppState) -> Result<(Element, Element), JsValue> {
    let total_packages = state.admin_total_packages.borrow().clone();
    
    // Calcular entregados desde todas las tournées
    let total_delivered: usize = state.admin_districts.borrow()
        .iter()
        .flat_map(|d| d.tournees.iter())
        .map(|t| t.delivered_count)
        .sum();
    
    // Calcular fallidos: buscar en sesiones almacenadas si están disponibles
    // Por ahora, intentar calcular desde sesiones seleccionadas (solución temporal)
    let mut total_failed = 0;
    if let Some(ref session) = *state.admin_selected_tournee_session.borrow() {
        total_failed = session.packages.values()
            .filter(|p| p.status.contains("NONLIV") || p.status.contains("ECHEC"))
            .count();
    }
    // TODO: Calcular desde todas las sesiones cuando se implemente backend completo
    
    // Calcular paradas (direcciones únicas) desde sesiones si están disponibles
    // Por ahora, usar aproximación temporal
    let total_addresses = if let Some(ref session) = *state.admin_selected_tournee_session.borrow() {
        session.stats.total_addresses
    } else {
        // Aproximación: usar número de tournées como aproximación de paradas
        state.admin_districts.borrow()
            .iter()
            .map(|d| d.tournees.len())
            .sum()
    };
    
    let completed_addresses = if let Some(ref session) = *state.admin_selected_tournee_session.borrow() {
        session.addresses.values()
            .filter(|address| {
                !address.package_ids.is_empty() && address.package_ids.iter().all(|pkg_id| {
                    session.packages.get(pkg_id)
                        .map(|pkg| !pkg.status.starts_with("STATUT_CHARGER"))
                        .unwrap_or(false)
                })
            })
            .count()
    } else {
        // Aproximación temporal
        total_addresses
    };
    
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
    
    // Texto de progreso (paradas tratadas, igual que chofer)
    let progress_text = ElementBuilder::new("div")?
        .class("progress-text")
        .build();
    
    let language = state.language.borrow().clone();
    let progress_count = ElementBuilder::new("span")?
        .class("progress-count")
        .text(&format!("✓ {}/{} {}", completed_addresses, total_addresses, t("traitees", &language)))
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
    
    // Crear body content (lista de districts) - id para actualización incremental
    let body_content = ElementBuilder::new("div")?
        .attr("id", "admin-package-list")?
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

/// Actualizar contenido del bottom sheet admin (progress + tournées) sin re-render completo.
/// Se llama cuando admin_districts cambia (p. ej. polling) para refrescar colis/livrés en las cards.
pub fn update_admin_bottom_sheet_content(state: &AppState) -> Result<(), JsValue> {
    use crate::dom::{get_element_by_id, remove_child, append_child, set_inner_html};

    let drag_handle_container = get_element_by_id("drag-handle-container")
        .ok_or_else(|| JsValue::from_str("admin bottom sheet not found"))?;

    // Reemplazar progress info y bar con datos actualizados
    if let Some(old_progress) = get_element_by_id("admin-progress-info") {
        let _ = remove_child(&drag_handle_container, &old_progress);
    }
    if let Some(old_bar) = get_element_by_id("admin-progress-bar-container") {
        let _ = remove_child(&drag_handle_container, &old_bar);
    }
    let (progress_info, progress_bar_container) = render_admin_progress_info(state)?;
    append_child(&drag_handle_container, &progress_info)?;
    append_child(&drag_handle_container, &progress_bar_container)?;

    // Reemplazar district/tournée cards con datos actualizados
    if let Some(package_list) = get_element_by_id("admin-package-list") {
        set_inner_html(&package_list, "");
        for (idx, district) in state.admin_districts.borrow().iter().enumerate() {
            let district_card = create_district_card(state, district, idx)?;
            append_child(&package_list, &district_card)?;
        }
    }
    Ok(())
}

/// Crear card de distrito (estilo package-card, igual que chofer)
fn create_district_card(state: &AppState, district: &AdminDistrict, index: usize) -> Result<Element, JsValue> {
    // Capturar estados antes de crear el card para evitar borrow conflicts
    let code_postal = district.code_postal.clone();
    let is_selected = state.admin_selected_district.borrow().as_ref().map_or(false, |selected| selected == &code_postal);
    let is_expanded = state.admin_expanded_districts.borrow().contains(&code_postal);
    
    // Usar clases de package-card para mantener consistencia
    let mut classes = vec!["package-card", "district-card"];
    if is_selected {
        classes.push("selected");
    }
    
    let card = ElementBuilder::new("div")?
        .class(&classes.join(" "))
        .build();
    
    // Click listener para seleccionar distrito (igual que chofer)
    {
        let state_clone = state.clone();
        let code_postal_clone = code_postal.clone();
        let card_clone = card.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            // Si ya está seleccionado, expandir/colapsar
            let currently_selected = state_clone.admin_selected_district.borrow().as_ref().map_or(false, |s| s == &code_postal_clone);
            if currently_selected {
                // Si está seleccionado, toggle expand
                {
                    let mut expanded = state_clone.admin_expanded_districts.borrow_mut();
                    if expanded.contains(&code_postal_clone) {
                        expanded.remove(&code_postal_clone);
                    } else {
                        expanded.insert(code_postal_clone.clone());
                    }
                } // Borrow liberado
            } else {
                // Si no está seleccionado, seleccionar este (deseleccionar el anterior si existe)
                {
                    *state_clone.admin_selected_district.borrow_mut() = Some(code_postal_clone.clone());
                } // Borrow liberado
            }
            crate::rerender_app();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        
        card_clone.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    // Header estilo package-header (vacío por ahora, botón info va en main content)
    let header = ElementBuilder::new("div")?
        .class("package-header")
        .build();
    
    append_child(&card, &header)?;
    
    // Main content estilo package-main (igual que chofer)
    let main = ElementBuilder::new("div")?
        .class("package-main")
        .build();
    
    let info = ElementBuilder::new("div")?
        .class("package-info")
        .build();
    
    // Recipient row: Botón info (izquierda) + Código postal (derecha)
    let recipient_row = ElementBuilder::new("div")?
        .class("package-recipient-row")
        .build();
    
    // Botón info (fondo izquierda, como en chofer)
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
    
    // Código postal (fondo derecha, elemento principal, sin "-")
    let recipient = ElementBuilder::new("div")?
        .class("package-recipient")
        .text(&district.code_postal)
        .build();
    
    append_child(&recipient_row, &info_btn)?;
    append_child(&recipient_row, &recipient)?;
    append_child(&info, &recipient_row)?;
    
    // Info resumida (tournées y colis) - equivalente a dirección en chofer
    let address = ElementBuilder::new("div")?
        .class("package-address")
        .text(&format!("🚚 {} {} • {} {}",
            district.tournees.len(),
            t("tournees", &state.language.borrow()),
            district.tournees.iter().map(|t| t.nb_colis).sum::<usize>(),
            t("paquets", &state.language.borrow())))
        .build();
    append_child(&info, &address)?;
    
    append_child(&main, &info)?;
    append_child(&card, &main)?;
    
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
        
        for (_letter, tournees) in tournees_by_letter {
            // Cards de tournées (sin título redundante)
            for (tournee_idx, tournee) in tournees.iter().enumerate() {
                let tournee_card = create_tournee_card(state, district, tournee, tournee_idx)?;
                append_child(&expanded_container, &tournee_card)?;
            }
        }
        
        append_child(&card, &expanded_container)?;
    }
    
    // EXPAND HANDLE - solo cuando está seleccionado (igual que chofer)
    // Aparece DESPUÉS del contenido, con pulse si no está expandido, sin pulse si está expandido
    if is_selected {
        let mut expand_handle_classes = vec!["expand-handle"];
        if !is_expanded {
            expand_handle_classes.push("pulse");
        }
        
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
                
                // Toggle expand
                {
                    let mut expanded = state_clone.admin_expanded_districts.borrow_mut();
                    if expanded.contains(&code_postal_clone) {
                        expanded.remove(&code_postal_clone);
                    } else {
                        expanded.insert(code_postal_clone.clone());
                    }
                } // Borrow liberado
                
                crate::rerender_app();
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            
            expand_handle_clone.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        // Expand handle siempre al final del card
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
                        
                        // Guardar sesión y seleccionar tournée; cambiar vista para mostrar grid de paquetes
                        *state_for_fetch.admin_selected_tournee_session.borrow_mut() = Some(session);
                        *state_for_fetch.admin_selected_tournee.borrow_mut() = Some(code_tournee_clone);
                        *state_for_fetch.admin_view.borrow_mut() = "packages".to_string();
                        
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
        .text(&format!("📦 {} {} • {} {} • {}",
            tournee.nb_colis,
            t("paquets", &state.language.borrow()),
            tournee.delivered_count,
            t("livres", &state.language.borrow()),
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
    let lang_grid = state.language.borrow().clone();
    let title_text = if let Some(ref session) = *state.admin_selected_tournee_session.borrow() {
        format!("{} {} – {} {}",
            t("tournee_word", &lang_grid),
            session.driver.driver_id,
            session.stats.total_packages,
            t("paquets", &lang_grid))
    } else {
        t("paquets_tournee", &lang_grid)
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
    let lang_ph = state.language.borrow().clone();
    let placeholder = ElementBuilder::new("div")?
        .class("packages-placeholder")
        .text(&t("chargement_paquets", &lang_ph))
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

/// Modal de détail d'une demande: infos + historial (traçabilité) + bouton Confirmer
pub fn render_status_request_detail_modal(state: &AppState) -> Result<Option<Element>, JsValue> {
    use crate::views::details_modal::create_traceability_section;
    use wasm_bindgen::closure::Closure;

    let req = state.admin_selected_status_request.borrow().clone();
    let req = match req {
        Some(r) => r,
        None => return Ok(None),
    };

    let tracking = req.tracking_code.clone();
    let sso_opt = state.admin_sso_token.borrow().clone();
    let username_opt = state.admin_username.borrow().clone();
    let societe_opt = state.admin_societe.borrow().clone();
    let loading = *state.admin_traceability_loading.borrow();
    let traceability_opt = state.package_traceability.borrow().clone();

    // Lancer le fetch de traçabilité si on a les creds et qu'on n'a pas encore chargé
    if traceability_opt.is_none() && !loading && sso_opt.is_some() && username_opt.is_some() && societe_opt.is_some() {
        *state.admin_traceability_loading.borrow_mut() = true;
        let state_fetch = state.clone();
        let tracking_fetch = tracking.clone();
        let sso = sso_opt.unwrap();
        let username = username_opt.unwrap();
        let societe = societe_opt.unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            let api = ApiClient::new();
            match api.fetch_package_traceability(&tracking_fetch, &sso, &username, &societe).await {
                Ok(resp) => {
                    *state_fetch.package_traceability.borrow_mut() = Some(resp);
                    *state_fetch.admin_traceability_loading.borrow_mut() = false;
                    crate::rerender_app();
                }
                Err(e) => {
                    console::error_1(&JsValue::from_str(&format!("❌ Traçabilité: {}", e)));
                    *state_fetch.admin_traceability_loading.borrow_mut() = false;
                    crate::rerender_app();
                }
            }
        });
    }

    let modal = ElementBuilder::new("div")?
        .class("modal active status-request-detail-modal")
        .build();
    let overlay = ElementBuilder::new("div")?
        .class("modal-overlay")
        .build();
    {
        let state_clone = state.clone();
        let c = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            *state_clone.admin_selected_status_request.borrow_mut() = None;
            *state_clone.package_traceability.borrow_mut() = None;
            *state_clone.admin_traceability_loading.borrow_mut() = false;
            crate::rerender_app();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        overlay.add_event_listener_with_callback("click", c.as_ref().unchecked_ref())?;
        c.forget();
    }
    append_child(&modal, &overlay)?;

    let content = ElementBuilder::new("div")?
        .class("modal-content")
        .build();
    {
        let c = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            e.stop_propagation();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        content.add_event_listener_with_callback("click", c.as_ref().unchecked_ref())?;
        c.forget();
    }

    let header = ElementBuilder::new("div")?
        .class("modal-header")
        .build();
    let title = ElementBuilder::new("h2")?
        .text(&format!("📋 {} – {}", t("demande", &state.language.borrow()), tracking))
        .build();
    let close_btn = ElementBuilder::new("button")?
        .class("btn-close")
        .text("✕")
        .build();
    {
        let state_clone = state.clone();
        let c = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            *state_clone.admin_selected_status_request.borrow_mut() = None;
            *state_clone.package_traceability.borrow_mut() = None;
            *state_clone.admin_traceability_loading.borrow_mut() = false;
            crate::rerender_app();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        close_btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref())?;
        c.forget();
    }
    append_child(&header, &title)?;
    append_child(&header, &close_btn)?;
    append_child(&content, &header)?;

    let body = ElementBuilder::new("div")?
        .class("modal-body")
        .build();

    let lang = state.language.borrow().clone();
    let section_req = ElementBuilder::new("div")?
        .class("detail-section")
        .build();
    let label_req = ElementBuilder::new("div")?
        .class("detail-label")
        .text(&t("demande", &lang))
        .build();
    let val_req = ElementBuilder::new("div")?
        .class("detail-value")
        .build();
    let req_text = format!(
        "👤 {} · 📍 {} · 👨‍✈️ {} · {}",
        req.customer_name,
        req.customer_address,
        req.driver_matricule,
        req.notes.as_deref().unwrap_or("–")
    );
    crate::dom::set_text_content(&val_req, &req_text);
    append_child(&section_req, &label_req)?;
    append_child(&section_req, &val_req)?;
    append_child(&body, &section_req)?;

    if let Some(ref traceability) = traceability_opt {
        let section_trace = create_traceability_section(traceability, &lang)?;
        append_child(&body, &section_trace)?;
    } else {
        let loading_section = ElementBuilder::new("div")?
            .class("detail-section")
            .build();
        let loading_label = ElementBuilder::new("div")?
            .class("detail-label")
            .text(&t("traçabilité", &lang))
            .build();
        let loading_value = ElementBuilder::new("div")?
            .class("detail-value")
            .text(&t("chargement", &lang))
            .build();
        append_child(&loading_section, &loading_label)?;
        append_child(&loading_section, &loading_value)?;
        append_child(&body, &loading_section)?;
    }

    let actions = ElementBuilder::new("div")?
        .class("modal-actions")
        .build();
    let cancel_btn = ElementBuilder::new("button")?
        .class("btn-cancel")
        .text(&t("fermer", &lang))
        .build();
    {
        let state_clone = state.clone();
        let c = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            *state_clone.admin_selected_status_request.borrow_mut() = None;
            *state_clone.package_traceability.borrow_mut() = None;
            *state_clone.admin_traceability_loading.borrow_mut() = false;
            crate::rerender_app();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        cancel_btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref())?;
        c.forget();
    }
    let confirm_btn = ElementBuilder::new("button")?
        .class("btn-confirm")
        .text(&format!("✅ {}", t("confirmer", &lang)))
        .build();
    {
        let state_clone = state.clone();
        let request_id = req.id.clone().unwrap_or_default();
        let admin_matricule = state.admin_username.borrow().clone().unwrap_or_else(|| "admin".to_string());
        let c = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            let state_c = state_clone.clone();
            let rid = request_id.clone();
            let am = admin_matricule.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let api = ApiClient::new();
                match api.confirm_request(&rid, &am).await {
                    Ok(_) => {
                        *state_c.admin_selected_status_request.borrow_mut() = None;
                        *state_c.package_traceability.borrow_mut() = None;
                        *state_c.admin_traceability_loading.borrow_mut() = false;
                        if let Ok(requests) = api.fetch_status_requests("confirm_modal").await {
                            *state_c.admin_status_requests.borrow_mut() = requests;
                        }
                        crate::rerender_app();
                    }
                    Err(e) => {
                        console::error_1(&JsValue::from_str(&format!("❌ Confirm: {}", e)));
                    }
                }
            });
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        confirm_btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref())?;
        c.forget();
    }
    append_child(&actions, &cancel_btn)?;
    append_child(&actions, &confirm_btn)?;
    append_child(&body, &actions)?;
    append_child(&content, &body)?;
    append_child(&modal, &content)?;

    Ok(Some(modal))
}

/// Crear vista de status requests (los pendings se actualizan con el polling del dashboard cada 60 s)
fn create_status_requests_view(state: &AppState) -> Result<Element, JsValue> {
    let container = ElementBuilder::new("div")?
        .class("status-requests-view")
        .build();
    
    let header = ElementBuilder::new("div")?
        .class("requests-header")
        .build();
    
    let title = ElementBuilder::new("h2")?
        .class("requests-title")
        .text(&format!("📋 {}", t("demandes_changement_statut", &state.language.borrow())))
        .build();
    
    let preview_visible = *state.admin_show_status_requests_preview.borrow();
    let confirmed_count = state.admin_status_requests.borrow().iter().filter(|r| r.status == "confirmed").count();
    let lang_toggle = state.language.borrow().clone();
    let toggle_text = if preview_visible { 
        t("masquer_apercu", &lang_toggle)
    } else { 
        format!("{} ({})", t("apercu_excel", &lang_toggle), confirmed_count)
    };
    
    let toggle_btn = ElementBuilder::new("button")?
        .class("btn-preview-toggle")
        .text(&toggle_text)
        .build();
    {
        let state_clone = state.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            let current = *state_clone.admin_show_status_requests_preview.borrow();
            *state_clone.admin_show_status_requests_preview.borrow_mut() = !current;
            crate::rerender_app();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        toggle_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    let header_row = ElementBuilder::new("div")?
        .class("requests-header-row")
        .build();
    append_child(&header_row, &title)?;
    append_child(&header_row, &toggle_btn)?;
    
    // Botones Exporter Excel y Fermer le jour (solo si hay confirmados)
    if confirmed_count > 0 {
        let export_btn = ElementBuilder::new("button")?
            .class("btn-export-excel")
            .text(&format!("📥 {}", t("exporter_excel", &state.language.borrow())))
            .build();
        {
            let state_clone = state.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                export_confirmed_to_csv(&state_clone);
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            export_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        append_child(&header_row, &export_btn)?;

        let close_day_btn = ElementBuilder::new("button")?
            .class("btn-close-day")
            .text(&format!("🔒 {}", t("fermer_le_jour", &state.language.borrow())))
            .build();
        {
            let state_clone = state.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
                if let Some(win) = web_sys::window() {
                    let msg = &t("voulez_fermer_jour", &state_clone.language.borrow());
                    if win.confirm_with_message(msg).unwrap_or(false) {
                        let societe_opt = state_clone.admin_societe.borrow().clone();
                        if let Some(societe) = societe_opt {
                            let state_for_close = state_clone.clone();
                            let societe_clone = societe.clone();
                            spawn_local(async move {
                                let api = ApiClient::new();
                                if let Ok(res) = api.close_day(&societe_clone).await {
                                    console::log_1(&JsValue::from_str(&format!("✅ Jour fermé: {} demandes, {} sessions supprimées", res.closed_count, res.sessions_deleted)));
                                    if let Ok(requests) = api.fetch_status_requests("close_day_button").await {
                                        *state_for_close.admin_status_requests.borrow_mut() = requests;
                                    }
                                    let u_opt = state_for_close.admin_username.borrow().clone();
                                    let p_opt = state_for_close.admin_password.borrow().clone();
                                    let s_opt = state_for_close.admin_societe.borrow().clone();
                                    if let (Some(u), Some(p), Some(s)) = (u_opt, p_opt, s_opt) {
                                        let today = js_sys::Date::new_0();
                                        let date_debut = format!("{:04}-{:02}-{:02}T00:00:00.000Z", today.get_full_year(), today.get_month() + 1, today.get_date());
                                        if let Ok(dash) = api.fetch_admin_dashboard(&u, &p, &s, &date_debut).await {
                                            *state_for_close.admin_districts.borrow_mut() = dash.districts;
                                            *state_for_close.admin_total_packages.borrow_mut() = dash.total_packages;
                                            *state_for_close.admin_selected_tournee_session.borrow_mut() = None;
                                            *state_for_close.admin_selected_tournee.borrow_mut() = None;
                                        }
                                    }
                                    crate::rerender_app();
                                } else {
                                    console::error_1(&JsValue::from_str("❌ Erreur lors de la fermeture du jour"));
                                }
                            });
                        } else {
                            console::error_1(&JsValue::from_str("❌ Societe non disponible"));
                        }
                    }
                }
            }) as Box<dyn FnMut(web_sys::MouseEvent)>);
            close_day_btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        append_child(&header_row, &close_day_btn)?;
    }
    
    append_child(&header, &header_row)?;
    append_child(&container, &header)?;
    
    // Preview tabular (REF COLIS + TYPE DE LIVRAISON par date)
    if preview_visible {
        let preview_section = create_status_requests_preview(state)?;
        append_child(&container, &preview_section)?;
    }
    
    let requests_list = ElementBuilder::new("div")?
        .class("requests-list")
        .build();
    
    // Solo mostrar los pendientes como cards (los confirmados solo en el aperçu)
    for request in state.admin_status_requests.borrow().iter().filter(|r| r.status == "pending") {
        let request_card = create_status_request_card(state, request)?;
        append_child(&requests_list, &request_card)?;
    }
    
    append_child(&container, &requests_list)?;
    
    Ok(container)
}

/// Formatear delivery_date (ej. "2026-01-05" -> "05/01/2026")
fn format_preview_date(s: &str) -> String {
    if s.len() >= 10 && s.as_bytes().get(4) == Some(&b'-') && s.as_bytes().get(7) == Some(&b'-') {
        let y = &s[0..4];
        let m = &s[5..7];
        let d = &s[8..10];
        format!("{}/{}/{}", d, m, y)
    } else {
        s.to_string()
    }
}

/// Crear sección preview: tablas REF COLIS (en majuscule) + TYPE DE LIVRAISON agrupadas por fecha
/// Solo muestra los requests confirmados (status == "confirmed")
fn create_status_requests_preview(state: &AppState) -> Result<Element, JsValue> {
    use std::collections::HashMap;
    let requests = state.admin_status_requests.borrow();
    let confirmed: Vec<&StatusChangeRequest> = requests.iter()
        .filter(|r| r.status == "confirmed")
        .collect();
    let mut by_date: HashMap<String, Vec<&StatusChangeRequest>> = HashMap::new();
    for r in confirmed {
        let key = r.delivery_date.clone();
        by_date.entry(key).or_default().push(r);
    }
    let mut dates: Vec<String> = by_date.keys().cloned().collect();
    dates.sort();

    let section = ElementBuilder::new("div")?
        .class("status-requests-preview")
        .build();

    for date_key in dates {
        let reqs = by_date.get(&date_key).unwrap();
        let date_label = format_preview_date(&date_key);

        let block = ElementBuilder::new("div")?
            .class("preview-date-block")
            .build();
        let date_header = ElementBuilder::new("div")?
            .class("preview-date-header")
            .text(&date_label)
            .build();
        append_child(&block, &date_header)?;

        let table = ElementBuilder::new("table")?
            .class("preview-table")
            .build();
        let thead = ElementBuilder::new("thead")?.build();
        let tr_head = ElementBuilder::new("tr")?.build();
        let th_ref = ElementBuilder::new("th")?
            .class("preview-th")
            .text(&t("ref_colis", &state.language.borrow()))
            .build();
        let th_type = ElementBuilder::new("th")?
            .class("preview-th")
            .text(&t("type_livraison", &state.language.borrow()))
            .build();
        append_child(&tr_head, &th_ref)?;
        append_child(&tr_head, &th_type)?;
        append_child(&thead, &tr_head)?;
        append_child(&table, &thead)?;

        let tbody = ElementBuilder::new("tbody")?.build();
        for req in reqs {
            let tr = ElementBuilder::new("tr")?.build();
            let td_ref = ElementBuilder::new("td")?
                .class("preview-td preview-td-ref")
                .text(&req.tracking_code.to_uppercase())
                .build();
            let type_livraison = req.notes.as_deref().unwrap_or("—");
            let td_type = ElementBuilder::new("td")?
                .class("preview-td")
                .text(type_livraison)
                .build();
            append_child(&tr, &td_ref)?;
            append_child(&tr, &td_type)?;
            append_child(&tbody, &tr)?;
        }
        append_child(&table, &tbody)?;
        append_child(&block, &table)?;
        append_child(&section, &block)?;
    }

    Ok(section)
}

/// Crear card de status request (clickeable para abrir modal con historial)
fn create_status_request_card(state: &AppState, request: &StatusChangeRequest) -> Result<Element, JsValue> {
    let card = ElementBuilder::new("div")?
        .class("status-request-card clickable")
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
        .text(&format!("🔴 {}", t("en_attente", &state.language.borrow())))
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
        .text(&format!("👨‍✈️ {} {}", t("signale_par", &state.language.borrow()), request.driver_matricule))
        .build();
    
    append_child(&info, &customer)?;
    append_child(&info, &address)?;
    append_child(&info, &driver)?;
    
    if let Some(notes) = &request.notes {
        let notes_el = ElementBuilder::new("div")?
            .class("request-notes")
            .text(&format!("📝 {} {}", t("notes_label", &state.language.borrow()), notes))
            .build();
        append_child(&info, &notes_el)?;
    }
    
    append_child(&card, &info)?;
    
    let action_hint = ElementBuilder::new("div")?
        .class("request-action-hint")
        .text(&t("cliquer_voir_historique_confirmer", &state.language.borrow()))
        .build();
    append_child(&card, &action_hint)?;
    
    // Clic sur la card ouvre le modal (historique + confirmer)
    {
        let state_clone = state.clone();
        let request_clone = request.clone();
        let closure = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            *state_clone.admin_selected_status_request.borrow_mut() = Some(request_clone.clone());
            *state_clone.package_traceability.borrow_mut() = None;
            *state_clone.admin_traceability_loading.borrow_mut() = false;
            crate::rerender_app();
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        card.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
    Ok(card)
}

/// Polling automático: dashboard (tournées) + status change requests pendientes (cada 30 s en modo admin).
/// Solo un intervalo activo; no recrear en cada re-render (evita loop infinito: poll→rerender→setup→spawn→fetch→rerender→setup...)
fn setup_dashboard_polling(state: &AppState) {
    use gloo_timers::callback::Interval;
    
    if state.admin_dashboard_polling_interval.borrow().is_some() {
        return;
    }
    
    let state_clone = state.clone();
    
    let interval = Interval::new(30_000, move || {
        if !*state_clone.admin_mode.borrow() {
            return;
        }
        let state_for_fetch = state_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            use crate::services::api_client::ApiClient;
            use web_sys::{Notification, NotificationOptions, NotificationPermission};
            
            let api = ApiClient::new();
            let prev_pending = state_for_fetch.admin_status_requests.borrow()
                .iter()
                .filter(|r| r.status == "pending")
                .count();
            
            if let Ok(requests) = api.fetch_status_requests("dashboard_polling").await {
                let new_pending = requests.iter()
                    .filter(|r| r.status == "pending")
                    .count();
                *state_for_fetch.admin_status_requests.borrow_mut() = requests;
                
                // Notificación del navegador si hay nuevas demandes pendientes
                if new_pending > prev_pending
                    && *state_for_fetch.admin_notifications_enabled.borrow()
                    && Notification::permission() == NotificationPermission::Granted
                {
                    let body = if new_pending == 1 {
                        "1 demande en attente".to_string()
                    } else {
                        format!("{} demandes en attente", new_pending)
                    };
                    let mut opts = NotificationOptions::new();
                    opts.set_body(&body);
                    opts.set_icon("/icon-192.png");
                    opts.set_tag("admin-demandes");
                    let _ = Notification::new_with_options("Route Optimizer", &opts);
                }
            }
            // Actualizar tournées siempre (bottom sheet visible en todas las vistas)
            let username_opt = state_for_fetch.admin_username.borrow().clone();
            let password_opt = state_for_fetch.admin_password.borrow().clone();
            let societe_opt = state_for_fetch.admin_societe.borrow().clone();
            if let (Some(username), Some(password), Some(societe)) = (username_opt, password_opt, societe_opt) {
                let today = js_sys::Date::new_0();
                let date_debut = format!(
                    "{:04}-{:02}-{:02}T00:00:00.000Z",
                    today.get_full_year(),
                    today.get_month() + 1,
                    today.get_date()
                );
                if let Ok(response) = api.fetch_admin_dashboard(&username, &password, &societe, &date_debut).await {
                    let expanded_districts = state_for_fetch.admin_expanded_districts.borrow().clone();
                    *state_for_fetch.admin_districts.borrow_mut() = response.districts;
                    *state_for_fetch.admin_total_packages.borrow_mut() = response.total_packages;
                    *state_for_fetch.admin_expanded_districts.borrow_mut() = expanded_districts;
                    *state_for_fetch.admin_sso_token.borrow_mut() = Some(response.sso_token.clone());
                    // Actualizar contenido del bottom sheet (tournées, colis, livrés) sin full re-render
                    let _ = crate::rerender_app_with_type(crate::state::app_state::UpdateType::Incremental(
                        crate::state::app_state::IncrementalUpdate::AdminBottomSheetContent,
                    ));
                }
            }
            crate::rerender_app();
        });
    });
    // Primera ejecución inmediata para tener datos al entrar
    wasm_bindgen_futures::spawn_local({
        let state_first = state.clone();
        async move {
            use crate::services::api_client::ApiClient;
            let api = ApiClient::new();
            if let Ok(requests) = api.fetch_status_requests("dashboard_polling").await {
                *state_first.admin_status_requests.borrow_mut() = requests;
                crate::rerender_app();
            }
        }
    });
    *state.admin_dashboard_polling_interval.borrow_mut() = Some(interval);
}

/// Exportar los requests confirmados a un archivo CSV (compatible con Excel)
fn export_confirmed_to_csv(state: &AppState) {
    use std::collections::HashMap;
    
    let requests = state.admin_status_requests.borrow();
    let confirmed: Vec<_> = requests.iter().filter(|r| r.status == "confirmed").collect();
    
    if confirmed.is_empty() {
        console::log_1(&JsValue::from_str("⚠️ Aucune demande confirmée à exporter"));
        return;
    }
    
    // Agrupar por fecha
    let mut by_date: HashMap<String, Vec<&StatusChangeRequest>> = HashMap::new();
    for r in &confirmed {
        by_date.entry(r.delivery_date.clone()).or_default().push(r);
    }
    let mut dates: Vec<String> = by_date.keys().cloned().collect();
    dates.sort();
    
    // Generar CSV con BOM para Excel (UTF-8)
    let mut csv = String::from("\u{FEFF}"); // BOM UTF-8
    
    for date_key in &dates {
        let reqs = by_date.get(date_key).unwrap();
        let date_label = format_preview_date(date_key);
        
        // Header con fecha
        csv.push_str(&format!("Date: {}\n", date_label));
        csv.push_str("REF COLIS;TYPE DE LIVRAISON\n");
        
        for req in reqs {
            let ref_colis = req.tracking_code.to_uppercase();
            let type_livraison = req.notes.as_deref().unwrap_or("");
            csv.push_str(&format!("{};{}\n", ref_colis, type_livraison));
        }
        csv.push('\n');
    }
    
    // Usar JavaScript para crear blob y descargar
    let today = js_sys::Date::new_0();
    let filename = format!(
        "livraisons_confirmees_{:04}-{:02}-{:02}.csv",
        today.get_full_year(),
        today.get_month() + 1,
        today.get_date()
    );
    
    let js_code = format!(
        r#"
        (function() {{
            const csv = {};
            const filename = "{}";
            const blob = new Blob([csv], {{ type: 'text/csv;charset=utf-8' }});
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        }})();
        "#,
        serde_json::to_string(&csv).unwrap_or_else(|_| "\"\"".to_string()),
        filename
    );
    
    let _ = js_sys::eval(&js_code);
    
    console::log_1(&JsValue::from_str(&format!("✅ Export CSV: {} demandes confirmées", confirmed.len())));
    
    // Diálogo: ¿Cerrar el día y vaciar el aperçu?
    if let Some(win) = web_sys::window() {
        let msg = &t("export_terminée_fermer", &state.language.borrow());
        if win.confirm_with_message(msg).unwrap_or(false) {
            let societe_opt = state.admin_societe.borrow().clone();
            if let Some(societe) = societe_opt {
                let state_clone = state.clone();
                let societe_clone = societe.clone();
                spawn_local(async move {
                    let api = ApiClient::new();
                    if let Ok(res) = api.close_day(&societe_clone).await {
                        console::log_1(&JsValue::from_str(&format!("✅ Jour fermé: {} demandes, {} sessions supprimées", res.closed_count, res.sessions_deleted)));
                        if let Ok(requests) = api.fetch_status_requests("close_day_after_export").await {
                            *state_clone.admin_status_requests.borrow_mut() = requests;
                        }
                        let u_opt = state_clone.admin_username.borrow().clone();
                        let p_opt = state_clone.admin_password.borrow().clone();
                        let s_opt = state_clone.admin_societe.borrow().clone();
                        if let (Some(u), Some(p), Some(s)) = (u_opt, p_opt, s_opt) {
                            let today = js_sys::Date::new_0();
                            let date_debut = format!("{:04}-{:02}-{:02}T00:00:00.000Z", today.get_full_year(), today.get_month() + 1, today.get_date());
                            if let Ok(dash) = api.fetch_admin_dashboard(&u, &p, &s, &date_debut).await {
                                *state_clone.admin_districts.borrow_mut() = dash.districts;
                                *state_clone.admin_total_packages.borrow_mut() = dash.total_packages;
                                *state_clone.admin_selected_tournee_session.borrow_mut() = None;
                                *state_clone.admin_selected_tournee.borrow_mut() = None;
                            }
                        }
                        crate::rerender_app();
                    } else {
                        console::error_1(&JsValue::from_str("❌ Erreur lors de la fermeture du jour"));
                    }
                });
            } else {
                console::error_1(&JsValue::from_str("❌ Societe non disponible"));
            }
        }
    }
}
