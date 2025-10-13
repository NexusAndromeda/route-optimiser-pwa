use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Package {
    pub id: String,
    pub recipient: String,
    pub address: String,
    pub status: String,
    pub coords: Option<[f64; 2]>, // [longitude, latitude]
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
    pub lst_lieu_article: Vec<OptimizedPackage>,
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

impl Package {
    pub fn demo_packages() -> Vec<Self> {
        vec![
            Package {
                id: "CP123456".to_string(),
                recipient: "Jean Dupont".to_string(),
                address: "15 Rue de la Paix, 75001 Paris".to_string(),
                status: "pending".to_string(),
                coords: Some([2.3316, 48.8698]), // Rue de la Paix
            },
            Package {
                id: "CP123457".to_string(),
                recipient: "Marie Martin".to_string(),
                address: "23 Avenue des Champs-Élysées, 75008 Paris".to_string(),
                status: "delivered".to_string(),
                coords: Some([2.3069, 48.8698]), // Champs-Élysées
            },
            Package {
                id: "CP123458".to_string(),
                recipient: "Sophie Leroy".to_string(),
                address: "8 Rue de Rivoli, 75004 Paris".to_string(),
                status: "pending".to_string(),
                coords: Some([2.3522, 48.8566]), // Rivoli
            },
            Package {
                id: "CP123459".to_string(),
                recipient: "Pierre Moreau".to_string(),
                address: "25 Boulevard Saint-Germain, 75005 Paris".to_string(),
                status: "delivered".to_string(),
                coords: Some([2.3488, 48.8534]), // Saint-Germain
            },
            Package {
                id: "CP123460".to_string(),
                recipient: "Claire Bernard".to_string(),
                address: "12 Place de la Bastille, 75011 Paris".to_string(),
                status: "pending".to_string(),
                coords: Some([2.3691, 48.8530]), // Bastille
            },
        ]
    }
}

