use crate::{shared_rw::SharedContext, ContextError};
use async_openai::types::CreateChatCompletionRequest;
use axum::async_trait;
use objs::ObjValidationError;
use services::{AliasNotFoundError, AppService, HubServiceError};
use std::sync::Arc;

#[async_trait]
pub trait RouterState: std::fmt::Debug + Send + Sync {
  fn app_service(&self) -> Arc<dyn AppService>;

  async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
  ) -> Result<reqwest::Response>;
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
}

type Result<T> = std::result::Result<T, RouterStateError>;

#[async_trait]
impl RouterState for DefaultRouterState {
  fn app_service(&self) -> Arc<dyn AppService> {
    self.app_service.clone()
  }

  async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
  ) -> Result<reqwest::Response> {
    let alias = self
      .app_service
      .data_service()
      .find_alias(&request.model)
      .ok_or_else(|| AliasNotFoundError(request.model.clone()))?;
    let response = self.ctx.chat_completions(request, alias).await?;
    Ok(response)
  }
}

#[cfg(test)]
mod test {
  use crate::{ContextError, DefaultRouterState, MockSharedContext, RouterState, RouterStateError};
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
      RouterStateError::AliasNotFound(err) => {
        assert_eq!("not-found", err.0);
      }
      err => {
        panic!("expected AliasNotFound error, got: {}", err);
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
      .return_once(|_, _| Ok(mock_response("")));
    let service = AppServiceStubBuilder::default()
      .with_temp_home_as(temp_dir)
      .with_hub_service()
      .with_data_service()
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
      .return_once(|_, _| {
        Err(ContextError::Server(ServerError::StartupError(
          "test error".to_string(),
        )))
      });
    let service = AppServiceStubBuilder::default()
      .with_temp_home_as(temp_dir)
      .with_data_service()
      .with_hub_service()
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
