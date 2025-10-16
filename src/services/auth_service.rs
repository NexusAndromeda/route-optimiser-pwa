use gloo_net::http::Request;
use crate::models::{Company, CompaniesResponse, LoginRequest, LoginResponse};
use crate::utils::{BACKEND_URL, save_to_storage, load_from_storage};

const COMPANIES_CACHE_KEY: &str = "routeOptimizer_companies";
const COMPANIES_CACHE_DURATION_HOURS: i64 = 24; // Cache por 24 horas

#[derive(serde::Serialize, serde::Deserialize)]
struct CompaniesCache {
    companies: Vec<Company>,
    timestamp: String,
}

/// Load available companies from the backend (with cache)
pub async fn load_companies() -> Result<Vec<Company>, String> {
    // Check cache first
    if let Some(cache) = load_from_storage::<CompaniesCache>(COMPANIES_CACHE_KEY) {
        if let Ok(cache_time) = chrono::DateTime::parse_from_rfc3339(&cache.timestamp) {
            let now = chrono::Utc::now();
            let cache_age = now.signed_duration_since(cache_time.with_timezone(&chrono::Utc));
            let cache_age_hours = cache_age.num_hours();
            
            if cache_age_hours < COMPANIES_CACHE_DURATION_HOURS {
                log::info!("📋 Usando empresas del cache ({} horas de antigüedad)", cache_age_hours);
                return Ok(cache.companies);
            } else {
                log::info!("📋 Cache de empresas expirado, obteniendo datos frescos...");
            }
        }
    }
    
    // Fetch from API
    log::info!("📋 Obteniendo lista de empresas del servidor...");
    let url = format!("{}/colis-prive/companies", BACKEND_URL);
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    let companies_response = response
        .json::<CompaniesResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let companies = companies_response.companies;
    
    // Save to cache
    let cache = CompaniesCache {
        companies: companies.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    save_to_storage(COMPANIES_CACHE_KEY, &cache);
    log::info!("💾 {} empresas guardadas en cache", companies.len());
    
    Ok(companies)
}

/// Perform login with username, password and company code
pub async fn perform_login(username: &str, password: &str, societe: &str) -> Result<LoginResponse, String> {
    let url = format!("{}/colis-prive/auth", BACKEND_URL);
    let request_body = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
        societe: societe.to_string(),
    };
    
    let response = Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    response
        .json::<LoginResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Register a new company
pub async fn register_company(
    company_name: String,
    company_address: String,
    company_siret: Option<String>,
    admin_full_name: String,
    admin_email: String,
    admin_password: String,
) -> Result<(), String> {
    let url = format!("{}/company/register", BACKEND_URL);
    
    let response = Request::post(&url)
        .json(&serde_json::json!({
            "company_name": company_name,
            "company_address": company_address,
            "company_siret": company_siret,
            "admin_full_name": admin_full_name,
            "admin_email": admin_email,
            "admin_password": admin_password,
        }))
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;
    
    if !response.ok() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("HTTP error {}: {}", status, error_text));
    }
    
    Ok(())
}

