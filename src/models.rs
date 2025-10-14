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

// Login models
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Company {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct CompaniesResponse {
    pub success: bool,
    pub companies: Vec<Company>,
    pub message: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub societe: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub success: bool,
    #[serde(default)]
    pub authentication: Option<AuthenticationInfo>,
    #[serde(default)]
    pub error: Option<ErrorInfo>,
    pub credentials_used: Option<CredentialsUsed>,
    pub timestamp: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct AuthenticationInfo {
    pub token: Option<String>,
    pub matricule: Option<String>,
    pub message: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct ErrorInfo {
    pub message: Option<String>,
    pub code: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct CredentialsUsed {
    pub username: String,
    pub societe: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct LoginData {
    pub username: String,
    pub token: String,
    pub company: Company,
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

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct OptimizationRequest {
    pub matricule: String,
    pub societe: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct OptimizationResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<OptimizationData>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct OptimizationData {
    pub matricule_chauffeur: Option<String>,
    pub date_tournee: Option<String>,
    pub optimized_packages: Vec<OptimizedPackage>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct OptimizedPackage {
    pub numero_ordre: Option<i32>,
    pub reference_colis: Option<String>,
    pub destinataire_nom: Option<String>,
    pub destinataire_adresse1: Option<String>,
    pub coord_x_destinataire: Option<f64>,
    pub coord_y_destinataire: Option<f64>,
    pub statut: Option<String>,
}

// Modelos para Mapbox Optimization API
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct OptimizationPackage {
    pub id: String,
    pub reference_colis: String,
    pub destinataire_nom: String,
    pub destinataire_adresse1: Option<String>,
    pub destinataire_cp: Option<String>,
    pub destinataire_ville: Option<String>,
    pub coord_x_destinataire: Option<f64>,
    pub coord_y_destinataire: Option<f64>,
    pub statut: Option<String>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct MapboxOptimizationRequest {
    pub matricule: String,
    pub societe: String,
    pub packages: Vec<OptimizationPackage>,
}
