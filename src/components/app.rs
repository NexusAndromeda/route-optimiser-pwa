use yew::prelude::*;
use crate::models::Package;
use super::{Header, MapContainer, PackageList, DetailsModal, BalModal, SettingsPopup};
use gloo_timers::callback::Timeout;
use std::collections::HashMap;

pub enum SheetState {
    Collapsed,
    Half,
    Full,
}

#[function_component(App)]
pub fn app() -> Html {
    let packages = use_state(|| Package::demo_packages());
    let selected_index = use_state(|| None::<usize>);
    let sheet_state = use_state(|| SheetState::Collapsed);
    let show_details = use_state(|| false);
    let details_package_index = use_state(|| None::<usize>);
    let show_bal_modal = use_state(|| false);
    let show_settings = use_state(|| false);
    let animations = use_state(|| HashMap::<usize, String>::new());
    
    // Calculate stats
    let total = packages.len();
    let delivered = packages.iter().filter(|p| p.status == "delivered").count();
    let percentage = if total > 0 { (delivered * 100) / total } else { 0 };
    
    // Toggle bottom sheet
    let toggle_sheet = {
        let sheet_state = sheet_state.clone();
        Callback::from(move |_: MouseEvent| {
            let new_state = match *sheet_state {
                SheetState::Collapsed => SheetState::Half,
                SheetState::Half => SheetState::Full,
                SheetState::Full => SheetState::Collapsed,
            };
            sheet_state.set(new_state);
        })
    };
    
    // Close on backdrop click
    let close_sheet = {
        let sheet_state = sheet_state.clone();
        Callback::from(move |_: MouseEvent| {
            sheet_state.set(SheetState::Collapsed);
        })
    };
    
    // Select package
    let on_select = {
        let selected_index = selected_index.clone();
        Callback::from(move |index: usize| {
            selected_index.set(Some(index));
        })
    };
    
    // Show details
    let on_show_details = {
        let show_details = show_details.clone();
        let details_package_index = details_package_index.clone();
        Callback::from(move |index: usize| {
            details_package_index.set(Some(index));
            show_details.set(true);
        })
    };
    
    // Navigate
    let on_navigate = Callback::from(move |index: usize| {
        log::info!("🧭 Navigate to package {}", index);
        // TODO: Open Waze/Maps
    });
    
    // Reorder con animaciones
    let on_reorder = {
        let packages = packages.clone();
        let animations = animations.clone();
        let selected_index = selected_index.clone();
        
        Callback::from(move |(index, direction): (usize, String)| {
            let pkgs = (*packages).clone();
            let mut anims = (*animations).clone();
            
            match direction.as_str() {
                "up" if index > 0 => {
                    // Aplicar animaciones
                    anims.insert(index, "moving-up".to_string());
                    anims.insert(index - 1, "moving-down".to_string());
                    animations.set(anims.clone());
                    
                    // Intercambiar paquetes después de 150ms
                    let packages_clone = packages.clone();
                    let animations_clone = animations.clone();
                    let selected_clone = selected_index.clone();
                    
                    Timeout::new(150, move || {
                        let mut pkgs = (*packages_clone).clone();
                        pkgs.swap(index, index - 1);
                        packages_clone.set(pkgs);
                        
                        // Flash effect después de mover
                        let animations_clone2 = animations_clone.clone();
                        Timeout::new(50, move || {
                            let mut anims = HashMap::new();
                            anims.insert(index - 1, "moved".to_string());
                            animations_clone2.set(anims.clone());
                            
                            // Limpiar animación después de 500ms
                            let animations_clone3 = animations_clone2.clone();
                            Timeout::new(500, move || {
                                animations_clone3.set(HashMap::new());
                            }).forget();
                        }).forget();
                        
                        // Actualizar índice seleccionado
                        selected_clone.set(Some(index - 1));
                    }).forget();
                },
                "down" if index < pkgs.len() - 1 => {
                    // Aplicar animaciones
                    anims.insert(index, "moving-down".to_string());
                    anims.insert(index + 1, "moving-up".to_string());
                    animations.set(anims.clone());
                    
                    // Intercambiar paquetes después de 150ms
                    let packages_clone = packages.clone();
                    let animations_clone = animations.clone();
                    let selected_clone = selected_index.clone();
                    
                    Timeout::new(150, move || {
                        let mut pkgs = (*packages_clone).clone();
                        pkgs.swap(index, index + 1);
                        packages_clone.set(pkgs);
                        
                        // Flash effect después de mover
                        let animations_clone2 = animations_clone.clone();
                        Timeout::new(50, move || {
                            let mut anims = HashMap::new();
                            anims.insert(index + 1, "moved".to_string());
                            animations_clone2.set(anims.clone());
                            
                            // Limpiar animación después de 500ms
                            let animations_clone3 = animations_clone2.clone();
                            Timeout::new(500, move || {
                                animations_clone3.set(HashMap::new());
                            }).forget();
                        }).forget();
                        
                        // Actualizar índice seleccionado
                        selected_clone.set(Some(index + 1));
                    }).forget();
                },
                _ => {}
            }
        })
    };
    
    // Toggle settings
    let toggle_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_: MouseEvent| {
            let current = *show_settings;
            show_settings.set(!current);
        })
    };
    
    let sheet_class = match *sheet_state {
        SheetState::Collapsed => "bottom-sheet collapsed",
        SheetState::Half => "bottom-sheet half",
        SheetState::Full => "bottom-sheet full",
    };
    
    let backdrop_class = if matches!(*sheet_state, SheetState::Collapsed) {
        "backdrop"
    } else {
        "backdrop active"
    };
    
    html! {
        <>
            <Header show_settings={*show_settings} on_toggle_settings={toggle_settings.clone()} />
            
            <MapContainer />
            
            // Backdrop
            <div class={backdrop_class} onclick={close_sheet}></div>
            
            // Package List (Bottom Sheet / Sidebar)
            <div class={sheet_class}>
                <PackageList
                    packages={(*packages).clone()}
                    selected_index={*selected_index}
                    delivered={delivered}
                    total={total}
                    percentage={percentage}
                    sheet_state={match *sheet_state {
                        SheetState::Collapsed => "collapsed",
                        SheetState::Half => "half",
                        SheetState::Full => "full",
                    }}
                    on_toggle={toggle_sheet}
                    on_select={on_select}
                    on_show_details={on_show_details}
                    on_navigate={on_navigate}
                    on_reorder={on_reorder}
                    animations={(*animations).clone()}
                />
            </div>
            
            // Modals
            {
                if *show_details && details_package_index.is_some() {
                    let index = (*details_package_index).unwrap();
                    if let Some(package) = packages.get(index) {
                        let show_bal = show_bal_modal.clone();
                        let show_det = show_details.clone();
                        Some(html! {
                            <DetailsModal
                                package={package.clone()}
                                on_close={Callback::from(move |_| show_det.set(false))}
                                on_edit_bal={Callback::from(move |_| show_bal.set(true))}
                            />
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            
            {
                if *show_bal_modal {
                    let show_bal = show_bal_modal.clone();
                    let show_bal2 = show_bal_modal.clone();
                    Some(html! {
                        <BalModal
                            on_close={Callback::from(move |_| show_bal.set(false))}
                            on_select={Callback::from(move |has_access: bool| {
                                log::info!("📬 BAL: {}", has_access);
                                show_bal2.set(false);
                            })}
                        />
                    })
                } else {
                    None
                }
            }
            
            {
                if *show_settings {
                    let show_set = show_settings.clone();
                    Some(html! {
                        <SettingsPopup
                            on_close={Callback::from(move |_| show_set.set(false))}
                            on_logout={Callback::from(move |_| {
                                log::info!("🚪 Logout");
                            })}
                        />
                    })
                } else {
                    None
                }
            }
        </>
    }
}

