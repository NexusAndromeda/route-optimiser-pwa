use web_sys::{window, Storage};
use serde::{Serialize, de::DeserializeOwned};

pub fn get_local_storage() -> Option<Storage> {
    window()?.local_storage().ok()?
}

pub fn save_to_storage<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    let storage = get_local_storage().ok_or("No se pudo acceder a localStorage")?;
    let json = serde_json::to_string(value)
        .map_err(|e| format!("Error serializando datos: {}", e))?;
    storage.set_item(key, &json)
        .map_err(|_| "Error guardando en localStorage".to_string())?;
    Ok(())
}

pub fn load_from_storage<T: DeserializeOwned>(key: &str) -> Option<T> {
    let storage = get_local_storage()?;
    let json = storage.get_item(key).ok()??;
    serde_json::from_str(&json).ok()
}

pub fn remove_from_storage(key: &str) -> Result<(), String> {
    let storage = get_local_storage().ok_or("No se pudo acceder a localStorage")?;
    storage.remove_item(key)
        .map_err(|_| "Error eliminando de localStorage".to_string())?;
    Ok(())
}

pub fn get_cache_key(societe: &str, username: &str) -> String {
    format!("{}_{}", societe, username)
}

