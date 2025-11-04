// ============================================================================
// SYNC STORE - SIN YEWDUX (por compatibilidad con Rust 1.90)
// ============================================================================

use crate::models::sync::{Change, SyncState};

/// Estado de sincronización
#[derive(Clone, Debug, PartialEq)]
pub struct SyncStore {
    pub pending_changes: Vec<Change>,
    pub sync_state: SyncState,
    pub last_sync_attempt: Option<i64>,
    pub is_online: bool,
    /// Último número de conflictos resueltos (para mostrar notificación)
    pub last_conflicts_resolved: Option<usize>,
}

impl Default for SyncStore {
    fn default() -> Self {
        Self {
            pending_changes: Vec::new(),
            sync_state: SyncState::Synced,
            last_sync_attempt: None,
            is_online: true,
            last_conflicts_resolved: None,
        }
    }
}
