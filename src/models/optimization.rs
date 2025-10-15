use serde::{Deserialize, Serialize};

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

