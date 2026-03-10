use serde::{Deserialize, Serialize};

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
