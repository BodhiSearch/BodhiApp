use crate::ContextError;
use crate::{obj_exts::update, tokenizer_config::TokenizerConfig};
use async_openai::types::CreateChatCompletionRequest;
use llamacpp_rs::{BodhiServerContext, CommonParams, CommonParamsBuilder, ServerContext};
use objs::{Alias, HubFile};
use std::ffi::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use validator::Validate;

type Result<T> = std::result::Result<T, ContextError>;

#[derive(Debug)]
struct CallbackUserdata {
  stream: bool,
  sender: Sender<String>,
  rx_state: Arc<AtomicBool>,
}

#[allow(clippy::unnecessary_cast)]
extern "C" fn callback_stream(
  contents: *const c_char,
  size: usize,
  callback_userdata: *mut c_void,
) -> usize {
  let userdata = unsafe { &mut *(callback_userdata as *mut CallbackUserdata) };
  if !userdata.rx_state.load(Ordering::SeqCst) {
    drop(unsafe { Box::from_raw(userdata) });
    return 0;
  }

  let slice = unsafe { slice::from_raw_parts(contents as *const u8, size) };
  let input_str = match std::str::from_utf8(slice) {
    Ok(s) => s,
    Err(_) => {
      drop(unsafe { Box::from_raw(userdata) });
      return 0;
    }
  }
  .to_owned();

  tokio::spawn(async move {
    let stream_done = input_str == "data: [DONE]\n\n";
    if userdata.sender.send(input_str).await.is_err() {
      tracing::warn!(
        "error sending generated token using callback, receiver closed, closing sender"
      );
      userdata.rx_state.store(false, Ordering::SeqCst);
    }
    if !userdata.stream || stream_done {
      drop(unsafe { Box::from_raw(userdata) });
    }
  });
  size
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait SharedContextRw: std::fmt::Debug + Send + Sync {
  async fn set_library_path(&mut self, path: PathBuf) -> Result<()>;

  async fn reload(&self, gpt_params: Option<CommonParams>) -> Result<()>;

  async fn try_stop(&self) -> Result<()>;

  async fn has_model(&self) -> bool;

  async fn get_common_params(&self) -> Result<Option<CommonParams>>;

  async fn chat_completions(
    &self,
    mut request: CreateChatCompletionRequest,
    alias: Alias,
    model_file: HubFile,
    tokenizer_file: HubFile,
    userdata: Sender<String>,
  ) -> Result<()>;

  fn disable_logging(&mut self);
}

pub trait ServerContextFactory: std::fmt::Debug + Send + Sync {
  fn create_server_context(&self) -> Box<dyn ServerContext>;
}

#[derive(Debug)]
pub struct DefaultServerContextFactory;

impl ServerContextFactory for DefaultServerContextFactory {
  fn create_server_context(&self) -> Box<dyn ServerContext> {
    Box::new(BodhiServerContext::default())
  }
}

#[derive(Debug)]
pub struct DefaultSharedContextRw {
  disable_logging: bool,
  library_path: Option<PathBuf>,
  factory: Box<dyn ServerContextFactory>,
  ctx: RwLock<Option<Box<dyn ServerContext + 'static>>>,
}

impl Default for DefaultSharedContextRw {
  fn default() -> Self {
    Self::new(true, Box::new(DefaultServerContextFactory), None)
  }
}

impl DefaultSharedContextRw {
  pub fn new(
    disable_logging: bool,
    factory: Box<dyn ServerContextFactory>,
    library_path: Option<PathBuf>,
  ) -> Self {
    Self {
      disable_logging,
      factory,
      ctx: RwLock::new(None),
      library_path,
    }
  }
}

#[async_trait::async_trait]
impl SharedContextRw for DefaultSharedContextRw {
  async fn set_library_path(&mut self, path: PathBuf) -> Result<()> {
    self.library_path = Some(path.to_path_buf());
    let common_params = {
      let params = self.ctx.read().await;
      params.as_ref().map(|ctx| ctx.get_common_params())
    };
    self.reload(common_params).await?;
    Ok(())
  }

  async fn has_model(&self) -> bool {
    let lock = self.ctx.read().await;
    lock.as_ref().is_some()
  }

  async fn reload(&self, gpt_params: Option<CommonParams>) -> crate::shared_rw::Result<()> {
    let mut lock = self.ctx.write().await;
    try_stop_with(&mut lock)?;
    let Some(gpt_params) = gpt_params else {
      return Ok(());
    };
    let ctx = self.factory.create_server_context();
    let library_path: &Path = match self.library_path.as_ref() {
      Some(path) => path,
      None => Err(ContextError::LibraryPathMissing)?,
    };
    ctx.load_library(library_path)?;
    if self.disable_logging {
      let _ = ctx.disable_logging();
    }
    ctx.create_context(&gpt_params)?;
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

  async fn get_common_params(&self) -> crate::shared_rw::Result<Option<CommonParams>> {
    let lock = self.ctx.read().await;
    if let Some(opt) = lock.as_ref() {
      Ok(Some(opt.get_common_params()))
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
    sender: Sender<String>,
  ) -> crate::shared_rw::Result<()> {
    let lock = self.ctx.read().await;
    let ctx = lock.as_ref();
    let loaded_model = ctx.map(|ctx| ctx.get_common_params().model.clone());
    let request_model = model_file.path().display().to_string();
    let chat_template: TokenizerConfig = TokenizerConfig::try_from(tokenizer_file)?;
    chat_template.validate()?;
    alias.request_params.update(&mut request);
    let prompt = chat_template.apply_chat_template(&request.messages)?;
    let stream = request.stream.unwrap_or(false);
    let mut input_value = serde_json::to_value(request)?;
    input_value["prompt"] = serde_json::Value::String(prompt);
    let input = serde_json::to_string(&input_value)?;
    let userdata = CallbackUserdata {
      stream,
      sender,
      rx_state: Arc::new(AtomicBool::new(true)),
    };
    match ModelLoadStrategy::choose(&loaded_model, &request_model) {
      ModelLoadStrategy::Continue => {
        let userdata_ptr: *mut c_void = Box::into_raw(Box::new(userdata)) as *mut _;
        ctx
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
          .completions(&input, "", callback_stream, userdata_ptr)?;
        Ok(())
      }
      ModelLoadStrategy::DropAndLoad => {
        drop(lock);
        let mut new_gpt_params = CommonParamsBuilder::default()
          .model(request_model)
          .build()?;
        update(&alias.context_params, &mut new_gpt_params);
        self.reload(Some(new_gpt_params)).await?;
        let lock = self.ctx.read().await;
        let ctx = lock.as_ref();
        let userdata_ptr: *mut c_void = Box::into_raw(Box::new(userdata)) as *mut _;
        ctx
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
          .completions(&input, "", callback_stream, userdata_ptr)?;
        Ok(())
      }
      ModelLoadStrategy::Load => {
        // TODO: take context params from alias
        // TODO: reload keeping lock and doing completions operation
        let mut new_gpt_params = CommonParamsBuilder::default()
          .model(request_model)
          .build()?;
        update(&alias.context_params, &mut new_gpt_params);
        drop(lock);
        self.reload(Some(new_gpt_params)).await?;
        let lock = self.ctx.read().await;
        let ctx = lock.as_ref();
        let userdata_ptr: *mut c_void = Box::into_raw(Box::new(userdata)) as *mut _;
        ctx
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
          .completions(&input, "", callback_stream, userdata_ptr)?;
        Ok(())
      }
    }
  }

  fn disable_logging(&mut self) {
    self.disable_logging = true;
  }
}

fn try_stop_with(
  lock: &mut tokio::sync::RwLockWriteGuard<'_, Option<Box<dyn ServerContext + 'static>>>,
) -> Result<()> {
  let opt = lock.take();
  if let Some(mut ctx) = opt {
    ctx.stop().map_err(ContextError::LlamaCpp)?;
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
  use std::path::PathBuf;

  use crate::{
    shared_rw::{DefaultSharedContextRw, ModelLoadStrategy, SharedContextRw},
    test_utils::{mock_server_ctx, test_channel, BodhiServerFactoryStub},
    ServerContextFactory,
  };
  use anyhow_trace::anyhow_trace;
  use async_openai::types::CreateChatCompletionRequest;
  use llamacpp_rs::{
    test_utils::llama2_7b_str, CommonParams, CommonParamsBuilder, MockServerContext,
  };
  use mockall::predicate::{always, eq};
  use objs::{test_utils::temp_hf_home, Alias, HubFileBuilder};
  use rstest::rstest;
  use serde_json::json;
  use serial_test::serial;
  use services::test_utils::{app_service_stub, AppServiceStub};
  use tempfile::TempDir;

  #[ignore]
  #[tokio::test]
  async fn test_shared_rw_new() -> anyhow::Result<()> {
    let ctx = DefaultSharedContextRw::default();
    assert!(!ctx.has_model().await);
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_new_reload(
    mock_server_ctx: MockServerContext,
    llama2_7b_str: String,
  ) -> anyhow::Result<()> {
    let gpt_params = CommonParams {
      model: llama2_7b_str,
      ..CommonParams::default()
    };
    let factory: Box<dyn ServerContextFactory> =
      Box::new(BodhiServerFactoryStub::new(Box::new(mock_server_ctx)));
    let ctx = DefaultSharedContextRw::new(
      false,
      factory,
      Some(PathBuf::from("/tmp/test_library.dylib")),
    );
    ctx.reload(Some(gpt_params)).await?;
    assert!(ctx.has_model().await);
    ctx.try_stop().await?;
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_reload_with_none(
    mock_server_ctx: MockServerContext,
    #[from(mock_server_ctx)] mock_server_ctx_2: MockServerContext,
    llama2_7b_str: String,
  ) -> anyhow::Result<()> {
    let gpt_params = CommonParams {
      model: llama2_7b_str.clone(),
      ..CommonParams::default()
    };
    let factory: Box<dyn ServerContextFactory> =
      Box::new(BodhiServerFactoryStub::new_with_instances(vec![
        Box::new(mock_server_ctx),
        Box::new(mock_server_ctx_2),
      ]));
    let ctx = DefaultSharedContextRw::new(
      false,
      factory,
      Some(PathBuf::from("/tmp/test_library.dylib")),
    );
    ctx.reload(Some(gpt_params.clone())).await?;
    let model_params = ctx.get_common_params().await?.unwrap();
    assert_eq!(llama2_7b_str, model_params.model);
    ctx.reload(None).await?;
    assert!(ctx.get_common_params().await?.is_none());
    ctx.reload(Some(gpt_params)).await?;
    let model_params = ctx.get_common_params().await?.unwrap();
    assert_eq!(llama2_7b_str, model_params.model);
    ctx.try_stop().await?;
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_try_stop(
    mock_server_ctx: MockServerContext,
    llama2_7b_str: String,
  ) -> anyhow::Result<()> {
    let gpt_params = CommonParams {
      model: llama2_7b_str,
      ..CommonParams::default()
    };
    let factory: Box<dyn ServerContextFactory> =
      Box::new(BodhiServerFactoryStub::new(Box::new(mock_server_ctx)));
    let ctx = DefaultSharedContextRw::new(
      false,
      factory,
      Some(PathBuf::from("/tmp/test_library.dylib")),
    );
    ctx.reload(Some(gpt_params)).await?;
    ctx.try_stop().await?;
    assert!(!ctx.has_model().await);
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
  #[awt]
  #[serial(BodhiServerContext)]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_chat_completions_continue_strategy(
    mut mock_server_ctx: MockServerContext,
    #[future] app_service_stub: AppServiceStub,
  ) -> anyhow::Result<()> {
    let hf_cache = app_service_stub.hf_cache();
    let model_file = HubFileBuilder::testalias()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let model_filepath = model_file.path().display().to_string();
    let tokenizer_file = HubFileBuilder::testalias_tokenizer()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let expected_input = r#"{"messages":[{"role":"user","content":"What day comes after Monday?"}],"model":"testalias:instruct","prompt":"<|begin_of_text|><|start_header_id|>user<|end_header_id|>\n\nWhat day comes after Monday?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n"}"#;
    mock_server_ctx
      .expect_completions()
      .with(eq(expected_input), eq(""), always(), always())
      .return_once(|_, _, _, _| Ok(()));
    let gpt_params = CommonParamsBuilder::default()
      .model(model_filepath)
      .build()?;
    let gpt_params_cl = gpt_params.clone();
    mock_server_ctx
      .expect_get_common_params()
      .return_once(move || gpt_params_cl);

    let bodhi_server_factory = BodhiServerFactoryStub::new(Box::new(mock_server_ctx));
    let shared_ctx = DefaultSharedContextRw::new(
      false,
      Box::new(bodhi_server_factory),
      Some(PathBuf::from("/tmp/test_library.dylib")),
    );
    shared_ctx.reload(Some(gpt_params)).await?;
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
    mut mock_server_ctx: MockServerContext,
    temp_hf_home: TempDir,
  ) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface").join("hub");
    let model_file = HubFileBuilder::testalias()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let tokenizer_file = HubFileBuilder::testalias_tokenizer()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let expected_input = r#"{"messages":[{"role":"user","content":"What day comes after Monday?"}],"model":"testalias:instruct","prompt":"<|begin_of_text|><|start_header_id|>user<|end_header_id|>\n\nWhat day comes after Monday?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n"}"#;
    mock_server_ctx
      .expect_completions()
      .with(eq(expected_input), eq(""), always(), always())
      .return_once(|_, _, _, _| Ok(()));

    let bodhi_server_factory = BodhiServerFactoryStub::new(Box::new(mock_server_ctx));
    let shared_ctx = DefaultSharedContextRw::new(
      false,
      Box::new(bodhi_server_factory),
      Some(PathBuf::from("/tmp/test_library.dylib")),
    );
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
    mut mock_server_ctx: MockServerContext,
    #[from(mock_server_ctx)] mut request_context: MockServerContext,
    temp_hf_home: TempDir,
  ) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface").join("hub");
    let loaded_model = HubFileBuilder::testalias()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let loaded_model_filepath = loaded_model.path().display().to_string();
    let loaded_params = CommonParamsBuilder::default()
      .model(loaded_model_filepath)
      .build()?;
    let loaded_params_cl = loaded_params.clone();
    mock_server_ctx
      .expect_get_common_params()
      .return_once(move || loaded_params_cl);
    let expected_input = r#"{"messages":[{"role":"user","content":"What day comes after Monday?"}],"model":"fakemodel:instruct","prompt":"<|begin_of_text|><|start_header_id|>user<|end_header_id|>\n\nWhat day comes after Monday?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n"}"#;
    mock_server_ctx
      .expect_completions()
      .with(eq(expected_input), eq(""), always(), always())
      .return_once(|_, _, _, _| Ok(()));

    let request_model = HubFileBuilder::fakemodel()
      .hf_cache(hf_cache.clone())
      .build()?;
    let request_model_filepath = request_model.path().display().to_string();
    let request_params = CommonParamsBuilder::default()
      .model(request_model_filepath)
      .build()?;
    let request_params_cl = request_params.clone();
    request_context
      .expect_get_common_params()
      .return_once(move || request_params_cl);
    let tokenizer_file = HubFileBuilder::testalias_tokenizer()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();

    let bodhi_server_factory = BodhiServerFactoryStub::new_with_instances(vec![
      Box::new(mock_server_ctx),
      Box::new(request_context),
    ]);
    let shared_ctx = DefaultSharedContextRw::new(
      false,
      Box::new(bodhi_server_factory),
      Some(PathBuf::from("/tmp/test_library.dylib")),
    );
    shared_ctx.reload(Some(loaded_params)).await?;
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "fakemodel:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    let (tx, _rx) = test_channel();
    shared_ctx
      .chat_completions(
        request,
        Alias::testalias(),
        loaded_model,
        tokenizer_file,
        tx,
      )
      .await?;
    Ok(())
  }
}
