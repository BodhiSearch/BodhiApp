use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({
    "code": "auth_code_123",
    "state": "random_state_456"
}))]
pub struct AuthCallbackRequest {
  /// OAuth authorization code from successful authentication (required for success flow)
  #[schema(example = "auth_code_123")]
  pub code: Option<String>,
  /// OAuth state parameter for CSRF protection (must match initiated request)
  #[schema(example = "random_state_456")]
  pub state: Option<String>,
  /// OAuth error code if authentication failed (e.g., "access_denied")
  #[schema(example = "access_denied")]
  pub error: Option<String>,
  /// Human-readable OAuth error description if authentication failed
  #[schema(example = "The user denied the request")]
  pub error_description: Option<String>,
  /// Additional OAuth 2.1 parameters sent by the authorization server
  #[serde(flatten)]
  #[schema(additional_properties = true)]
  pub additional_params: HashMap<String, String>,
}
