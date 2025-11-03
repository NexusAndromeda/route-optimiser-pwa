use yew::prelude::*;
use std::collections::HashMap;
use crate::models::package::Package;

#[derive(Debug, Clone, PartialEq)]
pub enum GroupBy {
    Address,
    Status,
    DeliveryType,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackageGroup {
    pub title: String,
    pub count: usize,
    pub packages: Vec<Package>,
}

pub fn group_packages(
    packages: Vec<Package>,
    group_by: GroupBy,
) -> Vec<PackageGroup> {
    match group_by {
        GroupBy::Address => group_by_address(&packages),
        GroupBy::Status => group_by_status(&packages),
        GroupBy::DeliveryType => group_by_delivery_type(&packages),
        GroupBy::None => vec![PackageGroup {
            title: "Tous les colis".to_string(),
            count: packages.len(),
            packages: packages.clone(),
        }],
    }
}

fn group_by_address(packages: &[Package]) -> Vec<PackageGroup> {
    let mut map: HashMap<String, Vec<Package>> = HashMap::new();
    
    // Agrupar por address_id
    for p in packages.iter().cloned() {
        let key = p.address_id.clone();
        map.entry(key).or_default().push(p);
    }
    
    // Ordenar paquetes dentro de cada grupo según original_order/route_order
    for packages in map.values_mut() {
        packages.sort_by_key(|p| {
            // Usar route_order si está optimizado, sino original_order (orden del backend)
            p.route_order.unwrap_or(p.original_order)
        });
    }
    
    // Convertir a Vec de grupos
    let mut groups: Vec<PackageGroup> = map
        .into_iter()
        .map(|(title, packages)| {
            PackageGroup { 
                title, 
                count: packages.len(), 
                packages 
            }
        })
        .collect();
    
    // ⭐ ORDENAR grupos según original_order/route_order del PRIMER paquete
    // Esto preserva el orden que el backend envió (basado en el orden del JSON)
    // Cuando hay múltiples paquetes en la misma dirección, el grupo usa el orden
    // del primer paquete de esa dirección que apareció en el JSON
    groups.sort_by_key(|group| {
        group.packages.first()
            .map(|p| p.route_order.unwrap_or(p.original_order))
            .unwrap_or(0)
    });
    
    groups
}

fn group_by_status(packages: &[Package]) -> Vec<PackageGroup> {
    let mut map: HashMap<String, Vec<Package>> = HashMap::new();
    for p in packages.iter().cloned() {
        let sup = p.status.to_uppercase();
        let key = if sup.contains("LIVR") {
            "DELIVERED".to_string()
        } else if sup.contains("NONLIV") || sup.contains("ECHEC") {
            "FAILED".to_string()
        } else if sup.contains("SCAN") || sup.contains("RECEPT") {
            "SCANNED".to_string()
        } else {
            "PENDING".to_string()
        };
        map.entry(key).or_default().push(p);
    }
    map.into_iter()
        .map(|(title, packages)| PackageGroup { title, count: packages.len(), packages })
        .collect()
}

fn group_by_delivery_type(packages: &[Package]) -> Vec<PackageGroup> {
    let mut map: HashMap<String, Vec<Package>> = HashMap::new();
    for p in packages.iter().cloned() {
        let key = match p.delivery_type {
            crate::models::package::DeliveryType::PickupPoint => "RELAIS".to_string(),
            crate::models::package::DeliveryType::Rcs => "RCS".to_string(),
            _ => "DOMICILE".to_string(),
        };
        map.entry(key).or_default().push(p);
    }
    map.into_iter()
        .map(|(title, packages)| PackageGroup { title, count: packages.len(), packages })
        .collect()
}


