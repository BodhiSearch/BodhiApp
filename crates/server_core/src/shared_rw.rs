use crate::{merge_server_args, ContextError, LlmEndpoint};
use llama_server_proc::{LlamaServer, LlamaServerArgs, LlamaServerArgsBuilder, Server};
use objs::Alias;
use serde_json::Value;
use services::{HubService, SettingService};
use std::fmt::Debug;
use std::{path::Path, sync::Arc};
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

  async fn forward_request(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
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
  setting_service: Arc<dyn SettingService>,
  factory: Box<dyn ServerFactory>,
  exec_variant: ExecVariant,
  server: RwLock<Option<Box<dyn Server>>>,
  state_listeners: RwLock<Vec<Arc<dyn ServerStateListener>>>,
}

impl DefaultSharedContext {
  pub fn new(hub_service: Arc<dyn HubService>, setting_service: Arc<dyn SettingService>) -> Self {
    Self::with_args(hub_service, setting_service, Box::new(DefaultServerFactory))
  }

  pub fn with_args(
    hub_service: Arc<dyn HubService>,
    setting_service: Arc<dyn SettingService>,
    factory: Box<dyn ServerFactory>,
  ) -> Self {
    let exec_variant = setting_service.exec_variant();
    Self {
      hub_service,
      setting_service,
      factory,
      exec_variant: ExecVariant::new(exec_variant),
      server: RwLock::new(None),
      state_listeners: RwLock::new(Vec::new()),
    }
  }

  async fn get_server_args(&self) -> Option<LlamaServerArgs> {
    let lock = self.server.read().await;
    lock.as_ref().map(|server| server.get_server_args())
  }

  fn get_setting_args(&self) -> Vec<String> {
    self
      .setting_service
      .get_server_args_common()
      .unwrap_or_default()
      .split_whitespace()
      .map(String::from)
      .collect()
  }

  fn get_setting_variant_args(&self, variant: &str) -> Vec<String> {
    self
      .setting_service
      .get_server_args_variant(variant)
      .unwrap_or_default()
      .split_whitespace()
      .map(String::from)
      .collect()
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
    let exec_path = self.setting_service.exec_path_from();
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

  async fn forward_request(
    &self,
    endpoint: LlmEndpoint,
    mut request: Value,
    alias: Alias,
  ) -> Result<reqwest::Response> {
    // Pattern match to extract local alias information and reject API aliases
    let (alias_name, repo, filename, snapshot, context_params) = match &alias {
      Alias::User(user_alias) => (
        &user_alias.alias,
        &user_alias.repo,
        &user_alias.filename,
        &user_alias.snapshot,
        &user_alias.context_params,
      ),
      Alias::Model(model_alias) => (
        &model_alias.alias,
        &model_alias.repo,
        &model_alias.filename,
        &model_alias.snapshot,
        // Model aliases don't have context_params, use empty vector
        &Vec::<String>::new(),
      ),
      Alias::Api(_) => {
        return Err(ContextError::Unreachable(
          "API aliases cannot be processed by SharedContext".to_string(),
        ));
      }
    };

    let lock = self.server.read().await;
    let server = lock.as_ref();
    let loaded_alias = server.map(|server| server.get_server_args().alias);
    let request_alias = alias_name;
    let model_file = self
      .hub_service
      .find_local_file(repo, filename, Some(snapshot.clone()))?
      .path();

    // Apply request parameters if this is a user alias
    if let Alias::User(user_alias) = &alias {
      // For Value, we need to deserialize, update, and re-serialize
      // This is only needed for chat completions, but we'll apply it universally
      if let Ok(mut chat_request) =
        serde_json::from_value::<async_openai::types::CreateChatCompletionRequest>(request.clone())
      {
        user_alias.request_params.update(&mut chat_request);
        request = serde_json::to_value(chat_request)?;
      }
    }

    let input_value = request;
    let setting_args = self.get_setting_args();
    let result = match ModelLoadStrategy::choose(loaded_alias, request_alias) {
      ModelLoadStrategy::Continue => {
        let server = server
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?;
        let response = match endpoint {
          LlmEndpoint::ChatCompletions => server.chat_completions(&input_value).await?,
          LlmEndpoint::Embeddings => server.embeddings(&input_value).await?,
        };
        Ok(response)
      }
      ModelLoadStrategy::DropAndLoad => {
        drop(lock);
        let variant = self.exec_variant.get().await;
        let setting_variant_args = self.get_setting_variant_args(&variant);
        let merged_args = merge_server_args(&setting_args, &setting_variant_args, context_params);
        let server_args = LlamaServerArgsBuilder::default()
          .alias(alias_name.clone())
          .model(model_file.to_string_lossy().to_string())
          .server_args(merged_args)
          .build()?;
        self.reload(Some(server_args)).await?;
        let lock = self.server.read().await;
        let server = lock
          .as_ref()
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?;
        let response = match endpoint {
          LlmEndpoint::ChatCompletions => server.chat_completions(&input_value).await?,
          LlmEndpoint::Embeddings => server.embeddings(&input_value).await?,
        };
        Ok(response)
      }
      ModelLoadStrategy::Load => {
        let variant = self.exec_variant.get().await;
        let setting_variant_args = self.get_setting_variant_args(&variant);
        let merged_args = merge_server_args(&setting_args, &setting_variant_args, context_params);
        let server_args = LlamaServerArgsBuilder::default()
          .alias(alias_name.clone())
          .model(model_file.to_string_lossy().to_string())
          .server_args(merged_args)
          .build()?;
        drop(lock);
        self.reload(Some(server_args)).await?;
        let lock = self.server.read().await;
        let server = lock
          .as_ref()
          .ok_or_else(|| ContextError::Unreachable("context should not be None".to_string()))?;
        let response = match endpoint {
          LlmEndpoint::ChatCompletions => server.chat_completions(&input_value).await?,
          LlmEndpoint::Embeddings => server.embeddings(&input_value).await?,
        };
        Ok(response)
      }
    };
    self
      .notify_state_listeners(ServerState::ChatCompletions {
        alias: alias_name.clone(),
      })
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
  use std::collections::HashMap;

  use crate::{
    shared_rw::{DefaultSharedContext, ModelLoadStrategy, SharedContext},
    test_utils::{bin_path, mock_server, ServerFactoryStub},
    LlmEndpoint,
  };
  use anyhow_trace::anyhow_trace;
  use async_openai::types::CreateChatCompletionRequest;
  use futures::FutureExt;
  use llama_server_proc::{
    test_utils::mock_response, LlamaServerArgsBuilder, MockServer, BUILD_TARGET, BUILD_VARIANTS,
    DEFAULT_VARIANT, EXEC_NAME,
  };
  use mockall::predicate::eq;
  use objs::{test_utils::temp_hf_home, Alias, HubFileBuilder, UserAlias};
  use rstest::rstest;
  use serde_json::{json, Value};
  use serial_test::serial;
  use services::{
    test_utils::{app_service_stub_builder, AppServiceStubBuilder},
    AppService, BODHI_EXEC_LOOKUP_PATH, BODHI_EXEC_NAME, BODHI_EXEC_TARGET, BODHI_EXEC_VARIANT,
    BODHI_EXEC_VARIANTS, BODHI_LLAMACPP_ARGS,
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
      .build()
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
    );
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

    let bodhi_server_factory = ServerFactoryStub::new(Box::new(mock_server));

    let shared_ctx = DefaultSharedContext::with_args(
      app_service_stub.hub_service(),
      app_service_stub.setting_service(),
      Box::new(bodhi_server_factory),
    );
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
      .build()
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
    );
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
}
