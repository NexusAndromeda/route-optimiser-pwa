use yew::prelude::*;
use std::collections::{HashSet, HashMap};
use crate::hooks::use_grouped_packages::PackageGroup;
use crate::components::package_card::PackageCard;

#[derive(Properties, PartialEq, Clone)]
pub struct PackageListProps {
    pub groups: Vec<PackageGroup>,
    pub addresses: HashMap<String, String>,
    pub on_info: Callback<String>,
    #[prop_or_default]
    pub on_package_selected: Option<Callback<usize>>, // Para conectar con el mapa
    #[prop_or_default]
    pub selected_index: Option<usize>, // Para sincronizar selección desde el mapa
}

#[function_component(PackageList)]
pub fn package_list(props: &PackageListProps) -> Html {
    let expanded_cards = use_state(|| HashSet::<usize>::new());
    let selected_key = use_state(|| props.selected_index);
    let previous_selected = use_state(|| None as Option<usize>);
    let animations = use_state(|| HashMap::<usize, String>::new());
    
    // Sincronizar cuando cambia desde el padre (click en mapa)
    {
        let selected_key = selected_key.clone();
        let prop_selected = props.selected_index;
        use_effect_with(prop_selected, move |&index| {
            selected_key.set(index);
            || ()
        });
    }

    // Toggle expand/collapse de card (si es grupo)
    let toggle_expand = {
        let expanded_cards = expanded_cards.clone();
        Callback::from(move |card_idx: usize| {
            let mut set = (*expanded_cards).clone();
            if set.contains(&card_idx) {
                set.remove(&card_idx);
            } else {
                set.insert(card_idx);
            }
            expanded_cards.set(set);
        })
    };

    // Seleccionar card - SIN animación flash (solo cambia el estado)
    let on_select = {
        let selected_key = selected_key.clone();
        let previous_selected = previous_selected.clone();
        let on_package_selected = props.on_package_selected.clone();
        Callback::from(move |idx: usize| {
            previous_selected.set(Some(idx));
            selected_key.set(Some(idx));
            
            // Notificar al padre (para sincronizar con mapa)
            if let Some(callback) = &on_package_selected {
                callback.emit(idx);
            }
        })
    };

    let on_navigate = Callback::from(|idx: usize| {
        log::info!("Navigate to package index: {}", idx);
    });

    html! {
        <div class="package-list">
            { for props.groups.iter().enumerate().map(|(group_idx, group)| {
                // Si el grupo tiene MÚLTIPLES paquetes, crear un card agrupado
                if group.packages.len() > 1 {
                    let address_id = group.title.clone();
                    let address_label = props.addresses.get(&address_id)
                        .cloned()
                        .unwrap_or_else(|| address_id.clone());
                    
                    let is_selected = (*selected_key) == Some(group_idx);
                    let is_expanded = (*expanded_cards).contains(&group_idx);
                    let animation_class = (*animations).get(&group_idx).cloned();
                    
                    // Crear un Package "virtual" para el grupo
                    let first_pkg = group.packages.first().unwrap();
                    let mut group_package = first_pkg.clone();
                    group_package.customer_name = format!("{} paquetes", group.packages.len());
                    group_package.is_group = true;
                    group_package.group_packages = Some(group.packages.clone());
                    
                    let on_select_card = {
                        let on_select = on_select.clone();
                        Callback::from(move |_| on_select.emit(group_idx))
                };
                    
                html!{
                        <PackageCard 
                            key={address_id.clone()}
                            package={group_package} 
                            index={group_idx} 
                            address={Some(address_label)} 
                            on_info={props.on_info.clone()} 
                            is_selected={is_selected}
                            is_expanded={is_expanded}
                            on_select={Some(on_select_card)}
                            on_navigate={Some(on_navigate.clone())}
                            on_toggle_expand={Some(toggle_expand.clone())}
                            animation_class={animation_class}
                        />
                    }
                } else {
                    // Paquete individual (sin agrupar)
                    let package = group.packages.first().unwrap();
                    let addr = props.addresses.get(&package.address_id).cloned();
                    let is_selected = (*selected_key) == Some(group_idx);
                    let animation_class = (*animations).get(&group_idx).cloned();
                    
                                            let on_select_card = {
                                                let on_select = on_select.clone();
                        Callback::from(move |_| on_select.emit(group_idx))
                                            };
                    
                                            html!{
                                                <PackageCard 
                            key={package.tracking.clone()}
                            package={package.clone()} 
                            index={group_idx} 
                                                    address={addr} 
                                                    on_info={props.on_info.clone()} 
                                                    is_selected={is_selected}
                            is_expanded={false}
                                                    on_select={Some(on_select_card)}
                                                    on_navigate={Some(on_navigate.clone())}
                            on_toggle_expand={None::<Callback<usize>>}
                            animation_class={animation_class}
                        />
                                }
                }
            }) }
        </div>
    }
}


