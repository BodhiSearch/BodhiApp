use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[schema(example = json!({}))]
pub struct EmptyResponse {}

#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[schema(example = json!({
    "location": "https://oauth.example.com/auth?client_id=test&redirect_uri=..."
}))]
pub struct RedirectResponse {
  /// The URL to redirect to (OAuth authorization URL or application home page)
  #[schema(example = "https://oauth.example.com/auth?client_id=test&redirect_uri=...")]
  pub location: String,
}
