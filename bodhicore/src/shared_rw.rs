use crate::objs::{Alias, LocalModelFile};
use crate::service::DataServiceError;
#[cfg(test)]
use crate::test_utils::MockBodhiServerContext as BodhiServerContext;
use crate::tokenizer_config::TokenizerConfig;
use async_openai::types::CreateChatCompletionRequest;
#[cfg(not(test))]
use llama_server_bindings::BodhiServerContext;
use llama_server_bindings::{Callback, GptParams};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct SharedContextRw {
  // TODO: remove pub access
  pub ctx: RwLock<Option<BodhiServerContext>>,
}

#[derive(Debug, Error)]
pub enum ContextError {
  #[error("{0}")]
  LlamaCpp(#[from] anyhow::Error),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
  #[error("{0}")]
  Unreachable(String),
  #[error(transparent)]
  SerdeJson(#[from] serde_json::Error),
  #[error(transparent)]
  AppError(#[from] crate::error::AppError),
}

pub type Result<T> = std::result::Result<T, ContextError>;

#[async_trait::async_trait]
pub trait SharedContextRwFn: std::fmt::Debug + Send + Sync {
  async fn reload(&self, gpt_params: Option<GptParams>) -> Result<()>;

  async fn try_stop(&self) -> Result<()>;

  async fn has_model(&self) -> bool;

  async fn get_gpt_params(&self) -> Result<Option<GptParams>>;

  #[allow(clippy::ptr_arg)]
  async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
    model_file: LocalModelFile,
    tokenizer_file: LocalModelFile,
    callback: Option<Callback>,
    userdata: &String,
  ) -> Result<()>;
}

impl SharedContextRw {
  pub async fn new_shared_rw(gpt_params: Option<GptParams>) -> Result<Self>
  where
    Self: Sized,
  {
    let ctx = SharedContextRw {
      ctx: RwLock::new(None),
    };
    ctx.reload(gpt_params).await?;
    Ok(ctx)
  }

  #[cfg(test)]
  pub fn new(bodhi_ctx: BodhiServerContext) -> Self {
    SharedContextRw {
      ctx: RwLock::new(Some(bodhi_ctx)),
    }
  }
}

#[async_trait::async_trait]
impl SharedContextRwFn for SharedContextRw {
  async fn has_model(&self) -> bool {
    let lock = self.ctx.read().await;
    lock.as_ref().is_some()
  }

  async fn reload(&self, gpt_params: Option<GptParams>) -> crate::shared_rw::Result<()> {
    let mut lock = self.ctx.write().await;
    try_stop_with(&mut lock)?;
    let Some(gpt_params) = gpt_params else {
      return Ok(());
    };
    let ctx = BodhiServerContext::new(gpt_params)?;
    *lock = Some(ctx);
    let Some(ctx) = lock.as_ref() else {
      unreachable!("just injected ctx in rwlock");
    };
    ctx.init()?;
    ctx.start_event_loop()?;
    // TODO - if stopping server immediately after starting, gets stuck in
    // `waiting for event_thread to complete`
    // sleep for .5 sec to avoid this scenario
    tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
    Ok(())
  }

  async fn try_stop(&self) -> crate::shared_rw::Result<()> {
    let mut lock = self.ctx.write().await;
    try_stop_with(&mut lock)?;
    Ok(())
  }

  async fn get_gpt_params(&self) -> crate::shared_rw::Result<Option<GptParams>> {
    let lock = self.ctx.read().await;
    if let Some(opt) = lock.as_ref() {
      Ok(Some(opt.get_gpt_params()))
    } else {
      Ok(None)
    }
  }

  async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
    model_file: LocalModelFile,
    tokenizer_file: LocalModelFile,
    callback: Option<Callback>,
    userdata: &String,
  ) -> crate::shared_rw::Result<()> {
    let lock = self.ctx.read().await;
    let ctx = lock.as_ref();
    let loaded_model = ctx.map(|ctx| ctx.get_gpt_params().model.clone());
    let request_model = model_file.path().display().to_string();
    let chat_template: TokenizerConfig = TokenizerConfig::try_from(tokenizer_file)?;
    let prompt = chat_template.apply_chat_template(&request.messages)?;
    let mut input_value = serde_json::to_value(request)?;
    input_value["prompt"] = serde_json::Value::String(prompt);
    let input = serde_json::to_string(&input_value)?;
    match ModelLoadStrategy::choose(&loaded_model, &request_model) {
      ModelLoadStrategy::Continue => {
        ctx
          .ok_or(ContextError::Unreachable(
            "context should not be None".to_string(),
          ))?
          .completions(&input, "", callback, userdata as *const _ as *mut _)?;
        Ok(())
      }
      ModelLoadStrategy::DropAndLoad => {
        todo!()
      }
      ModelLoadStrategy::Load => todo!(),
    }
  }
}

fn try_stop_with(
  lock: &mut tokio::sync::RwLockWriteGuard<'_, Option<BodhiServerContext>>,
) -> Result<()> {
  let opt = lock.take();
  if let Some(mut ctx) = opt {
    ctx
      .stop()
      .map_err(|err: anyhow::Error| ContextError::LlamaCpp(err))?;
    drop(ctx);
  };
  Ok(())
}

#[derive(Debug, PartialEq)]
enum ModelLoadStrategy {
  Continue,
  DropAndLoad,
  Load,
}

impl ModelLoadStrategy {
  fn choose(loaded_model: &Option<String>, request_model: &str) -> ModelLoadStrategy {
    if let Some(loaded_model) = loaded_model {
      if loaded_model.eq(request_model) {
        ModelLoadStrategy::Continue
      } else {
        ModelLoadStrategy::DropAndLoad
      }
    } else {
      ModelLoadStrategy::Load
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{
    objs::LocalModelFile,
    shared_rw::{ModelLoadStrategy, SharedContextRw, SharedContextRwFn},
    test_utils::{hf_cache, MockBodhiServerContext},
  };
  use anyhow::anyhow;
  use anyhow_trace::anyhow_trace;
  use async_openai::types::{CreateChatCompletionRequest, CreateChatCompletionResponse};
  use llama_server_bindings::{
    bindings::llama_server_disable_logging, disable_llama_log, GptParams,
  };
  use mockall::predicate::{always, eq};
  use rstest::{fixture, rstest};
  use serde_json::json;
  use std::{
    ffi::{c_char, c_void},
    path::PathBuf,
    slice,
  };
  use tempfile::TempDir;

  #[fixture]
  fn model_file() -> String {
    let user_home = dirs::home_dir()
      .ok_or_else(|| anyhow!("failed to get users home dir"))
      .unwrap();
    let model_file = user_home.join(".cache/huggingface/hub/models--TheBloke--Llama-2-7B-Chat-GGUF/snapshots/08a5566d61d7cb6b420c3e4387a39e0078e1f2fe5f055f3a03887385304d4bfa/llama-2-7b-chat.Q4_K_M.gguf");
    assert!(model_file.exists());
    model_file.to_string_lossy().into_owned()
  }

  #[ignore]
  #[tokio::test]
  async fn test_shared_rw_new() -> anyhow::Result<()> {
    let ctx = SharedContextRw::new_shared_rw(None).await?;
    assert!(!ctx.has_model().await);
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_new_with_params(model_file: String) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let gpt_params = GptParams {
      model: model_file,
      ..GptParams::default()
    };
    let ctx = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    assert!(ctx.has_model().await);
    ctx.try_stop().await?;
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_reload(model_file: String) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let gpt_params = GptParams {
      model: model_file.clone(),
      ..GptParams::default()
    };
    let ctx = SharedContextRw::new_shared_rw(Some(gpt_params.clone())).await?;
    let model_params = ctx.get_gpt_params().await?.unwrap();
    assert_eq!(model_file, model_params.model);
    ctx.reload(None).await?;
    assert!(ctx.get_gpt_params().await?.is_none());
    ctx.reload(Some(gpt_params)).await?;
    let model_params = ctx.get_gpt_params().await?.unwrap();
    assert_eq!(model_file, model_params.model);
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_try_stop(model_file: String) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let gpt_params = GptParams {
      model: model_file,
      ..GptParams::default()
    };
    let mut ctx = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    ctx.try_stop().await?;
    assert!(!ctx.has_model().await);
    Ok(())
  }

  pub unsafe extern "C" fn test_callback(
    contents: *const c_char,
    size: usize,
    userdata: *mut c_void,
  ) -> usize {
    let slice = unsafe { slice::from_raw_parts(contents as *const u8, size) };
    let input_str = match std::str::from_utf8(slice) {
      Ok(s) => s,
      Err(_) => return 0,
    };

    let user_data_str = unsafe { &mut *(userdata as *mut String) };
    user_data_str.push_str(input_str);
    size
  }

  #[fixture]
  pub fn chat_request() -> String {
    let request = json!({
      "seed": 42,
      "messages": [
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "What day comes after Monday?"},
      ],
    });
    serde_json::to_string(&request).expect("should serialize chat completion request to string")
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_completions(
    model_file: String,
    chat_request: String,
  ) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let gpt_params = GptParams {
      seed: Some(42),
      model: model_file,
      ..GptParams::default()
    };
    let ctx = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    let userdata = String::with_capacity(1024);
    let lock = ctx.ctx.read().await;
    let inner = lock.as_ref().expect("should have context loaded");
    inner.completions(
      &chat_request,
      "",
      Some(test_callback),
      &userdata as *const _ as *mut _,
    )?;
    let response: CreateChatCompletionResponse =
      serde_json::from_str(&userdata).expect("parse as chat completion response json");
    assert_eq!(
      "The day that comes after Monday is Tuesday.",
      response.choices[0]
        .message
        .content
        .as_ref()
        .expect("content does not exists")
    );
    Ok(())
  }

  #[rstest]
  fn test_model_load_strategy_continue_if_request_and_model_file_same() -> anyhow::Result<()> {
    let loaded_model = "/path/to/loaded_model.gguf".to_string();
    let request_model = loaded_model.clone();
    let result = ModelLoadStrategy::choose(&Some(loaded_model), &request_model);
    assert_eq!(result, ModelLoadStrategy::Continue);
    Ok(())
  }

  #[rstest]
  fn test_model_load_strategy_drop_and_load_if_loaded_model_different_from_request_model(
  ) -> anyhow::Result<()> {
    let loaded_model = "/path/to/loaded_model.gguf".to_string();
    let request_model = "/path/to/request_model.gguf";
    let result = ModelLoadStrategy::choose(&Some(loaded_model), request_model);
    assert_eq!(result, ModelLoadStrategy::DropAndLoad);
    Ok(())
  }

  #[rstest]
  fn test_model_load_strategy_load_if_no_model_loaded() -> anyhow::Result<()> {
    let request_model = "/path/to/request_model.gguf";
    let result = ModelLoadStrategy::choose(&None, request_model);
    assert_eq!(result, ModelLoadStrategy::Load);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_chat_completions_continue_strategy(
    hf_cache: (TempDir, PathBuf),
  ) -> anyhow::Result<()> {
    let model_file = LocalModelFile::testalias_builder()
      .hf_cache(hf_cache.1.clone())
      .build()
      .unwrap();
    let model_filepath = model_file.path().display().to_string();
    let tokenizer_file = LocalModelFile::testalias_tokenizer_builder()
      .hf_cache(hf_cache.1.clone())
      .build()
      .unwrap();
    let mut mock = MockBodhiServerContext::default();
    let expected_input = 
      "{\"messages\":[{\"content\":\"What day comes after Monday?\",\"role\":\"user\"}],\"model\":\"testalias:instruct\",\"prompt\":\"<|begin_of_text|><|start_header_id|>user<|end_header_id|>\\n\\nWhat day comes after Monday?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\\n\\n\"}";
    mock
      .expect_completions()
      .with(eq(expected_input), eq(""), eq(None), always())
      .return_once(|_, _, _, _| Ok(()));
    mock.expect_get_gpt_params().return_once(move || GptParams {
      model: model_filepath,
      ..Default::default()
    });
    let shared_ctx = SharedContextRw::new(mock);
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    let userdata = String::new();
    shared_ctx
      .chat_completions(request, model_file, tokenizer_file, None, &userdata)
      .await?;
    Ok(())
  }
}
