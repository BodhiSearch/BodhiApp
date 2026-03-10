use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentMode {
  #[default]
  Standalone,
  MultiTenant,
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
  /// User must select a tenant
  #[schema(rename = "tenant_selection")]
  TenantSelection,
}

/// Tenant represents an OAuth2 client registration (the "app instance") for this deployment.
/// In standalone mode there is at most one tenant; in multi-tenant mode there can be many.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tenant {
  pub id: String,
  pub client_id: String,
  pub client_secret: String,
  pub name: String,
  pub description: Option<String>,
  pub status: AppStatus,
  pub created_by: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<super::tenant_entity::TenantRow> for Tenant {
  fn from(row: super::tenant_entity::TenantRow) -> Self {
    Tenant {
      id: row.id,
      client_id: row.client_id,
      client_secret: row.client_secret,
      name: row.name,
      description: row.description,
      status: row.app_status,
      created_by: row.created_by,
      created_at: row.created_at,
      updated_at: row.updated_at,
    }
  }
}
