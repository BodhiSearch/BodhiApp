use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AppRole;

#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct UserInfo {
  #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
  pub user_id: String,
  #[schema(example = "user@example.com")]
  pub username: String,
  #[schema(example = "John")]
  pub first_name: Option<String>,
  #[schema(example = "Doe")]
  pub last_name: Option<String>,
  #[schema(example = "resource_user")]
  pub role: Option<AppRole>,
}
