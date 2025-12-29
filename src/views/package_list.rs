// ============================================================================
// PACKAGE LIST VIEW - Convertida de componente Yew a función Rust puro
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::Element;
use std::collections::HashMap;
use std::rc::Rc;
use crate::dom::{ElementBuilder, append_child};
use crate::views::package_card::render_package_card;
use crate::models::package::Package;
use crate::state::app_state::AppState;

/// Grupo de paquetes (equivalente a PackageGroup)
#[derive(Clone)]
pub struct PackageGroup {
    pub title: String,
    pub count: usize,
    pub packages: Vec<Package>,
}

/// Renderizar lista de paquetes
pub fn render_package_list(
    groups: Vec<PackageGroup>,
    addresses: &HashMap<String, String>,
    selected_index: Option<usize>,
    state: &AppState,
    on_select: Rc<dyn Fn(usize)>,
    on_info: Rc<dyn Fn(String)>,
) -> Result<Element, JsValue> {
    // Obtener grupos expandidos del estado
    let expanded_groups = state.expanded_groups.borrow().clone();
    let list = ElementBuilder::new("div")?
        .attr("id", "package-list")?
        .class("package-list")
        .build();
    
    // Renderizar cada grupo
    for (group_idx, group) in groups.iter().enumerate() {
        if group.packages.is_empty() {
            continue;
        }
        
        let is_selected = selected_index == Some(group_idx);
        
        // Obtener dirección
        let address = group.packages.first()
            .and_then(|p| addresses.get(&p.address_id))
            .map(|s| s.as_str());
        
        // Renderizar card del grupo
        if let Some(first_pkg) = group.packages.first() {
            // Si el grupo tiene múltiples paquetes, crear un paquete "virtual" para el grupo
            let is_group = group.packages.len() > 1;
            let is_expanded = expanded_groups.contains(&group_idx);
            
            let mut group_package = first_pkg.clone();
            if is_group {
                group_package.customer_name = format!("{} paquetes", group.packages.len());
                group_package.is_group = true;
                group_package.group_packages = Some(group.packages.clone());
            }
            
            let on_select_card = {
                let on_select_clone = on_select.clone();
                let idx = group_idx;
                Rc::new(move |_idx: usize| on_select_clone(idx)) as Rc<dyn Fn(usize)>
            };
            
            let on_info_card = {
                let on_info_clone = on_info.clone();
                let tracking = first_pkg.tracking.clone();
                Rc::new(move |track: String| on_info_clone(track)) as Rc<dyn Fn(String)>
            };
            
            // Toggle expand/collapse callback (solo para grupos)
            // Capturar group_idx correctamente en el closure
            let on_toggle_expand: Option<Rc<dyn Fn(usize)>> = if is_group {
                let state_clone = state.clone();
                let current_group_idx = group_idx; // Capturar el índice actual del grupo
                Some(Rc::new(move |_idx: usize| {
                    // Usar el índice capturado en lugar del parámetro
                    // Liberar el borrow antes de llamar a rerender_app_with_type
                    {
                    let mut expanded = state_clone.expanded_groups.borrow_mut();
                    if expanded.contains(&current_group_idx) {
                        expanded.remove(&current_group_idx);
                    } else {
                        expanded.insert(current_group_idx);
                    }
                    } // Borrow liberado aquí
                    
                    // Trigger re-render para actualizar UI (ahora es seguro)
                    crate::rerender_app_with_type(crate::state::app_state::UpdateType::Incremental(
                        crate::state::app_state::IncrementalUpdate::PackageList
                    ));
                }) as Rc<dyn Fn(usize)>)
            } else {
                None
            };
            
            let card = render_package_card(
                &group_package,
                group_idx,
                address,
                is_selected,
                is_expanded,
                on_select_card,
                on_info_card,
                on_toggle_expand,
            )?;
            
            // Agregar data-index para scroll automático
            crate::dom::set_attribute(&card, "data-index", &group_idx.to_string())?;
            
            append_child(&list, &card)?;
        }
    }
    
    Ok(list)
}

/// Agrupar paquetes por dirección (equivalente a use_grouped_packages)
pub fn group_packages_by_address(packages: Vec<Package>) -> Vec<PackageGroup> {
    use std::collections::HashMap;
    
    let mut grouped: HashMap<String, Vec<Package>> = HashMap::new();
    
    for package in packages {
        grouped
            .entry(package.address_id.clone())
            .or_insert_with(Vec::new)
            .push(package);
    }
    
    // Ordenar paquetes dentro de cada grupo
    for packages in grouped.values_mut() {
        packages.sort_by_key(|p| {
            p.route_order.unwrap_or(p.original_order)
        });
    }
    
    // Convertir a grupos
    let mut groups: Vec<PackageGroup> = grouped
        .into_iter()
        .map(|(address_id, packages)| PackageGroup {
            title: address_id,
            count: packages.len(),
            packages,
        })
        .collect();
    
    // Ordenar grupos por orden del primer paquete
    groups.sort_by_key(|group| {
        group.packages.first()
            .map(|p| p.route_order.unwrap_or(p.original_order))
            .unwrap_or(0)
    });
    
    groups
}

