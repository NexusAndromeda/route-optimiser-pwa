use yew::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, PartialEq, Debug)]
pub enum Language {
    French,
    Spanish,
}

impl Default for Language {
    fn default() -> Self {
        Language::French
    }
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::French => "FR",
            Language::Spanish => "ES",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "ES" => Language::Spanish,
            _ => Language::French,
        }
    }
}

#[derive(Clone)]
pub struct LanguageContext {
    pub language: Language,
    pub texts: Rc<HashMap<String, String>>,
}

impl PartialEq for LanguageContext {
    fn eq(&self, other: &Self) -> bool {
        self.language == other.language
    }
}

impl LanguageContext {
    pub fn new(language: Language) -> Self {
        let mut texts = HashMap::new();
        
        match language {
            Language::French => {
                // Login
                texts.insert("app_title".to_string(), "Route Optimizer".to_string());
                texts.insert("login_title".to_string(), "Connexion".to_string());
                texts.insert("username".to_string(), "Nom d'utilisateur".to_string());
                texts.insert("password".to_string(), "Mot de passe".to_string());
                texts.insert("company".to_string(), "Entreprise".to_string());
                texts.insert("login_button".to_string(), "Se connecter".to_string());
                texts.insert("register_text".to_string(), "Votre entreprise souhaite utiliser Route Optimizer? Contactez-nous pour un essai gratuit".to_string());
                texts.insert("register_button".to_string(), "Contactez-nous".to_string());
                
                // Navigation
                texts.insert("optimize".to_string(), "Optimiser".to_string());
                texts.insert("refresh".to_string(), "Actualiser".to_string());
                texts.insert("settings".to_string(), "Paramètres".to_string());
                texts.insert("logout".to_string(), "Déconnexion".to_string());
                
                // Package details
                texts.insert("package_details".to_string(), "Détails du Colis".to_string());
                texts.insert("recipient".to_string(), "Destinataire".to_string());
                texts.insert("address".to_string(), "Adresse".to_string());
                texts.insert("phone".to_string(), "Téléphone".to_string());
                texts.insert("door_codes".to_string(), "Codes de porte".to_string());
                texts.insert("mailbox_access".to_string(), "Accès boîte aux lettres (BAL)".to_string());
                texts.insert("client_instructions".to_string(), "Indications du client".to_string());
                texts.insert("driver_notes".to_string(), "Notes propres du chauffeur".to_string());
                texts.insert("not_provided".to_string(), "Non renseigné".to_string());
                texts.insert("modify".to_string(), "Modifier".to_string());
                texts.insert("edit_address".to_string(), "Modifier l'adresse (géocodage)".to_string());
                texts.insert("edit_door_code".to_string(), "Modifier Code de porte".to_string());
                texts.insert("edit_client_instructions".to_string(), "Modifier Indications du client".to_string());
                texts.insert("edit_driver_notes".to_string(), "Modifier Notes du chauffeur".to_string());
                texts.insert("add_note".to_string(), "Ajouter une note...".to_string());
                texts.insert("go".to_string(), "Aller".to_string());
                texts.insert("details".to_string(), "Détails".to_string());
                
                // Settings
                texts.insert("language".to_string(), "Langue".to_string());
                texts.insert("french".to_string(), "Français".to_string());
                texts.insert("spanish".to_string(), "Español".to_string());
                texts.insert("edit_mode".to_string(), "Éditer".to_string());
                texts.insert("color_codes".to_string(), "Code couleurs".to_string());
                texts.insert("ready_to_load".to_string(), "Prêt à charger".to_string());
                texts.insert("in_transit".to_string(), "En route".to_string());
                texts.insert("delivered_status".to_string(), "Livré".to_string());
                texts.insert("not_delivered".to_string(), "Non livré".to_string());
                
                // Geocoding
                texts.insert("geocoding_prompt".to_string(), "Modifier l'adresse pour géocodage:\n\nEntrez la nouvelle adresse complète:\n\n⚠️ Laisser vide pour marquer comme problématique".to_string());
                texts.insert("geocoding_success".to_string(), "Géocodage réussi".to_string());
                texts.insert("geocoding_error".to_string(), "Erreur de géocodage".to_string());
                texts.insert("package_marked_problematic".to_string(), "⚠️ Paquet marqué comme problématique\n\nIl a été retiré de la carte et placé en bas de la liste.".to_string());
                
                // Status
                texts.insert("pending".to_string(), "En attente".to_string());
                texts.insert("delivered".to_string(), "Livré".to_string());
                texts.insert("loading".to_string(), "Chargement...".to_string());
                texts.insert("packages_loaded".to_string(), "Paquetes obtenidos".to_string());
                texts.insert("packages_with_coords".to_string(), "Paquetes avec coordonnées".to_string());
            },
            Language::Spanish => {
                // Login
                texts.insert("app_title".to_string(), "Route Optimizer".to_string());
                texts.insert("login_title".to_string(), "Iniciar Sesión".to_string());
                texts.insert("username".to_string(), "Nombre de usuario".to_string());
                texts.insert("password".to_string(), "Contraseña".to_string());
                texts.insert("company".to_string(), "Empresa".to_string());
                texts.insert("login_button".to_string(), "Iniciar sesión".to_string());
                texts.insert("register_text".to_string(), "¿Su empresa desea usar Route Optimizer? Contáctenos para una prueba gratuita".to_string());
                texts.insert("register_button".to_string(), "Contáctenos".to_string());
                
                // Navigation
                texts.insert("optimize".to_string(), "Optimizar".to_string());
                texts.insert("refresh".to_string(), "Actualizar".to_string());
                texts.insert("settings".to_string(), "Configuración".to_string());
                texts.insert("logout".to_string(), "Cerrar sesión".to_string());
                
                // Package details
                texts.insert("package_details".to_string(), "Detalles del Paquete".to_string());
                texts.insert("recipient".to_string(), "Destinatario".to_string());
                texts.insert("address".to_string(), "Dirección".to_string());
                texts.insert("phone".to_string(), "Teléfono".to_string());
                texts.insert("door_codes".to_string(), "Códigos de puerta".to_string());
                texts.insert("mailbox_access".to_string(), "Acceso buzón (BAL)".to_string());
                texts.insert("client_instructions".to_string(), "Indicaciones del cliente".to_string());
                texts.insert("driver_notes".to_string(), "Notas del conductor".to_string());
                texts.insert("not_provided".to_string(), "No proporcionado".to_string());
                texts.insert("modify".to_string(), "Modificar".to_string());
                texts.insert("edit_address".to_string(), "Modificar dirección (geocodificación)".to_string());
                texts.insert("edit_door_code".to_string(), "Modificar código de puerta".to_string());
                texts.insert("edit_client_instructions".to_string(), "Modificar indicaciones del cliente".to_string());
                texts.insert("edit_driver_notes".to_string(), "Modificar notas del conductor".to_string());
                texts.insert("add_note".to_string(), "Agregar una nota...".to_string());
                texts.insert("go".to_string(), "Ir".to_string());
                texts.insert("details".to_string(), "Detalles".to_string());
                
                // Settings
                texts.insert("language".to_string(), "Idioma".to_string());
                texts.insert("french".to_string(), "Français".to_string());
                texts.insert("spanish".to_string(), "Español".to_string());
                texts.insert("edit_mode".to_string(), "Editar".to_string());
                texts.insert("color_codes".to_string(), "Código de colores".to_string());
                texts.insert("ready_to_load".to_string(), "Listo para cargar".to_string());
                texts.insert("in_transit".to_string(), "En ruta".to_string());
                texts.insert("delivered_status".to_string(), "Entregado".to_string());
                texts.insert("not_delivered".to_string(), "No entregado".to_string());
                
                // Geocoding
                texts.insert("geocoding_prompt".to_string(), "Modificar dirección para geocodificación:\n\nIngrese la nueva dirección completa:\n\n⚠️ Dejar vacío para marcar como problemático".to_string());
                texts.insert("geocoding_success".to_string(), "Geocodificación exitosa".to_string());
                texts.insert("geocoding_error".to_string(), "Error de geocodificación".to_string());
                texts.insert("package_marked_problematic".to_string(), "⚠️ Paquete marcado como problemático\n\nHa sido removido del mapa y colocado al final de la lista.".to_string());
                
                // Status
                texts.insert("pending".to_string(), "Pendiente".to_string());
                texts.insert("delivered".to_string(), "Entregado".to_string());
                texts.insert("loading".to_string(), "Cargando...".to_string());
                texts.insert("packages_loaded".to_string(), "Paquetes obtenidos".to_string());
                texts.insert("packages_with_coords".to_string(), "Paquetes con coordenadas".to_string());
                texts.insert("no_packages".to_string(), "Aucun colis disponible".to_string());
                texts.insert("packages_after_login".to_string(), "Les colis apparaîtront ici après la connexion".to_string());
                texts.insert("please_wait".to_string(), "Veuillez patienter".to_string());
            }
        }
        
        Self { language, texts: Rc::new(texts) }
    }

