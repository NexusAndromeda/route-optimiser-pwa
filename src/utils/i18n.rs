// ============================================================================
// MÓDULO DE INTERNACIONALIZACIÓN
// ============================================================================

use std::collections::HashMap;

/// Obtener diccionario de traducciones para un idioma
fn get_translations(lang: &str) -> HashMap<&'static str, &'static str> {
    let mut translations = HashMap::new();
    let lang_upper = lang.to_uppercase();
    
    match lang_upper.as_str() {
        "ES" => {
            // Details Modal
            translations.insert("destinataire", "Destinatario");
            translations.insert("adresse", "Dirección");
            translations.insert("telephone", "Teléfono");
            translations.insert("codes_porte", "Códigos de puerta");
            translations.insert("acces_bal", "Acceso BAL");
            translations.insert("indications_client", "Indicaciones cliente");
            translations.insert("notes_chauffeur", "Notas chofer");
            translations.insert("enregistrer", "Guardar");
            translations.insert("annuler", "Cancelar");
            translations.insert("modifier", "Modificar");
            translations.insert("non_renseigne", "No especificado");
            translations.insert("ajouter_note", "Agregar una nota");
            translations.insert("nouvelle_adresse", "Nueva dirección");
            translations.insert("code_de_porte", "Código de puerta");
            translations.insert("oui", "Sí");
            translations.insert("non", "No");
            
            // Settings Popup
            translations.insert("parametres", "Parámetros");
            translations.insert("langue", "Idioma");
            translations.insert("mode_edition", "Modo edición");
            translations.insert("filtrer", "Filtrar");
            translations.insert("codes_couleur", "🎨 Códigos color");
            translations.insert("deconnexion", "⎋ Desconexión");
            translations.insert("relais", "RELAIS");
            translations.insert("rcs_premium", "RCS (Premium)");
            translations.insert("livre", "Entregado");
            translations.insert("non_livre", "No entregado");
            translations.insert("en_transit", "En tránsito");
            translations.insert("receptionne", "Recibido");
            translations.insert("en_collecte", "En recogida");
            
            // App
            translations.insert("route_optimizer", "Route Optimizer");
            translations.insert("optimiser", "Optimizar");
            translations.insert("scanner", "Escanear");
            translations.insert("rafraichir", "Refrescar");
            translations.insert("aucun_colis", "No hay paquetes en la sesión");
            translations.insert("veuillez_rafraichir", "Por favor refrescar o recargar la ronda");
            translations.insert("traitees", "tratadas");
            translations.insert("paquets", "paquetes");
            translations.insert("oui_capital", "Sí");
            translations.insert("non_capital", "No");
            translations.insert("marquer_problematique", "Marcar como problemático");
            translations.insert("problematique", "Problemático");
        }
        "FR" | _ => {
            // Details Modal
            translations.insert("destinataire", "Destinataire");
            translations.insert("adresse", "Adresse");
            translations.insert("telephone", "Téléphone");
            translations.insert("codes_porte", "Codes de porte");
            translations.insert("acces_bal", "Accès BAL");
            translations.insert("indications_client", "Indications client");
            translations.insert("notes_chauffeur", "Notes chauffeur");
            translations.insert("enregistrer", "Enregistrer");
            translations.insert("annuler", "Annuler");
            translations.insert("modifier", "Modifier");
            translations.insert("non_renseigne", "Non renseigné");
            translations.insert("ajouter_note", "Ajouter une note");
            translations.insert("nouvelle_adresse", "Nouvelle adresse");
            translations.insert("code_de_porte", "Code de porte");
            translations.insert("oui", "Oui");
            translations.insert("non", "Non");
            
            // Settings Popup
            translations.insert("parametres", "Paramètres");
            translations.insert("langue", "Langue");
            translations.insert("mode_edition", "Mode édition");
            translations.insert("filtrer", "Filtrer");
            translations.insert("codes_couleur", "🎨 Codes couleur");
            translations.insert("deconnexion", "⎋ Déconnexion");
            translations.insert("relais", "RELAIS");
            translations.insert("rcs_premium", "RCS (Premium)");
            translations.insert("livre", "Livré");
            translations.insert("non_livre", "Non livré");
            translations.insert("en_transit", "En transit");
            translations.insert("receptionne", "Réceptionné");
            translations.insert("en_collecte", "En collecte");
            
            // App
            translations.insert("route_optimizer", "Route Optimizer");
            translations.insert("optimiser", "Optimiser");
            translations.insert("scanner", "Scanner");
            translations.insert("rafraichir", "Rafraîchir");
            translations.insert("aucun_colis", "Aucun colis dans la session");
            translations.insert("veuillez_rafraichir", "Veuillez rafraîchir ou recharger la tournée");
            translations.insert("traitees", "traitées");
            translations.insert("paquets", "paquets");
            translations.insert("oui_capital", "Oui");
            translations.insert("non_capital", "Non");
            translations.insert("marquer_problematique", "Marquer comme problématique");
            translations.insert("problematique", "Problématique");
        }
    }
    
    translations
}

/// Función de traducción
/// 
/// # Arguments
/// * `key` - Clave de traducción
/// * `lang` - Idioma ("ES" o "FR")
/// 
/// # Returns
/// String traducida o la clave si no se encuentra traducción
pub fn t(key: &str, lang: &str) -> String {
    let translations = get_translations(lang);
    
    if let Some(translation) = translations.get(key) {
        return translation.to_string();
    }
    
    // Fallback: devolver la clave si no hay traducción
    key.to_string()
}

