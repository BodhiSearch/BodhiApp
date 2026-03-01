use std::collections::HashMap;

use crate::{
  shared_rw::{DefaultSharedContext, ModelLoadStrategy, SharedContext},
  test_utils::{bin_path, mock_server, ServerFactoryStub},
  LlmEndpoint,
};
use anyhow_trace::anyhow_trace;
use async_openai::types::chat::CreateChatCompletionRequest;
use futures::FutureExt;
use llama_server_proc::{
  test_utils::mock_response, LlamaServerArgsBuilder, MockServer, BUILD_TARGET, BUILD_VARIANTS,
  DEFAULT_VARIANT, EXEC_NAME,
};
use mockall::predicate::eq;
use rstest::rstest;
use serde_json::{json, Value};
use serial_test::serial;
use services::test_utils::temp_hf_home;
use services::{
  test_utils::{app_service_stub_builder, AppServiceStubBuilder},
  AppService, BODHI_EXEC_LOOKUP_PATH, BODHI_EXEC_NAME, BODHI_EXEC_TARGET, BODHI_EXEC_VARIANT,
  BODHI_EXEC_VARIANTS, BODHI_LLAMACPP_ARGS,
};
use services::{Alias, HubFileBuilder, UserAlias};
use tempfile::TempDir;

#[rstest]
#[case(Some("testalias".to_string()), "testalias", ModelLoadStrategy::Continue)]
#[case(Some("testalias".to_string()), "testalias2", ModelLoadStrategy::DropAndLoad)]
#[case(None, "testalias", ModelLoadStrategy::Load)]
fn test_model_load_strategy(
  #[case] loaded_alias: Option<String>,
  #[case] request_alias: &str,
  #[case] expected: ModelLoadStrategy,
) -> anyhow::Result<()> {
  let result = ModelLoadStrategy::choose(loaded_alias, request_alias);
  assert_eq!(expected, result);
  Ok(())
}

