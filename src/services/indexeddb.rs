// ============================================================================
// INDEXEDDB SERVICE - PLACEHOLDER
// ============================================================================
// Por ahora usamos localStorage como fallback
// TODO: Implementar completamente IndexedDB cuando sea necesario
// ============================================================================

/// Servicio de IndexedDB para almacenamiento offline (placeholder)
pub struct IndexedDbService {
    db_name: String,
}

impl IndexedDbService {
    pub fn new() -> Self {
        Self {
            db_name: "route_optimizer_db".to_string(),
        }
    }
    
    /// Inicializar base de datos IndexedDB (placeholder)
    pub async fn init(&self) -> Result<(), String> {
        // Por ahora siempre falla - usar localStorage
        Err("IndexedDB not yet implemented - use localStorage".to_string())
    }
    
    /// Guardar objeto en IndexedDB (placeholder)
    pub async fn save(&self, _store_name: &str, _key: &str, _value: &serde_json::Value) -> Result<(), String> {
        Err("IndexedDB not yet implemented - use localStorage".to_string())
    }
    
    /// Cargar objeto desde IndexedDB (placeholder)
    pub async fn load(&self, _store_name: &str, _key: &str) -> Result<Option<serde_json::Value>, String> {
        Err("IndexedDB not yet implemented - use localStorage".to_string())
    }
}

impl Default for IndexedDbService {
    fn default() -> Self {
        Self::new()
    }
}
