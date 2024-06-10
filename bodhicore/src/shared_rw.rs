#[cfg(not(test))]
use llama_server_bindings::BodhiServerContext;
#[cfg(test)]
use crate::test_utils::MockBodhiServerContext as BodhiServerContext;

use validator::{Validate, ValidationErrors};
use crate::error::Common;
use crate::objs::{Alias, HubFile, ObjError};
use crate::service::DataServiceError;
use tokio::sync::mpsc::Sender;
use crate::tokenizer_config::TokenizerConfig;
use async_openai::types::CreateChatCompletionRequest;
use llama_server_bindings::{LlamaCppError, GptParams, GptParamsBuilder, GptParamsBuilderError};
use std::ffi::{c_char, c_void};
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct SharedContextRw {
  ctx: RwLock<Option<BodhiServerContext>>,
}

#[derive(Debug, Error)]
pub enum ContextError {
  #[error(transparent)]
  BodhiError(#[from] LlamaCppError),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
  #[error(transparent)]
  Common(#[from] Common),
  #[error(transparent)]
  BuilderError(#[from] GptParamsBuilderError),
  #[error(transparent)]
  ObjError(#[from] ObjError),
  #[error(transparent)]
  Validation(#[from] ValidationErrors),
  #[error(transparent)]
  Minijina(#[from] minijinja::Error),
  #[error("{0}")]
  Unreachable(String),
}

pub type Result<T> = std::result::Result<T, ContextError>;

unsafe extern "C" fn callback_stream(
  contents: *const c_char,
  size: usize,
  callback_userdata: *mut c_void,
) -> usize {
  let slice = unsafe { slice::from_raw_parts(contents as *const u8, size) };
  let input_str = match std::str::from_utf8(slice) {
    Ok(s) => s,
    Err(_) => return 0,
  }
  .to_owned();
  let userdata = &mut *(callback_userdata as *mut (Sender<String>, Arc<AtomicBool>));
  let sender = userdata.0.clone();
  let receiver_status = userdata.1.clone();

  if !receiver_status.load(Ordering::SeqCst) {
      return 0;
  }

  tokio::spawn(async move {
    if sender.send(input_str).await.is_err() {
      tracing::warn!("error sending generated token using callback, receiver closed, closing sender");
      receiver_status.store(false, Ordering::SeqCst);
    }
  });
  size
}

#[async_trait::async_trait]
pub trait SharedContextRwFn: std::fmt::Debug + Send + Sync {
  async fn reload(&self, gpt_params: Option<GptParams>) -> Result<()>;

  async fn try_stop(&self) -> Result<()>;

  async fn has_model(&self) -> bool;

  async fn get_gpt_params(&self) -> Result<Option<GptParams>>;

  #[allow(clippy::ptr_arg)]
  async fn chat_completions(
    &self,
    mut request: CreateChatCompletionRequest,
    alias: Alias,
    model_file: HubFile,
    tokenizer_file: HubFile,
    userdata: Sender<String>,
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
    mut request: CreateChatCompletionRequest,
    alias: Alias,
    model_file: HubFile,
    tokenizer_file: HubFile,
    userdata: Sender<String>,
  ) -> crate::shared_rw::Result<()> {
    let lock = self.ctx.read().await;
    let ctx = lock.as_ref();
    let loaded_model = ctx.map(|ctx| ctx.get_gpt_params().model.clone());
    let request_model = model_file.path().display().to_string();
    let chat_template: TokenizerConfig = TokenizerConfig::try_from(tokenizer_file)?;
    chat_template.validate()?;
    alias.request_params.update(&mut request);
    let prompt = chat_template.apply_chat_template(&request.messages)?;
    let mut input_value = serde_json::to_value(request).map_err(Common::SerdeJsonDeserialize)?;
    input_value["prompt"] = serde_json::Value::String(prompt);
    let input = serde_json::to_string(&input_value).map_err(Common::SerdeJsonDeserialize)?;
    let callback_userdata = (userdata, Arc::new(AtomicBool::new(true)));
    match ModelLoadStrategy::choose(&loaded_model, &request_model) {
      ModelLoadStrategy::Continue => {
        ctx
          .ok_or_else(||ContextError::Unreachable(
            "context should not be None".to_string(),
          ))?
          .completions(&input, "", Some(callback_stream), &callback_userdata as *const _ as *mut _)?;
        Ok(())
      }
      ModelLoadStrategy::DropAndLoad => {
        drop(lock);
        let mut new_gpt_params = GptParamsBuilder::default().model(request_model).build()?;
        alias.context_params.update(&mut new_gpt_params);
        self.reload(Some(new_gpt_params)).await?;
        let lock = self.ctx.read().await;
        let ctx = lock.as_ref();
        ctx.ok_or_else(||ContextError::Unreachable(
          "context should not be None".to_string(),
        ))?
        .completions(&input, "", Some(callback_stream), &callback_userdata as *const _ as *mut _)?;
        Ok(())
      }
      ModelLoadStrategy::Load => {
        // TODO: take context params from alias
        // TODO: reload keeping lock and doing completions operation
        let mut new_gpt_params = GptParamsBuilder::default().model(request_model).build()?;
        alias.context_params.update(&mut new_gpt_params);
        drop(lock);
        self.reload(Some(new_gpt_params)).await?;
        let lock = self.ctx.read().await;
        let ctx = lock.as_ref();
        ctx.ok_or_else(||ContextError::Unreachable(
          "context should not be None".to_string(),
        ))?
        .completions(&input, "", Some(callback_stream), &callback_userdata as *const _ as *mut _)?;
        Ok(())
      },
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
      .map_err(ContextError::BodhiError)?;
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
    objs::{Alias, HubFile},
    shared_rw::{ModelLoadStrategy, SharedContextRw, SharedContextRwFn},
    test_utils::{hf_cache, test_channel, MockBodhiServerContext},
  };
  use anyhow::anyhow;
  use anyhow_trace::anyhow_trace;
  use async_openai::types::{CreateChatCompletionRequest, CreateChatCompletionResponse};
  use llama_server_bindings::{
    bindings::llama_server_disable_logging, disable_llama_log, GptParams, GptParamsBuilder,
  };
  use mockall::predicate::{always, eq};
  use rstest::{fixture, rstest};
  use serde_json::json;
  use std::{
    ffi::{c_char, c_void},
    path::PathBuf, slice,
  };
  use tempfile::TempDir;
  use serial_test::serial;

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
    let ctx = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
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
  #[serial(BodhiServerContext)]
  #[anyhow_trace]
  async fn test_chat_completions_continue_strategy(
    hf_cache: (TempDir, PathBuf),
  ) -> anyhow::Result<()> {
    let (_temp, hf_cache) = hf_cache;
    let model_file = HubFile::testalias_builder()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let model_filepath = model_file.path().display().to_string();
    let tokenizer_file = HubFile::testalias_tokenizer_builder()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let mut mock = MockBodhiServerContext::default();
    let expected_input =
      "{\"messages\":[{\"content\":\"What day comes after Monday?\",\"role\":\"user\"}],\"model\":\"testalias:instruct\",\"prompt\":\"<|begin_of_text|><|start_header_id|>user<|end_header_id|>\\n\\nWhat day comes after Monday?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\\n\\n\"}";
    mock.expect_init().with().return_once(|| Ok(()));
    mock.expect_start_event_loop().with().return_once(|| Ok(()));
    mock
      .expect_completions()
      .with(eq(expected_input), eq(""), always(), always())
      .return_once(|_, _, _, _| Ok(()));
    let gpt_params = GptParamsBuilder::default().model(model_filepath).build()?;
    let gpt_params_cl = gpt_params.clone();
    mock.expect_get_gpt_params().return_once(move || gpt_params_cl);

    let ctx = MockBodhiServerContext::new_context();
    ctx.expect().with(eq(gpt_params.clone())).return_once(move |_| Ok(mock));

    let shared_ctx = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    let (tx, _rx) = test_channel();
    shared_ctx
      .chat_completions(request, Alias::testalias(), model_file, tokenizer_file, tx)
      .await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[serial(BodhiServerContext)]
  #[anyhow_trace]
  async fn test_chat_completions_load_strategy(
    hf_cache: (TempDir, PathBuf),
  ) -> anyhow::Result<()> {
    let (_temp, hf_cache) = hf_cache;
    let model_file = HubFile::testalias_builder()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let model_filepath = model_file.path().display().to_string();
    let tokenizer_file = HubFile::testalias_tokenizer_builder()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let mut mock = MockBodhiServerContext::default();
    let expected_input = 
      "{\"messages\":[{\"content\":\"What day comes after Monday?\",\"role\":\"user\"}],\"model\":\"testalias:instruct\",\"prompt\":\"<|begin_of_text|><|start_header_id|>user<|end_header_id|>\\n\\nWhat day comes after Monday?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\\n\\n\"}";
    mock.expect_init().with().return_once(|| Ok(()));
    mock.expect_start_event_loop().with().return_once(|| Ok(()));
    mock
      .expect_completions()
      .with(eq(expected_input), eq(""), always(), always())
      .return_once(|_, _, _, _| Ok(()));

    let ctx = MockBodhiServerContext::new_context();
    ctx.expect().with(eq(GptParams{model: model_filepath, ..Default::default()})).return_once(move |_| Ok(mock));

    let shared_ctx = SharedContextRw::new_shared_rw(None).await?;
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    let (tx, _rx) = test_channel();
    shared_ctx
      .chat_completions(request, Alias::testalias(), model_file, tokenizer_file, tx)
      .await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[serial(BodhiServerContext)]
  #[anyhow_trace]
  async fn test_chat_completions_drop_and_load_strategy(
    hf_cache: (TempDir, PathBuf),
  ) -> anyhow::Result<()> {
    let (_temp, hf_cache) = hf_cache;
    let loaded_model = HubFile::testalias_builder()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let loaded_model_filepath = loaded_model.path().display().to_string();
    let mut loaded_ctx = MockBodhiServerContext::default();
    loaded_ctx.expect_init().with().return_once(|| Ok(()));
    loaded_ctx.expect_start_event_loop().with().return_once(|| Ok(()));
    let loaded_params = GptParamsBuilder::default().model(loaded_model_filepath).build()?;
    let loaded_params_cl = loaded_params.clone();
    loaded_ctx.expect_get_gpt_params().return_once(move || loaded_params_cl);
    loaded_ctx.expect_stop().with().return_once(|| Ok(()));
    let expected_input =
      "{\"messages\":[{\"content\":\"What day comes after Monday?\",\"role\":\"user\"}],\"model\":\"fakemodel:instruct\",\"prompt\":\"<|begin_of_text|><|start_header_id|>user<|end_header_id|>\\n\\nWhat day comes after Monday?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\\n\\n\"}";
    loaded_ctx
      .expect_completions()
      .with(eq(expected_input), eq(""), always(), always())
      .return_once(|_, _, _, _| Ok(()));
    let ctx = MockBodhiServerContext::new_context();
    ctx.expect().with(eq(loaded_params.clone())).return_once(move |_| Ok(loaded_ctx));

    let mut request_context = MockBodhiServerContext::default();
    request_context.expect_init().with().return_once(|| Ok(()));
    request_context.expect_start_event_loop().with().return_once(|| Ok(()));

    let request_model = HubFile::fakemodel_builder().hf_cache(hf_cache.clone()).build()?;
    let request_model_filepath = request_model.path().display().to_string();
    let request_params = GptParamsBuilder::default().model(request_model_filepath).build()?;
    let request_params_cl = request_params.clone();
    request_context.expect_get_gpt_params().return_once(move || request_params_cl);
    let request_ctx = MockBodhiServerContext::new_context();
    request_ctx.expect().with(eq(request_params)).return_once(move |_| Ok(request_context));

    let tokenizer_file = HubFile::testalias_tokenizer_builder()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();

    let shared_ctx = SharedContextRw::new_shared_rw(Some(loaded_params)).await?;
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "fakemodel:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    let (tx, _rx) = test_channel();
    shared_ctx
      .chat_completions(request, Alias::testalias(), loaded_model, tokenizer_file, tx)
      .await?;
    Ok(())
  }}
