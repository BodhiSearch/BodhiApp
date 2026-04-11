use crate::{ApiError, AuthScope, DashboardAuthRouteError};
use axum::{
  body::Body,
  extract::Path,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde_json::json;
use services::{AppError, AuthServiceError, ErrorType, SerdeJsonError};
use services::{DbError, SessionServiceError, TenantError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DevError {
  #[error(transparent)]
  TenantError(#[from] TenantError),
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
  #[error(transparent)]
  DbError(#[from] DbError),
  #[error(transparent)]
  SessionServiceError(#[from] SessionServiceError),
  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
  #[error(transparent)]
  DashboardAuthRouteError(#[from] DashboardAuthRouteError),
  #[error("Not in multi-tenant mode.")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  NotMultiTenant,
  #[error("Tenant '{0}' not found in local database.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  TenantNotFoundLocal(String),
  #[error("KC SPI request failed (status {status}): {body}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  SpiRequestFailed { status: u16, body: String },
}

pub async fn dev_secrets_handler(auth_scope: AuthScope) -> Result<Response, ApiError> {
  let tenant_svc = auth_scope.tenants();
  let instance = tenant_svc.get_standalone_app().await.ok().flatten();
  let status = instance
    .as_ref()
    .map(|t| t.status.clone())
    .unwrap_or_default();

  let value = json! {{
    "status": status,
    "app_info": instance,
    "auth_context": auth_scope.auth_context(),
  }};
  Ok(
    Response::builder()
      .header("Content-Type", "application/json")
      .body(Body::from(value.to_string()))
      .unwrap(),
  )
}

pub async fn envs_handler(auth_scope: AuthScope) -> Result<Response, ApiError> {
  let envs = auth_scope
    .settings()
    .list()
    .await
    .into_iter()
    .collect::<Vec<_>>();
  Ok((StatusCode::OK, Json(envs)).into_response())
}

pub async fn dev_db_reset_handler(auth_scope: AuthScope) -> Result<Response, ApiError> {
  auth_scope
    .db()
    .reset_all_tables()
    .await
    .map_err(DevError::from)?;
  auth_scope
    .sessions()
    .clear_all_sessions()
    .await
    .map_err(DevError::from)?;
  Ok((StatusCode::OK, Json(json!({"status": "ok"}))).into_response())
}

pub async fn dev_clients_dag_handler(
  auth_scope: AuthScope,
  Path(client_id): Path<String>,
) -> Result<Response, ApiError> {
  if !auth_scope.auth_context().is_multi_tenant() {
    return Err(DevError::NotMultiTenant)?;
  }

  let dashboard_token = auth_scope.auth_context().require_dashboard_token()?;

  let (status, body) = auth_scope
    .auth_service()
    .forward_request(
      "POST".to_string(),
      format!("test/clients/{}/dag", client_id),
      Some(dashboard_token.to_string()),
      None,
    )
    .await
    .map_err(DevError::from)?;

  if status >= 400 {
    return Err(DevError::SpiRequestFailed {
      status,
      body: body.to_string(),
    })?;
  }

  // Look up local tenant to get client_secret
  let tenant = auth_scope
    .tenants()
    .get_tenant_by_client_id(&client_id)
    .await
    .map_err(DevError::from)?
    .ok_or_else(|| DevError::TenantNotFoundLocal(client_id.clone()))?;

  Ok(
    (
      StatusCode::OK,
      Json(json!({
        "client_id": tenant.client_id,
        "client_secret": tenant.client_secret,
      })),
    )
      .into_response(),
  )
}

pub async fn dev_tenants_cleanup_handler(auth_scope: AuthScope) -> Result<Response, ApiError> {
  if !auth_scope.auth_context().is_multi_tenant() {
    return Err(DevError::NotMultiTenant)?;
  }

  let dashboard_token = auth_scope.auth_context().require_dashboard_token()?;
  let user_id = auth_scope.auth_context().require_user_id()?;

  // Look up tenants created by this user, excluding [do-not-delete] protected tenants
  let tenants = auth_scope
    .tenants()
    .list_tenants_by_creator(user_id)
    .await
    .map_err(DevError::from)?;

  let client_ids: Vec<String> = tenants
    .iter()
    .filter(|t| !t.name.starts_with("[do-not-delete]"))
    .map(|t| t.client_id.clone())
    .collect();

  if client_ids.is_empty() {
    return Ok(
      (
        StatusCode::OK,
        Json(json!({ "deleted": [], "skipped": [], "errors": [] })),
      )
        .into_response(),
    );
  }

  // Send explicit client_ids to SPI for deletion
  let (status, body) = auth_scope
    .auth_service()
    .forward_request(
      "DELETE".to_string(),
      "test/tenants/cleanup".to_string(),
      Some(dashboard_token.to_string()),
      Some(json!({ "client_ids": client_ids })),
    )
    .await
    .map_err(DevError::from)?;

  if status >= 400 {
    return Err(DevError::SpiRequestFailed {
      status,
      body: body.to_string(),
    })?;
  }

  // Optimistically delete all sent client_ids from local DB
  for client_id in &client_ids {
    auth_scope
      .tenants()
      .delete_tenant_by_client_id(client_id)
      .await
      .map_err(DevError::from)?;
  }

  Ok((StatusCode::OK, Json(body)).into_response())
}

#[cfg(test)]
mod tests {
  use super::dev_db_reset_handler;
  use crate::AuthScope;
  use anyhow_trace::anyhow_trace;
  use axum::{body::Body, http::StatusCode};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;
  use serde_json::Value;
  use services::AliasSource;
  use services::AuthContext;
  use services::{
    test_utils::{app_service_stub, TEST_TENANT_ID},
    AppService, AuthScopedAppService, DownloadRequestEntity, DownloadStatus, ModelMetadataEntity,
    TokenEntity, TokenStatus,
  };
  use services::{ApiFormat, UserAlias};
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
    let auth_scope = AuthScope(AuthScopedAppService::new(
      app_service.clone(),
      AuthContext::Anonymous {
        deployment: services::DeploymentMode::Standalone,
      },
    ));

    let response = dev_db_reset_handler(auth_scope).await?;

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
    let download_req = DownloadRequestEntity {
      id: "test-download".to_string(),
      tenant_id: TEST_TENANT_ID.to_string(),
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
      .insert_pending_request(
        TEST_TENANT_ID,
        "test-user".to_string(),
        "test-user-id".to_string(),
      )
      .await?;

    // 3. Create API token
    let mut api_token = TokenEntity {
      id: "test-token-id".to_string(),
      tenant_id: TEST_TENANT_ID.to_string(),
      user_id: "test-user".to_string(),
      name: "Test Token".to_string(),
      token_prefix: "prefix".to_string(),
      token_hash: "hash".to_string(),
      scopes: "scope_token_user".to_string(),
      status: TokenStatus::Active,
      created_at: app_service.time_service().utc_now(),
      updated_at: app_service.time_service().utc_now(),
    };
    db_service
      .create_api_token(TEST_TENANT_ID, &mut api_token)
      .await?;

    // 4. Create user alias
    let user_alias = UserAlias {
      id: "test-alias-id".to_string(),
      alias: "test-alias".to_string(),
      repo: "test/repo".parse()?,
      filename: "test.gguf".to_string(),
      snapshot: "main".to_string(),
      request_params: Default::default(),
      context_params: Default::default(),
      created_at: app_service.time_service().utc_now(),
      updated_at: app_service.time_service().utc_now(),
    };
    db_service
      .create_user_alias(TEST_TENANT_ID, "", &user_alias)
      .await?;

    // Create API model alias
    let api_alias = services::ApiAlias {
      id: "test-api-alias".to_string(),
      api_format: ApiFormat::OpenAI,
      base_url: "http://localhost".to_string(),
      models: vec![services::ApiModel::OpenAI(
        async_openai::types::models::Model {
          id: "model1".to_string(),
          object: "model".to_string(),
          created: 0,
          owned_by: "openai".to_string(),
        },
      )]
      .into(),
      prefix: Some("test-".to_string()),
      forward_all_with_prefix: false,
      created_at: app_service.time_service().utc_now(),
      updated_at: app_service.time_service().utc_now(),
    };
    db_service
      .create_api_model_alias(TEST_TENANT_ID, "", &api_alias, None)
      .await?;

    // Create metadata
    let metadata = ModelMetadataEntity {
      id: String::new(),
      tenant_id: TEST_TENANT_ID.to_string(),
      source: AliasSource::Model,
      repo: Some("test/repo".to_string()),
      filename: Some("test.gguf".to_string()),
      snapshot: Some("main".to_string()),
      api_model_id: None,
      capabilities: None,
      context: None,
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
    let auth_scope = AuthScope(AuthScopedAppService::new(
      app_service.clone(),
      AuthContext::Anonymous {
        deployment: services::DeploymentMode::Standalone,
      },
    ));
    let response = dev_db_reset_handler(auth_scope).await?;
    assert_eq!(StatusCode::OK, response.status());

    // Verify all tables are empty
    assert_eq!(
      None,
      db_service
        .get_download_request(TEST_TENANT_ID, "test-download")
        .await?
    );
    assert_eq!(
      None,
      db_service
        .get_pending_request(TEST_TENANT_ID, "test-user-id".to_string())
        .await?
    );
    assert_eq!(
      None,
      db_service
        .get_api_token_by_id("test-user", "test-token-id", TEST_TENANT_ID)
        .await?
    );
    assert_eq!(
      None,
      db_service
        .get_user_alias_by_id(TEST_TENANT_ID, "", "test-alias-id")
        .await?
    );
    assert_eq!(
      None,
      db_service
        .get_api_model_alias(TEST_TENANT_ID, "", "test-api-alias")
        .await?
    );
    assert_eq!(
      None,
      db_service
        .get_model_metadata_by_file(TEST_TENANT_ID, "test/repo", "test.gguf", "main")
        .await?
    );

    // Verify sessions cleared
    assert_eq!(
      0,
      session_service.count_sessions_for_user("test-user").await?
    );

    Ok(())
  }
}
