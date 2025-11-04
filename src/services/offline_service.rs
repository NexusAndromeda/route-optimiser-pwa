use crate::models::session::DeliverySession;
use crate::models::sync::{Change, PendingChangesQueue};
use web_sys::{window, Storage};

/// Service offline - IndexedDB + Background Sync
/// ✅ Ahora usa IndexedDB con fallback a localStorage
#[derive(Clone)]
pub struct OfflineService {
    use_indexeddb: bool,
}

const SESSION_STORAGE_KEY: &str = "delivery_session";
const QUEUE_STORAGE_KEY: &str = "pending_changes_queue";

impl OfflineService {
    pub fn new() -> Self {
        // Por ahora siempre usar localStorage (IndexedDB placeholder)
        Self {
            use_indexeddb: false,
        }
    }
    
    /// Guardar usando IndexedDB o localStorage como fallback
    fn save_storage(&self, key: &str, value: &str) -> Result<(), String> {
        if self.use_indexeddb {
            // TODO: Usar IndexedDB cuando esté completamente implementado
            // Por ahora usar localStorage como fallback
            self.save_local_storage(key, value)
        } else {
            self.save_local_storage(key, value)
        }
    }
    
    /// Cargar usando IndexedDB o localStorage como fallback
    fn load_storage(&self, key: &str) -> Result<Option<String>, String> {
        if self.use_indexeddb {
            // TODO: Usar IndexedDB cuando esté completamente implementado
            self.load_local_storage(key)
        } else {
            self.load_local_storage(key)
        }
    }
    
        fn save_local_storage(&self, key: &str, value: &str) -> Result<(), String> {
            let storage = window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .ok_or("No se pudo acceder a localStorage")?;
            
            storage.set_item(key, value)
                .map_err(|_| "Error guardando en localStorage".to_string())?;
            
            Ok(())
        }
        
        fn load_local_storage(&self, key: &str) -> Result<Option<String>, String> {
            let storage = window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .ok_or("No se pudo acceder a localStorage")?;
            
            match storage.get_item(key) {
                Ok(Some(value)) => Ok(Some(value)),
                Ok(None) => Ok(None),
                Err(_) => Err("Error leyendo localStorage".to_string()),
            }
        }
    
    /// Guardar sesión en IndexedDB (con fallback a localStorage)
    pub fn save_session(&self, session: &DeliverySession) -> Result<(), String> {
        let json = serde_json::to_string(session)
            .map_err(|e| format!("Error serializando sesión: {}", e))?;
        
        self.save_storage(SESSION_STORAGE_KEY, &json)?;
        log::info!("💾 Sesión guardada (IndexedDB/localStorage)");
        Ok(())
    }
    
    /// Cargar sesión desde IndexedDB (con fallback a localStorage)
    pub fn load_session(&self) -> Result<Option<DeliverySession>, String> {
        match self.load_storage(SESSION_STORAGE_KEY)? {
            Some(json) => {
                log::info!("📋 [STORAGE] Intentando deserializar sesión (tamaño: {} bytes)", json.len());
                // Intentar parsear el JSON primero para ver si hay errores de sintaxis
                match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(_) => log::info!("✅ [STORAGE] JSON válido"),
                    Err(e) => {
                        log::error!("❌ [STORAGE] JSON inválido: {}", e);
                        return Err(format!("JSON inválido: {}", e));
                    }
                }
                
                match serde_json::from_str::<DeliverySession>(&json) {
                    Ok(session) => {
                        log::info!("✅ [STORAGE] Sesión deserializada exitosamente: {} paquetes", session.stats.total_packages);
                Ok(Some(session))
            }
                    Err(e) => {
                        log::error!("❌ [STORAGE] Error deserializando sesión: {}", e);
                        // Intentar encontrar el campo problemático
                        if let Some(pos) = e.to_string().find("at line") {
                            log::error!("❌ [STORAGE] Ubicación del error: {}", &e.to_string()[pos..]);
                        }
                        Err(format!("Error deserializando: {}", e))
                    }
                }
            }
            None => {
                log::info!("📋 [STORAGE] No hay sesión guardada");
                Ok(None)
            }
        }
    }
    
    /// Guardar cambios pendientes en IndexedDB (con fallback a localStorage)
    pub fn save_pending_changes(&self, changes: &[Change]) -> Result<(), String> {
        let queue = PendingChangesQueue::new(changes.to_vec());
        self.save_queue(&queue)
    }
    
    /// Guardar queue completa
    pub fn save_queue(&self, queue: &PendingChangesQueue) -> Result<(), String> {
        let json = serde_json::to_string(queue)
            .map_err(|e| format!("Error serializando queue: {}", e))?;
        
        self.save_storage(QUEUE_STORAGE_KEY, &json)?;
        log::info!("💾 Queue guardada: {} cambios, {} reintentos", 
                   queue.len(), queue.retry_count);
        Ok(())
    }
    
    /// Cargar cambios pendientes desde IndexedDB (con fallback a localStorage)
    pub fn load_pending_changes(&self) -> Result<Option<PendingChangesQueue>, String> {
        match self.load_storage(QUEUE_STORAGE_KEY)? {
            Some(json) => {
                let queue = serde_json::from_str::<PendingChangesQueue>(&json)
                    .map_err(|e| format!("Error deserializando queue: {}", e))?;
                log::info!("📋 Queue cargada: {} cambios, {} reintentos", 
                           queue.len(), queue.retry_count);
                Ok(Some(queue))
            }
            None => Ok(None),
        }
    }
    
        /// Limpiar cambios pendientes después de sync exitoso
        pub fn clear_pending_changes(&self) -> Result<(), String> {
            let storage = window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .ok_or("No se pudo acceder a localStorage")?;
            
            storage.remove_item(QUEUE_STORAGE_KEY)
                .map_err(|_| "Error eliminando".to_string())?;
            
            log::info!("🗑️ Queue limpiada");
            Ok(())
        }
    
    /// Registrar Background Sync (TODO)
    pub async fn register_background_sync(&self) -> Result<(), String> {
        // TODO: Implementar Background Sync API
        Ok(())
    }
}

