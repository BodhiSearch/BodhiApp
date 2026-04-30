use async_trait::async_trait;
use std::sync::Arc;

use crate::db::{DbError, DbService, TimeService};
use crate::models::llm_liberty_envelope::LlmLibertyEnvelopeUpdate;
use crate::models::{
  ApiAlias, ApiAliasResponse, ApiFormat, ApiKeyUpdate, ApiModel, ApiModelRequest,
  DefaultApiModelRequest, LlmLibertyApiModelRequest,
};
use crate::new_ulid;
use crate::{AiApiClient, AiApiService};
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
    validate_forward_all_enum(&form)?;
    let now = self.time_service.utc_now();
    let id = new_ulid();
    let api_format = form.api_format();

    match form {
      ApiModelRequest::LlmLibertyOauth(d) => {
        self
          .create_llm_liberty(tenant_id, user_id, id, api_format, now, d)
          .await
      }
      other => {
        let d = match other {
          ApiModelRequest::Openai(d)
          | ApiModelRequest::OpenaiResponses(d)
          | ApiModelRequest::Anthropic(d)
          | ApiModelRequest::AnthropicOauth(d)
          | ApiModelRequest::Gemini(d) => d,
          ApiModelRequest::LlmLibertyOauth(_) => unreachable!(),
        };
        self
          .create_default(tenant_id, user_id, id, api_format, now, d)
          .await
      }
    }
  }

  async fn update(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    form: ApiModelRequest,
  ) -> Result<ApiAliasResponse, ApiModelServiceError> {
    validate_forward_all_enum(&form)?;

    let api_alias = self
      .db_service
      .get_api_model_alias(tenant_id, user_id, id)
      .await?
      .ok_or_else(|| EntityError::NotFound(format!("API model '{}' not found", id)))?;

    let api_format = form.api_format();

    // Format is immutable on edit. The LlmLibertyOauth variant has its own
    // sibling-table credentials that would orphan on switch-out and silently
    // 404 on switch-in; the simpler contract is to forbid changes entirely.
    if api_format != api_alias.api_format {
      return Err(ApiModelServiceError::Validation(
        crate::ObjValidationError::ApiFormatImmutableOnEdit,
      ));
    }

    match form {
      ApiModelRequest::LlmLibertyOauth(d) => {
        self
          .update_llm_liberty(tenant_id, user_id, id, api_format, api_alias, d)
          .await
      }
      other => {
        let d = match other {
          ApiModelRequest::Openai(d)
          | ApiModelRequest::OpenaiResponses(d)
          | ApiModelRequest::Anthropic(d)
          | ApiModelRequest::AnthropicOauth(d)
          | ApiModelRequest::Gemini(d) => d,
          ApiModelRequest::LlmLibertyOauth(_) => unreachable!(),
        };
        self
          .update_default(tenant_id, user_id, id, api_format, api_alias, d)
          .await
      }
    }
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

    if api_alias.api_format == ApiFormat::LlmLibertyOauth {
      let summary = self
        .db_service
        .get_llm_liberty_summary(tenant_id, user_id, id)
        .await?;
      return Ok(ApiAliasResponse::from(api_alias).with_llm_liberty(summary));
    }

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

    let (client, api_key): (Box<dyn AiApiClient>, Option<String>) =
      if api_alias.api_format == ApiFormat::LlmLibertyOauth {
        let creds = self
          .db_service
          .get_llm_liberty_credentials(tenant_id, user_id, id)
          .await?
          .ok_or_else(|| {
            ApiModelServiceError::AiApi(format!(
              "LLM Liberty credentials not found for alias '{}'",
              id
            ))
          })?;
        let client = self
          .ai_api_service
          .for_resolved_credentials(&creds, &api_alias, tenant_id, user_id)
          .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;
        (client, None)
      } else {
        let key = self
          .db_service
          .get_api_key_for_alias(tenant_id, user_id, id)
          .await
          .ok()
          .flatten();
        let client = self
          .ai_api_service
          .for_alias(&api_alias, key.clone())
          .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;
        (client, key)
      };

    let models = client
      .fetch_models()
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

    if updated_alias.api_format == ApiFormat::LlmLibertyOauth {
      let summary = self
        .db_service
        .get_llm_liberty_summary(tenant_id, user_id, id)
        .await?;
      return Ok(ApiAliasResponse::from(updated_alias).with_llm_liberty(summary));
    }

    Ok(ApiAliasResponse::from(updated_alias).with_has_api_key(api_key.is_some()))
  }
}

