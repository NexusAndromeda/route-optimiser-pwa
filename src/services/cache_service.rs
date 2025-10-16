use crate::models::package::Package;
use serde::{Deserialize, Serialize};
use web_sys::window;
use chrono::{DateTime, Utc};

const CACHE_KEY_PACKAGES: &str = "tournee_packages_cache";
const CACHE_VERSION: u32 = 3;
const CACHE_TTL_HOURS: i64 = 24;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackagesCache {
    pub version: u32,
    pub tournee_id: String,
    pub packages: Vec<Package>,
    pub singles: Vec<Package>,
    pub groups: Vec<Package>,
    pub problematic: Vec<Package>,
    pub timestamp: DateTime<Utc>,
    pub checksum: String,
    pub optimization_data: Option<OptimizationCache>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptimizationCache {
    pub optimized: bool,
    pub order: Vec<usize>,
    pub timestamp: DateTime<Utc>,
    pub total_distance: Option<f64>,
    pub total_duration: Option<f64>,
}

impl PackagesCache {
    pub fn new(tournee_id: String, packages: Vec<Package>) -> Self {
        let (singles, groups, problematic) = Self::categorize_packages(&packages);
        let checksum = Self::calculate_checksum(&packages);
        
        Self {
            version: CACHE_VERSION,
            tournee_id,
            packages,
            singles,
            groups,
            problematic,
            timestamp: Utc::now(),
            checksum,
            optimization_data: None,
        }
    }

    fn categorize_packages(packages: &[Package]) -> (Vec<Package>, Vec<Package>, Vec<Package>) {
        let mut singles = Vec::new();
        let mut groups = Vec::new();
        let mut problematic = Vec::new();

        for pkg in packages {
            if pkg.is_problematic {
                problematic.push(pkg.clone());
            } else if pkg.is_group {
                groups.push(pkg.clone());
            } else {
                singles.push(pkg.clone());
            }
        }

        (singles, groups, problematic)
    }

    fn calculate_checksum(packages: &[Package]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        packages.len().hash(&mut hasher);
        
        for pkg in packages {
            pkg.id.hash(&mut hasher);
            if let Some(ref status) = pkg.code_statut_article {
                status.hash(&mut hasher);
            }
        }
        
        format!("{:x}", hasher.finish())
    }

    pub fn is_valid(&self) -> bool {
        // Verificar versión
        if self.version < CACHE_VERSION {
            log::info!("❌ Caché inválido: versión antigua {} < {}", self.version, CACHE_VERSION);
            return false;
        }

        // Verificar TTL
        let now = Utc::now();
        let age_hours = now.signed_duration_since(self.timestamp).num_hours();
        if age_hours > CACHE_TTL_HOURS {
            log::info!("❌ Caché expirado: {} horas de antigüedad", age_hours);
            return false;
        }

        // Verificar checksum
        let current_checksum = Self::calculate_checksum(&self.packages);
        if current_checksum != self.checksum {
            log::info!("❌ Caché corrupto: checksum no coincide");
            return false;
        }

        true
    }

    pub fn update_packages(&mut self, packages: Vec<Package>) {
        let (singles, groups, problematic) = Self::categorize_packages(&packages);
        
        self.packages = packages;
        self.singles = singles;
        self.groups = groups;
        self.problematic = problematic;
        self.timestamp = Utc::now();
        self.checksum = Self::calculate_checksum(&self.packages);
        self.version = CACHE_VERSION;
    }

    pub fn update_optimization(&mut self, order: Vec<usize>, total_distance: Option<f64>, total_duration: Option<f64>) {
        self.optimization_data = Some(OptimizationCache {
            optimized: true,
            order,
            timestamp: Utc::now(),
            total_distance,
            total_duration,
        });
        self.timestamp = Utc::now();
    }
}

/// Servicio de caché para gestión de paquetes
pub struct CacheService;

impl CacheService {
    /// Sincroniza el estado de la tournée con el backend
    pub async fn sync_with_backend(tournee_id: &str) -> Result<(), String> {
        let cache = Self::load_cache()?
            .ok_or("No hay caché para sincronizar")?;
        
        // Preparar datos para sincronizar
        let problematic_packages: Vec<String> = cache.problematic.iter()
            .map(|p| p.id.clone())
            .collect();
        
        let updated_coords: Vec<serde_json::Value> = cache.packages.iter()
            .filter(|p| p.coords.is_some() && !p.is_problematic)
            .map(|p| {
                let coords = p.coords.unwrap();
                serde_json::json!({
                    "package_id": p.id,
                    "lat": coords[1],
                    "lng": coords[0],
                    "address": p.address
                })
            })
            .collect();
        
        let request_body = serde_json::json!({
            "tournee_id": tournee_id,
            "version": cache.version,
            "problematic_packages": problematic_packages,
            "updated_coords": updated_coords,
            "checksum": cache.checksum
        });
        
        // Enviar al backend
        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://localhost:8080/sync/state/{}", tournee_id))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Error enviando sincronización: {}", e))?;
        
        if response.status().is_success() {
            log::info!("✅ Estado sincronizado con el servidor");
            Ok(())
        } else {
            let error_msg = format!("Error HTTP: {}", response.status());
            log::error!("❌ {}", error_msg);
            Err(error_msg)
        }
    }

    /// Guarda el caché de paquetes
    pub fn save_cache(cache: &PackagesCache) -> Result<(), String> {
        let storage = window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .ok_or("No se pudo acceder a localStorage")?;

        let json = serde_json::to_string(cache)
            .map_err(|e| format!("Error serializando caché: {}", e))?;

        storage
            .set_item(CACHE_KEY_PACKAGES, &json)
            .map_err(|_| "Error guardando en localStorage".to_string())?;

        log::info!("💾 Caché guardado: {} paquetes (versión {})", cache.packages.len(), cache.version);
        Ok(())
    }

    /// Carga el caché de paquetes
    pub fn load_cache() -> Result<Option<PackagesCache>, String> {
        let storage = window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .ok_or("No se pudo acceder a localStorage")?;

        let json = storage
            .get_item(CACHE_KEY_PACKAGES)
            .map_err(|_| "Error accediendo a localStorage".to_string())?;

        match json {
            Some(data) => {
                let cache: PackagesCache = serde_json::from_str(&data)
                    .map_err(|e| format!("Error deserializando caché: {}", e))?;

                if cache.is_valid() {
                    log::info!("✅ Caché válido cargado: {} paquetes", cache.packages.len());
                    Ok(Some(cache))
                } else {
                    log::info!("⚠️ Caché inválido o expirado, se descartará");
                    Self::clear_cache()?;
                    Ok(None)
                }
            }
            None => {
                log::info!("ℹ️ No hay caché guardado");
                Ok(None)
            }
        }
    }

    /// Limpia el caché
    pub fn clear_cache() -> Result<(), String> {
        let storage = window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .ok_or("No se pudo acceder a localStorage")?;

        storage
            .remove_item(CACHE_KEY_PACKAGES)
            .map_err(|_| "Error limpiando localStorage".to_string())?;

        log::info!("🗑️ Caché limpiado");
        Ok(())
    }

    /// Actualiza solo los paquetes en el caché existente
    pub fn update_packages(packages: Vec<Package>) -> Result<(), String> {
        let mut cache = Self::load_cache()?.unwrap_or_else(|| {
            // Si no hay caché, crear uno nuevo
            let tournee_id = window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .and_then(|s| s.get_item("login_data").ok())
                .flatten()
                .and_then(|data| {
                    serde_json::from_str::<serde_json::Value>(&data).ok()
                })
                .and_then(|v| v.get("username").and_then(|u| u.as_str()).map(String::from))
                .unwrap_or_else(|| "unknown".to_string());
            
            PackagesCache::new(tournee_id, vec![])
        });

        cache.update_packages(packages);
        Self::save_cache(&cache)
    }

    /// Actualiza la optimización en el caché
    pub fn update_optimization(order: Vec<usize>, total_distance: Option<f64>, total_duration: Option<f64>) -> Result<(), String> {
        let mut cache = Self::load_cache()?
            .ok_or("No hay caché para actualizar")?;

        cache.update_optimization(order, total_distance, total_duration);
        Self::save_cache(&cache)
    }

    /// Marca un paquete como problemático
    pub fn mark_package_problematic(package_id: &str) -> Result<(), String> {
        let mut cache = Self::load_cache()?
            .ok_or("No hay caché para actualizar")?;

        // Buscar y actualizar el paquete
        if let Some(pkg) = cache.packages.iter_mut().find(|p| p.id == package_id) {
            pkg.is_problematic = true;
            pkg.coords = None;
        }

        cache.update_packages(cache.packages.clone());
        Self::save_cache(&cache)
    }

    /// Actualiza las coordenadas de un paquete
    pub fn update_package_coords(package_id: &str, lat: f64, lng: f64, address: String) -> Result<(), String> {
        let mut cache = Self::load_cache()?
            .ok_or("No hay caché para actualizar")?;

        // Buscar y actualizar el paquete
        if let Some(pkg) = cache.packages.iter_mut().find(|p| p.id == package_id) {
            pkg.coords = Some([lng, lat]);
            pkg.address = address;
            pkg.is_problematic = false;
        }

        cache.update_packages(cache.packages.clone());
        Self::save_cache(&cache)
    }

    /// Obtiene estadísticas del caché
    pub fn get_stats() -> Result<CacheStats, String> {
        let cache = Self::load_cache()?;

        match cache {
            Some(cache) => {
                let age_hours = Utc::now().signed_duration_since(cache.timestamp).num_hours();
                
                Ok(CacheStats {
                    total_packages: cache.packages.len(),
                    singles: cache.singles.len(),
                    groups: cache.groups.len(),
                    problematic: cache.problematic.len(),
                    is_optimized: cache.optimization_data.is_some(),
                    age_hours,
                    version: cache.version,
                    checksum: cache.checksum.clone(),
                })
            }
            None => Ok(CacheStats::default()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_packages: usize,
    pub singles: usize,
    pub groups: usize,
    pub problematic: usize,
    pub is_optimized: bool,
    pub age_hours: i64,
    pub version: u32,
    pub checksum: String,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_packages: 0,
            singles: 0,
            groups: 0,
            problematic: 0,
            is_optimized: false,
            age_hours: 0,
            version: 0,
            checksum: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_categorization() {
        // Este test se ejecutaría en un entorno con localStorage disponible
    }
}

