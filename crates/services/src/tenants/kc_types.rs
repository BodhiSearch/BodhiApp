use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KcTenant {
  pub client_id: String,
  pub name: String,
  #[serde(default)]
  pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KcTenantListResponse {
  pub tenants: Vec<KcTenant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantRequest {
  pub name: String,
  pub description: String,
  pub redirect_uris: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KcCreateTenantResponse {
  pub client_id: String,
  pub client_secret: String,
}
