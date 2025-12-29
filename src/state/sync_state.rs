// ============================================================================
// SYNC STATE - Estado de sincronización (reemplaza SyncStore)
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;
use crate::models::sync::{Change, SyncState};

/// Estado de sincronización
#[derive(Clone)]
pub struct SyncStateWrapper {
    pub pending_changes: Rc<RefCell<Vec<Change>>>,
    pub sync_state: Rc<RefCell<SyncState>>,
    pub last_sync_attempt: Rc<RefCell<Option<i64>>>,
    pub is_online: Rc<RefCell<bool>>,
    pub last_conflicts_resolved: Rc<RefCell<Option<usize>>>,
}

impl SyncStateWrapper {
    /// Crear nuevo estado de sincronización
    pub fn new() -> Self {
        Self {
            pending_changes: Rc::new(RefCell::new(Vec::new())),
            sync_state: Rc::new(RefCell::new(SyncState::Synced)),
            last_sync_attempt: Rc::new(RefCell::new(None)),
            is_online: Rc::new(RefCell::new(true)),
            last_conflicts_resolved: Rc::new(RefCell::new(None)),
        }
    }
    
    /// Obtener pending changes
    pub fn get_pending_changes(&self) -> Vec<Change> {
        self.pending_changes.borrow().clone()
    }
    
    /// Agregar pending change
    pub fn add_pending_change(&self, change: Change) {
        self.pending_changes.borrow_mut().push(change);
    }
    
    /// Limpiar pending changes
    pub fn clear_pending_changes(&self) {
        self.pending_changes.borrow_mut().clear();
    }
    
    /// Establecer sync state
    pub fn set_sync_state(&self, state: SyncState) {
        *self.sync_state.borrow_mut() = state;
    }
    
    /// Obtener sync state
    pub fn get_sync_state(&self) -> SyncState {
        self.sync_state.borrow().clone()
    }
    
    /// Establecer is_online
    pub fn set_online(&self, online: bool) {
        *self.is_online.borrow_mut() = online;
    }
    
    /// Obtener is_online
    pub fn get_online(&self) -> bool {
        *self.is_online.borrow()
    }
    
    /// Establecer last_sync_attempt
    pub fn set_last_sync_attempt(&self, time: Option<i64>) {
        *self.last_sync_attempt.borrow_mut() = time;
    }
    
    /// Obtener last_sync_attempt
    pub fn get_last_sync_attempt(&self) -> Option<i64> {
        *self.last_sync_attempt.borrow()
    }
    
    /// Establecer last_conflicts_resolved
    pub fn set_last_conflicts_resolved(&self, count: Option<usize>) {
        *self.last_conflicts_resolved.borrow_mut() = count;
    }
    
    /// Obtener last_conflicts_resolved
    pub fn get_last_conflicts_resolved(&self) -> Option<usize> {
        *self.last_conflicts_resolved.borrow()
    }
}

impl Default for SyncStateWrapper {
    fn default() -> Self {
        Self::new()
    }
}

