use crate::{
  model_router::{DefaultModelRouter, ModelRouter, ModelRouterError, RouteDestination},
  shared_rw::SharedContext,
  ContextError,
};
use async_openai::types::CreateChatCompletionRequest;
use objs::ObjValidationError;
use services::{
  AiApiService, AiApiServiceError, AliasNotFoundError, AppService, DefaultAiApiService,
  HubServiceError,
};
use std::{future::Future, pin::Pin, sync::Arc};

pub trait RouterState: std::fmt::Debug + Send + Sync {
  fn app_service(&self) -> Arc<dyn AppService>;

  fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
  ) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send + '_>>;
}

#[derive(Debug, Clone)]
pub struct DefaultRouterState {
  pub(crate) ctx: Arc<dyn SharedContext>,
  pub(crate) app_service: Arc<dyn AppService>,
}

impl DefaultRouterState {
  pub fn new(ctx: Arc<dyn SharedContext>, app_service: Arc<dyn AppService>) -> Self {
    Self { ctx, app_service }
  }

  pub async fn stop(&self) -> Result<()> {
    self.ctx.stop().await?;
    Ok(())
  }

  pub fn model_router(&self) -> Box<dyn ModelRouter + Send + Sync> {
    Box::new(DefaultModelRouter::new(
      self.app_service.data_service(),
      self.app_service.db_service(),
    ))
  }

  pub fn ai_api_service(&self) -> Box<dyn AiApiService + Send + Sync> {
    Box::new(DefaultAiApiService::with_db_service(
      self.app_service.db_service(),
    ))
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = objs::AppError)]
pub enum RouterStateError {
  #[error(transparent)]
  ObjValidationError(#[from] ObjValidationError),
  #[error(transparent)]
  AliasNotFound(#[from] AliasNotFoundError),
  #[error(transparent)]
  HubService(#[from] HubServiceError),
  #[error(transparent)]
  ContextError(#[from] ContextError),
  #[error(transparent)]
  ModelRouter(#[from] ModelRouterError),
  #[error(transparent)]
  AiApiService(#[from] AiApiServiceError),
}

type Result<T> = std::result::Result<T, RouterStateError>;

impl RouterState for DefaultRouterState {
  fn app_service(&self) -> Arc<dyn AppService> {
    self.app_service.clone()
  }

  fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
  ) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send + '_>> {
    Box::pin(async move {
      // Use ModelRouter to determine routing destination
      let destination = self.model_router().route_request(&request.model).await?;

      match destination {
        RouteDestination::Local(alias) => {
          // Route to local model via SharedContext (existing behavior)
          let response = self.ctx.chat_completions(request, alias).await?;
          Ok(response)
        }
        RouteDestination::Remote(api_alias) => {
          // Route to remote API via AiApiService
          // The AiApiService returns an axum::Response, but we need a reqwest::Response
          // for compatibility with the existing interface
          let axum_response = self
            .ai_api_service()
            .forward_chat_completion(&api_alias.id, request)
            .await?;

          // Convert axum::Response to reqwest::Response
          // Extract parts and body from axum response
          let (parts, body) = axum_response.into_parts();

          // Convert body to bytes for reqwest
          let body_bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|e| {
            AiApiServiceError::ApiError(format!("Failed to read response body: {}", e))
          })?;

          // Build a response that can be converted to reqwest::Response
          // Use axum's re-exported http types
          let mut builder = axum::http::Response::builder().status(parts.status);
          for (key, value) in parts.headers {
            if let Some(key) = key {
              builder = builder.header(key, value);
            }
          }
          let http_response = builder
            .body(body_bytes.to_vec())
            .map_err(|e| AiApiServiceError::ApiError(format!("Failed to build response: {}", e)))?;

          // Convert to reqwest::Response
          Ok(reqwest::Response::from(http_response))
        }
      }
    })
  }
}

#[cfg(test)]
mod test {
  use crate::{
    ContextError, DefaultRouterState, MockSharedContext, ModelRouterError, RouterState,
    RouterStateError,
  };
  use anyhow_trace::anyhow_trace;
  use async_openai::types::CreateChatCompletionRequest;
  use llama_server_proc::{test_utils::mock_response, ServerError};
  use mockall::predicate::eq;
  use objs::{test_utils::temp_dir, Alias};
  use rstest::rstest;
  use serde_json::json;
  use services::test_utils::AppServiceStubBuilder;
  use std::sync::Arc;
  use tempfile::TempDir;

  #[rstest]
  #[tokio::test]
  async fn test_router_state_chat_completions_model_not_found() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .with_db_service()
      .await
      .build()?;
    let state = DefaultRouterState::new(Arc::new(MockSharedContext::default()), Arc::new(service));
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "not-found",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;
    let result = state.chat_completions(request).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
      RouterStateError::ModelRouter(ModelRouterError::ApiModelNotFound(model)) => {
        assert_eq!("not-found", model);
      }
      err => {
        panic!("expected ModelRouter::ApiModelNotFound error, got: {}", err);
      }
    }
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_router_state_chat_completions_delegate_to_context_with_alias(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_ctx = MockSharedContext::default();
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias-exists:instruct",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;
    mock_ctx
      .expect_chat_completions()
      .with(eq(request.clone()), eq(Alias::testalias_exists()))
      .times(1)
      .return_once(|_, _| Ok(mock_response("")));
    let service = AppServiceStubBuilder::default()
      .with_temp_home_as(temp_dir)
      .with_hub_service()
      .with_data_service()
      .with_db_service()
      .await
      .build()?;
    let state = DefaultRouterState::new(Arc::new(mock_ctx), Arc::new(service));
    state.chat_completions(request).await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_router_state_chat_completions_returns_context_err(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_ctx = MockSharedContext::default();
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias-exists:instruct",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;
    let alias = Alias::testalias_exists();
    mock_ctx
      .expect_chat_completions()
      .with(eq(request.clone()), eq(alias))
      .times(1)
      .return_once(|_, _| {
        Err(ContextError::Server(ServerError::StartupError(
          "test error".to_string(),
        )))
      });
    let service = AppServiceStubBuilder::default()
      .with_temp_home_as(temp_dir)
      .with_data_service()
      .with_hub_service()
      .with_db_service()
      .await
      .build()?;
    let state = DefaultRouterState::new(Arc::new(mock_ctx), Arc::new(service));
    let result = state.chat_completions(request).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
      RouterStateError::ContextError(ContextError::Server(ServerError::StartupError(msg))) => {
        assert_eq!("test error", msg);
      }
      err => {
        panic!("expected ContextError, got: {}", err);
      }
    }
    Ok(())
  }
}
