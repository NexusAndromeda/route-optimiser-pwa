// ============================================================================
// SERVICIO DE SINCRONIZACIÓN INTELIGENTE CON QUEUE PERSISTENTE
// ============================================================================
// ✅ COPIADO DEL ORIGINAL (app/src/services/sync_service.rs)
// Adaptado para usar OfflineService (IndexedDB) en lugar de localStorage
// ============================================================================

use gloo_net::http::Request;
use crate::models::session::DeliverySession;
use crate::models::sync::{Change, SyncRequest, SyncResponse, SyncResult, PendingChangesQueue};
use crate::services::{OfflineService, ApiClient, NetworkMonitor};
use crate::utils::constants::BACKEND_URL;
use wasm_bindgen_futures::spawn_local;

/// Servicio de sincronización inteligente con queue persistente
#[derive(Clone)]
pub struct SyncService {
    backend_url: String,
    offline_service: OfflineService,
    api_client: ApiClient,
}

impl SyncService {
    pub fn new() -> Self {
        Self {
            backend_url: BACKEND_URL.to_string(),
            offline_service: OfflineService::new(),
            api_client: ApiClient::new(),
        }
    }
    
    /// Sincronizar sesión con backend
    pub async fn sync_session(
        &self,
        local_session: &DeliverySession,
        changes: Vec<Change>,
    ) -> SyncResult {
        log::info!("🔄 Iniciando sincronización: {} cambios pendientes", changes.len());
        
        // Si no hay cambios, solo pull
        if changes.is_empty() {
            return self.pull_remote_changes(local_session).await;
        }
        
        // Guardar cambios en queue persistente antes de enviar
        if let Err(e) = self.offline_service.save_pending_changes(&changes) {
            log::error!("❌ Error guardando queue: {}", e);
        }
        
        // Push cambios locales
        match self.push_local_changes(local_session, changes.clone()).await {
            Ok(response) => {
                // Sincronización exitosa → limpiar queue
                let _ = self.offline_service.clear_pending_changes();
                
                if response.conflicts_resolved > 0 {
                    log::warn!("⚠️ {} conflictos resueltos por el backend", 
                              response.conflicts_resolved);
                    SyncResult::ConflictResolved {
                        merged_session: response.session,
                        conflicts_count: response.conflicts_resolved,
                    }
                } else {
                    log::info!("✅ Sincronización exitosa: {} cambios aplicados", 
                              response.changes_applied);
                    SyncResult::Success {
                        session: response.session,
                        changes_applied: response.changes_applied,
                    }
                }
            }
            Err(e) => {
                log::error!("❌ Error en sincronización: {}", e);
                
                // Incrementar contador de reintentos
                if let Ok(Some(mut queue)) = self.offline_service.load_pending_changes() {
                    queue.increment_retry();
                    let _ = self.offline_service.save_queue(&queue);
                }
                
                SyncResult::Error {
                    message: e,
                    pending_changes: changes,
                }
            }
        }
    }
    
    /// Push cambios locales al backend
    async fn push_local_changes(
        &self,
        session: &DeliverySession,
        changes: Vec<Change>,
    ) -> Result<SyncResponse, String> {
        let url = format!("{}/api/v1/sessions/{}/sync", self.backend_url, session.session_id);
        
        let request = SyncRequest {
            session_id: session.session_id.clone(),
            last_sync: session.last_sync,
            changes,
        };
        
        log::info!("📤 Enviando {} cambios al backend", request.changes.len());
        
        let response = Request::post(&url)
            .json(&request)
            .map_err(|e| format!("Request build error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        
        if !response.ok() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("HTTP {}: {}", status, error_text));
        }
        
        let sync_response = response
            .json::<SyncResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        
        Ok(sync_response)
    }
    
