use crate::{shared_rw::SharedContextRw, ContextError};
use async_openai::types::CreateChatCompletionRequest;
use axum::async_trait;
use objs::{ObjValidationError, Repo, TOKENIZER_CONFIG_JSON};
use services::{AliasNotFoundError, AppService, HubServiceError};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait RouterState: std::fmt::Debug + Send + Sync {
  fn app_service(&self) -> Arc<dyn AppService>;

  async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
    userdata: Sender<String>,
  ) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct DefaultRouterState {
  pub(crate) ctx: Arc<dyn SharedContextRw>,

  pub(crate) app_service: Arc<dyn AppService>,
}

impl DefaultRouterState {
  pub fn new(ctx: Arc<dyn SharedContextRw>, app_service: Arc<dyn AppService>) -> Self {
    Self { ctx, app_service }
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
    userdata: Sender<String>,
  ) -> Result<()> {
    let alias = self
      .app_service
      .data_service()
      .find_alias(&request.model)
      .ok_or_else(|| AliasNotFoundError(request.model.clone()))?;
    let model_file = self.app_service.hub_service().find_local_file(
      &alias.repo,
      &alias.filename,
      Some(alias.snapshot.clone()),
    )?;
    let tokenizer_repo = Repo::try_from(alias.chat_template.clone())?;
    let tokenizer_file = self.app_service.hub_service().find_local_file(
      &tokenizer_repo,
      TOKENIZER_CONFIG_JSON,
      None,
    )?;
    self
      .ctx
      .chat_completions(request, alias, model_file, tokenizer_file, userdata)
      .await?;
    Ok(())
  }
}

impl DefaultRouterState {
  pub async fn try_stop(&self) -> Result<()> {
    self.ctx.try_stop().await?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::{
    ContextError, test_utils::test_channel, DefaultRouterState, MockSharedContextRw,
    RouterState, RouterStateError,
  };
  use anyhow_trace::anyhow_trace;
  use async_openai::types::CreateChatCompletionRequest;
  use llama_server_bindings::LlamaCppError;
  use mockall::predicate::{always, eq};
  use objs::{test_utils::temp_dir, Alias, HubFileBuilder};
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
    let state =
      DefaultRouterState::new(Arc::new(MockSharedContextRw::default()), Arc::new(service));
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "not-found",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;
    let (tx, _rx) = test_channel();
    let result = state.chat_completions(request, tx).await;
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
    let mut mock_ctx = MockSharedContextRw::default();
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias-exists:instruct",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;
    let hf_cache = temp_dir.path().join("huggingface/hub").to_path_buf();
    let model_file = HubFileBuilder::testalias_exists()
      .hf_cache(hf_cache.clone())
      .build()?;
    let llama3_tokenizer = HubFileBuilder::llama3_tokenizer()
      .hf_cache(hf_cache.clone())
      .build()?;
    mock_ctx
      .expect_chat_completions()
      .with(
        eq(request.clone()),
        eq(Alias::testalias_exists()),
        eq(model_file),
        eq(llama3_tokenizer),
        always(),
      )
      .return_once(|_, _, _, _, _| Ok(()));
    let service = AppServiceStubBuilder::default()
      .with_temp_home_as(temp_dir)
      .with_data_service()
      .with_hub_service()
      .build()?;
    let state = DefaultRouterState::new(Arc::new(mock_ctx), Arc::new(service));
    let (tx, _rx) = test_channel();
    state.chat_completions(request, tx).await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_router_state_chat_completions_returns_context_err(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let hf_cache = temp_dir.path().join("huggingface/hub").to_path_buf();
    let mut mock_ctx = MockSharedContextRw::default();
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias-exists:instruct",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;
    let (tx, _rx) = test_channel();
    let model_file = HubFileBuilder::testalias_exists()
      .hf_cache(hf_cache.clone())
      .build()?;
    let llama3_tokenizer = HubFileBuilder::llama3_tokenizer()
      .hf_cache(hf_cache.clone())
      .build()?;
    let alias = Alias::testalias_exists();
    mock_ctx
      .expect_chat_completions()
      .with(
        eq(request.clone()),
        eq(alias),
        eq(model_file),
        eq(llama3_tokenizer),
        always(),
      )
      .return_once(|_, _, _, _, _| {
        Err(ContextError::LlamaCpp(
          LlamaCppError::BodhiServerChatCompletion("test error".to_string()),
        ))
      });
    let service = AppServiceStubBuilder::default()
      .with_temp_home_as(temp_dir)
      .with_hub_service()
      .with_data_service()
      .build()?;
    let state = DefaultRouterState::new(Arc::new(mock_ctx), Arc::new(service));
    let result = state.chat_completions(request, tx).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
      RouterStateError::ContextError(ContextError::LlamaCpp(
        LlamaCppError::BodhiServerChatCompletion(msg),
      )) => {
        assert_eq!("test error", msg);
      }
      err => {
        panic!("expected ContextError, got: {}", err);
      }
    }
    Ok(())
  }
}
