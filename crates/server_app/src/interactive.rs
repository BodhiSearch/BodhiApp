use crate::TaskJoinError;
use async_openai::{
  error::OpenAIError,
  types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
    CreateChatCompletionRequestArgs, CreateChatCompletionStreamResponse,
  },
};
use derive_new::new;
use dialoguer::{theme::ColorfulTheme, BasicHistory, Input};
use indicatif::{ProgressBar, ProgressStyle};
use llama_server_bindings::{disable_llama_log, CommonParamsBuilder, CommonParamsBuilderError};
use objs::{
  impl_error_from, Alias, AppError, BuilderError, ErrorType, ObjValidationError, SerdeJsonError,
};
use server_core::{
  obj_exts::update, ContextError, DefaultRouterState, DefaultSharedContextRw, RouterState,
  RouterStateError,
};
use services::{AppService, DataServiceError, HubServiceError};
use std::{
  io::{self, Write},
  sync::Arc,
  time::Duration,
};
use tokio::{
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

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum InteractiveError {
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
  #[error(transparent)]
  Join(#[from] TaskJoinError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400, code = "interactive_error-openai_error", args_delegate = false)]
  OpenAIError(#[from] OpenAIError),
  #[error(transparent)]
  BuilderError(#[from] BuilderError),
  #[error(transparent)]
  HubServiceError(#[from] HubServiceError),
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
  #[error(transparent)]
  ObjValidationError(#[from] ObjValidationError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "interactive_error-gpt_params_builder_error", args_delegate = false)]
  GptParamsBuilderError(#[from] CommonParamsBuilderError),
  #[error(transparent)]
  ContextError(#[from] ContextError),
  #[error(transparent)]
  RouterStateError(#[from] RouterStateError),
}

impl_error_from!(
  ::serde_json::Error,
  InteractiveError::SerdeJson,
  ::objs::SerdeJsonError
);
impl_error_from!(
  ::tokio::task::JoinError,
  InteractiveError::Join,
  crate::TaskJoinError
);

type Result<T> = std::result::Result<T, InteractiveError>;

impl Interactive {
  pub async fn execute(self, service: Arc<dyn AppService>) -> Result<()> {
    let alias = self.alias.clone();
    let model = service.hub_service().find_local_file(
      &alias.repo,
      &alias.filename,
      Some(alias.snapshot.clone()),
    )?;
    let pb = infinite_loading(String::from("Loading..."));
    let mut gpt_params = CommonParamsBuilder::default()
      .model(model.path().display().to_string())
      .build()?;
    update(&alias.context_params, &mut gpt_params);
    disable_llama_log();

    let shared_rw = DefaultSharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    let router_state = DefaultRouterState::new(Arc::new(shared_rw), service);
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
    router_state: &DefaultRouterState,
    input: &str,
    chat_history: Arc<Mutex<Vec<ChatCompletionRequestMessage>>>,
  ) -> Result<()> {
    let mut lock = chat_history.lock().await;
    (*lock).push(ChatCompletionRequestMessage::User(
      ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(input.to_string()),
        ..Default::default()
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
    let handle: JoinHandle<Result<()>> = tokio::spawn(async move {
      let mut deltas = String::new();
      while let Some(message) = rx.recv().await {
        if message.trim() == "data: [DONE]" {
          break;
        }
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
    (handle.await?)?;
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

  pub async fn execute(&self, alias: Alias, service: Arc<dyn AppService>) -> Result<()> {
    Interactive::new(alias).execute(service).await
  }
}

#[cfg(test)]
mod test {
  use crate::Interactive;
  use core::panic;
  use objs::{AliasBuilder, Repo};
  use rstest::rstest;
  use services::{test_utils::AppServiceStubBuilder, HubFileNotFoundError, HubServiceError};
  use std::sync::Arc;

  use super::InteractiveError;

  #[rstest]
  #[tokio::test]
  async fn test_interactive_non_remote_model_alias_local_model_not_found_raises_error(
  ) -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .with_hub_service()
      .build()?;
    let result = Interactive::new(
      AliasBuilder::default()
        .alias("notexists:instruct")
        .repo(Repo::testalias())
        .filename("notexists.Q8_0.gguf")
        .snapshot("main")
        .build()?,
    )
    .execute(Arc::new(service))
    .await;
    assert!(result.is_err());
    match result.unwrap_err() {
      InteractiveError::HubServiceError(HubServiceError::HubFileNotFound(
        HubFileNotFoundError {
          filename,
          repo,
          snapshot,
        },
      )) => {
        assert_eq!(filename, "notexists.Q8_0.gguf");
        assert_eq!(repo, "MyFactory/testalias-gguf");
        assert_eq!(snapshot, "5007652f7a641fe7170e0bad4f63839419bd9213");
      }
      err => panic!("expecting InteractiveError::HubServiceError, found {}", err),
    };
    Ok(())
  }
}
