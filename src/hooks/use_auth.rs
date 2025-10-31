// ============================================================================
// USE AUTH HOOK - Reemplazo de Yewdux AuthStore
// ============================================================================
// Hook nativo de Yew para manejar autenticación
// Compatible con Rust 1.90 (sin yewdux)
// ============================================================================

use yew::prelude::*;
use crate::stores::AuthStore;
use crate::models::company::Company;

#[derive(Clone)]
pub struct UseAuthHandle {
    pub state: UseStateHandle<AuthStore>,
    pub logout: Callback<()>,
    pub on_show_companies: Callback<()>,
    pub on_select_company: Callback<Company>,
    pub show_company_modal: bool,
}

#[hook]
pub fn use_auth() -> UseAuthHandle {
    let state = use_state(|| AuthStore::default());
    let show_company_modal = use_state(|| false);
    
    // Show companies modal
    let on_show_companies = {
        let show_company_modal = show_company_modal.clone();
        Callback::from(move |_| {
            show_company_modal.set(true);
        })
    };
    
    // Select company
    let on_select_company = {
        let show_company_modal = show_company_modal.clone();
        Callback::from(move |_company: Company| {
            show_company_modal.set(false);
        })
    };
    
    // Logout
    let logout = {
        let state = state.clone();
        Callback::from(move |_| {
            state.set(AuthStore::default());
        })
    };
    
    UseAuthHandle {
        state,
        logout,
        on_show_companies,
        on_select_company,
        show_company_modal: *show_company_modal,
    }
}

