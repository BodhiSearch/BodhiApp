use crate::tokenizer_config::TokenizerConfig;
use crate::ContextError;
use async_openai::types::CreateChatCompletionRequest;
use llama_server_proc::{LlamaServer, LlamaServerArgs, LlamaServerArgsBuilder, Server};
use objs::{Alias, HubFile};
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use validator::Validate;

type Result<T> = std::result::Result<T, ContextError>;
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait SharedContext: std::fmt::Debug + Send + Sync {
  async fn set_exec_path(&mut self, path: PathBuf) -> Result<()>;

  async fn reload(&self, server_args: Option<LlamaServerArgs>) -> Result<()>;

  async fn stop(&self) -> Result<()>;

  async fn is_loaded(&self) -> bool;

  async fn chat_completions(
    &self,
    mut request: CreateChatCompletionRequest,
    alias: Alias,
    model_file: HubFile,
    tokenizer_file: HubFile,
  ) -> Result<reqwest::Response>;
}

pub trait ServerFactory: std::fmt::Debug + Send + Sync {
  fn create_server(
    &self,
    executable_path: &Path,
    server_args: &LlamaServerArgs,
  ) -> Result<Box<dyn Server>>;
}

#[derive(Debug)]
pub struct DefaultServerFactory;

impl ServerFactory for DefaultServerFactory {
  fn create_server(
    &self,
    executable_path: &Path,
    server_args: &LlamaServerArgs,
  ) -> Result<Box<dyn Server>> {
    let server = LlamaServer::new(executable_path.to_path_buf(), server_args.clone())?;
    Ok(Box::new(server))
  }
}

#[derive(Debug)]
pub struct DefaultSharedContext {
  factory: Box<dyn ServerFactory>,
  exec_path: PathBuf,
  server: RwLock<Option<Box<dyn Server>>>,
}

impl DefaultSharedContext {
  pub fn new(exec_path: PathBuf) -> Self {
    Self::with_args(Box::new(DefaultServerFactory), exec_path)
  }

  pub fn with_args(factory: Box<dyn ServerFactory>, exec_path: PathBuf) -> Self {
    Self {
      exec_path,
      factory,
      server: RwLock::new(None),
    }
  }

  async fn get_server_args(&self) -> Option<LlamaServerArgs> {
    let lock = self.server.read().await;
    lock.as_ref().map(|server| server.get_server_args())
  }
}

#[async_trait::async_trait]
impl SharedContext for DefaultSharedContext {
  async fn set_exec_path(&mut self, path: PathBuf) -> Result<()> {
    self.exec_path = path;
    let server_args = self.get_server_args().await;
    self.reload(server_args).await?;
    Ok(())
  }

  async fn is_loaded(&self) -> bool {
    let lock = self.server.read().await;
    lock.as_ref().is_some()
  }

  async fn reload(&self, server_args: Option<LlamaServerArgs>) -> Result<()> {
    self.stop().await?;
    let Some(server_args) = server_args else {
      return Ok(());
    };
    let server = self
      .factory
      .create_server(self.exec_path.as_ref(), &server_args)?;
    server.start().await?;
    *self.server.write().await = Some(server);
    Ok(())
  }

  async fn stop(&self) -> Result<()> {
    let mut lock = self.server.write().await;
    let server = lock.take();
    if let Some(server) = server {
      server.stop().await?;
    };
    Ok(())
  }

  async fn chat_completions(
    &self,
    mut request: CreateChatCompletionRequest,
    alias: Alias,
    model_file: HubFile,
    tokenizer_file: HubFile,
  ) -> Result<reqwest::Response> {
    let lock = self.server.read().await;
    let server = lock.as_ref();
    let loaded_model = server.map(|server| server.get_server_args().model);
    let request_model = model_file.path();
    let chat_template: TokenizerConfig = TokenizerConfig::try_from(tokenizer_file)?;
    chat_template.validate()?;
    alias.request_params.update(&mut request);
    let prompt = chat_template.apply_chat_template(&request.messages)?;
    let mut input_value = serde_json::to_value(request)?;
    input_value["prompt"] = serde_json::Value::String(prompt);
    // TODO: instead of comparing model path, compare server args
    match ModelLoadStrategy::choose(loaded_model, &request_model) {
      ModelLoadStrategy::Continue => {
        let response = server
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
          .chat_completions(&input_value)
          .await?;
        Ok(response)
      }
      ModelLoadStrategy::DropAndLoad => {
        drop(lock);
        let server_args = LlamaServerArgsBuilder::default()
          .model(request_model)
          .server_params(&alias.context_params)
          .build()?;
        self.reload(Some(server_args)).await?;
        let lock = self.server.read().await;
        let server = lock.as_ref();
        let response = server
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
          .chat_completions(&input_value)
          .await?;
        Ok(response)
      }
      ModelLoadStrategy::Load => {
        // TODO: take context params from alias
        // TODO: reload keeping lock and doing completions operation
        let server_args = LlamaServerArgsBuilder::default()
          .model(request_model)
          .server_params(&alias.context_params)
          .build()?;
        drop(lock);
        self.reload(Some(server_args)).await?;
        let lock = self.server.read().await;
        let server = lock.as_ref();
        let response = server
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?
          .chat_completions(&input_value)
          .await?;
        Ok(response)
      }
    }
  }
}

