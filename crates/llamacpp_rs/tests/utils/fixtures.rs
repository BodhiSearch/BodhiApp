use std::{env, path::PathBuf};

use async_openai::types::{
  ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
  ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use llamacpp_rs::{BodhiServerContext, CommonParams, ServerContext};
use rstest::fixture;

#[fixture]
pub fn chat_completion_request() -> String {
  let messages = vec![
    ChatCompletionRequestMessage::System(
      ChatCompletionRequestSystemMessageArgs::default()
        .content("You are a helpful assistant.")
        .build()
        .expect("error building system message"),
    ),
    ChatCompletionRequestMessage::User(
      ChatCompletionRequestUserMessageArgs::default()
        .content("What day comes after Monday?")
        .build()
        .expect("expected to build user messages but failed"),
    ),
  ];
  let request = CreateChatCompletionRequestArgs::default()
    .messages(messages)
    .seed(42)
    .build()
    .expect("expected to build request, but failed");
  serde_json::to_string(&request).expect("should serialize chat completion request to string")
}

#[fixture]
pub fn common_params_default() -> CommonParams {
  let mut model_path = dirs::home_dir().expect("Home directory not found");
  model_path.push(".cache/huggingface/hub/models--TheBloke--Llama-2-7b-Chat-GGUF/snapshots/191239b3e26b2882fb562ffccdd1cf0f65402adb/llama-2-7b-chat.Q4_K_M.gguf");
  let model_path = model_path
    .canonicalize()
    .expect("error finding path for test LLM model file");
  if !model_path.exists() {
    panic!("model file does not exists at: {}", model_path.display());
  }
  let params = CommonParams {
    seed: Some(42),
    n_predict: None,
    n_ctx: None,
    model: model_path
      .to_str()
      .expect("error while unwrapping model_path")
      .to_string(),
    embedding: Some(false),
    n_parallel: None,
    n_keep: None,
  };
  params
}

#[fixture]
pub fn bodhi_server_ctx(common_params_default: CommonParams) -> BodhiServerContext {
  let ctx = BodhiServerContext::default();
  let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../llamacpp-sys/libs")
    .join(llamacpp_sys::BUILD_TARGET)
    .join(llamacpp_sys::DEFAULT_VARIANT)
    .join(llamacpp_sys::LIBRARY_NAME);
  assert!(
    lib_path.exists(),
    "library path does not exist: {}",
    lib_path.display()
  );
  let lib_path = lib_path.canonicalize().unwrap();
  ctx
    .load_library(&lib_path)
    .expect("error while loading library");
  ctx
    .disable_logging()
    .expect("error while disabling logging");
  ctx
    .create_context(&common_params_default)
    .expect("error while building fixture bodhi server");
  ctx
}
