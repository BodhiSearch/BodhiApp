use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, derive_builder::Builder)]
pub struct AppRegInfo {
  pub client_id: String,
  pub client_secret: String,
}

#[derive(
  Debug,
  Serialize,
  Deserialize,
  PartialEq,
  strum::Display,
  Clone,
  Default,
  strum::EnumString,
  ToSchema,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[schema(example = "ready")]
pub enum AppStatus {
  #[default]
  /// Initial setup required
  #[schema(rename = "setup")]
  Setup,
  /// Application is ready
  #[schema(rename = "ready")]
  Ready,
  /// Admin setup required
  #[schema(rename = "resource-admin")]
  ResourceAdmin,
}
