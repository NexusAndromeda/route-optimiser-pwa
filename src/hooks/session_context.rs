// ============================================================================
// SESSION CONTEXT - Compartir estado de sesión entre componentes
// ============================================================================
// Usa Context API de Yew para compartir SessionStore globalmente
// ============================================================================

use yew::prelude::*;
use crate::hooks::use_session::{use_session, UseSessionHandle};

/// Provider component que envuelve la app y proporciona el estado de sesión
#[function_component(SessionContextProvider)]
pub fn session_context_provider(props: &SessionContextProviderProps) -> Html {
    // Crear el estado de sesión (esto se ejecutará una sola vez en el provider)
    // Como no hay Context aún, use_session() creará un estado local
    let session_handle = use_session();
    
    html! {
        <ContextProvider<UseSessionHandle> context={session_handle}>
            {props.children.clone()}
        </ContextProvider<UseSessionHandle>>
    }
}

#[derive(Properties, PartialEq)]
pub struct SessionContextProviderProps {
    pub children: Children,
}

