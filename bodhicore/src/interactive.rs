use crate::error::AppError;
use crate::objs::{Alias, REFS_MAIN, TOKENIZER_CONFIG_JSON};
use crate::service::AppServiceFn;
use crate::tokenizer_config::{self, ChatMessage, TokenizerConfig};
use crate::Repo;
use async_openai::types::CreateChatCompletionStreamResponse;
use derive_new::new;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{BasicHistory, Input};
use indicatif::{ProgressBar, ProgressStyle};
use llama_server_bindings::{disable_llama_log, BodhiServerContext, GptParams};
use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;
use std::{
  ffi::{c_char, c_void},
  io::stdout,
  path::Path,
  slice,
  time::Duration,
};
use tokio::{runtime::Builder, sync::mpsc::Sender};

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

unsafe extern "C" fn callback_stream(
  contents: *const c_char,
  size: usize,
  userdata: *mut c_void,
) -> usize {
  let slice = unsafe { slice::from_raw_parts(contents as *const u8, size) };
  let input_str = match std::str::from_utf8(slice) {
    Ok(s) => s,
    Err(_) => return 0,
  }
  .to_owned();
  let sender = unsafe { &mut *(userdata as *mut Sender<String>) }.clone();
  // TODO: handle closed receiver
  tokio::spawn(async move { sender.send(input_str).await.unwrap() });
  size
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
    let alias = self.alias.clone();
    let chat_repo: Repo = alias.chat_template.try_into()?;
    let tokenizer_file = service
      .find_local_file(&chat_repo, TOKENIZER_CONFIG_JSON, REFS_MAIN)?
      .ok_or_else(move || {
        let filepath = service
          .model_file_path(&chat_repo, TOKENIZER_CONFIG_JSON, REFS_MAIN)
          .display()
          .to_string();
        AppError::AliasModelFilesNotFound {
          alias: alias.alias.clone(),
          filepath,
        }
      })?;
    let tokenizer_config: TokenizerConfig = tokenizer_file.try_into()?;
    let pb = infinite_loading(String::from("Loading..."));
    let gpt_params = GptParams {
      model: model.path().to_string_lossy().into_owned(),
      ..Default::default()
    };
    disable_llama_log();
    let mut ctx = BodhiServerContext::new(gpt_params)?;
    ctx.init()?;
    ctx.start_event_loop()?;
    pb.finish_and_clear();
    let mut history = BasicHistory::new().max_entries(100).no_duplicates(false);
    loop {
      if let Ok(cmd) = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(">>> ")
        .history_with(&mut history)
        .interact_text()
      {
        if cmd == "/bye" {
          break;
        }
        self.process_input(&ctx, &cmd, &tokenizer_config)?;
      }
    }
    let pb = infinite_loading(String::from("Stopping..."));
    ctx.stop()?;
    pb.finish_and_clear();
    Ok(())
  }

  fn process_input(
    &self,
    ctx: &BodhiServerContext,
    input: &str,
    tokenizer_config: &TokenizerConfig,
  ) -> anyhow::Result<()> {
    let messages = vec![ChatMessage::new(String::from("user"), String::from(input))];
    let chat_template = "";
    let prompt = tokenizer_config.apply_chat_template(&messages)?;
    let mut request: Value = json! {{"prompt": prompt}};
    request["model"] = Value::String("".to_string());
    request["stream"] = Value::Bool(true);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
    let json_request = serde_json::to_string(&request)?;
    tokio::spawn(async move {
      while let Some(token) = rx.recv().await {
        let token = if token.starts_with("data:") {
          token.strip_prefix("data: ").unwrap().trim()
        } else {
          token.as_str()
        };
        let token: CreateChatCompletionStreamResponse = serde_json::from_str(token).unwrap();
        if let Some(delta) = token.choices[0].delta.content.as_ref() {
          print!("{}", delta);
          let _ = stdout().flush();
        }
      }
    });
    ctx.completions(
      &json_request,
      chat_template,
      Some(callback_stream),
      &tx as *const _ as *mut _,
    )?;
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
  use crate::{
    objs::{Alias, LocalModelFile, REFS_MAIN, TOKENIZER_CONFIG_JSON},
    service::HubService,
    test_utils::{app_service_stub, mock_app_service, AppServiceTuple, MockAppServiceFn},
    Repo,
  };
  use mockall::predicate::eq;
  use rstest::rstest;
  use std::path::PathBuf;

  #[rstest]
  #[tokio::test]
  async fn test_interactive_local_model_not_found_raises_error(
    #[from(mock_app_service)] mut mock: MockAppServiceFn,
  ) -> anyhow::Result<()> {
    let alias = Alias::test_alias();
    let alias_clone = alias.clone();
    mock
      .hub_service
      .expect_find_local_file()
      .with(
        eq(alias.repo.clone()),
        eq(alias.filename.clone()),
        eq(alias.snapshot.clone()),
      )
      .return_once(|_, _, _| Ok(None));
    mock
      .hub_service
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

  #[rstest]
  #[tokio::test]
  async fn test_interactive_chat_template_not_found(
    #[from(mock_app_service)] mut mock: MockAppServiceFn,
  ) -> anyhow::Result<()> {
    let alias = Alias::test_alias();
    mock
      .hub_service
      .expect_find_local_file()
      .with(
        eq(alias.repo.clone()),
        eq(alias.filename.clone()),
        eq(alias.snapshot.clone()),
      )
      .return_once(|_, _, _| Ok(Some(LocalModelFile::testalias())));
    let llama3 = Repo::try_new("meta-llama/Meta-Llama-3-8B-Instruct".to_string())?;
    mock
      .hub_service
      .expect_find_local_file()
      .with(eq(llama3.clone()), eq(TOKENIZER_CONFIG_JSON), eq(REFS_MAIN))
      .return_once(|_, _, _| Ok(None));
    mock
      .hub_service
      .expect_model_file_path()
      .with(eq(llama3), eq(TOKENIZER_CONFIG_JSON), eq(REFS_MAIN))
      .return_once(|_, _, _| {
        PathBuf::from(
          "/tmp/huggingface/hub/models--meta-llama-repo/snapshots/xyz/tokenizer_config.json",
        )
      });
    let result = Interactive::new(alias).execute(&mock).await;
    assert!(result.is_err());
    assert_eq!(
      r#"model files for model alias 'testalias:instruct' not found in huggingface cache directory. Check if file in the expected filepath exists.
filepath: /tmp/huggingface/hub/models--meta-llama-repo/snapshots/xyz/tokenizer_config.json
"#,
      result.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_interactive_chat_with_llama3(
    app_service_stub: AppServiceTuple,
  ) -> anyhow::Result<()> {
    let AppServiceTuple(_temp_bodhi, _temp_hf, _bodhi_home, _hf_cache, service) = app_service_stub;
    let handle = tokio::spawn(async move {
      let alias = Alias::tinyllama();
      Interactive::new(alias).execute(&service).await
    });
    todo!()
  }
}
