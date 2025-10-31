use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SyncState {
    Synced,
    Pending { count: usize },
    Syncing,
    Offline {
        last_error: String,
        pending_count: usize,
    },
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Change {
    PackageScanned {
        tracking: String,
        timestamp: i64,
        new_status: String,
    },
    AddressUpdated {
        address_id: String,
        new_label: String,
        new_latitude: f64,
        new_longitude: f64,
        timestamp: i64,
    },
    OrderChanged {
        package_internal_id: String,
        old_position: usize,
        new_position: usize,
        timestamp: i64,
    },
    PackageDelivered {
        tracking: String,
        timestamp: i64,
        delivery_proof: Option<String>,
    },
    PackageFailed {
        tracking: String,
        timestamp: i64,
        reason: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub session_id: String,
    pub last_sync: i64,
    pub changes: Vec<Change>,
}

/// Response de sincronización del backend
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub success: bool,
    pub session: crate::models::session::DeliverySession,
    pub conflicts_resolved: usize,
    pub changes_applied: usize,
}

/// Resultado de sincronización (para frontend)
#[derive(Debug, Clone)]
pub enum SyncResult {
    /// Sincronización exitosa
    Success {
        session: crate::models::session::DeliverySession,
        changes_applied: usize,
    },
    /// No hay cambios
    NoChanges,
    /// Conflicto detectado y resuelto
    ConflictResolved {
        merged_session: crate::models::session::DeliverySession,
        conflicts_count: usize,
    },
    /// Error (modo offline activado)
    Error {
        message: String,
        pending_changes: Vec<Change>,
    },
}

// ============================================================================
// QUEUE PERSISTENTE DE CAMBIOS PARA MODO OFFLINE
// ============================================================================

/// Queue persistente de cambios para modo offline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingChangesQueue {
    pub changes: Vec<Change>,
    pub created_at: i64,
    pub retry_count: usize,
    pub last_retry: Option<i64>,
}

impl PendingChangesQueue {
    pub fn new(changes: Vec<Change>) -> Self {
        Self {
            changes,
            created_at: chrono::Utc::now().timestamp(),
            retry_count: 0,
            last_retry: None,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.changes.len()
    }
    
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
        self.last_retry = Some(chrono::Utc::now().timestamp());
    }
    
    /// Determinar si debemos reintentar basado en backoff exponencial
    pub fn should_retry(&self) -> bool {
        if self.retry_count == 0 {
            return true;
        }
        
        let last_retry = match self.last_retry {
            Some(ts) => ts,
            None => return true,
        };
        
        let now = chrono::Utc::now().timestamp();
        let time_since_retry = now - last_retry;
        
        // Backoff exponencial: 30s, 60s, 120s, 240s, max 300s (5 min)
        let backoff_seconds = std::cmp::min(30 * (2_i64.pow(self.retry_count as u32)), 300);
        
        time_since_retry >= backoff_seconds
    }
}

impl Change {
    pub fn timestamp(&self) -> i64 {
        match self {
            Change::PackageScanned { timestamp, .. } => *timestamp,
            Change::AddressUpdated { timestamp, .. } => *timestamp,
            Change::OrderChanged { timestamp, .. } => *timestamp,
            Change::PackageDelivered { timestamp, .. } => *timestamp,
            Change::PackageFailed { timestamp, .. } => *timestamp,
        }
    }
    
    /// Crea un cambio de paquete escaneado
    pub fn scanned(tracking: String, new_status: String) -> Self {
        Change::PackageScanned {
            tracking,
            timestamp: chrono::Utc::now().timestamp(),
            new_status,
        }
    }
    
    /// Crea un cambio de dirección actualizada
    pub fn address_updated(
        address_id: String,
        new_label: String,
        new_latitude: f64,
        new_longitude: f64,
    ) -> Self {
        Change::AddressUpdated {
            address_id,
            new_label,
            new_latitude,
            new_longitude,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

