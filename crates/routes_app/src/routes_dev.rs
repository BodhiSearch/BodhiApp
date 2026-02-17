use auth_middleware::{SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN};
use axum::{
  body::Body,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use objs::{ApiError, AppError, SerdeJsonError};
use serde_json::json;
use server_core::RouterState;
use services::{db::DbError, SecretServiceError, SecretServiceExt, SessionServiceError};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DevError {
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
  #[error(transparent)]
  DbError(#[from] DbError),
  #[error(transparent)]
  SessionServiceError(#[from] SessionServiceError),
}

pub async fn dev_secrets_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Response, ApiError> {
  let secret_service = state.app_service().secret_service();

  // Read session tokens
  let access_token = session
    .get::<String>(SESSION_KEY_ACCESS_TOKEN)
    .await
    .ok()
    .flatten();
  let refresh_token = session
    .get::<String>(SESSION_KEY_REFRESH_TOKEN)
    .await
    .ok()
    .flatten();

  #[allow(unused_mut)]
  let mut value = json! {{
    "status": secret_service.app_status()?,
    "app_info": secret_service.app_reg_info()?,
    "session": {
      "access_token": access_token,
      "refresh_token": refresh_token,
    }
  }};
  #[cfg(debug_assertions)]
  {
    value["dump"] = serde_json::Value::String(secret_service.dump()?);
  }
  Ok(
    Response::builder()
      .header("Content-Type", "application/json")
      .body(Body::from(value.to_string()))
      .unwrap(),
  )
}

pub async fn envs_handler(State(state): State<Arc<dyn RouterState>>) -> Result<Response, ApiError> {
  let envs = state
    .app_service()
    .setting_service()
    .list()
    .into_iter()
    .collect::<Vec<_>>();
  Ok((StatusCode::OK, Json(envs)).into_response())
}

pub async fn dev_db_reset_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Response, ApiError> {
  let app_service = state.app_service();
  app_service
    .db_service()
    .reset_all_tables()
    .await
    .map_err(DevError::from)?;
  app_service
    .session_service()
    .clear_all_sessions()
    .await
    .map_err(DevError::from)?;
  Ok((StatusCode::OK, Json(json!({"status": "ok"}))).into_response())
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow_trace::anyhow_trace;
  use axum::{body::Body, extract::State, http::StatusCode};
  use objs::{AliasSource, ApiFormat, UserAlias};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::Value;
  use server_core::{DefaultRouterState, MockSharedContext, RouterState};
  use services::{
    db::{ApiToken, DownloadRequest, DownloadStatus, ModelMetadataRow, TokenStatus, ToolsetRow},
    test_utils::app_service_stub,
    AppService,
  };
  use std::sync::Arc;
  use tower_sessions::SessionStore;

  async fn body_to_json(body: Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_dev_db_reset_returns_ok(
    #[future] app_service_stub: services::test_utils::AppServiceStub,
  ) -> anyhow::Result<()> {
    let app_service: Arc<dyn AppService> = Arc::new(app_service_stub.await);
    let mock_context = Arc::new(MockSharedContext::default());
    let router_state = Arc::new(DefaultRouterState::new(mock_context, app_service.clone()));

    let response = dev_db_reset_handler(State(router_state as Arc<dyn RouterState>)).await?;

    assert_eq!(StatusCode::OK, response.status());
    let body = body_to_json(response.into_body()).await;
    assert_eq!(json!({"status": "ok"}), body);

    Ok(())
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_dev_db_reset_clears_all_tables(
    #[future] app_service_stub: services::test_utils::AppServiceStub,
  ) -> anyhow::Result<()> {
    let app_service: Arc<dyn AppService> = Arc::new(app_service_stub.await);
    let db_service = app_service.db_service();

    // Populate database with test data
    // 1. Create download request
    let download_req = DownloadRequest {
      id: "test-download".to_string(),
      repo: "test/repo".to_string(),
      filename: "test.gguf".to_string(),
      status: DownloadStatus::Pending,
      error: None,
      created_at: app_service.time_service().utc_now(),
      updated_at: app_service.time_service().utc_now(),
      total_bytes: None,
      downloaded_bytes: 0,
      started_at: None,
    };
    db_service.create_download_request(&download_req).await?;

    // 2. Create access request
    db_service
      .insert_pending_request("test-user".to_string(), "test-user-id".to_string())
      .await?;

    // 3. Create API token
    let mut api_token = ApiToken {
      id: "test-token-id".to_string(),
      user_id: "test-user".to_string(),
      name: "Test Token".to_string(),
      token_prefix: "prefix".to_string(),
      token_hash: "hash".to_string(),
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: app_service.time_service().utc_now(),
      updated_at: app_service.time_service().utc_now(),
    };
    db_service.create_api_token(&mut api_token).await?;

    // 4. Create user alias
    let user_alias = UserAlias {
      id: "test-alias-id".to_string(),
      alias: "test-alias".to_string(),
      repo: "test/repo".parse()?,
      filename: "test.gguf".to_string(),
      snapshot: "main".to_string(),
      request_params: Default::default(),
      context_params: vec![],
      created_at: app_service.time_service().utc_now(),
      updated_at: app_service.time_service().utc_now(),
    };
    db_service.create_user_alias(&user_alias).await?;

    // 5. Create toolset
    let toolset = ToolsetRow {
      id: "test-toolset-id".to_string(),
      user_id: "test-user".to_string(),
      toolset_type: "builtin-exa-search".to_string(),
      slug: "my-search".to_string(),
      description: Some("Test toolset".to_string()),
      enabled: true,
      encrypted_api_key: None,
      salt: None,
      nonce: None,
      created_at: app_service.time_service().utc_now().timestamp(),
      updated_at: app_service.time_service().utc_now().timestamp(),
    };
    db_service.create_toolset(&toolset).await?;

    // Create API model alias
    let api_alias = objs::ApiAlias {
      id: "test-api-alias".to_string(),
      api_format: ApiFormat::OpenAI,
      base_url: "http://localhost".to_string(),
      models: vec!["model1".to_string()],
      prefix: Some("test-".to_string()),
      forward_all_with_prefix: false,
      models_cache: vec![],
      cache_fetched_at: app_service.time_service().utc_now(),
      created_at: app_service.time_service().utc_now(),
      updated_at: app_service.time_service().utc_now(),
    };
    db_service.create_api_model_alias(&api_alias, None).await?;

    // Create metadata
    let metadata = ModelMetadataRow {
      id: 1,
      source: AliasSource::Model.to_string(),
      repo: Some("test/repo".to_string()),
      filename: Some("test.gguf".to_string()),
      snapshot: Some("main".to_string()),
      api_model_id: None,
      capabilities_vision: Some(0),
      capabilities_audio: Some(0),
      capabilities_thinking: Some(0),
      capabilities_function_calling: Some(0),
      capabilities_structured_output: Some(0),
      context_max_input_tokens: None,
      context_max_output_tokens: None,
      architecture: None,
      additional_metadata: None,
      chat_template: None,
      extracted_at: app_service.time_service().utc_now(),
      created_at: app_service.time_service().utc_now(),
      updated_at: app_service.time_service().utc_now(),
    };
    db_service.upsert_model_metadata(&metadata).await?;

    // Create session
    let session_service = app_service.session_service();
    let session_store = session_service.get_session_store();
    let session_id = tower_sessions::session::Id::default();
    let mut data = std::collections::HashMap::new();
    data.insert(
      "user_id".to_string(),
      serde_json::Value::String("test-user".to_string()),
    );
    let record = tower_sessions::session::Record {
      id: session_id,
      data,
      expiry_date: time::OffsetDateTime::now_utc() + time::Duration::hours(1),
    };
    session_store.save(&record).await?;

    // Reset database
    let mock_context = Arc::new(MockSharedContext::default());
    let router_state = Arc::new(DefaultRouterState::new(mock_context, app_service.clone()));
    let response = dev_db_reset_handler(State(router_state as Arc<dyn RouterState>)).await?;
    assert_eq!(StatusCode::OK, response.status());

    // Verify all tables are empty
    assert_eq!(
      None,
      db_service.get_download_request("test-download").await?
    );
    assert_eq!(
      None,
      db_service
        .get_pending_request("test-user-id".to_string())
        .await?
    );
    assert_eq!(
      None,
      db_service
        .get_api_token_by_id("test-user", "test-token-id")
        .await?
    );
    assert_eq!(
      None,
      db_service.get_user_alias_by_id("test-alias-id").await?
    );
    assert_eq!(None, db_service.get_toolset("test-toolset-id").await?);
    assert_eq!(
      None,
      db_service.get_api_model_alias("test-api-alias").await?
    );
    assert_eq!(
      None,
      db_service
        .get_model_metadata_by_file("test/repo", "test.gguf", "main")
        .await?
    );

    // Verify sessions cleared
    assert_eq!(0, session_store.count_sessions_for_user("test-user").await?);

    // Verify app_toolset_configs re-seeded
    let config = db_service
      .get_app_toolset_config("builtin-exa-search")
      .await?;
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!("builtin-exa-search", config.toolset_type);
    assert_eq!(false, config.enabled);
    assert_eq!("system", config.updated_by);

    Ok(())
  }
}
