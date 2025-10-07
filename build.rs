use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Cargar variables de entorno desde .env si existe
    let env_file = Path::new(".env");
    
    if env_file.exists() {
        println!("cargo:rerun-if-changed=.env");
        
        // Leer el archivo .env
        if let Ok(contents) = fs::read_to_string(env_file) {
            for line in contents.lines() {
                // Ignorar comentarios y líneas vacías
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                
                // Parsear KEY=VALUE
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    
                    // Solo configurar si no está ya definida
                    if env::var(key).is_err() {
                        println!("cargo:rustc-env={}={}", key, value);
                    }
                }
            }
        }
    } else {
        println!("cargo:warning=No .env file found. Using default values. Copy .env.example to .env and configure your settings.");
    }
    
    // Recompilar si cambia el archivo de configuración
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.env.example");
}

