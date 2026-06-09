use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Request body for initiating OAuth authentication
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({"client_id": "my-client-id"}))]
pub struct AuthInitiateRequest {
  /// The OAuth client_id of the tenant to authenticate with
  #[schema(example = "my-client-id")]
  pub client_id: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({
    "code": "auth_code_123",
    "state": "random_state_456"
}))]
pub struct AuthCallbackRequest {
  /// Required for the success flow
  #[schema(example = "auth_code_123")]
  pub code: Option<String>,
  /// CSRF protection — must match the initiated request
  #[schema(example = "random_state_456")]
  pub state: Option<String>,
  /// OAuth error code if authentication failed (e.g., "access_denied")
  #[schema(example = "access_denied")]
  pub error: Option<String>,
  #[schema(example = "The user denied the request")]
  pub error_description: Option<String>,
  /// Additional OAuth 2.1 parameters sent by the authorization server
  #[serde(flatten)]
  #[schema(additional_properties = true)]
  pub additional_params: HashMap<String, String>,
}
