// ============================================================================
// SYNC INDICATOR VIEW - Indicador de estado de sincronización
// ============================================================================

// ============================================================================
// SYNC INDICATOR VIEW - Indicador de estado de sincronización
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::Element;
use crate::dom::{ElementBuilder, add_class};
use crate::state::app_state::AppState;
use crate::models::sync::SyncState;

/// Renderizar indicador de sincronización
/// Retorna None cuando está Synced (no mostrar nada)
pub fn render_sync_indicator(state: &AppState) -> Result<Option<Element>, JsValue> {
    let sync_state = state.sync.get_sync_state();
    
    // Si está sincronizado, no mostrar nada
    if matches!(sync_state, SyncState::Synced) {
        return Ok(None);
    }
    
    // Container principal
    let indicator = ElementBuilder::new("div")?
        .class("sync-indicator")
        .build();
    
    // Icono y texto según estado
    let (text_content, is_error_state) = match sync_state {
        SyncState::Synced => {
            // Ya manejado arriba, pero necesario para el match
            unreachable!()
        }
        SyncState::Syncing => {
            ("⏳ Syncing...".to_string(), false)
        }
        SyncState::Pending { count } => {
            (format!("⏳ Pending ({})", count), false)
        }
        SyncState::Offline { pending_count, .. } => {
            let text = if pending_count > 0 {
                format!("📴 Offline ({} pending)", pending_count)
            } else {
                "📴 Offline".to_string()
            };
            (text, true)
        }
        SyncState::Error { message } => {
            (format!("❌ Error: {}", message), true)
        }
    };
    
    // Agregar contenido
    crate::dom::set_text_content(&indicator, &text_content);
    
    // Aplicar clase de error para estados offline/error
    if is_error_state {
        add_class(&indicator, "sync-indicator--error")?;
    }
    
    Ok(Some(indicator))
}
