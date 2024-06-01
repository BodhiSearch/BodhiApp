use crate::{
  oai::OpenAIApiError,
  objs::{REFS_MAIN, TOKENIZER_CONFIG_JSON},
  service::AppServiceFn,
  shared_rw::{SharedContextRw, SharedContextRwFn},
  Repo,
};
use anyhow::{anyhow, bail};
use async_openai::types::CreateChatCompletionRequest;
use futures_util::TryFutureExt;
use llama_server_bindings::{Callback, GptParams};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct RouterState {
  pub(crate) ctx: Arc<dyn SharedContextRwFn>,
  pub(crate) app_service: Arc<dyn AppServiceFn>,
}

impl RouterState {
  pub(crate) fn new(ctx: Arc<dyn SharedContextRwFn>, app_service: Arc<dyn AppServiceFn>) -> Self {
    Self { ctx, app_service }
  }
}

impl RouterState {
  pub async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
    callback: Option<Callback>,
    userdata: &String,
  ) -> crate::oai::Result<()> {
    let Some(alias) = self.app_service.find_alias(&request.model) else {
      return Err(crate::oai::OpenAIApiError::ModelNotFound(
        request.model.clone(),
      ));
    };
    let model_file = self
      .app_service
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
      .chat_completions(request, model_file, tokenizer_file, callback, userdata)
      .await
      .map_err(OpenAIApiError::ContextError)?;
    Ok(())
  }
  //   let Some(alias) = self.app_service.find_alias(&request.model) else {
  //     bail!("model alias not found: '{}'", request.model)
  //   };
  //   let Some(local_model) =
  //     self
  //       .app_service
  //       .find_local_file(&alias.repo, &alias.filename, &alias.snapshot)?
  //   else {
  //     bail!("local model not found: {:?}", alias);
  //   };
  //   let lock = self.ctx.read().await;
  //   let ctx = lock.as_ref();
  //   let local_model_path = local_model.path().to_string_lossy().into_owned();
  //   match ctx {
  //     Some(ctx) => {
  //       let gpt_params = ctx.gpt_params.clone();
  //       let loaded_model = gpt_params.model.clone();
  //       if loaded_model.eq(&local_model_path) {
  //         ctx.completions(
  //           input,
  //           chat_template,
  //           callback,
  //           userdata as *const _ as *mut _,
  //         )
  //       } else {
  //         tracing::info!(
  //           loaded_model,
  //           ?local_model,
  //           "requested model not loaded, loading model"
  //         );
  //         drop(lock);
  //         let new_gpt_params = GptParams {
  //           model: local_model_path,
  //           ..gpt_params
  //         };
  //         self.ctx.reload(Some(new_gpt_params)).await?;
  //         let lock = self.ctx.read().await;
  //         let ctx = lock.as_ref().ok_or(anyhow!("context not present"))?;
  //         ctx.completions(
  //           input,
  //           chat_template,
  //           callback,
  //           userdata as *const _ as *mut _,
  //         )
  //       }
  //     }
  //     None => {
  //       let gpt_params = GptParams {
  //         model: local_model_path,
  //         ..Default::default()
  //       };
  //       drop(lock);
  //       self.ctx.reload(Some(gpt_params)).await?;
  //       let lock = self.ctx.read().await;
  //       let ctx = lock.as_ref().ok_or(anyhow!("context not present"))?;
  //       ctx.completions(
  //         input,
  //         chat_template,
  //         callback,
  //         userdata as *const _ as *mut _,
  //       )
  //     }
  //   }
  // }

  pub async fn completions(
    &self,
    model: &str,
    input: &str,
    chat_template: &str,
    callback: Option<Callback>,
    userdata: &String,
  ) -> anyhow::Result<()> {
    let Some(alias) = self.app_service.find_alias(model) else {
      bail!("model alias not found: '{}'", model)
    };
    let Some(local_model) =
      self
        .app_service
        .find_local_file(&alias.repo, &alias.filename, &alias.snapshot)?
    else {
      bail!("local model not found: {:?}", alias);
    };
    todo!()
    // let lock = self.ctx.read().await;
    // let ctx = lock.as_ref();
    // let local_model_path = local_model.path().to_string_lossy().into_owned();
    // match ctx {
    //   Some(ctx) => {
    //     let gpt_params = ctx.gpt_params.clone();
    //     let loaded_model = gpt_params.model.clone();
    //     if loaded_model.eq(&local_model_path) {
    //       ctx.completions(
    //         input,
    //         chat_template,
    //         callback,
    //         userdata as *const _ as *mut _,
    //       )
    //     } else {
    //       tracing::info!(
    //         loaded_model,
    //         ?local_model,
    //         "requested model not loaded, loading model"
    //       );
    //       drop(lock);
    //       let new_gpt_params = GptParams {
    //         model: local_model_path,
    //         ..gpt_params
    //       };
    //       self.ctx.reload(Some(new_gpt_params)).await?;
    //       let lock = self.ctx.ctx.read().await;
    //       let ctx = lock.as_ref().ok_or(anyhow!("context not present"))?;
    //       ctx.completions(
    //         input,
    //         chat_template,
    //         callback,
    //         userdata as *const _ as *mut _,
    //       )
    //     }
    //   }
    //   None => {
    //     let gpt_params = GptParams {
    //       model: local_model_path,
    //       ..Default::default()
    //     };
    //     drop(lock);
    //     self.ctx.reload(Some(gpt_params)).await?;
    //     let lock = self.ctx.ctx.read().await;
    //     let ctx = lock.as_ref().ok_or(anyhow!("context not present"))?;
    //     ctx.completions(
    //       input,
    //       chat_template,
    //       callback,
    //       userdata as *const _ as *mut _,
    //     )
    // }
  }
}

