use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  strum::EnumString,
  strum::Display,
  PartialEq,
  ToSchema,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DownloadStatus {
  Pending,
  Completed,
  Error,
}

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  strum::EnumString,
  strum::Display,
  PartialEq,
  ToSchema,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TokenStatus {
  Active,
  Inactive,
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
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
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
  #[schema(rename = "resource_admin")]
  ResourceAdmin,
}