#[rstest]
#[awt]
#[serial(BodhiServerContext)]
#[anyhow_trace]
#[tokio::test]
async fn test_chat_completions_continue_strategy(
  mut mock_server: MockServer,
  #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  bin_path: TempDir,
) -> anyhow::Result<()> {
  let app_service_stub = app_service_stub_builder
    .with_settings(HashMap::from([
      (BODHI_EXEC_VARIANT, DEFAULT_VARIANT),
      (BODHI_EXEC_TARGET, BUILD_TARGET),
      (BODHI_EXEC_VARIANTS, BUILD_VARIANTS.join(",").as_str()),
      (BODHI_EXEC_NAME, EXEC_NAME),
      (
        BODHI_EXEC_LOOKUP_PATH,
        bin_path.path().display().to_string().as_str(),
      ),
    ]))
    .await
    .build()
    .await
    .unwrap();
  let hf_cache = app_service_stub.hf_cache();
  let model_file = HubFileBuilder::testalias()
    .hf_cache(hf_cache.clone())
    .build()
    .unwrap();
  let expected_input: Value = serde_json::from_str(
    r#"{"messages":[{"role":"user","content":"What day comes after Monday?"}],"model":"testalias:instruct"}"#,
  )?;
  mock_server
    .expect_chat_completions()
    .with(eq(expected_input))
    .times(1)
    .return_once(|_| async { Ok(mock_response("")) }.boxed());
  let server_args = LlamaServerArgsBuilder::default()
    .alias("testalias:instruct")
    .model(model_file.path())
    .build()?;
  let server_args_cl = server_args.clone();
  mock_server
    .expect_get_server_args()
    .times(1)
    .return_once(move || server_args_cl);
  mock_server
    .expect_stop()
    .times(1)
    .return_once(|| async { Ok(()) }.boxed());

  let server_factory = ServerFactoryStub::new(Box::new(mock_server));
  let shared_ctx = DefaultSharedContext::with_args(
    app_service_stub.hub_service(),
    app_service_stub.setting_service(),
    Box::new(server_factory),
  )
  .await;
  shared_ctx.reload(Some(server_args)).await?;
  let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
    "model": "testalias:instruct",
    "messages": [{"role": "user", "content": "What day comes after Monday?"}]
  }})?;
  let request_value = serde_json::to_value(&request)?;
  shared_ctx
    .forward_request(
      LlmEndpoint::ChatCompletions,
      request_value,
      Alias::User(UserAlias::testalias()),
    )
    .await?;
  shared_ctx.stop().await?;
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[serial(BodhiServerContext)]
#[anyhow_trace]
async fn test_chat_completions_load_strategy(
  #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  mut mock_server: MockServer,
  bin_path: TempDir,
) -> anyhow::Result<()> {
  let app_service_stub = app_service_stub_builder
    .with_settings(HashMap::from([
      (BODHI_EXEC_VARIANT, DEFAULT_VARIANT),
      (BODHI_EXEC_TARGET, BUILD_TARGET),
      (BODHI_EXEC_VARIANTS, BUILD_VARIANTS.join(",").as_str()),
      (BODHI_EXEC_NAME, EXEC_NAME),
      (
        BODHI_EXEC_LOOKUP_PATH,
        bin_path.path().display().to_string().as_str(),
      ),
      (BODHI_LLAMACPP_ARGS, "--verbose"),
    ]))
    .await
    .build()
    .await
    .unwrap();
  let expected_input: Value = serde_json::from_str(
    r#"{"messages":[{"role":"user","content":"What day comes after Monday?"}],"model":"testalias:instruct"}"#,
  )?;
  mock_server
    .expect_chat_completions()
    .with(eq(expected_input))
    .times(1)
    .return_once(|_| async { Ok(mock_response("")) }.boxed());

  let bodhi_server_factory = ServerFactoryStub::new(Box::new(mock_server));

  let shared_ctx = DefaultSharedContext::with_args(
    app_service_stub.hub_service(),
    app_service_stub.setting_service(),
    Box::new(bodhi_server_factory),
  )
  .await;
  let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
    "model": "testalias:instruct",
    "messages": [{"role": "user", "content": "What day comes after Monday?"}]
  }})?;
  let request_value = serde_json::to_value(&request)?;
  shared_ctx
    .forward_request(
      LlmEndpoint::ChatCompletions,
      request_value,
      Alias::User(UserAlias::testalias()),
    )
    .await?;
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
#[serial(BodhiServerContext)]
async fn test_chat_completions_drop_and_load_strategy(
  mut mock_server: MockServer,
  #[from(mock_server)] mut request_server: MockServer,
  #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  temp_hf_home: TempDir,
  bin_path: TempDir,
) -> anyhow::Result<()> {
  let app_service_stub = app_service_stub_builder
    .with_settings(HashMap::from([
      (BODHI_EXEC_VARIANT, DEFAULT_VARIANT),
      (BODHI_EXEC_TARGET, BUILD_TARGET),
      (BODHI_EXEC_VARIANTS, BUILD_VARIANTS.join(",").as_str()),
      (BODHI_EXEC_NAME, EXEC_NAME),
      (
        BODHI_EXEC_LOOKUP_PATH,
        bin_path.path().display().to_string().as_str(),
      ),
      (BODHI_LLAMACPP_ARGS, "--verbose"),
    ]))
    .await
    .build()
    .await
    .unwrap();
  let hf_cache = temp_hf_home.path().join("huggingface").join("hub");
  let loaded_model = HubFileBuilder::testalias()
    .hf_cache(hf_cache.clone())
    .build()
    .unwrap();
  let loaded_params = LlamaServerArgsBuilder::default()
    .alias("testalias:instruct")
    .model(loaded_model.path())
    .build()?;
  let expected_input: Value = serde_json::from_str(
    r#"{"messages":[{"role":"user","content":"What day comes after Monday?"}],"model":"fakemodel:instruct"}"#,
  )?;
  mock_server
    .expect_chat_completions()
    .with(eq(expected_input))
    .times(1)
    .return_once(|_| async { Ok(mock_response("")) }.boxed());
  mock_server
    .expect_stop()
    .times(1)
    .return_once(|| async { Ok(()) }.boxed());

  let request_model = HubFileBuilder::fakemodel()
    .hf_cache(hf_cache.clone())
    .build()?;
  let request_params = LlamaServerArgsBuilder::default()
    .alias("fakemodel:instruct")
    .model(request_model.path())
    .build()?;
  request_server
    .expect_get_server_args()
    .times(1)
    .return_once(move || request_params);
  request_server
    .expect_stop()
    .times(1)
    .return_once(|| async { Ok(()) }.boxed());
  let server_factory =
    ServerFactoryStub::new_with_instances(vec![Box::new(mock_server), Box::new(request_server)]);
  let shared_ctx = DefaultSharedContext::with_args(
    app_service_stub.hub_service(),
    app_service_stub.setting_service(),
    Box::new(server_factory),
  )
  .await;
  shared_ctx.reload(Some(loaded_params)).await?;
  let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
    "model": "fakemodel:instruct",
    "messages": [{"role": "user", "content": "What day comes after Monday?"}]
  }})?;
  let request_value = serde_json::to_value(&request)?;
  shared_ctx
    .forward_request(
      LlmEndpoint::ChatCompletions,
      request_value,
      Alias::User(UserAlias::testalias()),
    )
    .await?;
  shared_ctx.stop().await?;
  Ok(())
}