#[cfg(test)]
mod test {
  use super::RouterState;
  use crate::{
    bindings::{disable_llama_log, llama_server_disable_logging},
    oai::ApiError,
    objs::{Alias, LocalModelFile, REFS_MAIN, TOKENIZER_CONFIG_JSON},
    shared_rw::ContextError,
    test_utils::{
      app_service_stub, init_test_tracing, test_callback, AppServiceTuple, MockAppService,
      MockSharedContext, ResponseTestExt,
    },
    Repo, SharedContextRw, SharedContextRwFn,
  };
  use anyhow::anyhow;
  use anyhow_trace::anyhow_trace;
  use async_openai::types::{CreateChatCompletionRequest, CreateChatCompletionResponse};
  use axum::http::StatusCode;
  use axum::response::{IntoResponse, Response};
  use llama_server_bindings::GptParams;
  use mockall::predicate::eq;
  use rstest::{fixture, rstest};
  use serde_json::json;
  use serial_test::serial;
  use std::sync::Arc;
  use tempfile::TempDir;

  fn setup() {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    init_test_tracing();
  }

  struct RouterStateTuple(TempDir, TempDir, RouterState);

  async fn state(app_service_stub: AppServiceTuple) -> RouterStateTuple {
    setup();
    let model_path = dirs::home_dir()
      .ok_or(anyhow!("unable to locate home dir"))
      .unwrap()
      .join(".cache/huggingface/hub/models--TheBloke--Llama-2-7B-Chat-GGUF/snapshots/08a5566d61d7cb6b420c3e4387a39e0078e1f2fe5f055f3a03887385304d4bfa/llama-2-7b-chat.Q4_K_M.gguf")
      .canonicalize()
      .unwrap()
      .to_str()
      .unwrap()
      .to_owned();
    let gpt_params = GptParams {
      model: model_path,
      ..Default::default()
    };
    let ctx = SharedContextRw::new_shared_rw(Some(gpt_params))
      .await
      .unwrap();
    let AppServiceTuple(temp_bodhi_home, temp_hf_home, _, _, service) = app_service_stub;
    RouterStateTuple(
      temp_bodhi_home,
      temp_hf_home,
      RouterState::new(Arc::new(ctx), Arc::new(service)),
    )
  }

  async fn empty_state(app_service_stub: AppServiceTuple) -> RouterStateTuple {
    let ctx = SharedContextRw::new_shared_rw(None).await.unwrap();
    let AppServiceTuple(temp_bodhi_home, temp_hf_home, _, _, service) = app_service_stub;
    RouterStateTuple(
      temp_bodhi_home,
      temp_hf_home,
      RouterState::new(Arc::new(ctx), Arc::new(service)),
    )
  }

