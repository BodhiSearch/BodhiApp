use crate::ContextError;
use async_openai::types::CreateChatCompletionRequest;
use llama_server_proc::{
  exec_path_from, LlamaServer, LlamaServerArgs, LlamaServerArgsBuilder, Server,
};
use objs::Alias;
use services::HubService;
use std::fmt::Debug;
use std::{
  path::{Path, PathBuf},
  sync::Arc,
};
use tokio::sync::RwLock;
use tracing::info;

type Result<T> = std::result::Result<T, ContextError>;

#[derive(Debug, Clone)]
pub enum ServerState {
  Start,
  Stop,
  ChatCompletions { alias: String },
  Variant { variant: String },
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait SharedContext: std::fmt::Debug + Send + Sync {
  async fn set_exec_variant(&self, variant: &str) -> Result<()>;

  async fn reload(&self, server_args: Option<LlamaServerArgs>) -> Result<()>;

  async fn stop(&self) -> Result<()>;

  async fn is_loaded(&self) -> bool;

  async fn chat_completions(
    &self,
    mut request: CreateChatCompletionRequest,
    alias: Alias,
  ) -> Result<reqwest::Response>;

  async fn add_state_listener(&self, listener: Arc<dyn ServerStateListener>);

  async fn notify_state_listeners(&self, state: ServerState);
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ServerStateListener: Debug + Send + Sync {
  async fn on_state_change(&self, state: ServerState);
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
    let server = LlamaServer::new(executable_path, server_args.clone())?;
    Ok(Box::new(server))
  }
}

#[derive(Debug)]
pub struct DefaultSharedContext {
  hub_service: Arc<dyn HubService>,
  factory: Box<dyn ServerFactory>,
  exec_lookup_path: PathBuf,
  exec_variant: ExecVariant,
  server: RwLock<Option<Box<dyn Server>>>,
  state_listeners: RwLock<Vec<Arc<dyn ServerStateListener>>>,
}

impl DefaultSharedContext {
  pub fn new(
    hub_service: Arc<dyn HubService>,
    exec_lookup_path: &Path,
    exec_variant: &str,
  ) -> Self {
    Self::with_args(
      hub_service,
      Box::new(DefaultServerFactory),
      exec_lookup_path,
      exec_variant,
    )
  }

  pub fn with_args(
    hub_service: Arc<dyn HubService>,
    factory: Box<dyn ServerFactory>,
    exec_lookup_path: &Path,
    exec_variant: &str,
  ) -> Self {
    Self {
      hub_service,
      exec_lookup_path: exec_lookup_path.to_path_buf(),
      exec_variant: ExecVariant::new(exec_variant.to_string()),
      factory,
      server: RwLock::new(None),
      state_listeners: RwLock::new(Vec::new()),
    }
  }

  async fn get_server_args(&self) -> Option<LlamaServerArgs> {
    let lock = self.server.read().await;
    lock.as_ref().map(|server| server.get_server_args())
  }
}

#[async_trait::async_trait]
impl SharedContext for DefaultSharedContext {
  async fn set_exec_variant(&self, variant: &str) -> Result<()> {
    self.exec_variant.set(variant.to_string()).await;
    let server_args = self.get_server_args().await;
    self
      .notify_state_listeners(ServerState::Variant {
        variant: variant.to_string(),
      })
      .await;
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
    let exec_path = exec_path_from(
      self.exec_lookup_path.as_ref(),
      self.exec_variant.get().await.as_ref(),
    );
    if !exec_path.exists() {
      return Err(ContextError::ExecNotExists(
        exec_path.to_string_lossy().to_string(),
      ))?;
    }
    let server = self.factory.create_server(&exec_path, &server_args)?;
    server.start().await?;
    *self.server.write().await = Some(server);
    self.notify_state_listeners(ServerState::Start).await;
    info!(?exec_path, "server started");
    Ok(())
  }

  async fn stop(&self) -> Result<()> {
    let mut lock = self.server.write().await;
    let server = lock.take();
    if let Some(server) = server {
      server.stop().await?;
      self.notify_state_listeners(ServerState::Stop).await;
    };
    Ok(())
  }

