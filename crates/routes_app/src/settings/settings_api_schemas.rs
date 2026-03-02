use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to update a setting value
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "value": "debug"
}))]
pub struct UpdateSettingRequest {
  /// New value for the setting (type depends on setting metadata)
  #[schema(example = "debug")]
  pub value: serde_json::Value,
}
