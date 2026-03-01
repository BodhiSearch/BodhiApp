use crate::db::{DbError, DefaultDbService};
use crate::models::model_metadata_entity as model_metadata;
use crate::models::{AliasSource, ModelMetadataRow};
use sea_orm::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{Condition, QueryOrder, Set};

#[async_trait::async_trait]
pub trait ModelMetadataRepository: Send + Sync {
  async fn upsert_model_metadata(&self, metadata: &ModelMetadataRow) -> Result<(), DbError>;

  async fn get_model_metadata_by_file(
    &self,
    repo: &str,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<ModelMetadataRow>, DbError>;

  async fn batch_get_metadata_by_files(
    &self,
    files: &[(String, String, String)],
  ) -> Result<std::collections::HashMap<(String, String, String), ModelMetadataRow>, DbError>;

  async fn list_model_metadata(&self) -> Result<Vec<ModelMetadataRow>, DbError>;
}

#[async_trait::async_trait]
impl ModelMetadataRepository for DefaultDbService {
  async fn upsert_model_metadata(&self, metadata: &ModelMetadataRow) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    let id = if metadata.id.is_empty() {
      ulid::Ulid::new().to_string()
    } else {
      metadata.id.clone()
    };

    // When api_model_id is NULL, we use delete-then-insert instead of ON CONFLICT upsert.
    // SQL composite unique indexes treat each NULL as distinct, so a row with
    // api_model_id=NULL never conflicts with another NULL row. We must manually
    // delete the matching record first, then insert the replacement.
    if metadata.api_model_id.is_none() {
      let mut delete_query = model_metadata::Entity::delete_many()
        .filter(model_metadata::Column::Source.eq(metadata.source))
        .filter(model_metadata::Column::ApiModelId.is_null());

      if let Some(ref repo) = metadata.repo {
        delete_query = delete_query.filter(model_metadata::Column::Repo.eq(repo.clone()));
      } else {
        delete_query = delete_query.filter(model_metadata::Column::Repo.is_null());
      }
      if let Some(ref filename) = metadata.filename {
        delete_query = delete_query.filter(model_metadata::Column::Filename.eq(filename.clone()));
      } else {
        delete_query = delete_query.filter(model_metadata::Column::Filename.is_null());
      }
      if let Some(ref snapshot) = metadata.snapshot {
        delete_query = delete_query.filter(model_metadata::Column::Snapshot.eq(snapshot.clone()));
      } else {
        delete_query = delete_query.filter(model_metadata::Column::Snapshot.is_null());
      }

      delete_query.exec(&self.db).await.map_err(DbError::from)?;
    }

    let model = model_metadata::ActiveModel {
      id: Set(id),
      source: Set(metadata.source),
      repo: Set(metadata.repo.clone()),
      filename: Set(metadata.filename.clone()),
      snapshot: Set(metadata.snapshot.clone()),
      api_model_id: Set(metadata.api_model_id.clone()),
      capabilities: Set(metadata.capabilities.clone()),
      context: Set(metadata.context.clone()),
      architecture: Set(metadata.architecture.clone()),
      additional_metadata: Set(metadata.additional_metadata.clone()),
      chat_template: Set(metadata.chat_template.clone()),
      extracted_at: Set(metadata.extracted_at),
      created_at: Set(now),
      updated_at: Set(now),
    };

    if metadata.api_model_id.is_some() {
      model_metadata::Entity::insert(model)
        .on_conflict(
          OnConflict::columns([
            model_metadata::Column::Source,
            model_metadata::Column::Repo,
            model_metadata::Column::Filename,
            model_metadata::Column::Snapshot,
            model_metadata::Column::ApiModelId,
          ])
          .update_columns([
            model_metadata::Column::Capabilities,
            model_metadata::Column::Context,
            model_metadata::Column::Architecture,
            model_metadata::Column::AdditionalMetadata,
            model_metadata::Column::ChatTemplate,
            model_metadata::Column::ExtractedAt,
            model_metadata::Column::UpdatedAt,
          ])
          .to_owned(),
        )
        .exec(&self.db)
        .await
        .map_err(DbError::from)?;
    } else {
      model_metadata::Entity::insert(model)
        .exec(&self.db)
        .await
        .map_err(DbError::from)?;
    }

    Ok(())
  }

  async fn get_model_metadata_by_file(
    &self,
    repo: &str,
    filename: &str,
    snapshot: &str,
  ) -> Result<Option<ModelMetadataRow>, DbError> {
    let result = model_metadata::Entity::find()
      .filter(model_metadata::Column::Source.eq(AliasSource::Model))
      .filter(model_metadata::Column::Repo.eq(repo))
      .filter(model_metadata::Column::Filename.eq(filename))
      .filter(model_metadata::Column::Snapshot.eq(snapshot))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result)
  }

  async fn batch_get_metadata_by_files(
    &self,
    files: &[(String, String, String)],
  ) -> Result<std::collections::HashMap<(String, String, String), ModelMetadataRow>, DbError> {
    use std::collections::HashMap;

    if files.is_empty() {
      return Ok(HashMap::new());
    }

    let mut condition = Condition::any();
    for (repo, filename, snapshot) in files {
      condition = condition.add(
        Condition::all()
          .add(model_metadata::Column::Repo.eq(repo.clone()))
          .add(model_metadata::Column::Filename.eq(filename.clone()))
          .add(model_metadata::Column::Snapshot.eq(snapshot.clone())),
      );
    }

    let results = model_metadata::Entity::find()
      .filter(model_metadata::Column::Source.eq(AliasSource::Model))
      .filter(condition)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    let mut map = HashMap::new();
    for m in results {
      if let (Some(repo), Some(filename), Some(snapshot)) =
        (m.repo.clone(), m.filename.clone(), m.snapshot.clone())
      {
        map.insert((repo, filename, snapshot), m);
      }
    }

    Ok(map)
  }

  async fn list_model_metadata(&self) -> Result<Vec<ModelMetadataRow>, DbError> {
    let results = model_metadata::Entity::find()
      .order_by_asc(model_metadata::Column::Source)
      .order_by_asc(model_metadata::Column::Repo)
      .order_by_asc(model_metadata::Column::Filename)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(results)
  }
}
