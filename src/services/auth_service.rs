use gloo_net::http::Request;
use crate::models::{Company, CompaniesResponse, LoginRequest, LoginResponse};
use crate::utils::BACKEND_URL;

/// Load available companies from the backend
pub async fn load_companies() -> Result<Vec<Company>, String> {
    let url = format!("{}/api/colis-prive/companies", BACKEND_URL);
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
    
    Ok(companies_response.companies)
}

/// Perform login with username, password and company code
pub async fn perform_login(username: &str, password: &str, societe: &str) -> Result<LoginResponse, String> {
    let url = format!("{}/api/colis-prive/auth", BACKEND_URL);
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
    let url = format!("{}/api/company/register", BACKEND_URL);
    
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

