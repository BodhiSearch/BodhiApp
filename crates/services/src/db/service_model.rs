use crate::db::{
  encryption::{decrypt_api_key, encrypt_api_key},
  entities::{api_model_alias, download_request, model_metadata},
  ApiKeyUpdate, DbError, DefaultDbService, DownloadRequest, ModelMetadataRow, ModelRepository,
};
use chrono::{DateTime, Utc};
use objs::ApiAlias;
use sea_orm::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{Condition, NotSet, QueryOrder, QuerySelect, Set};

use super::entities::api_model_alias::ApiAliasView;

#[async_trait::async_trait]
impl ModelRepository for DefaultDbService {
  async fn create_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    let model = download_request::ActiveModel {
      id: Set(request.id.clone()),
      repo: Set(request.repo.clone()),
      filename: Set(request.filename.clone()),
      status: Set(request.status.clone()),
      error: Set(request.error.clone()),
      total_bytes: Set(request.total_bytes),
      downloaded_bytes: Set(request.downloaded_bytes),
      started_at: Set(request.started_at),
      created_at: Set(request.created_at),
      updated_at: Set(request.updated_at),
    };
    download_request::Entity::insert(model)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn get_download_request(&self, id: &str) -> Result<Option<DownloadRequest>, DbError> {
    let result = download_request::Entity::find_by_id(id.to_string())
      .one(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(result)
  }

  async fn update_download_request(&self, request: &DownloadRequest) -> Result<(), DbError> {
    let model = download_request::ActiveModel {
      id: Set(request.id.clone()),
      repo: NotSet,
      filename: NotSet,
      status: Set(request.status.clone()),
      error: Set(request.error.clone()),
      total_bytes: Set(request.total_bytes),
      downloaded_bytes: Set(request.downloaded_bytes),
      started_at: Set(request.started_at),
      created_at: NotSet,
      updated_at: Set(request.updated_at),
    };
    download_request::Entity::update(model)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn list_download_requests(
    &self,
    page: usize,
    page_size: usize,
  ) -> Result<(Vec<DownloadRequest>, usize), DbError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);

    let total = download_request::Entity::find()
      .count(&self.db)
      .await
      .map_err(DbError::from)? as usize;

    let results = download_request::Entity::find()
      .order_by_desc(download_request::Column::UpdatedAt)
      .offset(((page - 1) * page_size) as u64)
      .limit(page_size as u64)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok((results, total))
  }

  async fn find_download_request_by_repo_filename(
    &self,
    repo: &str,
    filename: &str,
  ) -> Result<Vec<DownloadRequest>, DbError> {
    let results = download_request::Entity::find()
      .filter(download_request::Column::Repo.eq(repo))
      .filter(download_request::Column::Filename.eq(filename))
      .order_by_desc(download_request::Column::CreatedAt)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(results)
  }

