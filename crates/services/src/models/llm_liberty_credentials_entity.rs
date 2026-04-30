use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "api_model_oauth_credentials")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub api_alias_id: String,
  pub tenant_id: String,
  pub user_id: String,
  pub envelope_version: String,
  pub provider: String,
  pub encrypted_access_token: String,
  pub access_salt: String,
  pub access_nonce: String,
  pub encrypted_refresh_token: String,
  pub refresh_salt: String,
  pub refresh_nonce: String,
  pub expires_at: DateTime<Utc>,
  pub auth_in: String,
  pub auth_key: String,
  pub auth_scheme: String,
  pub oauth_authorize_url: String,
  pub oauth_token_url: String,
  pub oauth_revoke_url: Option<String>,
  pub oauth_client_id: String,
  pub oauth_client_secret: Option<String>,
  pub api_base_url: String,
  pub api_chat_url: String,
  pub api_models_url: Option<String>,
  pub headers_json: Json,
  pub body_json: Json,
  pub extra_json: Option<Json>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::api_model_alias_entity::Entity",
    from = "Column::ApiAliasId",
    to = "super::api_model_alias_entity::Column::Id",
    on_delete = "Cascade",
    on_update = "Cascade"
  )]
  ApiModelAlias,
}

impl Related<super::api_model_alias_entity::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::ApiModelAlias.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// Read-only view excluding encryption fields.
#[derive(Debug, Clone, DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Entity")]
pub struct LlmLibertyCredentialsView {
  pub tenant_id: String,
  pub user_id: String,
  pub envelope_version: String,
  pub provider: String,
  pub expires_at: DateTime<Utc>,
}

pub type LlmLibertyCredentialsEntity = Model;