    /// Pull cambios remotos del backend (cuando no hay cambios locales)
    async fn pull_remote_changes(&self, local_session: &DeliverySession) -> SyncResult {
        let url = format!("{}/api/v1/sessions/{}", self.backend_url, local_session.session_id);
        
        log::info!("📥 Verificando cambios remotos...");
        
        match Request::get(&url).send().await {
            Ok(response) => {
                if !response.ok() {
                    return SyncResult::Error {
                        message: format!("HTTP error: {}", response.status()),
                        pending_changes: Vec::new(),
                    };
                }
                
                match response.json::<DeliverySession>().await {
                    Ok(remote_session) => {
                        // Comparar timestamps
                        if remote_session.last_sync > local_session.last_sync {
                            log::info!("📥 Cambios remotos detectados (remote: {}, local: {})", 
                                      remote_session.last_sync, local_session.last_sync);
                            
                            SyncResult::Success {
                                session: remote_session,
                                changes_applied: 0, // Pull no cuenta como "cambios aplicados"
                            }
                        } else {
                            log::info!("✅ Sin cambios remotos");
                            SyncResult::NoChanges
                        }
                    }
                    Err(e) => {
                        SyncResult::Error {
                            message: format!("Parse error: {}", e),
                            pending_changes: Vec::new(),
                        }
                    }
                }
            }
            Err(e) => {
                SyncResult::Error {
                    message: format!("Network error: {}", e),
                    pending_changes: Vec::new(),
                }
            }
        }
    }
    
    // ==========================================
    // MÉTODOS DE QUEUE PERSISTENTE (delegados a OfflineService)
    // ==========================================
    
    /// Guardar cambios pendientes en IndexedDB
    pub fn save_pending_changes(&self, changes: &[Change]) -> Result<(), String> {
        self.offline_service.save_pending_changes(changes)
    }
    
    /// Cargar cambios pendientes desde IndexedDB
    pub fn load_pending_changes(&self) -> Result<Option<PendingChangesQueue>, String> {
        self.offline_service.load_pending_changes()
    }
    
    /// Limpiar cambios pendientes después de sync exitoso
    pub fn clear_pending_changes(&self) {
        let _ = self.offline_service.clear_pending_changes();
        log::info!("🗑️ Queue limpiada");
    }
    
    /// Verificar si hay cambios pendientes
    pub fn has_pending_changes(&self) -> bool {
        self.load_pending_changes()
            .ok()
            .flatten()
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }
    
    /// Obtener número de cambios pendientes
    pub fn pending_changes_count(&self) -> usize {
        self.load_pending_changes()
            .ok()
            .flatten()
            .map(|q| q.len())
            .unwrap_or(0)
    }
    
    /// Procesar queue pendiente automáticamente
    pub async fn process_pending_queue(&self) -> Result<(), String> {
        // 1. Cargar queue
        let mut queue = match self.offline_service.load_pending_changes()? {
            Some(q) => q,
            None => {
                log::info!("📭 No hay cambios pendientes");
                return Ok(());
            }
        };
        
        // 2. Verificar backoff exponencial
        if !queue.should_retry() {
            let remaining = queue.backoff_remaining();
            log::info!("⏳ Esperando backoff: {}s restantes", remaining);
            return Ok(());
        }
        
        // 3. Cargar sesión actual
        let session = self.offline_service.load_session()?
            .ok_or_else(|| "No hay sesión local".to_string())?;
        
        log::info!("🔄 Procesando queue: {} cambios pendientes (intento {})", 
            queue.len(), queue.retry_count + 1);
        
        // 4. Sincronizar
        let changes = queue.changes.clone();
        match self.sync_session(&session, changes).await {
            SyncResult::Success { .. } | SyncResult::ConflictResolved { .. } => {
                self.clear_pending_changes();
                log::info!("✅ Queue procesada exitosamente");
                Ok(())
            }
            SyncResult::Error { message, .. } => {
                queue.increment_retry();
                if let Err(e) = self.offline_service.save_queue(&queue) {
                    log::error!("❌ Error guardando queue actualizada: {}", e);
                }
                log::warn!("⚠️ Error en sync, reintentando más tarde: {}", message);
                Ok(()) // No retornar error, solo loguear
            }
            SyncResult::NoChanges => {
                self.clear_pending_changes();
                log::info!("✅ No hay cambios que sincronizar");
                Ok(())
            }
        }
    }
    
    /// Iniciar sincronización automática cuando vuelve la conexión
    pub fn start_auto_sync(&mut self) {
        let mut network_monitor = NetworkMonitor::new();
        let sync_service = self.clone();
        
        network_monitor.on_online(move || {
            let sync_service = sync_service.clone();
            spawn_local(async move {
                log::info!("🌐 Conexión restaurada - procesando queue automáticamente");
                if let Err(e) = sync_service.process_pending_queue().await {
                    log::error!("❌ Error procesando queue automática: {}", e);
                }
            });
        });
        
        log::info!("🚀 Auto-sync iniciado - procesará queue cuando vuelva la conexión");
    }
}

impl Default for SyncService {
    fn default() -> Self {
        Self::new()
    }
}
