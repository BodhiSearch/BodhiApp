use chrono::{DateTime, Utc};
pub use objs::AppStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppInstance {
  pub client_id: String,
  pub client_secret: String,
  pub status: AppStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}