  async fn chat_completions(
    &self,
    mut request: CreateChatCompletionRequest,
    alias: Alias,
  ) -> Result<reqwest::Response> {
    let lock = self.server.read().await;
    let server = lock.as_ref();
    let loaded_alias = server.map(|server| server.get_server_args().alias);
    let request_alias = &alias.alias;
    let model_file = self
      .hub_service
      .find_local_file(&alias.repo, &alias.filename, Some(alias.snapshot.clone()))?
      .path();
    alias.request_params.update(&mut request);
    let input_value = serde_json::to_value(request)?;
    let alias_name = alias.alias.clone();
    let result = match ModelLoadStrategy::choose(loaded_alias, request_alias) {
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
          .alias(alias.alias)
          .model(model_file.to_string_lossy().to_string())
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
          .alias(alias.alias)
          .model(model_file.to_string_lossy().to_string())
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
    };
    self
      .notify_state_listeners(ServerState::ChatCompletions { alias: alias_name })
      .await;
    result
  }

  async fn add_state_listener(&self, listener: Arc<dyn ServerStateListener>) {
    let mut listeners = self.state_listeners.write().await;
    if !listeners
      .iter()
      .any(|existing| std::ptr::eq(existing.as_ref(), listener.as_ref()))
    {
      listeners.push(listener);
    }
  }

  async fn notify_state_listeners(&self, state: ServerState) {
    let listeners = self.state_listeners.read().await;
    for listener in listeners.iter() {
      let listener_clone = listener.clone();
      let state_clone = state.clone();
      tokio::spawn(async move {
        listener_clone.on_state_change(state_clone).await;
      });
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
  fn choose(loaded_alias: Option<String>, request_alias: &str) -> ModelLoadStrategy {
    if let Some(loaded_model) = loaded_alias {
      if loaded_model.eq(request_alias) {
        ModelLoadStrategy::Continue
      } else {
        ModelLoadStrategy::DropAndLoad
      }
    } else {
      ModelLoadStrategy::Load
    }
  }
}

#[derive(Debug)]
pub struct ExecVariant {
  inner: RwLock<String>,
}

impl ExecVariant {
  pub fn new(variant: String) -> Self {
    Self {
      inner: RwLock::new(variant),
    }
  }

  pub async fn get(&self) -> String {
    self.inner.read().await.clone()
  }

  pub async fn set(&self, variant: String) {
    *self.inner.write().await = variant;
  }
}

impl Default for ExecVariant {
  fn default() -> Self {
    Self::new(llama_server_proc::DEFAULT_VARIANT.to_string())
  }
}

#[cfg(test)]
mod test {
  use crate::{
    shared_rw::{DefaultSharedContext, ModelLoadStrategy, SharedContext},
    test_utils::{bin_path, mock_server, ServerFactoryStub},
  };
  use anyhow_trace::anyhow_trace;
  use async_openai::types::CreateChatCompletionRequest;
  use futures::FutureExt;
  use llama_server_proc::{test_utils::mock_response, LlamaServerArgsBuilder, MockServer};
  use mockall::predicate::eq;
  use objs::{test_utils::temp_hf_home, Alias, HubFileBuilder};
  use rstest::rstest;
  use serde_json::{json, Value};
  use serial_test::serial;
  use services::{
    test_utils::{app_service_stub, AppServiceStub},
    AppService,
  };
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
    #[future] app_service_stub: AppServiceStub,
    bin_path: TempDir,
  ) -> anyhow::Result<()> {
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
      Box::new(server_factory),
      bin_path.path(),
      llama_server_proc::DEFAULT_VARIANT,
    );
    shared_ctx.reload(Some(server_args)).await?;
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    shared_ctx
      .chat_completions(request, Alias::testalias())
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
    #[future] app_service_stub: AppServiceStub,
    bin_path: TempDir,
    mut mock_server: MockServer,
  ) -> anyhow::Result<()> {
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
      Box::new(bodhi_server_factory),
      bin_path.path(),
      llama_server_proc::DEFAULT_VARIANT,
    );
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "testalias:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    shared_ctx
      .chat_completions(request, Alias::testalias())
      .await?;
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  #[serial(BodhiServerContext)]
  #[anyhow_trace]
  async fn test_chat_completions_drop_and_load_strategy(
    mut mock_server: MockServer,
    #[from(mock_server)] mut request_server: MockServer,
    #[future] app_service_stub: AppServiceStub,
    bin_path: TempDir,
    temp_hf_home: TempDir,
  ) -> anyhow::Result<()> {
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
      Box::new(server_factory),
      bin_path.path(),
      llama_server_proc::DEFAULT_VARIANT,
    );
    shared_ctx.reload(Some(loaded_params)).await?;
    let request = serde_json::from_value::<CreateChatCompletionRequest>(json! {{
      "model": "fakemodel:instruct",
      "messages": [{"role": "user", "content": "What day comes after Monday?"}]
    }})?;
    shared_ctx
      .chat_completions(request, Alias::testalias())
      .await?;
    shared_ctx.stop().await?;
    Ok(())
  }
}
