use yew::prelude::*;
use web_sys::MouseEvent;

#[derive(Clone, Copy, PartialEq)]
pub enum SheetState {
    Collapsed,
    Half,
    Full,
}

pub struct UseSheetHandle {
    pub state: UseStateHandle<SheetState>,
    pub toggle: Callback<MouseEvent>,
    pub close: Callback<MouseEvent>,
    pub set_half: Callback<()>,
}

#[hook]
pub fn use_sheet() -> UseSheetHandle {
    let state = use_state(|| SheetState::Collapsed);
    
    // Toggle sheet
    let toggle = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            let new_state = match *state {
                SheetState::Collapsed => SheetState::Half,
                SheetState::Half => SheetState::Full,
                SheetState::Full => SheetState::Collapsed,
            };
            state.set(new_state);
            update_backdrop(new_state);
        })
    };
    
    // Close sheet
    let close = {
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            state.set(SheetState::Collapsed);
            update_backdrop(SheetState::Collapsed);
        })
    };
    
    // Set to half
    let set_half = {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(SheetState::Half);
            update_backdrop(SheetState::Half);
        })
    };
    
    UseSheetHandle {
        state,
        toggle,
        close,
        set_half,
    }
}

fn update_backdrop(new_state: SheetState) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(backdrop) = document.get_element_by_id("backdrop") {
                match new_state {
                    SheetState::Collapsed => {
                        let _ = backdrop.class_list().remove_1("active");
                    }
                    SheetState::Half | SheetState::Full => {
                        let _ = backdrop.class_list().add_1("active");
                    }
                }
            }
        }
    }
}

impl SheetState {
    pub fn to_class(&self) -> &'static str {
        match self {
            SheetState::Collapsed => "bottom-sheet collapsed",
            SheetState::Half => "bottom-sheet half",
            SheetState::Full => "bottom-sheet full",
        }
    }
    
    pub fn to_str(&self) -> &'static str {
        match self {
            SheetState::Collapsed => "collapsed",
            SheetState::Half => "half",
            SheetState::Full => "full",
        }
    }
}

