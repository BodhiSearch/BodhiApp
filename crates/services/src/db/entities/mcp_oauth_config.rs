use chrono::{DateTime, Utc};
use objs::RegistrationType;
use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_oauth_configs")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub name: String,
  pub mcp_server_id: String,
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
  pub created_by: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::mcp_server::Entity",
    from = "Column::McpServerId",
    to = "super::mcp_server::Column::Id"
  )]
  McpServer,
  #[sea_orm(has_many = "super::mcp_oauth_token::Entity")]
  McpOAuthToken,
}

impl Related<super::mcp_server::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpServer.def()
  }
}

impl Related<super::mcp_oauth_token::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::McpOAuthToken.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// View that excludes encryption salt/nonce fields for domain reads.
/// Includes encrypted_client_secret and encrypted_registration_access_token
/// for is_some() checks only (ciphertext is useless without salt/nonce).
#[derive(Debug, Clone, DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
pub struct McpOAuthConfigView {
  pub id: String,
  pub name: String,
  pub mcp_server_id: String,
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
  pub created_by: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<McpOAuthConfigView> for objs::McpOAuthConfig {
  fn from(v: McpOAuthConfigView) -> Self {
    objs::McpOAuthConfig {
      id: v.id,
      name: v.name,
      mcp_server_id: v.mcp_server_id,
      registration_type: v.registration_type,
      client_id: v.client_id,
      authorization_endpoint: v.authorization_endpoint,
      token_endpoint: v.token_endpoint,
      registration_endpoint: v.registration_endpoint,
      client_id_issued_at: v.client_id_issued_at.map(|dt| dt.timestamp()),
      token_endpoint_auth_method: v.token_endpoint_auth_method,
      scopes: v.scopes,
      has_client_secret: v.encrypted_client_secret.is_some(),
      has_registration_access_token: v.encrypted_registration_access_token.is_some(),
      created_by: v.created_by,
      created_at: v.created_at,
      updated_at: v.updated_at,
    }
  }
}

impl From<Model> for crate::db::McpOAuthConfigRow {
  fn from(m: Model) -> Self {
    crate::db::McpOAuthConfigRow {
      id: m.id,
      name: m.name,
      mcp_server_id: m.mcp_server_id,
      registration_type: m.registration_type,
      client_id: m.client_id,
      encrypted_client_secret: m.encrypted_client_secret,
      client_secret_salt: m.client_secret_salt,
      client_secret_nonce: m.client_secret_nonce,
      authorization_endpoint: m.authorization_endpoint,
      token_endpoint: m.token_endpoint,
      registration_endpoint: m.registration_endpoint,
      encrypted_registration_access_token: m.encrypted_registration_access_token,
      registration_access_token_salt: m.registration_access_token_salt,
      registration_access_token_nonce: m.registration_access_token_nonce,
      client_id_issued_at: m.client_id_issued_at,
      token_endpoint_auth_method: m.token_endpoint_auth_method,
      scopes: m.scopes,
      created_by: m.created_by,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
