use crate::{
  db::DbService,
  error::{BodhiError, Common},
  objs::{Alias, ObjError},
  server::{RouterState, RouterStateFn},
  service::{AppServiceFn, HubServiceError},
  SharedContextRw,
};
use async_openai::types::{
  ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
  ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
  CreateChatCompletionRequestArgs, CreateChatCompletionStreamResponse, Role,
};
use derive_new::new;
use dialoguer::{theme::ColorfulTheme, BasicHistory, Input};
use indicatif::{ProgressBar, ProgressStyle};
use llama_server_bindings::{disable_llama_log, GptParamsBuilder};
use std::{
  io::{self, Write},
  sync::Arc,
  time::Duration,
};
use tokio::{
  runtime::Builder,
  sync::{mpsc::channel, Mutex},
  task::JoinHandle,
};

fn infinite_loading(msg: String) -> ProgressBar {
  let spinner_style = ProgressStyle::with_template("{spinner:.green} {wide_msg}")
    .unwrap()
    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

  let pb = ProgressBar::new_spinner();
  pb.enable_steady_tick(Duration::from_millis(100));
  pb.set_style(spinner_style);
  pb.set_message(msg);
  pb
}

#[derive(Debug, new)]
pub struct Interactive {
  alias: Alias,
}

impl Interactive {
  pub async fn execute(self, service: Arc<dyn AppServiceFn>) -> crate::error::Result<()> {
    let alias = self.alias.clone();
    let model = service
      .hub_service()
      .find_local_file(&alias.repo, &alias.filename, &alias.snapshot)?
      .ok_or_else(|| {
        let filepath = &service
          .hub_service()
          .model_file_path(&alias.repo, &alias.filename, &alias.snapshot)
          .display()
          .to_string();
        let (dirname, filename) = match filepath.rsplit_once('/') {
          Some((dir, file)) => (dir.to_string(), file.to_string()),
          None => ("".to_string(), filepath.to_string()),
        };
        let relative_dir = dirname
          .strip_prefix(&service.env_service().hf_home().display().to_string())
          .unwrap_or(&dirname)
          .to_string();
        HubServiceError::FileMissing {
          filename,
          dirname: relative_dir,
        }
      })?;
    let pb = infinite_loading(String::from("Loading..."));
    let mut gpt_params = GptParamsBuilder::default()
      .model(model.path().display().to_string())
      .build()
      .map_err(ObjError::from)?;
    alias.context_params.update(&mut gpt_params);
    disable_llama_log();

    let shared_rw = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    let router_state = RouterState::new(Arc::new(shared_rw), service, Arc::new(DbService::no_op()));
    pb.finish_and_clear();
    let mut shell_history = BasicHistory::new().max_entries(100).no_duplicates(false);
    let chat_history = Arc::new(Mutex::new(Vec::<ChatCompletionRequestMessage>::new()));
    loop {
      if let Ok(user_prompt) = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(">>> ")
        .history_with(&mut shell_history)
        .interact_text()
      {
        if user_prompt.starts_with('/') {
          match user_prompt.as_str() {
            "/?" => {
              println!("/bye: exit the interactive mode");
              println!("/?: show help");
              continue;
            }
            "/bye" => {
              break;
            }
            _ => {
              println!("unknown command `{user_prompt}`. type `/?` for list of commands.");
              continue;
            }
          }
        }
        self
          .process_input(&router_state, &user_prompt, chat_history.clone())
          .await?;
      }
    }
    let pb = infinite_loading(String::from("Stopping..."));
    router_state.try_stop().await?;
    pb.finish_and_clear();
    Ok(())
  }

