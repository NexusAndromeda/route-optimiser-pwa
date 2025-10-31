// ============================================================================
// DRAGGABLE PACKAGE LIST COMPONENT
// ============================================================================
// ✅ COPIADO DEL ORIGINAL - Preserva HTML/CSS exacto
// Adaptado para usar Yewdux + ViewModels
// ============================================================================

use yew::prelude::*;
use web_sys::DragEvent;
use crate::hooks::use_sync_state;
use crate::models::sync::Change;
use crate::models::package::Package;

#[derive(Properties, PartialEq)]
pub struct DraggablePackageListProps {
    pub packages: Vec<Package>,
}

/// Componente de lista de paquetes draggable
/// ✅ HTML EXACTO DEL ORIGINAL preservado
#[function_component(DraggablePackageList)]
pub fn draggable_package_list(props: &DraggablePackageListProps) -> Html {
    let sync_handle = use_sync_state();
    let dragged_index = use_state(|| None::<usize>);
    let drag_over_index = use_state(|| None::<usize>);
    
    let on_drag_start = {
        let dragged_index = dragged_index.clone();
        Callback::from(move |(index, event): (usize, DragEvent)| {
            dragged_index.set(Some(index));
            
            if let Some(dt) = event.data_transfer() {
                dt.set_effect_allowed("move");
                let _ = dt.set_data("text/plain", &index.to_string());
            }
            
            log::info!("🎯 Drag started: index {}", index);
        })
    };
    
    let on_drag_over = {
        let drag_over_index = drag_over_index.clone();
        Callback::from(move |(index, event): (usize, DragEvent)| {
            event.prevent_default();
            drag_over_index.set(Some(index));
            
            if let Some(dt) = event.data_transfer() {
                dt.set_drop_effect("move");
            }
        })
    };
    
    let on_drag_leave = {
        let drag_over_index = drag_over_index.clone();
        Callback::from(move |_: DragEvent| {
            drag_over_index.set(None);
        })
    };
    
    let on_drop = {
        let dragged_index = dragged_index.clone();
        let drag_over_index = drag_over_index.clone();
        let add_pending_change = sync_handle.add_pending_change.clone();
        let packages = props.packages.clone();
        
        Callback::from(move |(target_index, event): (usize, DragEvent)| {
            event.prevent_default();
            
            if let Some(from_index) = *dragged_index {
                if from_index != target_index {
                    log::info!("📦 Drop: moving package from {} to {}", from_index, target_index);
                    
                    // Crear cambio de reordenamiento
                    if let Some(package) = packages.get(from_index) {
                        let change = Change::OrderChanged {
                                package_internal_id: package.tracking.clone(),
                            old_position: from_index,
                            new_position: target_index,
                            timestamp: chrono::Utc::now().timestamp(),
                        };
                        
                        // Agregar a cambios pendientes usando hook
                        add_pending_change.emit(change.clone());
                        
                        log::info!("✅ Reorder change queued for sync");
                    }
                }
            }
            
            dragged_index.set(None);
            drag_over_index.set(None);
        })
    };
    
    let on_drag_end = {
        let dragged_index = dragged_index.clone();
        let drag_over_index = drag_over_index.clone();
        
        Callback::from(move |_: DragEvent| {
            dragged_index.set(None);
            drag_over_index.set(None);
        })
    };
    
    // ✅ HTML EXACTO DEL ORIGINAL
    html! {
        <div class="draggable-package-list">
            <div class="list-header">
                <h3>{"Paquetes"}</h3>
                <span class="drag-hint">{"🤚 Arrastra para reordenar"}</span>
            </div>
            
            <div class="packages-container">
                {
                    props.packages.iter().enumerate().map(|(index, package)| {
                        let is_dragging = *dragged_index == Some(index);
                        let is_drag_over = *drag_over_index == Some(index);
                        
                        let class = classes!(
                            "package-item",
                            "draggable",
                            is_dragging.then(|| "dragging"),
                            is_drag_over.then(|| "drag-over")
                        );
                        
                        let on_dragstart = {
                            let on_drag_start = on_drag_start.clone();
                            Callback::from(move |e: DragEvent| {
                                on_drag_start.emit((index, e));
                            })
                        };
                        
                        let on_dragover = {
                            let on_drag_over = on_drag_over.clone();
                            Callback::from(move |e: DragEvent| {
                                on_drag_over.emit((index, e));
                            })
                        };
                        
                        html! {
                            <div
                                key={package.tracking.clone()}
                                class={class}
                                draggable="true"
                                ondragstart={on_dragstart}
                                ondragover={on_dragover}
                                ondragleave={on_drag_leave.clone()}
                                ondrop={Callback::from({
                                    let on_drop = on_drop.clone();
                                    let target_idx = index;
                                    move |e: DragEvent| {
                                        on_drop.emit((target_idx, e));
                                    }
                                })}
                                ondragend={on_drag_end.clone()}
                            >
                                <div class="drag-handle">{"⋮⋮"}</div>
                                
                                <div class="package-content">
                                    <div class="package-number">
                                        {
                                            if let Some(route_order) = package.route_order {
                                                format!("#{}", route_order + 1)
                                            } else {
                                                format!("#{}", package.original_order + 1)
                                            }
                                        }
                                    </div>
                                    
                                    <div class="package-info">
                                        <div class="customer-name">{&package.customer_name}</div>
                                        <div class="tracking">{&package.tracking}</div>
                                    </div>
                                    
                                    <div class="package-status">
                                        {
                                            match package.status.as_str() {
                                                "STATUT_CHARGER" => html! { <span class="status loaded">{"📦 Cargado"}</span> },
                                                "STATUT_SCANNED" => html! { <span class="status scanned">{"✅ Escaneado"}</span> },
                                                "STATUT_LIVRE" => html! { <span class="status delivered">{"🎉 Entregado"}</span> },
                                                "STATUT_ECHEC" => html! { <span class="status failed">{"❌ Fallido"}</span> },
                                                _ => html! { <span class="status unknown">{"❓"}</span> }
                                            }
                                        }
                                    </div>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
        </div>
    }
}

