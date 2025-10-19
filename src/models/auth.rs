use serde::{Deserialize, Serialize};

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
pub struct SavedCredentials {
    pub username: String,
    pub password: String,
    pub company: Company,
}

