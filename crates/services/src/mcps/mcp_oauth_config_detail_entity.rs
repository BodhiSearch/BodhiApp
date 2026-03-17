use crate::mcps::RegistrationType;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_oauth_config_details")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub auth_config_id: String,
  pub tenant_id: String,
  pub registration_type: RegistrationType,
  pub client_id: String,
  pub encrypted_client_secret: Option<String>,
  pub client_secret_salt: Option<String>,
  pub client_secret_nonce: Option<String>,
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  pub registration_endpoint: Option<String>,
  pub encrypted_registration_access_token: Option<String>,
  pub registration_access_token_salt: Option<String>,
  pub registration_access_token_nonce: Option<String>,
  pub client_id_issued_at: Option<DateTime<Utc>>,
  pub token_endpoint_auth_method: Option<String>,
  pub scopes: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::mcp_auth_config_entity::Entity",
    from = "Column::AuthConfigId",
    to = "super::mcp_auth_config_entity::Column::Id"
  )]
  McpAuthConfig,
}

impl Related<super::mcp_auth_config_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpAuthConfig.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// View that excludes encryption salt/nonce fields for domain reads.
/// Includes encrypted_client_secret and encrypted_registration_access_token
/// for is_some() checks only (ciphertext is useless without salt/nonce).
#[derive(Debug, Clone, DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
pub struct McpOAuthConfigDetailView {
  #[allow(dead_code)]
  pub auth_config_id: String,
  pub registration_type: RegistrationType,
  pub client_id: String,
  pub encrypted_client_secret: Option<String>,
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  pub registration_endpoint: Option<String>,
  pub encrypted_registration_access_token: Option<String>,
  pub client_id_issued_at: Option<DateTime<Utc>>,
  pub token_endpoint_auth_method: Option<String>,
  pub scopes: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Type alias — McpOAuthConfigDetailEntity is the entity Model.
pub type McpOAuthConfigDetailEntity = Model;