#[derive(Debug, PartialEq)]
enum ModelLoadStrategy {
  Continue,
  DropAndLoad,
  Load,
}

impl ModelLoadStrategy {
  fn choose(loaded_model: Option<PathBuf>, request_model: &Path) -> ModelLoadStrategy {
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
    shared_rw::{DefaultSharedContext, ModelLoadStrategy, SharedContext},
    test_utils::{mock_server, ServerFactoryStub},
    ServerFactory,
  };
  use anyhow_trace::anyhow_trace;
  use async_openai::types::CreateChatCompletionRequest;
  use futures::FutureExt;
  use llama_server_proc::{
    test_utils::{llama2_7b, mock_response},
    LlamaServerArgsBuilder, MockServer,
  };
  use mockall::predicate::eq;
  use objs::{test_utils::temp_hf_home, Alias, HubFileBuilder};
  use rstest::rstest;
  use serde_json::{json, Value};
  use serial_test::serial;
  use services::test_utils::{app_service_stub, AppServiceStub};
  use std::path::PathBuf;
  use tempfile::TempDir;

  #[ignore]
  #[tokio::test]
  async fn test_shared_rw_new() -> anyhow::Result<()> {
    let ctx = DefaultSharedContext::new(PathBuf::from("/tmp/test_server.exe"));
    assert!(!ctx.is_loaded().await);
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_new_reload(
    mock_server: MockServer,
    llama2_7b: PathBuf,
  ) -> anyhow::Result<()> {
    let server_args = LlamaServerArgsBuilder::default()
      .model(llama2_7b)
      .build()
      .unwrap();
    let factory: Box<dyn ServerFactory> = Box::new(ServerFactoryStub::new(Box::new(mock_server)));
    let ctx = DefaultSharedContext::with_args(factory, PathBuf::from("/tmp/test_server.exe"));
    ctx.reload(Some(server_args)).await?;
    assert!(ctx.is_loaded().await);
    ctx.stop().await?;
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_reload_with_none(
    mock_server: MockServer,
    #[from(mock_server)] mock_server_2: MockServer,
    llama2_7b: PathBuf,
  ) -> anyhow::Result<()> {
    let server_args = LlamaServerArgsBuilder::default()
      .model(&llama2_7b)
      .build()
      .unwrap();
    let factory: Box<dyn ServerFactory> = Box::new(ServerFactoryStub::new_with_instances(vec![
      Box::new(mock_server),
      Box::new(mock_server_2),
    ]));
    let ctx = DefaultSharedContext::with_args(factory, PathBuf::from("/tmp/test_server.exe"));
    ctx.reload(Some(server_args.clone())).await?;
    let model_params = ctx.get_server_args().await.unwrap();
    assert_eq!(llama2_7b, model_params.model);
    ctx.reload(None).await?;
    assert!(ctx.get_server_args().await.is_none());
    ctx.reload(Some(server_args)).await?;
    let server_args = ctx.get_server_args().await.unwrap();
    assert_eq!(llama2_7b, server_args.model);
    ctx.stop().await?;
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_try_stop(
    mock_server: MockServer,
    llama2_7b: PathBuf,
  ) -> anyhow::Result<()> {
    let server_args = LlamaServerArgsBuilder::default()
      .model(llama2_7b)
      .build()
      .unwrap();
    let factory: Box<dyn ServerFactory> = Box::new(ServerFactoryStub::new(Box::new(mock_server)));
    let ctx = DefaultSharedContext::with_args(factory, PathBuf::from("/tmp/test_server.exe"));
    ctx.reload(Some(server_args)).await?;
    ctx.stop().await?;
    assert!(!ctx.is_loaded().await);
    Ok(())
  }

  #[rstest]
  fn test_model_load_strategy_continue_if_request_and_model_file_same() -> anyhow::Result<()> {
    let loaded_model = PathBuf::from("/path/to/loaded_model.gguf");
    let request_model = loaded_model.clone();
    let result = ModelLoadStrategy::choose(Some(loaded_model), &request_model);
    assert_eq!(result, ModelLoadStrategy::Continue);
    Ok(())
  }

  #[rstest]
  fn test_model_load_strategy_drop_and_load_if_loaded_model_different_from_request_model(
  ) -> anyhow::Result<()> {
    let loaded_model = PathBuf::from("/path/to/loaded_model.gguf");
    let request_model = PathBuf::from("/path/to/request_model.gguf");
    let result = ModelLoadStrategy::choose(Some(loaded_model), &request_model);
    assert_eq!(result, ModelLoadStrategy::DropAndLoad);
    Ok(())
  }

  #[rstest]
  fn test_model_load_strategy_load_if_no_model_loaded() -> anyhow::Result<()> {
    let request_model = PathBuf::from("/path/to/request_model.gguf");
    let result = ModelLoadStrategy::choose(None, &request_model);
    assert_eq!(result, ModelLoadStrategy::Load);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[serial(BodhiServerContext)]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_chat_completions_continue_strategy(
    mut mock_server: MockServer,
    #[future] app_service_stub: AppServiceStub,
  ) -> anyhow::Result<()> {
    let hf_cache = app_service_stub.hf_cache();
    let model_file = HubFileBuilder::testalias()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let tokenizer_file = HubFileBuilder::testalias_tokenizer()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let expected_input: Value = serde_json::from_str(
      r#"{"messages":[{"role":"user","content":"What day comes after Monday?"}],"model":"testalias:instruct","prompt":"<|begin_of_text|><|start_header_id|>user<|end_header_id|>\n\nWhat day comes after Monday?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n"}"#,
    )?;
    mock_server
      .expect_chat_completions()
      .with(eq(expected_input))
      .return_once(|_| async { Ok(mock_response("")) }.boxed());
    let server_args = LlamaServerArgsBuilder::default()
      .model(model_file.path())
      .build()?;
    let server_args_cl = server_args.clone();
    mock_server
      .expect_get_server_args()
      .return_once(move || server_args_cl);
    mock_server
      .expect_stop()
      .return_once(|| async { Ok(()) }.boxed());

    let server_factory = ServerFactoryStub::new(Box::new(mock_server));
    let shared_ctx = DefaultSharedContext::with_args(
      Box::new(server_factory),
      PathBuf::from("/tmp/test_server.exe"),
    );
    shared_ctx.reload(Some(server_args)).await?;
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    shared_ctx
      .chat_completions(request, Alias::testalias(), model_file, tokenizer_file)
      .await?;
    shared_ctx.stop().await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[serial(BodhiServerContext)]
  #[anyhow_trace]
  async fn test_chat_completions_load_strategy(
    mut mock_server: MockServer,
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
    let expected_input: Value = serde_json::from_str(
      r#"{"messages":[{"role":"user","content":"What day comes after Monday?"}],"model":"testalias:instruct","prompt":"<|begin_of_text|><|start_header_id|>user<|end_header_id|>\n\nWhat day comes after Monday?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n"}"#,
    )?;
    mock_server
      .expect_chat_completions()
      .with(eq(expected_input))
      .return_once(|_| async { Ok(mock_response("")) }.boxed());

    let bodhi_server_factory = ServerFactoryStub::new(Box::new(mock_server));
    let shared_ctx = DefaultSharedContext::with_args(
      Box::new(bodhi_server_factory),
      PathBuf::from("/tmp/test_library.dylib"),
    );
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    shared_ctx
      .chat_completions(request, Alias::testalias(), model_file, tokenizer_file)
      .await?;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[serial(BodhiServerContext)]
  #[anyhow_trace]
  async fn test_chat_completions_drop_and_load_strategy(
    mut mock_server: MockServer,
    #[from(mock_server)] mut request_server: MockServer,
    temp_hf_home: TempDir,
  ) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface").join("hub");
    let loaded_model = HubFileBuilder::testalias()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();
    let loaded_params = LlamaServerArgsBuilder::default()
      .model(loaded_model.path())
      .build()?;
    let loaded_params_cl = loaded_params.clone();
    mock_server
      .expect_get_server_args()
      .return_once(move || loaded_params_cl);
    let expected_input: Value = serde_json::from_str(
      r#"{"messages":[{"role":"user","content":"What day comes after Monday?"}],"model":"fakemodel:instruct","prompt":"<|begin_of_text|><|start_header_id|>user<|end_header_id|>\n\nWhat day comes after Monday?<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n"}"#,
    )?;
    mock_server
      .expect_chat_completions()
      .with(eq(expected_input))
      .return_once(|_| async { Ok(mock_response("")) }.boxed());
    mock_server
      .expect_stop()
      .return_once(|| async { Ok(()) }.boxed());

    let request_model = HubFileBuilder::fakemodel()
      .hf_cache(hf_cache.clone())
      .build()?;
    let request_params = LlamaServerArgsBuilder::default()
      .model(request_model.path())
      .build()?;
    request_server
      .expect_get_server_args()
      .return_once(move || request_params);
    request_server
      .expect_stop()
      .return_once(|| async { Ok(()) }.boxed());
    let tokenizer_file = HubFileBuilder::testalias_tokenizer()
      .hf_cache(hf_cache.clone())
      .build()
      .unwrap();

    let server_factory =
      ServerFactoryStub::new_with_instances(vec![Box::new(mock_server), Box::new(request_server)]);
    let shared_ctx = DefaultSharedContext::with_args(
      Box::new(server_factory),
      PathBuf::from("/tmp/test_server.exe"),
    );
    shared_ctx.reload(Some(loaded_params)).await?;
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "fakemodel:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    shared_ctx
      .chat_completions(request, Alias::testalias(), loaded_model, tokenizer_file)
      .await?;
    shared_ctx.stop().await?;
    Ok(())
  }
}
