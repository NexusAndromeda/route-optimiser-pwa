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
            translations.insert("notifications_navigateur", "Notificaciones del navegador");
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
            translations.insert("livres", "entregados");
            translations.insert("paquets", "paquetes");
            translations.insert("oui_capital", "Sí");
            translations.insert("non_capital", "No");
            translations.insert("marquer_problematique", "Marcar como problemático");
            translations.insert("problematique", "Problemático");
            translations.insert("traçabilité", "Trazabilidad");

            // Admin Dashboard
            translations.insert("rechercher_tracking", "Buscar tracking");
            translations.insert("rechercher", "Buscar");
            translations.insert("saisir_tracking_rechercher", "Introducir tracking y clicar en Buscar");
            translations.insert("aucun_colis_trouve", "No se encontraron paquetes");
            translations.insert("erreur", "Error");
            translations.insert("admin_dashboard", "Panel Admin");
            translations.insert("buscar_tracking", "Buscar tracking");
            translations.insert("demandes_en_attente", "solicitudes pendientes");
            translations.insert("tableau_bord", "Panel de control");
            translations.insert("chargement_paquets", "Cargando paquetes...");
            translations.insert("demande", "Solicitud");
            translations.insert("chargement", "Cargando...");
            translations.insert("fermer", "Cerrar");
            translations.insert("confirmer", "Confirmar");
            translations.insert("demandes_changement_statut", "Solicitudes de cambio de estado");
            translations.insert("apercu_excel", "Vista previa Excel");
            translations.insert("masquer_apercu", "Ocultar vista previa");
            translations.insert("exporter_excel", "Exportar Excel");
            translations.insert("fermer_le_jour", "Cerrar el día");
            translations.insert("ref_colis", "REF COLIS (en mayúscula)");
            translations.insert("type_livraison", "TIPO DE ENTREGA");
            translations.insert("en_attente", "En espera");
            translations.insert("cliquer_voir_historique_confirmer", "Clic para ver historial y confirmar");
            translations.insert("voulez_fermer_jour", "¿Desea cerrar el día? Las solicitudes confirmadas pasarán a resueltas. Las sesiones de tournées también se eliminarán.");
            translations.insert("export_terminée_fermer", "Exportación terminada. ¿Desea cerrar el día y vaciar la vista previa? Las sesiones de tournées también se eliminarán.");
            translations.insert("necessite_changement_statut", "Requiere cambio de estado");
            translations.insert("type_livraison_label", "Tipo de entrega");
            translations.insert("choisir_type_livraison", "Elija el tipo de entrega para el Excel (C=CLIENT, G=GARDIEN, BAL, A=ACCUEIL, AH=ACCUEIL HOTEL).");
            translations.insert("tournees_colis_format", "{} tournées • {} paquetes");
            translations.insert("tournees", "tournées");
            translations.insert("colis_livres_statut", "{} paquetes • {} entregados • {}");
            translations.insert("signale_par", "Reportado por:");
            translations.insert("notes_label", "Notas:");
            translations.insert("tournee_word", "Tournée");
            translations.insert("paquets_tournee", "Paquetes de la tournée");

            // Login
            translations.insert("optimisation_routes", "Optimización de Rutas de Entrega");
            translations.insert("type_utilisateur", "Tipo de usuario");
            translations.insert("chauffeur", "Conductor");
            translations.insert("admin", "Admin");
            translations.insert("connexion", "Conectar");
            translations.insert("utilisateur", "Usuario");
            translations.insert("utilisateur_placeholder", "Introduzca su nombre de usuario");
            translations.insert("mot_de_passe", "Contraseña");
            translations.insert("mot_de_passe_placeholder", "Introduzca su contraseña");
            translations.insert("entreprise", "Empresa");
            translations.insert("selectionner_entreprise", "Seleccionar empresa");
            translations.insert("buscar_empresa", "Buscar empresa...");
            translations.insert("veuillez_remplir_champs", "Por favor complete todos los campos");
            translations.insert("chargement_entreprises", "Cargando empresas...");
            translations.insert("aucune_entreprise", "No se encontraron empresas");

            // Tracking modal & scanner
            translations.insert("buscar_tracking_title", "Buscar Tracking");
            translations.insert("buscar_tracking_placeholder", "Buscar tracking...");
            translations.insert("aucun_tracking", "No se encontraron trackings");
            translations.insert("scanner_title", "Escanear");

            // App
            translations.insert("cargando_sesion", "Cargando sesión...");
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
            translations.insert("notifications_navigateur", "Notifications du navigateur");
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
            translations.insert("livres", "livrés");
            translations.insert("paquets", "paquets");
            translations.insert("oui_capital", "Oui");
            translations.insert("non_capital", "Non");
            translations.insert("marquer_problematique", "Marquer comme problématique");
            translations.insert("problematique", "Problématique");
            translations.insert("traçabilité", "Traçabilité");

            // Admin Dashboard
            translations.insert("rechercher_tracking", "Rechercher un tracking");
            translations.insert("rechercher", "Rechercher");
            translations.insert("saisir_tracking_rechercher", "Saisir un tracking et cliquer sur Rechercher");
            translations.insert("aucun_colis_trouve", "Aucun colis trouvé");
            translations.insert("erreur", "Erreur");
            translations.insert("admin_dashboard", "Admin Dashboard");
            translations.insert("buscar_tracking", "Rechercher tracking");
            translations.insert("demandes_en_attente", "demandes en attente");
            translations.insert("tableau_bord", "Tableau de bord");
            translations.insert("chargement_paquets", "Chargement des paquets...");
            translations.insert("demande", "Demande");
            translations.insert("chargement", "Chargement...");
            translations.insert("fermer", "Fermer");
            translations.insert("confirmer", "Confirmer");
            translations.insert("demandes_changement_statut", "Demandes de changement de statut");
            translations.insert("apercu_excel", "Aperçu Excel");
            translations.insert("masquer_apercu", "Masquer aperçu");
            translations.insert("exporter_excel", "Exporter Excel");
            translations.insert("fermer_le_jour", "Fermer le jour");
            translations.insert("ref_colis", "REF COLIS (en majuscule)");
            translations.insert("type_livraison", "TYPE DE LIVRAISON");
            translations.insert("en_attente", "En attente");
            translations.insert("cliquer_voir_historique_confirmer", "Cliquer pour voir l'historique et confirmer");
            translations.insert("voulez_fermer_jour", "Voulez-vous fermer le jour ? Les demandes confirmées passeront à résolues. Les sessions de tournées seront également supprimées.");
            translations.insert("export_terminée_fermer", "Exportation terminée. Voulez-vous fermer le jour et vider l'aperçu ? Les sessions de tournées seront également supprimées.");
            translations.insert("necessite_changement_statut", "Nécessite changement de statut");
            translations.insert("type_livraison_label", "Type de livraison");
            translations.insert("choisir_type_livraison", "Choisissez le type de livraison pour l'Excel (C=CLIENT, G=GARDIEN, BAL, A=ACCUEIL, AH=ACCUEIL HOTEL).");
            translations.insert("tournees_colis_format", "{} tournées • {} colis");
            translations.insert("tournees", "tournées");
            translations.insert("colis_livres_statut", "{} colis • {} livrés • {}");
            translations.insert("signale_par", "Signalé par:");
            translations.insert("notes_label", "Notes:");
            translations.insert("tournee_word", "Tournée");
            translations.insert("paquets_tournee", "Paquets de la tournée");

            // Login
            translations.insert("optimisation_routes", "Optimisation de Routes de Livraison");
            translations.insert("type_utilisateur", "Type d'utilisateur");
            translations.insert("chauffeur", "Chauffeur");
            translations.insert("admin", "Admin");
            translations.insert("connexion", "Se connecter");
            translations.insert("utilisateur", "Utilisateur");
            translations.insert("utilisateur_placeholder", "Entrez votre nom d'utilisateur");
            translations.insert("mot_de_passe", "Mot de passe");
            translations.insert("mot_de_passe_placeholder", "Entrez votre mot de passe");
            translations.insert("entreprise", "Entreprise");
            translations.insert("selectionner_entreprise", "Sélectionner l'entreprise");
            translations.insert("buscar_empresa", "Rechercher entreprise...");
            translations.insert("veuillez_remplir_champs", "Veuillez remplir tous les champs");
            translations.insert("chargement_entreprises", "Chargement des entreprises...");
            translations.insert("aucune_entreprise", "Aucune entreprise trouvée");

            // Tracking modal & scanner
            translations.insert("buscar_tracking_title", "Rechercher Tracking");
            translations.insert("buscar_tracking_placeholder", "Rechercher un tracking...");
            translations.insert("aucun_tracking", "Aucun tracking trouvé");
            translations.insert("scanner_title", "Scanner");

            // App
            translations.insert("cargando_sesion", "Chargement de la session...");
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

