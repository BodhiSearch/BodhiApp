use crate::error::AppError;
use crate::objs::Alias;
use crate::server::{RouterState, RouterStateFn};
use crate::service::AppServiceFn;
use crate::{AppService, SharedContextRw};
use async_openai::types::{
  ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
  ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
  CreateChatCompletionRequestArgs, CreateChatCompletionStreamResponse, Role,
};
use derive_new::new;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{BasicHistory, Input};
use indicatif::{ProgressBar, ProgressStyle};
use llama_server_bindings::{disable_llama_log, GptParams};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

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
pub(crate) struct Interactive {
  alias: Alias,
}

impl Interactive {
  pub(crate) async fn execute(self, service: &dyn AppServiceFn) -> crate::error::Result<()> {
    let alias = self.alias.clone();
    let model = service
      .find_local_file(&alias.repo, &alias.filename, &alias.snapshot)?
      .ok_or_else(move || {
        let filepath = service
          .model_file_path(&alias.repo, &alias.filename, &alias.snapshot)
          .display()
          .to_string();
        AppError::AliasModelFilesNotFound {
          alias: alias.alias.to_string(),
          filepath,
        }
      })?;
    let pb = infinite_loading(String::from("Loading..."));
    let gpt_params = GptParams {
      model: model.path().display().to_string(),
      ..Default::default()
    };
    disable_llama_log();
    let app_service = AppService::default();
    let shared_rw = SharedContextRw::new_shared_rw(Some(gpt_params))
      .await
      // TODO - fix error hierarchy with context error throwing app error
      .map_err(|err| AppError::Anyhow(anyhow::anyhow!(err.to_string())))?;
    let router_state = RouterState::new(Arc::new(shared_rw), Arc::new(app_service));
    pb.finish_and_clear();
    let mut shell_history = BasicHistory::new().max_entries(100).no_duplicates(false);
    let chat_history = Arc::new(Mutex::new(Vec::<ChatCompletionRequestMessage>::new()));
    loop {
      if let Ok(user_prompt) = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(">>> ")
        .history_with(&mut shell_history)
        .interact_text()
      {
        if user_prompt == "/bye" {
          break;
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
  ) -> anyhow::Result<()> {
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
      .build()?;
    let (tx, mut rx) = channel::<String>(100);
    let handle: JoinHandle<crate::error::Result<()>> = tokio::spawn(async move {
      let mut deltas = String::new();
      while let Some(message) = rx.recv().await {
        let message = if message.starts_with("data: ") {
          message.strip_prefix("data: ").unwrap()
        } else {
          message.as_ref()
        };
        let result = serde_json::from_str::<CreateChatCompletionStreamResponse>(message)?;
        let delta = result.choices[0]
          .delta
          .content
          .clone()
          .unwrap_or_default()
          .to_string();
        deltas.push_str(&delta);
        print!("{delta}");
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
    router_state.chat_completions(request, tx).await?;
    (handle.await?)?;
    println!();
    Ok(())
  }
}

pub(super) fn launch_interactive(alias: Alias, service: &dyn AppServiceFn) -> anyhow::Result<()> {
  let runtime = Builder::new_multi_thread().enable_all().build();
  match runtime {
    Ok(runtime) => {
      runtime.block_on(async move { Interactive::new(alias).execute(service).await })?;
      Ok(())
    }
    Err(err) => Err(err.into()),
  }
}

#[cfg(test)]
mod test {
  use super::Interactive;
  use crate::{objs::Alias, test_utils::MockAppService};
  use mockall::predicate::eq;
  use rstest::rstest;
  use std::path::PathBuf;

  #[rstest]
  #[tokio::test]
  async fn test_interactive_local_model_not_found_raises_error() -> anyhow::Result<()> {
    let alias = Alias::testalias();
    let alias_clone = alias.clone();
    let mut mock = MockAppService::default();
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
    let result = Interactive::new(alias_clone).execute(&mock).await;
    assert!(result.is_err());
    assert_eq!(
      r#"model files for model alias 'testalias:instruct' not found in huggingface cache directory. Check if file in the expected filepath exists.
filepath: /tmp/huggingface/hub/models--MyFactory--testalias-gguf/snapshots/5007652f7a641fe7170e0bad4f63839419bd9213/testalias.Q8_0.gguf
"#,
      result.unwrap_err().to_string()
    );
    Ok(())
  }
}
