use crate::app_access_requests::access_request_objs::AppAccessRequestRow;
use crate::{AppAccessRequestStatus, FlowType};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "app_access_requests")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub app_client_id: String,
  pub app_name: Option<String>,
  pub app_description: Option<String>,
  pub flow_type: FlowType,
  pub redirect_uri: Option<String>,
  pub status: AppAccessRequestStatus,
  pub requested: String,
  pub approved: Option<String>,
  pub user_id: Option<String>,
  pub requested_role: String,
  pub approved_role: Option<String>,
  pub access_request_scope: Option<String>,
  pub error_message: Option<String>,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for AppAccessRequestRow {
  fn from(m: Model) -> Self {
    AppAccessRequestRow {
      id: m.id,
      app_client_id: m.app_client_id,
      app_name: m.app_name,
      app_description: m.app_description,
      flow_type: m.flow_type,
      redirect_uri: m.redirect_uri,
      status: m.status,
      requested: m.requested,
      approved: m.approved,
      user_id: m.user_id,
      requested_role: m.requested_role,
      approved_role: m.approved_role,
      access_request_scope: m.access_request_scope,
      error_message: m.error_message,
      expires_at: m.expires_at,
      created_at: m.created_at,
      updated_at: m.updated_at,
    }
  }
}
