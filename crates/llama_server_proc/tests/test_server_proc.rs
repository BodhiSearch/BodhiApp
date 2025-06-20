use axum::http::StatusCode;
use llama_server_proc::{
  LlamaServer, LlamaServerArgsBuilder, Result, Server, BUILD_TARGET, DEFAULT_VARIANT, EXEC_NAME,
};
use objs::{HubFile, Repo};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use serde_json::Value;
use std::path::PathBuf;

#[fixture]
async fn server() -> LlamaServer {
  let executable_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("bin")
    .join(BUILD_TARGET)
    .join(DEFAULT_VARIANT)
    .join(EXEC_NAME);
  assert!(
    executable_path.exists(),
    "executable path does not exist: {}",
    executable_path.display()
  );
  let hf_cache = dirs::home_dir()
    .unwrap()
    .join(".cache")
    .join("huggingface")
    .join("hub");
  let model_file = HubFile::new(
    hf_cache,
    Repo::phi4_mini_instruct(),
    Repo::PHI4_MINI_INSTRUCT_Q4_K_M.to_string(),
    "7ff82c2aaa4dde30121698a973765f39be5288c0".to_string(),
    Some(1000),
  )
  .path();
  assert!(
    model_file.exists(),
    "model file does not exist: {}",
    model_file.display()
  );
  let args = LlamaServerArgsBuilder::default()
    .model(model_file)
    .alias("phi4:mini-instruct")
    .verbose(true)
    .build()
    .unwrap();

  let server = LlamaServer::new(&executable_path, &args).unwrap();
  server.start().await.unwrap_or_else(|_| {
    panic!(
      "failed to start server, executable_path={}, args={}",
      executable_path.display(),
      args
    )
  });
  server
}

fn chat_request(stream: bool) -> Value {
  serde_json::json!({
    "model": "phi4:mini-instruct",
    "seed": 42,
    "stream": stream,
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant."
      },
      {
        "role": "user",
        "content": "Answer in one word. What day comes after Monday?"
      }
    ]
  })
}

#[rstest]
#[case::messages_format(chat_request(false), "Tuesday")]
#[awt]
#[tokio::test]
async fn test_server_proc_chat_completions(
  #[future] server: LlamaServer,
  #[case] request_body: Value,
  #[case] expected_content: &str,
) -> Result<()> {
  let response = server.chat_completions(&request_body).await?;
  let status = response.status();
  let response_body = response.json::<Value>().await.unwrap();
  assert_eq!(StatusCode::OK, status, "body: {:?}", response_body);

  let response = response_body["choices"][0]["message"]["content"]
    .as_str()
    .unwrap();
  assert!(
    response.contains(expected_content),
    "expected_content: {} not found in response: {}",
    expected_content,
    response
  );
  assert_eq!("phi4:mini-instruct", response_body["model"]);
  assert_eq!("stop", response_body["choices"][0]["finish_reason"]);

  Ok(())
}

#[rstest]
#[case::messages_format(chat_request(true), "Tuesday")]
#[awt]
#[tokio::test]
async fn test_server_proc_chat_completions_streamed(
  #[future] server: LlamaServer,
  #[case] request_body: Value,
  #[case] expected_content: &str,
) -> Result<()> {
  let response = server.chat_completions(&request_body).await?;
  let status = response.status();
  let response_text = response.text().await.unwrap();
  assert_eq!(200, status, "body: {}", response_text);

  let streams = response_text
    .lines()
    .filter_map(|line| {
      if line.is_empty() || line == "data: [DONE]" {
        None
      } else if line.starts_with("data: ") {
        let value: Value = serde_json::from_str(line.strip_prefix("data: ").unwrap()).unwrap();
        Some(value)
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  let actual = streams[0..streams.len() - 1]
    .iter()
    .map(|stream| {
      stream["choices"][0]["delta"]["content"]
        .as_str()
        .unwrap_or_default()
    })
    .collect::<Vec<_>>()
    .join(" ");

  assert!(actual.contains(expected_content), "actual: {} not found in response: {}", actual, expected_content);

  let expected: Value =
    serde_json::from_str(r#"[{"delta":{},"finish_reason":"stop","index":0}]"#).unwrap();
  let last = streams.last().unwrap()["choices"].clone();
  assert_eq!(expected, last);

  Ok(())
}
