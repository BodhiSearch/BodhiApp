/*
// TODO: clean up/delete
  #[fixture]
  fn setup() -> () {
    init_test_tracing();
  }

  /*
  #[ignore]
  #[rstest]
  #[tokio::test]
  #[serial]
  #[anyhow_trace]
  async fn test_routes_chat_completions_stream(
    app_service_stub: AppServiceTuple,
  ) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let request = serde_json::to_string(&json! {{
      "stream": true,
      "model": "TheBloke/Llama-2-7B-Chat-GGUF:llama-2-7b-chat.Q4_K_M.gguf",
      "seed": 42,
      "messages": [{"role": "user", "content": "You are a helpful assistant. What day comes after Monday?"}]
    }})
    .unwrap();
    let model_path = dirs::home_dir()
      .ok_or_else(|| anyhow!("unable to locate home dir"))?
      .join(".cache/huggingface/hub/models--TheBloke--Llama-2-7B-Chat-GGUF/snapshots/08a5566d61d7cb6b420c3e4387a39e0078e1f2fe5f055f3a03887385304d4bfa/llama-2-7b-chat.Q4_K_M.gguf")
      .canonicalize()?
      .to_str()
      .unwrap()
      .to_owned();
    let gpt_params = GptParams {
      model: model_path,
      ..Default::default()
    };
    let AppServiceTuple(_temp_bodhi_home, _temp_hf_home, _, _, service) = app_service_stub;
    let wrapper = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    let app = llm_router().with_state(RouterState::new(Arc::new(wrapper), Arc::new(service)));
    let response = app
      .oneshot(Request::post("/v1/chat/completions").json(request).unwrap())
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let response: Vec<CreateChatCompletionStreamResponse> = response.sse().await.unwrap();
    let content = response.into_iter().fold(String::new(), |mut f, r| {
      let content = r
        .choices
        .first()
        .unwrap()
        .delta
        .content
        .as_deref()
        .unwrap_or_default();
      f.push_str(content);
      f
    });
    assert_eq!("The day that comes after Monday is Tuesday.", content);
    Ok(())
  }
  */

*/