  #[fixture]
  fn inputs() -> (String, String) {
    let model = String::from("TheBloke/Llama-2-7B-Chat-GGUF:llama-2-7b-chat.Q4_K_M.gguf");
    let request = serde_json::to_string(&json! {{
      "model": "TheBloke/Llama-2-7B-Chat-GGUF:llama-2-7b-chat.Q4_K_M.gguf",
      "seed": 42,
      "prompt": "<s>[INST] <<SYS>>\nyou are a helpful assistant\n<</SYS>>\n\nwhat day comes after Monday? [/INST]"
    }})
    .unwrap();
    (model, request)
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  #[serial(router_state)]
  #[anyhow_trace]
  async fn test_router_state_read_from_same_model(
    inputs: (String, String),
    app_service_stub: AppServiceTuple,
  ) -> anyhow::Result<()> {
    let RouterStateTuple(_temp_bodhi_home, _temp_hf_home, state) = state(app_service_stub).await;
    let (model, request) = inputs;
    let userdata = String::with_capacity(2048);
    state
      .completions(&model, &request, "", Some(test_callback), &userdata)
      .await?;
    let response: CreateChatCompletionResponse = serde_json::from_str(&userdata)?;
    let loaded_model = state
      .ctx
      .get_gpt_params()
      .await?
      .ok_or(anyhow!("gpt params not present"))?
      .model
      .clone();
    let (repo, file) = model.split_once(':').ok_or(anyhow!("failed to split"))?;
    let repo = format!("models--{}", repo.replace('/', "--"));
    assert!(loaded_model.contains(&repo));
    assert!(loaded_model.ends_with(file));
    assert_eq!(
      "  Great, I'm glad you asked! The day that comes after Monday is Tuesday! 😊",
      response
        .choices
        .first()
        .ok_or(anyhow!("choices not present"))?
        .message
        .content
        .as_ref()
        .ok_or(anyhow!("content not present"))?
    );
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  #[serial(router_state)]
  #[anyhow_trace]
  async fn test_router_state_read_from_same_model_empty_state(
    inputs: (String, String),
    app_service_stub: AppServiceTuple,
  ) -> anyhow::Result<()> {
    let RouterStateTuple(_temp_bodhi_home, _temp_hf_home, state) =
      empty_state(app_service_stub).await;
    let (model, request) = inputs;
    let userdata = String::with_capacity(2048);
    state
      .completions(&model, &request, "", Some(test_callback), &userdata)
      .await?;
    let response: CreateChatCompletionResponse = serde_json::from_str(&userdata)?;
    let loaded_model = state
      .ctx
      .get_gpt_params()
      .await?
      .ok_or(anyhow!("gpt params not present"))?
      .model
      .clone();
    let (repo, file) = model.split_once(':').ok_or(anyhow!("failed to split"))?;
    let repo = format!("models--{}", repo.replace('/', "--"));
    assert!(loaded_model.contains(&repo));
    assert!(loaded_model.ends_with(file));
    assert_eq!(
      "  Great, I'm glad you asked! The day that comes after Monday is Tuesday! 😊",
      response
        .choices
        .first()
        .ok_or(anyhow!("choices not present"))?
        .message
        .content
        .as_ref()
        .ok_or(anyhow!("content not present"))?
    );
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  #[serial(router_state)]
  #[anyhow_trace]
  async fn test_router_state_fails_if_model_not_found(
    app_service_stub: AppServiceTuple,
  ) -> anyhow::Result<()> {
    let RouterStateTuple(_temp_bodhi_home, _temp_hf_home, state) = state(app_service_stub).await;
    let model = "non-existing-model";
    let result = state.completions(model, "", "", None, &String::new()).await;
    assert!(result.is_err());
    assert_eq!(
      format!("model alias not found: '{}'", model),
      result.unwrap_err().to_string()
    );
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  #[serial(router_state)]
  #[anyhow_trace]
  async fn test_router_state_load_new_model(
    app_service_stub: AppServiceTuple,
  ) -> anyhow::Result<()> {
    let RouterStateTuple(_temp_bodhi_home, _temp_hf_home, state) = state(app_service_stub).await;
    let model = "TheBloke/Llama-2-7B-Chat-GGUF:llama-2-7b-chat.Q8_0.gguf";
    let request = serde_json::to_string(&json! {{
      "model": model,
      "seed": 42,
      "prompt": "<s>[INST] <<SYS>>\nyou are a helpful assistant\n<</SYS>>\n\nwhat day comes after Monday? [/INST]"
    }})
    .unwrap();
    let userdata = String::with_capacity(2048);
    state
      .completions(model, &request, "", Some(test_callback), &userdata)
      .await?;
    let response: CreateChatCompletionResponse = serde_json::from_str(&userdata).unwrap();
    assert_eq!(model, response.model);
    let loaded_model = state
      .ctx
      .get_gpt_params()
      .await?
      .ok_or(anyhow!("gpt params not present"))?
      .model
      .clone();
    let (repo, file) = model.split_once(':').ok_or(anyhow!("failed to split"))?;
    let repo = format!("models--{}", repo.replace('/', "--"));
    assert!(loaded_model.contains(&repo));
    assert!(loaded_model.ends_with(file));
    assert_eq!(
      "  Great question! The day that comes after Monday is Tuesday! 😊",
      response
        .choices
        .first()
        .ok_or(anyhow!("choices not present"))?
        .message
        .content
        .as_ref()
        .ok_or(anyhow!("content not present"))?
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_router_state_chat_completions_model_not_found() -> anyhow::Result<()> {
    let mut mock_app_service = MockAppService::default();
    mock_app_service
      .expect_find_alias()
      .with(eq("not-found"))
      .return_once(|_| None);
    let mock_ctx = MockSharedContext::default();
    let state = RouterState::new(Arc::new(mock_ctx), Arc::new(mock_app_service));
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "not-found",
      "messages": [
        {"role": "user", "content": "What day comes after Monday?"}
      ]
    }})?;
    let result = state.chat_completions(request, None, &String::new()).await;
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
    let mut mock_app_service = MockAppService::default();
    mock_app_service
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| Some(Alias::test_alias()));
    let testalias = Alias::test_alias();
    mock_app_service
      .expect_find_local_file()
      .with(
        eq(testalias.repo),
        eq(testalias.filename),
        eq(testalias.snapshot),
      )
      .return_once(|_, _, _| Ok(Some(LocalModelFile::testalias())));
    mock_app_service
      .expect_find_local_file()
      .with(eq(Repo::llama3()), eq(TOKENIZER_CONFIG_JSON), eq(REFS_MAIN))
      .return_once(|_, _, _| Ok(Some(LocalModelFile::llama3_tokenizer())));
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
        eq(LocalModelFile::testalias()),
        eq(LocalModelFile::llama3_tokenizer()),
        eq(None),
        eq(String::new()),
      )
      .return_once(|_, _, _, _, _| Ok(()));
    let state = RouterState::new(Arc::new(mock_ctx), Arc::new(mock_app_service));
    state
      .chat_completions(request, None, &String::new())
      .await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_router_state_chat_completions_returns_context_err() -> anyhow::Result<()> {
    let mut mock_app_service = MockAppService::default();
    mock_app_service
      .expect_find_alias()
      .with(eq("testalias:instruct"))
      .return_once(|_| Some(Alias::test_alias()));
    let testalias = Alias::test_alias();
    mock_app_service
      .expect_find_local_file()
      .with(
        eq(testalias.repo),
        eq(testalias.filename),
        eq(testalias.snapshot),
      )
      .return_once(|_, _, _| Ok(Some(LocalModelFile::testalias())));
    mock_app_service
      .expect_find_local_file()
      .with(eq(Repo::llama3()), eq(TOKENIZER_CONFIG_JSON), eq(REFS_MAIN))
      .return_once(|_, _, _| Ok(Some(LocalModelFile::llama3_tokenizer())));
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
        eq(LocalModelFile::testalias()),
        eq(LocalModelFile::llama3_tokenizer()),
        eq(None),
        eq(String::new()),
      )
      .return_once(|_, _, _, _, _| Err(ContextError::LlamaCpp(anyhow!("context error"))));
    let state = RouterState::new(Arc::new(mock_ctx), Arc::new(mock_app_service));
    let result = state.chat_completions(request, None, &String::new()).await;
    assert!(result.is_err());
    let response = result.unwrap_err().into_response();
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    assert_eq!(
      ApiError {
        message: "context error".to_string(),
        r#type: "internal_server_error".to_string(),
        param: None,
        code: "internal_server_error".to_string()
      },
      response.json::<ApiError>().await?
    );
    Ok(())
  }
}