    pub fn get(&self, key: &str) -> String {
        self.texts.get(key).cloned().unwrap_or_else(|| key.to_string())
    }
}

#[derive(Properties, PartialEq, Default)]
pub struct LanguageProviderProps {
    pub children: Children,
}

#[function_component]
pub fn LanguageProvider(props: &LanguageProviderProps) -> Html {
    // Get language from localStorage or default to French
    let initial_language = use_memo((), |_| {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(lang)) = storage.get_item("routeOptimizer_language") {
                    return Language::from_str(&lang);
                }
            }
        }
        Language::default()
    });

    let language_context = use_memo((*initial_language).clone(), |lang| {
        LanguageContext::new(lang.clone())
    });

    html! {
        <ContextProvider<LanguageContext> context={(*language_context).clone()}>
            <crate::views::App />
        </ContextProvider<LanguageContext>>
    }
}

// Simple function to get language from localStorage
pub fn get_current_language() -> Language {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(lang)) = storage.get_item("routeOptimizer_language") {
                return Language::from_str(&lang);
            }
        }
    }
    Language::default()
}

// Simple function to set language in localStorage
pub fn set_language(language: &Language) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("routeOptimizer_language", language.as_str());
        }
    }
}

// Simple function to get text by key
pub fn get_text(key: &str) -> String {
    let language = get_current_language();
    let context = LanguageContext::new(language);
    context.get(key)
}
