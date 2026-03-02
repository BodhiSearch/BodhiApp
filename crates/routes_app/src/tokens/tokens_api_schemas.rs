use serde::{Deserialize, Serialize};
use services::ApiToken;
use services::TokenScope;
use services::TokenStatus;
use utoipa::ToSchema;

/// Request to create a new API token
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "My Integration Token",
    "scope": "scope_token_user"
}))]
pub struct CreateApiTokenRequest {
  /// Descriptive name for the API token (minimum 3 characters)
  #[serde(default)]
  #[schema(min_length = 3, max_length = 100, example = "My Integration Token")]
  pub name: Option<String>,
  /// Token scope defining access level
  #[schema(example = "scope_token_user")]
  pub scope: TokenScope,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "token": "bodhiapp_1234567890abcdef"
}))]
pub struct ApiTokenResponse {
  /// API token with bodhiapp_ prefix for programmatic access
  #[schema(example = "bodhiapp_1234567890abcdef")]
  pub(crate) token: String,
}

/// Paginated list of API tokens
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedApiTokenResponse {
  pub data: Vec<ApiToken>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

/// Request to update an existing API token
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "Updated Token Name",
    "status": "inactive"
}))]
pub struct UpdateApiTokenRequest {
  /// New descriptive name for the token (minimum 3 characters)
  #[schema(min_length = 3, max_length = 100, example = "Updated Token Name")]
  pub name: String,
  /// New status for the token (active/inactive)
  #[schema(example = "inactive")]
  pub status: TokenStatus,
}
