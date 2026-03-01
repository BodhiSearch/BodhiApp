use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// UserAccessRequestStatus - Status for user-initiated access requests
// ============================================================================

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
pub enum UserAccessRequestStatus {
  Pending,
  Approved,
  Rejected,
}
