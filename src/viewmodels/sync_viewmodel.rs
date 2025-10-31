// ============================================================================
// SYNC VIEWMODEL - LÓGICA DE SINCRONIZACIÓN
// ============================================================================
// Lógica de negocio de sincronización - SIN yewdux
// Devuelve valores, los hooks actualizan el estado
// ============================================================================

use crate::services::{SyncService, NetworkMonitor};
use crate::models::sync::{SyncResult, SyncState, Change};
use crate::services::OfflineService;
use crate::models::session::DeliverySession;

/// ViewModel de sincronización - SOLO lógica de negocio
pub struct SyncViewModel {
    sync_service: SyncService,
    network_monitor: NetworkMonitor,
    offline_service: OfflineService,
}

impl SyncViewModel {
    pub fn new() -> Self {
        Self {
            sync_service: SyncService::new(),
            network_monitor: NetworkMonitor::new(),
            offline_service: OfflineService::new(),
        }
    }
    
    /// Sincronizar ahora - necesita sesión y cambios pendientes
    pub async fn sync_now(
        &self,
        session: &DeliverySession,
        pending_changes: Vec<Change>,
    ) -> Result<SyncResult, String> {
        log::info!("🔄 Iniciando sincronización manual");
        
        // Ejecutar sincronización
        let result = self.sync_service.sync_session(session, pending_changes).await;
        
        // Guardar sesión si es exitoso
        match &result {
            SyncResult::Success { session: updated_session, .. } => {
                if let Err(e) = self.offline_service.save_session(updated_session) {
                    log::error!("❌ Error guardando sesión: {}", e);
                }
            }
            SyncResult::ConflictResolved { merged_session, .. } => {
                if let Err(e) = self.offline_service.save_session(merged_session) {
                    log::error!("❌ Error guardando sesión: {}", e);
                }
            }
            _ => {}
        }
        
        Ok(result)
    }
    
    /// Agregar cambio pendiente
    pub async fn add_pending_change(&self, change: Change) {
        log::info!("📝 Cambio agregado: {:?}", change);
        
        // Guardar en queue persistente
        if let Ok(Some(mut queue)) = self.offline_service.load_pending_changes() {
            queue.changes.push(change);
            let _ = self.offline_service.save_pending_changes(&queue.changes);
        } else {
            let changes = vec![change];
            let _ = self.offline_service.save_pending_changes(&changes);
        }
    }
    
    /// Iniciar auto-sync (helper para hooks)
    pub fn start_auto_sync(&self) {
        log::info!("⏰ Auto-sync iniciado (manejado por hook)");
        // La lógica de auto-sync se maneja en use_sync_state hook
    }
    
    /// Iniciar monitoreo de red (helper para hooks)
    pub fn start_network_monitor(&mut self) {
        log::info!("🌐 Network monitor iniciado (manejado por hook)");
        // La lógica de monitoreo se maneja en use_sync_state hook
    }
    
    /// Cargar queue persistente al iniciar
    pub async fn load_pending_queue(&self) -> Result<Option<crate::models::sync::PendingChangesQueue>, String> {
        self.offline_service.load_pending_changes()
    }
    
    /// Verificar si está online
    pub fn is_online(&self) -> bool {
        self.network_monitor.is_online()
    }
}

// SyncViewModel no es Clone porque contiene servicios no-Clone
// Usar new() directamente donde sea necesario

impl Default for SyncViewModel {
    fn default() -> Self {
        Self::new()
    }
}
