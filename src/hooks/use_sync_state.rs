// ============================================================================
// USE SYNC STATE HOOK - Reemplazo de Yewdux Store
// ============================================================================

use yew::prelude::*;
use crate::stores::SyncStore;
use crate::viewmodels::SyncViewModel;
use crate::models::sync::{SyncResult, SyncState, Change};

#[derive(Clone)]
pub struct UseSyncStateHandle {
    pub state: UseStateHandle<SyncStore>,
    pub sync_now: Callback<()>,
    pub add_pending_change: Callback<Change>,
    pub update_conflicts_resolved: Callback<usize>,
}

#[hook]
pub fn use_sync_state() -> UseSyncStateHandle {
    let state = use_state(|| SyncStore::default());
    let _viewmodel = use_state(|| SyncViewModel::new());
    
    // Sync now
    let sync_now = {
        let _state = state.clone();
        // Necesitamos obtener la sesión actual - se hace en el hook que llama
        Callback::from(move |_| {
            log::info!("🔄 Sync manual iniciado desde hook");
            // La implementación real se hace donde tenemos acceso a la sesión
        })
    };
    
    // Add pending change
    let add_pending_change = {
        let state = state.clone();
        Callback::from(move |change: Change| {
            let state = state.clone();
            let vm = SyncViewModel::new(); // Crear nuevo
            let change_clone = change.clone();
            wasm_bindgen_futures::spawn_local(async move {
                vm.add_pending_change(change_clone).await;
                
                // Actualizar estado
                let mut new_state = (*state).clone();
                new_state.pending_changes.push(change);
                new_state.sync_state = SyncState::Pending {
                    count: new_state.pending_changes.len(),
                };
                state.set(new_state);
            });
        })
    };
    
    // Update conflicts resolved
    let update_conflicts_resolved = {
        let state = state.clone();
        Callback::from(move |count: usize| {
            let mut new_state = (*state).clone();
            new_state.last_conflicts_resolved = Some(count);
            state.set(new_state);
        })
    };
    
    UseSyncStateHandle {
        state,
        sync_now,
        add_pending_change,
        update_conflicts_resolved,
    }
}