// =============================================================================
// Per-variant helpers (default / llm_liberty)
// =============================================================================

impl DefaultApiModelService {
  async fn create_default(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: String,
    api_format: ApiFormat,
    now: chrono::DateTime<chrono::Utc>,
    form: DefaultApiModelRequest,
  ) -> Result<ApiAliasResponse, ApiModelServiceError> {
    validate_extra_headers(&form.extra_headers)?;

    let api_key_option = match form.api_key {
      ApiKeyUpdate::Set(key) => key.into_inner(),
      ApiKeyUpdate::Keep => None,
    };

    let base_url = form.base_url.trim_end_matches('/').to_string();

    let provider_models = self
      .ai_api_service
      .for_alias(
        &ApiAlias::new(
          String::new(),
          api_format.clone(),
          base_url.clone(),
          vec![],
          form.prefix.clone(),
          false,
          now,
          form.extra_headers.clone(),
          form.extra_body.clone(),
        ),
        api_key_option.clone(),
      )
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?
      .fetch_models()
      .await
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;

    let models = filter_models(provider_models, form.forward_all_with_prefix, &form.models)?;

    let api_alias = ApiAlias::new(
      id,
      api_format,
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

  async fn create_llm_liberty(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: String,
    api_format: ApiFormat,
    now: chrono::DateTime<chrono::Utc>,
    form: LlmLibertyApiModelRequest,
  ) -> Result<ApiAliasResponse, ApiModelServiceError> {
    let envelope = match form.envelope {
      LlmLibertyEnvelopeUpdate::Set(env) => env,
      LlmLibertyEnvelopeUpdate::Keep => {
        return Err(ApiModelServiceError::Validation(
          crate::ObjValidationError::LlmLibertyEnvelopeRequired,
        ));
      }
    };
    envelope.validate_supported()?;

    let provider_models = self
      .ai_api_service
      .for_envelope(&envelope)
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?
      .fetch_models()
      .await
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;

    let models = filter_models(provider_models, form.forward_all_with_prefix, &form.models)?;

    let api_alias = ApiAlias::new(
      id,
      api_format,
      envelope.api.base_url.clone(),
      models,
      form.prefix,
      form.forward_all_with_prefix,
      now,
      None,
      None,
    );

    self
      .db_service
      .create_api_model_alias(tenant_id, user_id, &api_alias, None)
      .await?;

    self
      .db_service
      .create_llm_liberty_credentials(tenant_id, user_id, &api_alias.id, &envelope)
      .await?;

    let summary = self
      .db_service
      .get_llm_liberty_summary(tenant_id, user_id, &api_alias.id)
      .await?;

    Ok(ApiAliasResponse::from(api_alias).with_llm_liberty(summary))
  }

  async fn update_default(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    api_format: ApiFormat,
    mut api_alias: ApiAlias,
    form: DefaultApiModelRequest,
  ) -> Result<ApiAliasResponse, ApiModelServiceError> {
    validate_extra_headers(&form.extra_headers)?;

    // Format equality is enforced upstream in `update`; api_format == api_alias.api_format here.
    let api_key_update = form.api_key.into_raw_update();
    let base_url = form.base_url.trim_end_matches('/').to_string();

    let fetch_key = match &api_key_update {
      crate::RawApiKeyUpdate::Set(key_opt) => key_opt.clone(),
      crate::RawApiKeyUpdate::Keep => {
        self
          .db_service
          .get_api_key_for_alias(tenant_id, user_id, id)
          .await?
      }
    };

    let provider_models = self
      .ai_api_service
      .for_alias(
        &ApiAlias::new(
          api_alias.id.clone(),
          api_format.clone(),
          base_url.clone(),
          vec![],
          form.prefix.clone(),
          false,
          api_alias.created_at,
          form.extra_headers.clone(),
          form.extra_body.clone(),
        ),
        fetch_key,
      )
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?
      .fetch_models()
      .await
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;

    let models = filter_models(provider_models, form.forward_all_with_prefix, &form.models)?;

    api_alias.api_format = api_format;
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

    let has_api_key = self
      .db_service
      .get_api_key_for_alias(tenant_id, user_id, id)
      .await?
      .is_some();

    Ok(ApiAliasResponse::from(api_alias).with_has_api_key(has_api_key))
  }

  async fn update_llm_liberty(
    &self,
    tenant_id: &str,
    user_id: &str,
    id: &str,
    api_format: ApiFormat,
    mut api_alias: ApiAlias,
    form: LlmLibertyApiModelRequest,
  ) -> Result<ApiAliasResponse, ApiModelServiceError> {
    let (client, new_base_url, new_envelope) = match form.envelope {
      LlmLibertyEnvelopeUpdate::Set(env) => {
        env.validate_supported()?;
        let base_url = env.api.base_url.clone();
        let client = self
          .ai_api_service
          .for_envelope(&env)
          .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;
        (client, base_url, Some(env))
      }
      LlmLibertyEnvelopeUpdate::Keep => {
        let creds = self
          .db_service
          .get_llm_liberty_credentials(tenant_id, user_id, id)
          .await?
          .ok_or_else(|| {
            ApiModelServiceError::AiApi(format!(
              "LLM Liberty credentials not found for alias '{}'",
              id
            ))
          })?;
        let base_url = creds.api_base_url.clone();
        let client = self
          .ai_api_service
          .for_resolved_credentials(&creds, &api_alias, tenant_id, user_id)
          .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;
        (client, base_url, None)
      }
    };

    let provider_models = client
      .fetch_models()
      .await
      .map_err(|e| ApiModelServiceError::AiApi(e.to_string()))?;

    let models = filter_models(provider_models, form.forward_all_with_prefix, &form.models)?;

    api_alias.api_format = api_format;
    api_alias.base_url = new_base_url;
    api_alias.models = models.into();
    api_alias.prefix = if form.prefix.as_ref().is_some_and(|p| p.is_empty()) {
      None
    } else {
      form.prefix
    };
    api_alias.forward_all_with_prefix = form.forward_all_with_prefix;
    api_alias.extra_headers = None;
    api_alias.extra_body = None;
    api_alias.updated_at = self.time_service.utc_now();

    self
      .db_service
      .update_api_model_alias(
        tenant_id,
        user_id,
        id,
        &api_alias,
        crate::RawApiKeyUpdate::Keep,
      )
      .await?;

    if let Some(envelope) = new_envelope {
      self
        .db_service
        .update_llm_liberty_credentials(tenant_id, user_id, id, &envelope)
        .await?;
    }

    let summary = self
      .db_service
      .get_llm_liberty_summary(tenant_id, user_id, id)
      .await?;

    Ok(ApiAliasResponse::from(api_alias).with_llm_liberty(summary))
  }
}

// =============================================================================
// Shared helpers
// =============================================================================

fn filter_models(
  provider_models: Vec<ApiModel>,
  forward_all: bool,
  selected: &[String],
) -> Result<Vec<ApiModel>, ApiModelServiceError> {
  if forward_all {
    return Ok(provider_models);
  }
  let provider_ids: HashSet<&str> = provider_models.iter().map(|m| m.id()).collect();
  for s in selected {
    if !provider_ids.contains(s.as_str()) {
      return Err(ApiModelServiceError::ModelNotFoundAtProvider(s.clone()));
    }
  }
  let selected_ids: HashSet<&str> = selected.iter().map(|s| s.as_str()).collect();
  Ok(
    provider_models
      .into_iter()
      .filter(|m| selected_ids.contains(m.id()))
      .collect(),
  )
}

fn validate_forward_all_enum(form: &ApiModelRequest) -> Result<(), ApiModelServiceError> {
  let forward_all = form.forward_all_with_prefix();
  let prefix = form.prefix();
  let models = form.models();
  if forward_all {
    if prefix.is_none() || prefix.is_some_and(|p| p.trim().is_empty()) {
      return Err(ApiModelServiceError::Validation(
        crate::ObjValidationError::ForwardAllRequiresPrefix,
      ));
    }
  } else if models.is_empty() {
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
