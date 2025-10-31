use serde::{Deserialize, Serialize};

// ============================================================================
// PACKAGE - SIMPLIFICADO (Sin CRDT, sin internal_id)
// ============================================================================

/// Paquete individual - ESTRUCTURA SIMPLIFICADA
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Package {
    // ========== ID ÚNICO ==========
    /// Número de tracking de Colis Privé (ES EL ID)
    pub tracking: String,
    
    /// ID de la dirección asociada
    pub address_id: String,
    
    // ========== LAST-WRITE-WINS ==========
    pub last_modified_at: i64,
    
    // ========== ORDEN Y POSICIÓN ==========
    /// Orden ORIGINAL (sin optimizar) - posición en la que llegó
    pub original_order: usize,
    
    /// Orden OPTIMIZADO - posición después de optimizar
    /// None = no optimizado aún
    pub route_order: Option<usize>,
    
    /// Posición visual en la lista (para reordenamiento manual)
    pub visual_position: usize,
    
    // ========== INFO DEL CLIENTE ==========
    pub customer_name: String,
    pub phone_number: Option<String>,
    pub customer_indication: Option<String>,
    
    // ========== ESTADO Y TIPO ==========
    pub status: String,
    pub delivery_type: DeliveryType,
    pub is_problematic: bool,
    pub optimization_priority: u8,
    
    // ========== FLAGS DE ESTADO ==========
    /// ¿Fue modificado por el conductor?
    pub modified_by_driver: bool,
    
    // ========== AGRUPAMIENTO ==========
    /// Indica si este Package representa un grupo de paquetes (para misma dirección)
    #[serde(default)]
    pub is_group: bool,
    
    /// Paquetes internos si es un grupo (None si es paquete simple)
    #[serde(default)]
    pub group_packages: Option<Vec<Package>>,
}

impl Package {
    /// Helper: ¿Está entregado?
    pub fn is_delivered(&self) -> bool {
        self.status.contains("LIVRE")
    }
    
    /// Helper: ¿Falló la entrega?
    pub fn is_failed(&self) -> bool {
        self.status.contains("NONLIV") || self.status.contains("ECHEC")
    }
    
    /// Helper: ¿Está pendiente?
    pub fn is_pending(&self) -> bool {
        !self.is_delivered() && !self.is_failed()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryType {
    #[serde(rename = "DOMICILE")]
    Home,
    #[serde(rename = "RCS")]
    Rcs,
    #[serde(rename = "RELAIS")]
    PickupPoint,
}
