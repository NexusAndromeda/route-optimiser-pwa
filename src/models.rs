use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Package {
    pub id: String,
    pub recipient: String,
    pub address: String,
    pub status: String,
}

impl Package {
    pub fn demo_packages() -> Vec<Self> {
        vec![
            Package {
                id: "CP123456".to_string(),
                recipient: "Jean Dupont".to_string(),
                address: "15 Rue de la Paix, 75001 Paris".to_string(),
                status: "pending".to_string(),
            },
            Package {
                id: "CP123457".to_string(),
                recipient: "Marie Martin".to_string(),
                address: "23 Avenue des Champs-Élysées, 75008 Paris".to_string(),
                status: "delivered".to_string(),
            },
            Package {
                id: "CP123458".to_string(),
                recipient: "Sophie Leroy".to_string(),
                address: "8 Rue de Rivoli, 75004 Paris".to_string(),
                status: "pending".to_string(),
            },
            Package {
                id: "CP123459".to_string(),
                recipient: "Pierre Moreau".to_string(),
                address: "25 Boulevard Saint-Germain, 75005 Paris".to_string(),
                status: "delivered".to_string(),
            },
            Package {
                id: "CP123460".to_string(),
                recipient: "Claire Bernard".to_string(),
                address: "12 Place de la Bastille, 75011 Paris".to_string(),
                status: "pending".to_string(),
            },
        ]
    }
}

