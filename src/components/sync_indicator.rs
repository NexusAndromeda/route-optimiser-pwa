// ============================================================================
// SYNC INDICATOR COMPONENT
// ============================================================================
// ✅ COPIADO DEL ORIGINAL - Preserva UI/UX exacta
// Solo cambiado para usar Yewdux en lugar de hooks
// ============================================================================

use yew::prelude::*;
use crate::hooks::use_sync_state;
use crate::models::sync::SyncState;
use crate::viewmodels::SyncViewModel;

/// Componente SyncIndicator - ✅ HTML EXACTO DEL ORIGINAL
#[function_component(SyncIndicator)]
pub fn sync_indicator() -> Html {
    let sync_handle = use_sync_state();
    let sync_store = sync_handle.state;
    
    // ✅ HTML/CSS EXACTO DEL ORIGINAL
    let (icon, text, class) = match &sync_store.sync_state {
        SyncState::Synced => {
            ("✅", "Sincronizado".to_string(), "sync-indicator synced")
        }
        SyncState::Pending { count } => {
            ("🔄", format!("{} cambios pendientes", count), "sync-indicator pending")
        }
        SyncState::Syncing => {
            ("⏳", "Sincronizando...".to_string(), "sync-indicator syncing")
        }
        SyncState::Offline { pending_count, .. } => {
            ("📴", format!("Offline - {} pendientes", pending_count), "sync-indicator offline")
        }
        SyncState::Error { message } => {
            ("⚠️", format!("Error: {}", message), "sync-indicator error")
        }
    };
    
    let onclick = {
        let sync_now = sync_handle.sync_now.clone();
        Callback::from(move |_| {
            sync_now.emit(());
        })
    };
    
    html! {
        <div class={class} onclick={onclick} title="Click para sincronizar ahora">
            <span class="sync-icon">{icon}</span>
            <span class="sync-text">{text}</span>
        </div>
    }
}
