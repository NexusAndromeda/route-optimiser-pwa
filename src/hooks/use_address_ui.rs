use yew::prelude::*;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct AddressUIState {
    pub selected_address_id: Option<String>,
    pub expanded_addresses: Vec<String>,
    pub reorder_mode: bool,
    pub reorder_origin: Option<String>,
    pub animations: HashMap<String, String>,
}

pub struct UseAddressUIHandle {
    pub state: UseStateHandle<AddressUIState>,
    pub select_address: Callback<String>,
    pub toggle_expand: Callback<String>,
    pub toggle_reorder_mode: Callback<()>,
    pub start_reorder: Callback<String>,
    pub end_reorder: Callback<()>,
    pub add_animation: Callback<(String, String)>,
    pub clear_animations: Callback<()>,
}

#[hook]
pub fn use_address_ui() -> UseAddressUIHandle {
    let state = use_state(|| AddressUIState {
        selected_address_id: None,
        expanded_addresses: Vec::new(),
        reorder_mode: false,
        reorder_origin: None,
        animations: HashMap::new(),
    });
    
    // Seleccionar dirección
    let select_address = {
        let state = state.clone();
        Callback::from(move |address_id: String| {
            let mut current_state = (*state).clone();
            current_state.selected_address_id = Some(address_id);
            state.set(current_state);
        })
    };
    
    // Toggle expandir/colapsar dirección
    let toggle_expand = {
        let state = state.clone();
        Callback::from(move |address_id: String| {
            let mut current_state = (*state).clone();
            if current_state.expanded_addresses.contains(&address_id) {
                current_state.expanded_addresses.retain(|id| id != &address_id);
            } else {
                current_state.expanded_addresses.push(address_id);
            }
            state.set(current_state);
        })
    };
    
    // Toggle modo reordenar
    let toggle_reorder_mode = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut current_state = (*state).clone();
            current_state.reorder_mode = !current_state.reorder_mode;
            if !current_state.reorder_mode {
                current_state.reorder_origin = None;
            }
            state.set(current_state);
        })
    };
    
    // Iniciar reordenamiento
    let start_reorder = {
        let state = state.clone();
        Callback::from(move |address_id: String| {
            let mut current_state = (*state).clone();
            current_state.reorder_origin = Some(address_id);
            state.set(current_state);
        })
    };
    
    // Terminar reordenamiento
    let end_reorder = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut current_state = (*state).clone();
            current_state.reorder_origin = None;
            state.set(current_state);
        })
    };
    
    // Agregar animación
    let add_animation = {
        let state = state.clone();
        Callback::from(move |(address_id, animation_class): (String, String)| {
            let mut current_state = (*state).clone();
            current_state.animations.insert(address_id.clone(), animation_class);
            state.set(current_state);
            
            // Limpiar animación después de un tiempo
            let state_clone = state.clone();
            let address_id_clone = address_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(1000).await;
                let mut current_state = (*state_clone).clone();
                current_state.animations.remove(&address_id_clone);
                state_clone.set(current_state);
            });
        })
    };
    
    // Limpiar todas las animaciones
    let clear_animations = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut current_state = (*state).clone();
            current_state.animations.clear();
            state.set(current_state);
        })
    };
    
    UseAddressUIHandle {
        state,
        select_address,
        toggle_expand,
        toggle_reorder_mode,
        start_reorder,
        end_reorder,
        add_animation,
        clear_animations,
    }
}
