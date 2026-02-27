use chrono::{DateTime, Utc};
use objs::{ContextLimits, ModelArchitecture, ModelCapabilities};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema, DeriveEntityModel)]
#[sea_orm(table_name = "model_metadata")]
#[schema(as = ModelMetadataRow)]
#[cfg_attr(
  any(test, feature = "test-utils"),
  derive(Default, derive_builder::Builder)
)]
#[cfg_attr(
  any(test, feature = "test-utils"),
  builder(setter(into, strip_option), default, build_fn(error = objs::BuilderError))
)]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: String,
  pub source: String,
  pub repo: Option<String>,
  pub filename: Option<String>,
  pub snapshot: Option<String>,
  pub api_model_id: Option<String>,
  #[sea_orm(column_type = "JsonBinary", nullable)]
  pub capabilities: Option<ModelCapabilities>,
  #[sea_orm(column_type = "JsonBinary", nullable)]
  pub context: Option<ContextLimits>,
  #[sea_orm(column_type = "JsonBinary", nullable)]
  pub architecture: Option<ModelArchitecture>,
  pub additional_metadata: Option<String>,
  pub chat_template: Option<String>,
  #[schema(value_type = String, format = "date-time")]
  pub extracted_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

pub type ModelMetadataRow = Model;

#[cfg(any(test, feature = "test-utils"))]
pub type ModelMetadataRowBuilder = ModelBuilder;

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for objs::ModelMetadata {
  fn from(row: Model) -> Self {
    objs::ModelMetadata {
      capabilities: row.capabilities.unwrap_or_else(|| objs::ModelCapabilities {
        vision: None,
        audio: None,
        thinking: None,
        tools: objs::ToolCapabilities {
          function_calling: None,
          structured_output: None,
        },
      }),
      context: row.context.unwrap_or_else(|| objs::ContextLimits {
        max_input_tokens: None,
        max_output_tokens: None,
      }),
      architecture: row.architecture.unwrap_or_else(|| objs::ModelArchitecture {
        family: None,
        parameter_count: None,
        quantization: None,
        format: "gguf".to_string(),
      }),
      chat_template: row.chat_template,
    }
  }
}
