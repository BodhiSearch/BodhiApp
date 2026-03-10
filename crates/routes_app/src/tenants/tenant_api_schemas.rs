use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TenantListItem {
  pub client_id: String,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub status: services::AppStatus,
  pub is_active: bool,
  pub logged_in: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TenantListResponse {
  pub tenants: Vec<TenantListItem>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateTenantRequest {
  #[validate(length(min = 1, max = 255))]
  pub name: String,
  #[serde(default)]
  #[validate(length(max = 1000))]
  pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTenantResponse {
  pub client_id: String,
}
