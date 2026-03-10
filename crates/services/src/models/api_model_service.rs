use async_trait::async_trait;
use std::sync::Arc;

use crate::db::{DbError, DbService, TimeService};
use crate::models::{
  ApiAlias, ApiKeyUpdate, ApiModelOutput, ApiModelRequest, PaginatedApiModelOutput,
};
use crate::new_ulid;
use crate::AiApiService;
use errmeta::{AppError, EntityError, ErrorType};

// =============================================================================
// ApiModelServiceError
// =============================================================================

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ApiModelServiceError {
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::BadRequest, code = "api_model_service_error-validation")]
  Validation(#[from] crate::ObjValidationError),

  #[error(transparent)]
  #[error_meta(args_delegate = false)]
  Db(#[from] DbError),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::NotFound, code = "api_model_service_error-not_found")]
  NotFound(#[from] EntityError),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication, code = "api_model_service_error-auth")]
  Auth(#[from] crate::auth::AuthContextError),

  #[error("AI API error: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  AiApi(String),
}

// =============================================================================
// ApiModelService trait
// =============================================================================

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait ApiModelService: Send + Sync + std::fmt::Debug {
  /// Create a new API model configuration
  async fn create(
    &self,
    tenant_id: &str,
    user_id: &str,
    form: ApiModelRequest,
  ) -> Result<ApiModelOutput, ApiModelServiceError>;

  /// Update an existing API model configuration
  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    form: ApiModelRequest,
  ) -> Result<ApiModelOutput, ApiModelServiceError>;

  /// Delete an API model configuration
  async fn delete(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), ApiModelServiceError>;

  /// Get a specific API model configuration
  async fn get(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<ApiModelOutput, ApiModelServiceError>;

  /// List API model configurations with pagination
  async fn list(
    &self,
    tenant_id: &str,
    user_id: &str,
    page: usize,
    page_size: usize,
  ) -> Result<PaginatedApiModelOutput, ApiModelServiceError>;

  /// Synchronously fetch and cache models for an API model alias
  async fn sync_cache(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<ApiModelOutput, ApiModelServiceError>;
}

// =============================================================================
// DefaultApiModelService
// =============================================================================

#[derive(Debug, derive_new::new)]
pub struct DefaultApiModelService {
  db_service: Arc<dyn DbService>,
  time_service: Arc<dyn TimeService>,
  ai_api_service: Arc<dyn AiApiService>,
}

#[async_trait]
impl ApiModelService for DefaultApiModelService {
  async fn create(
    &self,
    tenant_id: &str,
    user_id: &str,
    form: ApiModelRequest,
  ) -> Result<ApiModelOutput, ApiModelServiceError> {
    validate_forward_all(&form)?;

    let now = self.time_service.utc_now();
    let id = new_ulid();

    // Reset models to empty if forward_all_with_prefix is true
    let models = if form.forward_all_with_prefix {
      Vec::new()
    } else {
      form.models
    };

    let api_alias = ApiAlias::new(
      id,
      form.api_format,
      form.base_url.trim_end_matches('/').to_string(),
      models,
      form.prefix,
      form.forward_all_with_prefix,
      now,
    );

    // Extract API key from form
    let api_key_option = match form.api_key {
      ApiKeyUpdate::Set(key) => key.into_inner(),
      ApiKeyUpdate::Keep => None, // For create, Keep means no key
    };

    self
      .db_service
      .create_api_model_alias(tenant_id, user_id, &api_alias, api_key_option)
      .await?;

    let has_api_key = self
      .db_service
      .get_api_key_for_alias(tenant_id, user_id, &api_alias.id)
      .await?
      .is_some();

    // NOTE: For forward_all models, cache population should be handled by a proper
    // async job/queue system. The previous spawn_cache_refresh has been removed as
    // it was a fire-and-forget pattern that couldn't be properly tested or monitored.

    Ok(ApiModelOutput::from_alias(api_alias, has_api_key))
  }

  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    form: ApiModelRequest,
  ) -> Result<ApiModelOutput, ApiModelServiceError> {
    validate_forward_all(&form)?;

    // Get existing API model
    let mut api_alias = self
      .db_service
      .get_api_model_alias(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| EntityError::NotFound(format!("API model '{}' not found", id)))?;

    // Update all fields
    api_alias.api_format = form.api_format;
    api_alias.base_url = form.base_url.trim_end_matches('/').to_string();
    api_alias.models = form.models.into();
    api_alias.prefix = if form.prefix.as_ref().is_some_and(|p| p.is_empty()) {
      None
    } else {
      form.prefix
    };
    api_alias.forward_all_with_prefix = form.forward_all_with_prefix;
    api_alias.updated_at = self.time_service.utc_now();

    // Convert ApiKeyUpdate to the raw form for repository
    let api_key_update = form.api_key.into_raw_update();

    self
      .db_service
      .update_api_model_alias(tenant_id, user_id, id, &api_alias, api_key_update)
      .await?;

    // Check if API key exists after update
    let has_api_key = self
      .db_service
      .get_api_key_for_alias(tenant_id, user_id, id)
      .await?
      .is_some();

    Ok(ApiModelOutput::from_alias(api_alias, has_api_key))
  }

  async fn delete(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<(), ApiModelServiceError> {
    // Check if API model exists
    if self
      .db_service
      .get_api_model_alias(tenant_id, user_id, id)
      .await?
      .is_none()
    {
      return Err(ApiModelServiceError::NotFound(EntityError::NotFound(
        format!("API model '{}' not found", id),
      )));
    }

    self
      .db_service
      .delete_api_model_alias(tenant_id, user_id, id)
      .await?;

    Ok(())
  }

  async fn get(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<ApiModelOutput, ApiModelServiceError> {
    let api_alias = self
      .db_service
      .get_api_model_alias(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| EntityError::NotFound(format!("API model '{}' not found", id)))?;

    let has_api_key = self
      .db_service
      .get_api_key_for_alias(tenant_id, user_id, id)
      .await?
      .is_some();

    Ok(ApiModelOutput::from_alias(api_alias, has_api_key))
  }

  async fn list(
    &self,
    tenant_id: &str,
    user_id: &str,
    page: usize,
    page_size: usize,
  ) -> Result<PaginatedApiModelOutput, ApiModelServiceError> {
    let aliases = self
      .db_service
      .list_api_model_aliases(tenant_id, user_id)
      .await?;

    let total = aliases.len();
    let start = (page - 1) * page_size;

    // P0-8: Bounds check — if start >= total, return empty page instead of panicking
    let page_data: Vec<ApiModelOutput> = if start >= total {
      Vec::new()
    } else {
      let end = std::cmp::min(start + page_size, total);
      aliases[start..end]
        .iter()
        .map(|alias| {
          // TODO(P1-7): has_api_key is hardcoded to true for list view for efficiency;
          // checking individual key existence would require N queries
          ApiModelOutput::from_alias(alias.clone(), true)
        })
        .collect()
    };

    Ok(PaginatedApiModelOutput {
      data: page_data,
      total,
      page,
      page_size,
    })
  }

  async fn sync_cache(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<ApiModelOutput, ApiModelServiceError> {
    let api_alias = self
      .db_service
      .get_api_model_alias(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| EntityError::NotFound(format!("API model '{}' not found", id)))?;

    // Get API key (optional)
    let api_key = self
      .db_service
      .get_api_key_for_alias(tenant_id, user_id, id)
      .await
      .ok()
      .flatten();

    // Fetch models from remote API synchronously
    let models = self
      .ai_api_service
      .fetch_models(api_key.clone(), &api_alias.base_url)
      .await
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;

    // Update cache in DB
    let now = self.time_service.utc_now();
    self
      .db_service
      .update_api_model_cache(tenant_id, id, models.clone(), now)
      .await?;

    // Get refreshed alias
    let updated_alias = self
      .db_service
      .get_api_model_alias(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| EntityError::NotFound(format!("API model '{}' not found", id)))?;

    let has_api_key = api_key.is_some();

    Ok(ApiModelOutput::from_alias(updated_alias, has_api_key))
  }
}

// =============================================================================
// Shared validation helper
// =============================================================================

fn validate_forward_all(form: &ApiModelRequest) -> Result<(), ApiModelServiceError> {
  if form.forward_all_with_prefix {
    if form.prefix.is_none() || form.prefix.as_ref().is_none_or(|p| p.trim().is_empty()) {
      return Err(ApiModelServiceError::Validation(
        crate::ObjValidationError::ForwardAllRequiresPrefix,
      ));
    }
  } else if form.models.is_empty() {
    let mut errors = validator::ValidationErrors::new();
    let mut err = validator::ValidationError::new("models_required");
    err.message =
      Some("At least one model must be selected when not using forward_all mode".into());
    errors.add("models", err);
    return Err(ApiModelServiceError::Validation(
      crate::ObjValidationError::ValidationErrors(errors),
    ));
  }
  Ok(())
}
