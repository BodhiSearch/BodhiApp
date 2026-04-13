use async_trait::async_trait;
use std::sync::Arc;

use crate::db::{DbError, DbService, TimeService};
use crate::models::{ApiAlias, ApiAliasResponse, ApiKeyUpdate, ApiModel, ApiModelRequest};
use crate::new_ulid;
use crate::AiApiService;
use errmeta::{AppError, EntityError, ErrorType};
use std::collections::HashSet;

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

  #[error("Model '{0}' not found in the provider's model list. Verify the model ID is correct.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ModelNotFoundAtProvider(String),
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
  ) -> Result<ApiAliasResponse, ApiModelServiceError>;

  /// Update an existing API model configuration
  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    form: ApiModelRequest,
  ) -> Result<ApiAliasResponse, ApiModelServiceError>;

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
  ) -> Result<ApiAliasResponse, ApiModelServiceError>;

  /// Synchronously fetch and update models for a forward_all_with_prefix API model alias
  async fn sync_models(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<ApiAliasResponse, ApiModelServiceError>;
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
  ) -> Result<ApiAliasResponse, ApiModelServiceError> {
    validate_forward_all(&form)?;
    validate_extra_headers(&form.extra_headers)?;

    let now = self.time_service.utc_now();
    let id = new_ulid();

    // Extract API key from form
    let api_key_option = match form.api_key {
      ApiKeyUpdate::Set(key) => key.into_inner(),
      ApiKeyUpdate::Keep => None, // For create, Keep means no key
    };

    let base_url = form.base_url.trim_end_matches('/').to_string();

    // Fetch all models from provider and validate/filter based on mode
    let provider_models = self
      .ai_api_service
      .fetch_models(
        api_key_option.clone(),
        &base_url,
        &form.api_format,
        form.extra_headers.clone(),
        form.extra_body.clone(),
      )
      .await
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;

    let models: Vec<ApiModel> = if form.forward_all_with_prefix {
      provider_models
    } else {
      let provider_ids: HashSet<&str> = provider_models.iter().map(|m| m.id()).collect();
      for selected_id in &form.models {
        if !provider_ids.contains(selected_id.as_str()) {
          return Err(ApiModelServiceError::ModelNotFoundAtProvider(
            selected_id.clone(),
          ));
        }
      }
      let selected_ids: HashSet<&str> = form.models.iter().map(|s| s.as_str()).collect();
      provider_models
        .into_iter()
        .filter(|m| selected_ids.contains(m.id()))
        .collect()
    };

    let api_alias = ApiAlias::new(
      id,
      form.api_format,
      base_url,
      models,
      form.prefix,
      form.forward_all_with_prefix,
      now,
      form.extra_headers,
      form.extra_body,
    );

    self
      .db_service
      .create_api_model_alias(tenant_id, user_id, &api_alias, api_key_option)
      .await?;

    let has_api_key = self
      .db_service
      .get_api_key_for_alias(tenant_id, user_id, &api_alias.id)
      .await?
      .is_some();

    Ok(ApiAliasResponse::from(api_alias).with_has_api_key(has_api_key))
  }

  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    form: ApiModelRequest,
  ) -> Result<ApiAliasResponse, ApiModelServiceError> {
    validate_forward_all(&form)?;
    validate_extra_headers(&form.extra_headers)?;

    // Get existing API model
    let mut api_alias = self
      .db_service
      .get_api_model_alias(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| EntityError::NotFound(format!("API model '{}' not found", id)))?;

    // Changing api_format invalidates the stored api_key (different provider).
    // Require callers to supply a new key — rejecting `Keep` binds frontend/backend.
    if form.api_format != api_alias.api_format && matches!(form.api_key, ApiKeyUpdate::Keep) {
      return Err(ApiModelServiceError::Validation(
        crate::ObjValidationError::ApiFormatChangedRequiresNewKey,
      ));
    }

    // Convert ApiKeyUpdate to the raw form for repository
    let api_key_update = form.api_key.into_raw_update();

    let base_url = form.base_url.trim_end_matches('/').to_string();

    // Resolve API key for model fetching (use updated key if provided, else existing)
    let fetch_key = match &api_key_update {
      crate::RawApiKeyUpdate::Set(key_opt) => key_opt.clone(),
      crate::RawApiKeyUpdate::Keep => {
        self
          .db_service
          .get_api_key_for_alias(tenant_id, user_id, id)
          .await?
      }
    };

    // Fetch all models from provider and validate/filter based on mode
    let provider_models = self
      .ai_api_service
      .fetch_models(
        fetch_key,
        &base_url,
        &form.api_format,
        form.extra_headers.clone(),
        form.extra_body.clone(),
      )
      .await
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;

    let models: Vec<ApiModel> = if form.forward_all_with_prefix {
      provider_models
    } else {
      let provider_ids: HashSet<&str> = provider_models.iter().map(|m| m.id()).collect();
      for selected_id in &form.models {
        if !provider_ids.contains(selected_id.as_str()) {
          return Err(ApiModelServiceError::ModelNotFoundAtProvider(
            selected_id.clone(),
          ));
        }
      }
      let selected_ids: HashSet<&str> = form.models.iter().map(|s| s.as_str()).collect();
      provider_models
        .into_iter()
        .filter(|m| selected_ids.contains(m.id()))
        .collect()
    };

    // Update all fields
    api_alias.api_format = form.api_format;
    api_alias.base_url = base_url;
    api_alias.models = models.into();
    api_alias.prefix = if form.prefix.as_ref().is_some_and(|p| p.is_empty()) {
      None
    } else {
      form.prefix
    };
    api_alias.forward_all_with_prefix = form.forward_all_with_prefix;
    api_alias.extra_headers = form.extra_headers;
    api_alias.extra_body = form.extra_body;
    api_alias.updated_at = self.time_service.utc_now();

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

    Ok(ApiAliasResponse::from(api_alias).with_has_api_key(has_api_key))
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
  ) -> Result<ApiAliasResponse, ApiModelServiceError> {
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

    Ok(ApiAliasResponse::from(api_alias).with_has_api_key(has_api_key))
  }

  async fn sync_models(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
  ) -> Result<ApiAliasResponse, ApiModelServiceError> {
    let api_alias = self
      .db_service
      .get_api_model_alias(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| EntityError::NotFound(format!("API model '{}' not found", id)))?;

    if !api_alias.forward_all_with_prefix {
      return Err(ApiModelServiceError::Validation(
        crate::ObjValidationError::SyncRequiresForwardAll,
      ));
    }

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
      .fetch_models(
        api_key.clone(),
        &api_alias.base_url,
        &api_alias.api_format,
        api_alias.extra_headers.clone(),
        api_alias.extra_body.clone(),
      )
      .await
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;

    // Update models in DB
    self
      .db_service
      .update_api_model_models(tenant_id, id, models.clone())
      .await?;

    // Get refreshed alias
    let updated_alias = self
      .db_service
      .get_api_model_alias(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| EntityError::NotFound(format!("API model '{}' not found", id)))?;

    let has_api_key = api_key.is_some();

    Ok(ApiAliasResponse::from(updated_alias).with_has_api_key(has_api_key))
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

/// Defense-in-depth wrapper around `validate_extra_headers_no_auth` that maps the
/// `validator::ValidationError` into `ApiModelServiceError`. Keeps enforcement at
/// the service layer in addition to DTO-level validation.
pub(crate) fn validate_extra_headers(
  extra_headers: &Option<serde_json::Value>,
) -> Result<(), ApiModelServiceError> {
  let Some(value) = extra_headers else {
    return Ok(());
  };
  let Some(map) = value.as_object() else {
    return Ok(());
  };
  for key in map.keys() {
    let lower = key.to_ascii_lowercase();
    if lower == "authorization" || lower == "x-api-key" || lower == "x-goog-api-key" {
      return Err(ApiModelServiceError::Validation(
        crate::ObjValidationError::ExtraHeadersForbiddenKey(key.clone()),
      ));
    }
  }
  Ok(())
}
