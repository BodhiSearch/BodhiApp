/*
// TODO: clean up/delete
use super::RouterState;
  use crate::{
    bindings::{disable_llama_log, llama_server_disable_logging},
    oai::ApiError,
    objs::{Alias, LocalModelFile, REFS_MAIN, TOKENIZER_CONFIG_JSON},
    server::RouterStateFn,
    shared_rw::ContextError,
    test_utils::{
      app_service_stub, init_test_tracing, test_callback, test_channel, AppServiceTuple,
      MockAppService, MockSharedContext, ResponseTestExt,
    },
    Repo, SharedContextRw,
  };
  use anyhow::anyhow;
  use anyhow_trace::anyhow_trace;
  use async_openai::types::{CreateChatCompletionRequest, CreateChatCompletionResponse};
  use axum::http::StatusCode;
  use axum::response::{IntoResponse, Response};
  use llama_server_bindings::GptParams;
  use mockall::predicate::{always, eq};
  use rstest::{fixture, rstest};
  use serde_json::json;
  use serial_test::serial;
  use std::sync::Arc;
  use tempfile::TempDir;

struct RouterStateTuple(TempDir, TempDir, RouterState);
  fn setup() {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    init_test_tracing();
  }


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

*/