  async fn process_input(
    &self,
    router_state: &RouterState,
    input: &str,
    chat_history: Arc<Mutex<Vec<ChatCompletionRequestMessage>>>,
  ) -> crate::error::Result<()> {
    let mut lock = chat_history.lock().await;
    (*lock).push(ChatCompletionRequestMessage::User(
      ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(input.to_string()),
        role: Role::User,
        name: None,
      },
    ));
    let msgs_clone = (*lock).clone();
    drop(lock);
    let model = self.alias.alias.clone();
    let request = CreateChatCompletionRequestArgs::default()
      .model(model)
      .stream(true)
      .messages(msgs_clone)
      .build()
      .map_err(BodhiError::BuildError)?;
    let (tx, mut rx) = channel::<String>(100);
    let handle: JoinHandle<crate::error::Result<()>> =
      tokio::spawn(async move {
        let mut deltas = String::new();
        while let Some(message) = rx.recv().await {
          let message = if message.starts_with("data: ") {
            message.strip_prefix("data: ").unwrap()
          } else {
            message.as_ref()
          };
          let result = serde_json::from_str::<CreateChatCompletionStreamResponse>(message)
            .map_err(|err| Common::SerdeJsonSerialize {
              source: err,
              value: message.to_string(),
            })?;
          let delta = result.choices[0]
            .delta
            .content
            .clone()
            .unwrap_or_default()
            .to_string();
          deltas.push_str(&delta);
          print!("{delta}");
          _ = io::stdout().flush();
        }
        let mut msgs = chat_history.lock().await;
        (*msgs).push(ChatCompletionRequestMessage::Assistant(
          ChatCompletionRequestAssistantMessageArgs::default()
            .content(deltas)
            .build()
            .unwrap(),
        ));
        Ok(())
      });
    let result = router_state.chat_completions(request, tx).await;
    (handle.await.map_err(|err| Common::Stdlib(Arc::new(err)))?)?;
    match result {
      Ok(()) => {}
      Err(err) => eprintln!("error: {err}"),
    }
    println!();
    Ok(())
  }
}

#[allow(unused)]
// MockInteractiveRuntime used in cfg(test)
pub struct InteractiveRuntime {}

#[allow(unused)]
// MockInteractiveRuntime used in cfg(test)
impl InteractiveRuntime {
  #[allow(clippy::new_without_default)]
  // MockInteractiveRuntime used in cfg(test)
  pub fn new() -> Self {
    InteractiveRuntime {}
  }

  pub fn execute(&self, alias: Alias, service: Arc<dyn AppServiceFn>) -> crate::error::Result<()> {
    let runtime = Builder::new_multi_thread()
      .enable_all()
      .build()
      .map_err(Common::Io)?;
    runtime.block_on(async move { Interactive::new(alias).execute(service).await })?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::Interactive;
  use crate::{
    objs::Alias,
    service::{MockDataService, MockEnvServiceFn, MockHubService},
    test_utils::AppServiceStubMock,
  };
  use mockall::predicate::eq;
  use rstest::rstest;
  use std::{path::PathBuf, sync::Arc};

  #[rstest]
  #[tokio::test]
  async fn test_interactive_non_remote_model_alias_local_model_not_found_raises_error(
  ) -> anyhow::Result<()> {
    let alias = Alias::testalias();
    let alias_clone = alias.clone();
    let mut mock = MockHubService::default();
    mock
      .expect_find_local_file()
      .with(
        eq(alias.repo.clone()),
        eq(alias.filename.clone()),
        eq(alias.snapshot.clone()),
      )
      .return_once(|_, _, _| Ok(None));
    mock
      .expect_model_file_path()
      .with(eq(alias.repo), eq(alias.filename), eq(alias.snapshot))
      .return_once(|_, _, _| PathBuf::from("/tmp/huggingface/hub/models--MyFactory--testalias-gguf/snapshots/5007652f7a641fe7170e0bad4f63839419bd9213/testalias.Q8_0.gguf"));
    let mut mock_env_service = MockEnvServiceFn::default();
    mock_env_service
      .expect_hf_home()
      .with()
      .return_once(|| PathBuf::from("/tmp/huggingface/hub"));

    let service = AppServiceStubMock::new(mock_env_service, mock, MockDataService::new());
    let result = Interactive::new(alias_clone)
      .execute(Arc::new(service))
      .await;
    assert!(result.is_err());
    assert_eq!(
      r#"file 'testalias.Q8_0.gguf' not found in $HF_HOME/models--MyFactory--testalias-gguf/snapshots/5007652f7a641fe7170e0bad4f63839419bd9213.
Check Huggingface Home is set correctly using environment variable $HF_HOME or using command-line or settings file."#,
      result.unwrap_err().to_string()
    );
    Ok(())
  }
}
