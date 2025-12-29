// ============================================================================
// BOTTOM SHEET - Componente de lista de paquetes con estados collapsed/half/full
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::Element;
use std::rc::Rc;
use crate::dom::{ElementBuilder, append_child, set_attribute, add_class, remove_class};
use crate::dom::events::on_click;
use crate::state::app_state::AppState;
use crate::models::session::DeliverySession;
use crate::views::{PackageGroup, render_package_list};
use crate::utils::i18n::t;

/// Renderizar bottom sheet completo
pub fn render_bottom_sheet(
    state: &AppState,
    session: &DeliverySession,
    groups: &[PackageGroup],
    on_toggle_sheet: Rc<dyn Fn()>,
    on_close_sheet: Rc<dyn Fn()>,
    on_package_selected: Rc<dyn Fn(usize)>,
) -> Result<Element, JsValue> {
    let sheet_state = state.sheet_state.borrow().clone();
    
    // Container principal
    let container = ElementBuilder::new("div")?
        .attr("id", "package-container")?
        .class("package-container")
        .build();
    
    // Backdrop (solo visible cuando sheet no está collapsed)
    let backdrop = ElementBuilder::new("div")?
        .attr("id", "backdrop")?
        .class("backdrop")
        .build();
    
    if sheet_state != "collapsed" {
        add_class(&backdrop, "active")?;
    }
    
    // Event listener para cerrar sheet al hacer click en backdrop
    {
        let on_close = on_close_sheet.clone();
        on_click(&backdrop, move |_| {
            on_close();
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
    
    // Event listener para toggle sheet size
    {
        let on_toggle = on_toggle_sheet.clone();
        on_click(&drag_handle_container, move |_| {
            on_toggle();
        })?;
    }
    
    append_child(&drag_handle_container, &drag_handle)?;
    
    // Progress Info y Progress Bar (solo cuando sheet no está collapsed)
    // En Yew original: progress-info y progress-bar-container son hermanos dentro de drag-handle-container
    if sheet_state != "collapsed" {
        let (progress_info, progress_bar_container) = render_progress_info(session, state)?;
        append_child(&drag_handle_container, &progress_info)?;
        append_child(&drag_handle_container, &progress_bar_container)?;
    }
    
    append_child(&bottom_sheet, &drag_handle_container)?;
    
    // Package List (contenido scrolleable)
    if session.packages.is_empty() {
        // Estado vacío
        let no_packages = ElementBuilder::new("div")?
            .class("no-packages")
            .build();
        
        let icon = ElementBuilder::new("div")?
            .class("no-packages-icon")
            .text("📦")
            .build();
        
        let language = state.language.borrow().clone();
        let text = ElementBuilder::new("div")?
            .class("no-packages-text")
            .text(&t("aucun_colis", &language))
            .build();
        
        let subtitle = ElementBuilder::new("div")?
            .class("no-packages-subtitle")
            .text(&t("veuillez_rafraichir", &language))
            .build();
        
        append_child(&no_packages, &icon)?;
        append_child(&no_packages, &text)?;
        append_child(&no_packages, &subtitle)?;
        append_child(&bottom_sheet, &no_packages)?;
    } else {
        // Lista de paquetes
        let addresses_map: std::collections::HashMap<String, String> = session.addresses
            .iter()
            .map(|(k, v)| (k.clone(), v.label.clone()))
            .collect();
        
        let selected_index = *state.selected_package_index.borrow();
        
        let package_list = render_package_list(
            groups.to_vec(),
            &addresses_map,
            selected_index,
            state,
            on_package_selected.clone(),
            {
                let state_clone = state.clone();
                let session_clone = session.clone();
                Rc::new(move |tracking: String| {
                    log::info!("📦 Abriendo detalles para tracking: {}", tracking);
                    
                    // Guardar posición de scroll antes de abrir el modal
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("💾 [SCROLL] Guardando posición antes de abrir modal de detalles"));
                    state_clone.save_package_list_scroll_position();
                    
                    // Buscar el paquete en la sesión
                    if let Some(pkg) = session_clone.packages.get(&tracking) {
                        // Obtener la dirección asociada
                        if let Some(addr) = session_clone.addresses.get(&pkg.address_id) {
                            log::info!("✅ Paquete y dirección encontrados");
                            // Primero establecer details_package
                            {
                                let mut details = state_clone.details_package.borrow_mut();
                                *details = Some((pkg.clone(), addr.clone()));
                            }
                            // Luego mostrar el modal (esto puede necesitar re-render completo si el modal no existe)
                            state_clone.set_show_details(true);
                        } else {
                            log::warn!("⚠️ Dirección no encontrada para address_id: {}", pkg.address_id);
                        }
                    } else {
                        log::warn!("⚠️ Paquete no encontrado: {}", tracking);
                    }
                })
            },
        )?;
        
        append_child(&bottom_sheet, &package_list)?;
    }
    
    append_child(&container, &bottom_sheet)?;
    
    Ok(container)
}

/// Renderizar progress info (direcciones tratadas y paquetes) y progress bar
/// Retorna (progress_info, progress_bar_container) como elementos hermanos
/// Estructura igual que en Yew: ambos son hijos directos de drag-handle-container
pub fn render_progress_info(session: &DeliverySession, state: &AppState) -> Result<(Element, Element), JsValue> {
    // Calcular direcciones tratadas
    let total_addresses = session.stats.total_addresses;
    let completed_addresses = session.addresses.values()
        .filter(|address| {
            // Dirección tratada = TODOS los paquetes están hechos (no CHARGER)
            !address.package_ids.is_empty() && address.package_ids.iter().all(|pkg_id| {
                session.packages.get(pkg_id)
                    .map(|pkg| !pkg.status.starts_with("STATUT_CHARGER"))
                    .unwrap_or(false)
            })
        })
        .count();
    
    // Calcular paquetes
    let total_packages = session.stats.total_packages;
    let delivered_packages = session.packages.values()
        .filter(|p| p.status.contains("LIVRER"))
        .count();
    let failed_packages = session.packages.values()
        .filter(|p| p.status.contains("NONLIV") || p.status.contains("ECHEC"))
        .count();
    
    // Porcentajes para la barra de progreso
    let delivered_percent = if total_packages > 0 {
        (delivered_packages * 100) / total_packages
    } else {
        0
    };
    
    let failed_percent = if total_packages > 0 {
        (failed_packages * 100) / total_packages
    } else {
        0
    };
    
    // ===== PROGRESS INFO (primer hermano) =====
    let progress_info = ElementBuilder::new("div")?
        .attr("id", "progress-info")?
        .class("progress-info")
        .build();
    
    // Texto de progreso (direcciones tratadas)
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
        .text(&format!("{}/{} {}", delivered_packages, total_packages, t("paquets", &language)))
        .build();
    
    append_child(&progress_packages, &packages_count)?;
    
    // Agregar texto y paquetes a progress-info (hermanos dentro de progress-info)
    append_child(&progress_info, &progress_text)?;
    append_child(&progress_info, &progress_packages)?;
    
    // ===== PROGRESS BAR CONTAINER (segundo hermano) =====
    let progress_bar_container = ElementBuilder::new("div")?
        .attr("id", "progress-bar-container")?
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

