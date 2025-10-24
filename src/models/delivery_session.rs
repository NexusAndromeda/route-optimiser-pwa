// ============================================================================
// ESTRUCTURA UNIFICADA PARA TODO EL SISTEMA - FRONTEND
// ============================================================================
// Esta estructura es idéntica al backend para garantizar compatibilidad
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// ============================================================================
// MODELOS PRINCIPALES
// ============================================================================

/// Estructura PRINCIPAL que contiene TODO
/// Esta es la que se guarda en:
/// - Backend: Redis Cache
/// - Frontend: LocalStorage
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Dirección - puede tener múltiples paquetes
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    pub total_packages: usize,
    pub total_addresses: usize,
    pub problematic_count: usize,
    pub by_type: HashMap<String, usize>,
    pub by_status: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    pub driver_id: String,
    pub name: String,
    pub company_id: String,
    pub vehicle_id: Option<String>,
}

// ============================================================================
// IMPLEMENTACIONES PARA MANIPULACIÓN
// ============================================================================

impl DeliverySession {
    /// Crear nueva sesión al login
    pub fn new(driver: DriverInfo) -> Self {
        Self {
            session_id: js_sys::Math::random().to_string(), // Usar Math.random() en lugar de Uuid
            last_fetch: js_sys::Date::now() as i64 / 1000, // Timestamp en segundos
            last_sync: js_sys::Date::now() as i64 / 1000,
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
    
    /// Marcar como escaneado (actualizar status)
    pub fn mark_scanned(&mut self, tracking: &str) -> Result<(), String> {
        let internal_id = self.indices.by_tracking.get(tracking)
            .ok_or("Package not found")?
            .clone();
        
        if let Some(package) = self.packages.get_mut(&internal_id) {
            // Actualizar status a escaneado
            package.status = "STATUT_SCANNED".to_string();
            package.modified_by_driver = true;
            self.update_stats();
        }
        Ok(())
    }
    
    /// Actualizar dirección (cuando el conductor corrige)
    pub fn update_address(&mut self, address_id: &str, new_label: String, 
                          new_lat: f64, new_lng: f64) -> Result<(), String> {
        if let Some(address) = self.addresses.get_mut(address_id) {
            if !address.corrected_by_driver {
                address.original_label = Some(address.label.clone());
            }
            address.label = new_label;
            address.latitude = new_lat;
            address.longitude = new_lng;
            address.corrected_by_driver = true;
            
            // Marcar todos los paquetes de esta dirección como modificados
            for pkg_id in &address.package_ids {
                if let Some(pkg) = self.packages.get_mut(pkg_id) {
                    pkg.modified_by_driver = true;
                }
            }
            
            self.last_sync = js_sys::Date::now() as i64 / 1000;
            Ok(())
        } else {
            Err("Address not found".to_string())
        }
    }
    
    /// Aplicar optimización (desde Mapbox)
    pub fn apply_optimization(&mut self, optimized_order: Vec<String>) -> Result<(), String> {
        // optimized_order es una lista de internal_ids en orden optimizado
        
        self.indices.by_route_order.clear();
        
        for (route_order, internal_id) in optimized_order.iter().enumerate() {
            if let Some(package) = self.packages.get_mut(internal_id) {
                package.route_order = Some(route_order);
                package.visual_position = route_order;
                
                self.indices.by_route_order.insert(route_order, internal_id.clone());
            }
        }
        
        self.is_optimized = true;
        self.last_optimization = Some(js_sys::Date::now() as i64 / 1000);
        
        Ok(())
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
    
    /// Actualizar estadísticas
    pub fn update_stats(&mut self) {
        self.stats.total_packages = self.packages.len();
        self.stats.total_addresses = self.addresses.len();
        self.stats.problematic_count = self.indices.problematic_packages.len();
        
        // Recalcular stats por tipo y estado
        self.stats.by_type.clear();
        self.stats.by_status.clear();
        
        for package in self.packages.values() {
            let type_key = format!("{:?}", package.delivery_type);
            *self.stats.by_type.entry(type_key).or_insert(0) += 1;
            *self.stats.by_status.entry(package.status.clone()).or_insert(0) += 1;
        }
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

// ============================================================================
// DTOs PARA API
// ============================================================================

/// Respuesta al login/fetch inicial
#[derive(Debug, Serialize, Deserialize)]
pub struct InitialFetchResponse {
    pub session: DeliverySession,
    pub new_packages_count: usize,
}

/// Request para sincronización
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub session_id: String,
    pub session: DeliverySession,
}

/// Respuesta de sincronización
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub updated_session: DeliverySession,
    pub new_packages: Vec<Package>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SyncStatus {
    UpToDate,
    NewPackagesAdded,
    ConflictsResolved,
}

/// Request para optimización
#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationRequest {
    pub session_id: String,
    /// Lista de internal_ids en orden actual
    pub package_ids: Vec<String>,
}

/// Respuesta de optimización
#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationResponse {
    pub optimized_order: Vec<String>, // internal_ids ordenados
    pub total_distance_km: f64,
    pub total_duration_min: f64,
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
}

/// Request para crear sesión
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub driver: DriverInfo,
    pub sso_token: String,
}

/// Parámetros para cargar sesión
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadSessionParams {
    pub session_id: String,
}
