use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Package {
    pub id: String,
    pub recipient: String,
    pub address: String,
    pub status: String,
    pub coords: Option<[f64; 2]>, // [longitude, latitude]
    pub phone: Option<String>,
    pub phone_fixed: Option<String>,
    pub instructions: Option<String>,
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
}

