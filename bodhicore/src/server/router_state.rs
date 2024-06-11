use crate::{
  db::DbServiceFn,
  oai::OpenAIApiError,
  objs::{REFS_MAIN, TOKENIZER_CONFIG_JSON},
  service::AppServiceFn,
  shared_rw::SharedContextRwFn,
  Repo,
};
use async_openai::types::CreateChatCompletionRequest;
use axum::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait RouterStateFn: Send + Sync {
  fn app_service(&self) -> Arc<dyn AppServiceFn>;

  fn db_service(&self) -> Arc<dyn DbServiceFn>;

  async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
    userdata: Sender<String>,
  ) -> crate::oai::Result<()>;
}

#[derive(Debug, Clone)]
pub struct RouterState {
  pub(crate) ctx: Arc<dyn SharedContextRwFn>,
  pub(crate) app_service: Arc<dyn AppServiceFn>,
  pub(crate) db_service: Arc<dyn DbServiceFn>,
}

impl RouterState {
  pub(crate) fn new(
    ctx: Arc<dyn SharedContextRwFn>,
    app_service: Arc<dyn AppServiceFn>,
    db_service: Arc<dyn DbServiceFn>,
  ) -> Self {
    Self {
      ctx,
      app_service,
      db_service,
    }
  }
}

#[async_trait]
impl RouterStateFn for RouterState {
  fn app_service(&self) -> Arc<dyn AppServiceFn> {
    self.app_service.clone()
  }

  fn db_service(&self) -> Arc<dyn DbServiceFn> {
    self.db_service.clone()
  }

  async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
    userdata: Sender<String>,
  ) -> crate::oai::Result<()> {
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
      .map_err(OpenAIApiError::ContextError)?;
    Ok(())
  }
}

impl RouterState {
  pub async fn try_stop(&self) -> crate::error::Result<()> {
    self.ctx.try_stop().await?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::RouterState;
  use crate::{
    oai::ApiError,
    objs::{Alias, HubFile, REFS_MAIN, TOKENIZER_CONFIG_JSON},
    server::RouterStateFn,
    service::{MockDataService, MockEnvServiceFn, MockHubService},
    shared_rw::ContextError,
    test_utils::{
      test_channel, AppServiceStubMock, MockDbService, MockSharedContext, ResponseTestExt,
    },
    Repo,
  };
  use async_openai::types::CreateChatCompletionRequest;
  use axum::http::StatusCode;
  use axum::response::{IntoResponse, Response};
  use llama_server_bindings::LlamaCppError;
  use mockall::predicate::{always, eq};
  use rstest::rstest;
  use serde_json::json;
  use std::sync::Arc;

  #[rstest]
  #[tokio::test]
  async fn test_router_state_chat_completions_model_not_found() -> anyhow::Result<()> {
    let mut mock_data_service = MockDataService::default();
    mock_data_service
      .expect_find_alias()
      .with(eq("not-found"))
      .return_once(|_| None);
    let mock_ctx = MockSharedContext::default();
    let service = AppServiceStubMock::new(
      MockEnvServiceFn::new(),
      MockHubService::new(),
      mock_data_service,
    );
    let state = RouterState::new(
      Arc::new(mock_ctx),
      Arc::new(service),
      Arc::new(MockDbService::new()),
    );
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
    let response: ApiError = response.json_obj().await?;
    let expected = ApiError {
      message: "The model 'not-found' does not exist".to_string(),
      r#type: "model_not_found".to_string(),
      param: Some("model".to_string()),
      code: "model_not_found".to_string(),
    };
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
    let mut mock_ctx = MockSharedContext::default();
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
    let service =
      AppServiceStubMock::new(MockEnvServiceFn::new(), mock_hub_service, mock_data_service);
    let state = RouterState::new(
      Arc::new(mock_ctx),
      Arc::new(service),
      Arc::new(MockDbService::new()),
    );
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
    let mut mock_ctx = MockSharedContext::default();
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
    let service =
      AppServiceStubMock::new(MockEnvServiceFn::new(), mock_hub_service, mock_data_service);
    let state = RouterState::new(
      Arc::new(mock_ctx),
      Arc::new(service),
      Arc::new(MockDbService::new()),
    );
    let result = state.chat_completions(request, tx).await;
    assert!(result.is_err());
    let response = result.unwrap_err().into_response();
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    assert_eq!(
      ApiError {
        message: "bodhi_server_chat_completion: test error".to_string(),
        r#type: "internal_server_error".to_string(),
        param: None,
        code: "internal_server_error".to_string()
      },
      response.json::<ApiError>().await?
    );
    Ok(())
  }
}
