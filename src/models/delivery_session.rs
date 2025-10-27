use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use yew::html::ImplicitClone;

// ============================================================================
// MODELOS PRINCIPALES - IDÉNTICOS A idea.rs
// ============================================================================

/// Estructura PRINCIPAL que contiene TODO
/// Esta es la que se guarda en:
/// - Backend: Redis Cache
/// - Frontend: LocalStorage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeliverySession {
    /// ID único de la sesión (generado al login)
    pub session_id: String,
    
    /// Timestamp del último fetch de Colis Privé
    pub last_fetch: i64,
    
    /// Timestamp de última sincronización
    pub last_sync: i64,
    
    /// Timestamp de última optimización
    pub last_optimization: Option<i64>,
    
    /// ¿La ruta está optimizada?
    pub is_optimized: bool,
    
    /// Todos los paquetes indexados por internal_id
    pub packages: HashMap<String, Package>,
    
    /// Todas las direcciones indexadas por address_id
    pub addresses: HashMap<String, Address>,
    
    /// Índices para búsquedas rápidas
    pub indices: Indices,
    
    /// Stats generales
    pub stats: Stats,
    
    /// Info del conductor
    pub driver: DriverInfo,
}

/// Paquete individual - ESTRUCTURA DEFINITIVA
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Package {
    // ========== IDs ÚNICOS ==========
    /// ID interno único generado al crear (NUNCA CAMBIA)
    pub internal_id: String,
    
    /// Número de tracking de Colis Privé
    pub tracking: String,
    
    /// ID de la dirección asociada
    pub address_id: String,
    
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
}

impl ImplicitClone for Address {}

/// Dirección - puede tener múltiples paquetes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Address {
    pub address_id: String,
    pub label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub mailbox_access: bool,
    pub door_code: Option<String>,
    pub driver_notes: String,
    
    /// IDs de paquetes en esta dirección
    pub package_ids: Vec<String>,
    
    /// Orden de visita en la ruta optimizada
    pub visit_order: Option<usize>,
    
    /// ¿Dirección corregida por el conductor?
    pub corrected_by_driver: bool,
    
    /// Dirección original antes de corrección
    pub original_label: Option<String>,
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

// ============================================================================
// ÍNDICES PARA BÚSQUEDAS RÁPIDAS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Indices {
    /// tracking_number -> internal_id
    /// CLAVE PARA EL SCANNER
    pub by_tracking: HashMap<String, String>,
    
    /// delivery_type -> [internal_ids]
    pub by_type: HashMap<String, Vec<String>>,
    
    /// status -> [internal_ids]
    pub by_status: HashMap<String, Vec<String>>,
    
    /// address_id -> [internal_ids]
    pub by_address: HashMap<String, Vec<String>>,
    
    /// problematic -> [internal_ids]
    pub problematic_packages: Vec<String>,
    
    /// route_order -> internal_id (solo después de optimizar)
    pub by_route_order: HashMap<usize, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Stats {
    pub total_packages: usize,
    pub total_addresses: usize,
    pub problematic_count: usize,
    pub by_type: HashMap<String, usize>,
    pub by_status: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriverInfo {
    pub driver_id: String,
    pub name: String,
    pub company_id: String,
    pub vehicle_id: Option<String>,
}

// ============================================================================
// DTOs PARA API
// ============================================================================

/// Request para crear sesión (login)
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub username: String,
    pub password: String,
    pub societe: String,
}

/// Respuesta al crear sesión
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub success: bool,
    pub session: Option<DeliverySession>,
    pub session_id: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}

/// Request para obtener paquetes
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchPackagesRequest {
    pub username: String,
    pub password: String,
    pub societe: String,
}

/// Respuesta de obtener paquetes
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchPackagesResponse {
    pub success: bool,
    pub session: Option<DeliverySession>,
    pub new_packages_count: Option<usize>,
    pub message: Option<String>,
    pub error: Option<String>,
}

/// Request de escaneo
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanRequest {
    pub session_id: String,
    pub tracking: String,
}

/// Respuesta de escaneo
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResponse {
    pub found: bool,
    pub package: Option<Package>,
    pub route_position: Option<usize>,
    pub total_packages: usize,
    pub is_scanned: bool,
    pub message: Option<String>,
}

// ============================================================================
// IMPLEMENTACIONES PARA MANIPULACIÓN
// ============================================================================

impl DeliverySession {
    /// Crear nueva sesión al login
    pub fn new(driver: DriverInfo) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            last_fetch: chrono::Utc::now().timestamp(),
            last_sync: chrono::Utc::now().timestamp(),
            last_optimization: None,
            is_optimized: false,
            packages: HashMap::new(),
            addresses: HashMap::new(),
            indices: Indices::default(),
            stats: Stats::default(),
            driver,
        }
    }
    
    /// Buscar paquete por tracking (PARA EL SCANNER)
    pub fn find_by_tracking(&self, tracking: &str) -> Option<&Package> {
        self.indices.by_tracking.get(tracking)
            .and_then(|internal_id| self.packages.get(internal_id))
    }
    
    /// Obtener posición en ruta optimizada por tracking
    pub fn get_route_position(&self, tracking: &str) -> Option<usize> {
        self.find_by_tracking(tracking)
            .and_then(|pkg| pkg.route_order)
    }
    
    /// Obtener paquetes en orden (optimizado o original)
    pub fn get_ordered_packages(&self) -> Vec<&Package> {
        let mut packages: Vec<&Package> = self.packages.values().collect();
        
        if self.is_optimized {
            packages.sort_by_key(|p| p.route_order.unwrap_or(usize::MAX));
        } else {
            packages.sort_by_key(|p| p.original_order);
        }
        
        packages
    }
    
    /// Obtener direcciones en orden de visita
    pub fn get_ordered_addresses(&self) -> Vec<&Address> {
        let mut addresses: Vec<&Address> = self.addresses.values().collect();
        
        if self.is_optimized {
            addresses.sort_by_key(|a| a.visit_order.unwrap_or(usize::MAX));
        } else {
            // Ordenar por número de paquetes (más paquetes primero)
            addresses.sort_by_key(|a| std::cmp::Reverse(a.package_ids.len()));
        }
        
        addresses
    }
    
    /// Serializar para localStorage/cache
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    /// Deserializar desde localStorage/cache
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
