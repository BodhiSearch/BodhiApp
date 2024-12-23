use llama_server_proc::{
  LlamaCppServer, LlamaServerArgsBuilder, Result, BUILD_TARGET, DEFAULT_VARIANT, EXEC_NAME,
};
use objs::{HubFile, Repo};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use serde_json::Value;
use std::path::PathBuf;

#[fixture]
async fn server() -> LlamaCppServer {
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
    Repo::try_from("TheBloke/Llama-2-7B-Chat-GGUF").unwrap(),
    "llama-2-7b-chat.Q4_K_M.gguf".to_string(),
    "191239b3e26b2882fb562ffccdd1cf0f65402adb".to_string(),
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
    .verbose(true)
    .build()
    .unwrap();
  LlamaCppServer::start_server(&executable_path, args)
    .await
    .unwrap()
}

fn chat_request(stream: bool) -> Value {
  serde_json::json!({
    "model": "llama2:7b-chat",
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

fn prompt_request(stream: bool) -> Value {
  serde_json::json!({
    "model": "llama2:7b-chat",
    "seed": 42,
    "stream": stream,
    "prompt": r#"<s>[INST] <<SYS>>
You are a helpful assistant.
<</SYS>>

Answer in one word. What day comes after Monday? [/INST]"#,
    "add_special": false
  })
}

#[rstest]
#[case::messages_format(chat_request(false), "The day that comes after Monday is Tuesday.")]
#[case::prompt_format(prompt_request(false), "  Tuesday")]
#[awt]
#[tokio::test]
async fn test_server_proc_chat_completions(
  #[future] server: LlamaCppServer,
  #[case] request_body: Value,
  #[case] expected_content: &str,
) -> Result<()> {
  let response = server.chat_completions(request_body).await?;
  assert_eq!(200, response.status());
  let response_body = response.json::<Value>().await.unwrap();

  assert_eq!(
    expected_content,
    response_body["choices"][0]["message"]["content"]
      .as_str()
      .unwrap()
  );
  assert_eq!("llama2:7b-chat", response_body["model"]);
  assert_eq!("stop", response_body["choices"][0]["finish_reason"]);

  Ok(())
}

#[rstest]
#[case::messages_format(chat_request(true), [
  "The", " day", " that", " comes", " after", " Monday", " is", " T", "ues", "day", ".", "",
].as_slice())]
#[case::prompt_format(prompt_request(true), [" ", " T", "ues", "day", ""].as_slice())]
#[awt]
#[tokio::test]
async fn test_server_proc_chat_completions_streamed(
  #[future] server: LlamaCppServer,
  #[case] request_body: Value,
  #[case] expected_content: &[&str],
) -> Result<()> {
  let response = server.chat_completions(request_body).await?;
  assert_eq!(200, response.status());
  let response_text = response.text().await.unwrap();

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
    .map(|stream| stream["choices"][0]["delta"]["content"].as_str().unwrap())
    .collect::<Vec<_>>();

  assert_eq!(expected_content, actual);

  let expected: Value =
    serde_json::from_str(r#"[{"delta":{},"finish_reason":"stop","index":0}]"#).unwrap();
  let last = streams.last().unwrap()["choices"].clone();
  assert_eq!(expected, last);

  Ok(())
}