  async fn create_api_model_alias(
    &self,
    alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<(), DbError> {
    if let Some(ref prefix) = alias.prefix {
      if !prefix.is_empty() && self.check_prefix_exists(prefix, None).await? {
        return Err(DbError::PrefixExists(prefix.clone()));
      }
    }

    let (encrypted_api_key, salt, nonce) = if let Some(ref key) = api_key {
      let (enc, s, n) = encrypt_api_key(&self.encryption_key, key)
        .map_err(|e| DbError::EncryptionError(e.to_string()))?;
      (Some(enc), Some(s), Some(n))
    } else {
      (None, None, None)
    };

    let model = api_model_alias::ActiveModel {
      id: Set(alias.id.clone()),
      api_format: Set(alias.api_format.clone()),
      base_url: Set(alias.base_url.clone()),
      models: Set(alias.models.clone()),
      prefix: Set(alias.prefix.clone()),
      forward_all_with_prefix: Set(alias.forward_all_with_prefix),
      models_cache: Set(alias.models_cache.clone()),
      cache_fetched_at: Set(alias.cache_fetched_at),
      encrypted_api_key: Set(encrypted_api_key),
      salt: Set(salt),
      nonce: Set(nonce),
      created_at: Set(alias.created_at),
      updated_at: Set(alias.updated_at),
    };
    api_model_alias::Entity::insert(model)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn get_api_model_alias(&self, id: &str) -> Result<Option<ApiAlias>, DbError> {
    let result = api_model_alias::Entity::find_by_id(id.to_string())
      .into_partial_model::<ApiAliasView>()
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result.map(Into::into))
  }

  async fn update_api_model_alias(
    &self,
    id: &str,
    model: &ApiAlias,
    api_key: ApiKeyUpdate,
  ) -> Result<(), DbError> {
    if let Some(ref prefix) = model.prefix {
      if !prefix.is_empty()
        && self
          .check_prefix_exists(prefix, Some(id.to_string()))
          .await?
      {
        return Err(DbError::PrefixExists(prefix.clone()));
      }
    }

    let now = self.time_service.utc_now();

    let mut active: api_model_alias::ActiveModel = Default::default();
    active.id = Set(id.to_string());
    active.api_format = Set(model.api_format.clone());
    active.base_url = Set(model.base_url.clone());
    active.models = Set(model.models.clone());
    active.prefix = Set(model.prefix.clone());
    active.forward_all_with_prefix = Set(model.forward_all_with_prefix);
    active.updated_at = Set(now);

    match api_key {
      ApiKeyUpdate::Set(api_key_opt) => match api_key_opt {
        Some(api_key) => {
          let (encrypted, s, n) = encrypt_api_key(&self.encryption_key, &api_key)
            .map_err(|e| DbError::EncryptionError(e.to_string()))?;
          active.encrypted_api_key = Set(Some(encrypted));
          active.salt = Set(Some(s));
          active.nonce = Set(Some(n));
        }
        None => {
          active.encrypted_api_key = Set(None);
          active.salt = Set(None);
          active.nonce = Set(None);
        }
      },
      ApiKeyUpdate::Keep => {}
    }

    api_model_alias::Entity::update(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn update_api_model_cache(
    &self,
    id: &str,
    models: Vec<String>,
    fetched_at: DateTime<Utc>,
  ) -> Result<(), DbError> {
    let mut active: api_model_alias::ActiveModel = Default::default();
    active.id = Set(id.to_string());
    active.models_cache = Set(models.into());
    active.cache_fetched_at = Set(fetched_at);

    api_model_alias::Entity::update(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn delete_api_model_alias(&self, id: &str) -> Result<(), DbError> {
    api_model_alias::Entity::delete_by_id(id.to_string())
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;
    Ok(())
  }

  async fn list_api_model_aliases(&self) -> Result<Vec<ApiAlias>, DbError> {
    let results = api_model_alias::Entity::find()
      .order_by_desc(api_model_alias::Column::CreatedAt)
      .into_partial_model::<ApiAliasView>()
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(results.into_iter().map(Into::into).collect())
  }

  async fn get_api_key_for_alias(&self, id: &str) -> Result<Option<String>, DbError> {
    let result = api_model_alias::Entity::find_by_id(id.to_string())
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    match result {
      Some(m) => match (m.encrypted_api_key, m.salt, m.nonce) {
        (Some(encrypted), Some(salt), Some(nonce)) => {
          let api_key = decrypt_api_key(&self.encryption_key, &encrypted, &salt, &nonce)
            .map_err(|e| DbError::EncryptionError(e.to_string()))?;
          Ok(Some(api_key))
        }
        (None, None, None) => Ok(None),
        _ => Err(DbError::EncryptionError(format!(
          "Data corruption: API key encryption fields are partially NULL for alias '{}'",
          id
        ))),
      },
      None => Ok(None),
    }
  }

  async fn check_prefix_exists(
    &self,
    prefix: &str,
    exclude_id: Option<String>,
  ) -> Result<bool, DbError> {
    let mut query =
      api_model_alias::Entity::find().filter(api_model_alias::Column::Prefix.eq(prefix));

    if let Some(id) = exclude_id {
      query = query.filter(api_model_alias::Column::Id.ne(id));
    }

    let count = query.count(&self.db).await.map_err(DbError::from)?;
    Ok(count > 0)
  }

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
        .filter(model_metadata::Column::Source.eq(metadata.source.clone()))
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
      source: Set(metadata.source.clone()),
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
      .filter(model_metadata::Column::Source.eq(objs::AliasSource::Model.to_string()))
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
      .filter(model_metadata::Column::Source.eq(objs::AliasSource::Model.to_string()))
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
