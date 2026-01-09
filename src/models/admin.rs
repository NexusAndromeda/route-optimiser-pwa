use serde::{Deserialize, Serialize};

/// Distrito (código postal) con sus tournées asociadas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDistrict {
    pub code_postal: String,
    pub nom_ville: Option<String>,
    pub tournees: Vec<AdminTournee>,
}

/// Tournée dentro de un distrito
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminTournee {
    pub letter: String, // "A", "B", "C" extraída del matricule
    pub code_tournee: String, // "PCP0010699_B187518-20260106"
    pub matricule: String, // "PCP0010699_B187518" o "B187518"
    pub nom_chauffeur: Option<String>,
    pub nb_colis: usize,
    pub delivered_count: usize, // Número de paquetes entregados
    pub statut: String, // "EN_COURS", "TERMINE", "Non optimisé", etc.
}

/// Status change request completo (para responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChangeRequest {
    pub id: Option<String>,
    pub tracking_code: String,
    pub session_id: String,
    pub driver_matricule: String,
    pub notes: Option<String>,
    pub customer_name: String,
    pub customer_address: String,
    pub delivery_date: String,
    pub status: String,
    pub created_at: Option<String>,
}

/// Acción técnica en el historial de traçabilité
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityAction {
    pub date_action: String,
    pub type_action: String,
    pub origine_action: Option<String>,
    pub description: String,
    pub commentaire: String,
}

/// Response de traçabilité de un paquete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTraceabilityResponse {
    pub success: bool,
    pub code_colis: String,
    pub nomprenom_destinataire: Option<String>,
    pub adresse_destinataire: Option<String>,
    pub telephone_destinataire: Option<String>,
    pub email_destinataire: Option<String>,
    pub datelivraison_prevu: Option<String>,
    pub datelivraison_reel: Option<String>,
    pub code_statut_colis: Option<String>,
    pub code_etat_colis: Option<String>,
    pub actions: Vec<TraceabilityAction>,
}


