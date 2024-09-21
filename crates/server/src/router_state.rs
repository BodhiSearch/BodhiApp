use crate::{oai::OpenAIApiError, shared_rw::SharedContextRw, ContextError};
use async_openai::types::CreateChatCompletionRequest;
use axum::async_trait;
use objs::{Repo, REFS_MAIN, TOKENIZER_CONFIG_JSON};
use services::AppService;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait RouterState: Send + Sync {
  fn app_service(&self) -> Arc<dyn AppService>;

  async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
    userdata: Sender<String>,
  ) -> std::result::Result<(), OpenAIApiError>;
}

#[derive(Debug, Clone)]
pub struct DefaultRouterState {
  pub(crate) ctx: Arc<dyn SharedContextRw>,

  pub(crate) app_service: Arc<dyn AppService>,
}

impl DefaultRouterState {
  pub(crate) fn new(ctx: Arc<dyn SharedContextRw>, app_service: Arc<dyn AppService>) -> Self {
    Self { ctx, app_service }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum RouterStateError {
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
  ) -> std::result::Result<(), OpenAIApiError> {
    let Some(alias) = self.app_service.data_service().find_alias(&request.model) else {
      return Err(crate::oai::OpenAIApiError::ModelNotFound(request.model));
    };
    let model_file = self
      .app_service
      .hub_service()
      .find_local_file(&alias.repo, &alias.filename, &alias.snapshot)
      .map_err(|err| OpenAIApiError::InternalServer(err.to_string()))?;
    let Some(model_file) = model_file else {
      return Err(OpenAIApiError::InternalServer(format!(
        "file required by LLM model not found in huggingface cache: filename: '{}', repo: '{}'",
        alias.filename, alias.repo
      )));
    };
    let tokenizer_repo = Repo::try_from(alias.chat_template.clone())
      .map_err(|err| OpenAIApiError::InternalServer(err.to_string()))?;
    let tokenizer_file = self
      .app_service
      .hub_service()
      .find_local_file(&tokenizer_repo, TOKENIZER_CONFIG_JSON, REFS_MAIN)
      .map_err(|err| OpenAIApiError::InternalServer(err.to_string()))?;
    let Some(tokenizer_file) = tokenizer_file else {
      return Err(OpenAIApiError::InternalServer(format!(
        "file required by LLM model not found in huggingface cache: filename: '{}', repo: '{}'",
        TOKENIZER_CONFIG_JSON, tokenizer_repo
      )));
    };
    self
      .ctx
      .chat_completions(request, alias, model_file, tokenizer_file, userdata)
      .await
      .map_err(|e| OpenAIApiError::ContextError(e.to_string()))?;
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
    shared_rw::ContextError,
    test_utils::{test_channel, ResponseTestExt},
    DefaultRouterState, MockSharedContextRw, RouterState,
  };
  use async_openai::types::CreateChatCompletionRequest;
  use axum::http::StatusCode;
  use axum::response::{IntoResponse, Response};
  use llama_server_bindings::LlamaCppError;
  use mockall::predicate::{always, eq};
  use objs::{Alias, HubFile, Repo, REFS_MAIN, TOKENIZER_CONFIG_JSON};
  use rstest::rstest;
  use serde_json::{json, Value};
  use services::{test_utils::AppServiceStubMock, MockDataService, MockHubService};
  use std::sync::Arc;

  #[rstest]
  #[tokio::test]
  async fn test_router_state_chat_completions_model_not_found() -> anyhow::Result<()> {
    let mut mock_data_service = MockDataService::default();
    mock_data_service
      .expect_find_alias()
      .with(eq("not-found"))
      .return_once(|_| None);
    let mock_ctx = MockSharedContextRw::default();
    let service = AppServiceStubMock::builder()
      .data_service(mock_data_service)
      .build()?;
    let state = DefaultRouterState::new(Arc::new(mock_ctx), Arc::new(service));
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "not-found",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;
    let (tx, _rx) = test_channel();
    let result = state.chat_completions(request, tx).await;
    assert!(result.is_err());
    let response: Response = result.unwrap_err().into_response();
    assert_eq!(StatusCode::NOT_FOUND, response.status());
    let response: Value = response.json().await?;
    let expected = json! {{
      "message": "The model 'not-found' does not exist",
      "type": "invalid_request_error",
      "param": "model",
      "code": "model_not_found",
    }};
    assert_eq!(expected, response);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_router_state_chat_completions_delegate_to_context_with_alias() -> anyhow::Result<()>
  {
    let mut mock_data_service = MockDataService::default();
    mock_data_service
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| Some(Alias::testalias()));
    let testalias = Alias::testalias();
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_find_local_file()
      .with(
        eq(testalias.repo),
        eq(testalias.filename),
        eq(testalias.snapshot),
      )
      .return_once(|_, _, _| Ok(Some(HubFile::testalias())));
    mock_hub_service
      .expect_find_local_file()
      .with(eq(Repo::llama3()), eq(TOKENIZER_CONFIG_JSON), eq(REFS_MAIN))
      .return_once(|_, _, _| Ok(Some(HubFile::llama3_tokenizer())));
    let mut mock_ctx = MockSharedContextRw::default();
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias:instruct",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;
    mock_ctx
      .expect_chat_completions()
      .with(
        eq(request.clone()),
        eq(Alias::testalias()),
        eq(HubFile::testalias()),
        eq(HubFile::llama3_tokenizer()),
        always(),
      )
      .return_once(|_, _, _, _, _| Ok(()));
    let service = AppServiceStubMock::builder()
      .hub_service(mock_hub_service)
      .data_service(mock_data_service)
      .build()?;
    let state = DefaultRouterState::new(Arc::new(mock_ctx), Arc::new(service));
    let (tx, _rx) = test_channel();
    state.chat_completions(request, tx).await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_router_state_chat_completions_returns_context_err() -> anyhow::Result<()> {
    let mut mock_data_service = MockDataService::new();
    mock_data_service
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| Some(Alias::testalias()));
    let testalias = Alias::testalias();
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_find_local_file()
      .with(
        eq(testalias.repo),
        eq(testalias.filename),
        eq(testalias.snapshot),
      )
      .return_once(|_, _, _| Ok(Some(HubFile::testalias())));
    mock_hub_service
      .expect_find_local_file()
      .with(eq(Repo::llama3()), eq(TOKENIZER_CONFIG_JSON), eq(REFS_MAIN))
      .return_once(|_, _, _| Ok(Some(HubFile::llama3_tokenizer())));
    let mut mock_ctx = MockSharedContextRw::default();
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias:instruct",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;
    let (tx, _rx) = test_channel();
    mock_ctx
      .expect_chat_completions()
      .with(
        eq(request.clone()),
        eq(Alias::testalias()),
        eq(HubFile::testalias()),
        eq(HubFile::llama3_tokenizer()),
        always(),
      )
      .return_once(|_, _, _, _, _| {
        Err(ContextError::BodhiError(
          LlamaCppError::BodhiServerChatCompletion("test error".to_string()),
        ))
      });
    let service = AppServiceStubMock::builder()
      .hub_service(mock_hub_service)
      .data_service(mock_data_service)
      .build()?;
    let state = DefaultRouterState::new(Arc::new(mock_ctx), Arc::new(service));
    let result = state.chat_completions(request, tx).await;
    assert!(result.is_err());
    let response = result.unwrap_err().into_response();
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    assert_eq!(
      json! {{
        "message": "bodhi_server_chat_completion: test error",
        "type": "internal_server_error",
        "param": null,
        "code": "internal_server_error"
      }},
      response.json::<Value>().await?
    );
    Ok(())
  }
}
