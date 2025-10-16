use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Package {
    pub id: String,
    pub recipient: String,
    pub address: String,
    pub status: String,
    pub code_statut_article: Option<String>,
    pub coords: Option<[f64; 2]>, // [longitude, latitude]
    pub phone: Option<String>,
    pub phone_fixed: Option<String>,
    pub instructions: Option<String>,
    
    // Campos para grupos
    #[serde(default)]
    pub is_group: bool,
    #[serde(default)]
    pub total_packages: Option<usize>,
    #[serde(default)]
    pub group_packages: Option<Vec<GroupPackageInfo>>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct GroupPackageInfo {
    pub id: String,
    pub tracking: String,
    pub customer_name: String,
    pub phone_number: Option<String>,
    pub customer_indication: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct PackageRequest {
    pub matricule: String,
    pub societe: String,
    pub date: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct PackagesCache {
    pub packages: Vec<Package>,
    pub timestamp: String,
    #[serde(default)]
    pub version: u32, // Version del cache para invalidar cuando cambia la estructura
}

