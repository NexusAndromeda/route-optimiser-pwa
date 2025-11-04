// ============================================================================
// SYNC INDICATOR COMPONENT
// ============================================================================
// ✅ COPIADO DEL ORIGINAL - Preserva UI/UX exacta
// Solo cambiado para usar Yewdux en lugar de hooks
// ============================================================================

use yew::prelude::*;
use crate::hooks::use_sync_state;
use crate::models::sync::SyncState;
use crate::services::{SyncService, OfflineService};
use gloo_timers::callback::Timeout;
use wasm_bindgen_futures::spawn_local;

/// Componente SyncIndicator - ✅ HTML EXACTO DEL ORIGINAL
#[function_component(SyncIndicator)]
pub fn sync_indicator() -> Html {
    let sync_handle = use_sync_state();
    
    // Estado para mostrar notificación de conflictos
    let show_conflict_notification = use_state(|| false);
    let conflict_count = use_state(|| 0usize);
    
    // Verificar si hay conflictos resueltos recientes
    {
        let sync_handle_state = sync_handle.state.clone();
        let show_conflict_notification_clone = show_conflict_notification.clone();
        let conflict_count_clone = conflict_count.clone();
        
        // Re-ejecutar cuando cambie last_conflicts_resolved
        use_effect_with(sync_handle_state.last_conflicts_resolved.clone(), move |last_conflicts| {
            let show_conflict_notification = show_conflict_notification_clone.clone();
            let conflict_count = conflict_count_clone.clone();
            
            if let Some(count) = last_conflicts {
                if *count > 0 {
                    log::info!("⚠️ Mostrando notificación de {} conflictos resueltos", count);
                    conflict_count.set(*count);
                    show_conflict_notification.set(true);
                    
                    // Ocultar notificación después de 5 segundos
                    let show_conflict_notification_timeout = show_conflict_notification.clone();
                    Timeout::new(5_000, move || {
                        show_conflict_notification_timeout.set(false);
                    }).forget();
                }
            }
            || ()
        });
    }
    
    // ✅ HTML/CSS EXACTO DEL ORIGINAL
    let sync_store = (*sync_handle.state).clone();
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
        let sync_handle_state = sync_handle.state.clone();
        let update_conflicts = sync_handle.update_conflicts_resolved.clone();
        
        Callback::from(move |_| {
            let sync_handle_state = sync_handle_state.clone();
            let update_conflicts = update_conflicts.clone();
            
            // Actualizar estado a syncing
            let mut new_sync_state = (*sync_handle_state).clone();
            new_sync_state.sync_state = SyncState::Syncing;
            sync_handle_state.set(new_sync_state);
            
            spawn_local(async move {
                // Obtener sesión y cambios pendientes
                let offline_service = OfflineService::new();
                let sync_service = SyncService::new();
                
                if let Ok(Some(session)) = offline_service.load_session() {
                    let pending_changes = offline_service.load_pending_changes()
                        .ok()
                        .flatten()
                        .map(|q| q.changes)
                        .unwrap_or_default();
                    
                    // Ejecutar sync
                    let result = sync_service.sync_session(&session, pending_changes).await;
                    
                    // Actualizar estado según resultado
                    let sync_handle_state_for_update = sync_handle_state.clone();
                    let mut new_sync_state = (*sync_handle_state_for_update).clone();
                    match &result {
                        crate::models::sync::SyncResult::Success { changes_applied, .. } => {
                            new_sync_state.sync_state = SyncState::Synced;
                            log::info!("✅ Sync exitoso: {} cambios aplicados", changes_applied);
                        }
                        crate::models::sync::SyncResult::ConflictResolved { conflicts_count, .. } => {
                            new_sync_state.sync_state = SyncState::Synced;
                            // Actualizar conflictos resueltos
                            update_conflicts.emit(*conflicts_count);
                            log::warn!("⚠️ {} conflictos resueltos", conflicts_count);
                        }
                        crate::models::sync::SyncResult::Error { message, .. } => {
                            new_sync_state.sync_state = SyncState::Error { 
                                message: message.clone() 
                            };
                            log::error!("❌ Error en sync: {}", message);
                        }
                        crate::models::sync::SyncResult::NoChanges => {
                            new_sync_state.sync_state = SyncState::Synced;
                        }
                    }
                    sync_handle_state_for_update.set(new_sync_state);
                } else {
                    let sync_handle_state_for_error = sync_handle_state.clone();
                    let mut new_sync_state = (*sync_handle_state_for_error).clone();
                    new_sync_state.sync_state = SyncState::Error { 
                        message: "No hay sesión disponible".to_string() 
                    };
                    sync_handle_state_for_error.set(new_sync_state);
                }
            });
        })
    };
    
    html! {
        <>
            <div class={class} onclick={onclick.clone()} title="Click para sincronizar ahora">
                <span class="sync-icon">{icon}</span>
                <span class="sync-text">{text}</span>
            </div>
            {if *show_conflict_notification {
                html! {
                    <div class="conflict-notification" onclick={onclick} style="position: fixed; top: 20px; right: 20px; background: #ff9800; color: white; padding: 12px 20px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); z-index: 1000; cursor: pointer; animation: slideIn 0.3s ease-out;">
                        <span style="font-weight: bold;">{"⚠️ "}</span>
                        <span>{format!("{} conflictos detectados y resueltos automáticamente", *conflict_count)}</span>
                    </div>
                }
            } else {
                html! {}
            }}
        </>
    }
}
