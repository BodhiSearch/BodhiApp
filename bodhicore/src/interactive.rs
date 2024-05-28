use crate::error::AppError;
use crate::hf_tokenizer::{ChatMessage, HubTokenizerConfig};
use crate::objs::Alias;
use crate::service::AppServiceFn;
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
struct Interactive {
  alias: Alias,
}

impl Interactive {
  async fn execute(&self, service: &dyn AppServiceFn) -> crate::error::Result<()> {
    let model = service
      .find_local_model(&self.alias.repo, &self.alias.filename, &self.alias.snapshot)
      .ok_or(AppError::AliasModelNotFound {
        repo: self.alias.repo.to_string(),
        filename: self.alias.filename.clone(),
        snapshot: self.alias.snapshot.clone().unwrap_or(String::from("main")),
      })?;
    let pb = infinite_loading(String::from("Loading..."));
    let gpt_params = GptParams {
      model: model.path(),
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
        self.process_input(&ctx, &cmd)?;
      }
    }
    let pb = infinite_loading(String::from("Stopping..."));
    ctx.stop()?;
    pb.finish_and_clear();
    Ok(())
  }

  fn process_input(&self, ctx: &BodhiServerContext, input: &str) -> anyhow::Result<()> {
    let messages = vec![ChatMessage::new(String::from("user"), String::from(input))];
    let chat_template = "";
    let prompt = self.config.apply_chat_template(&messages)?;
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

pub(super) fn launch_interactive(alias: Alias) -> anyhow::Result<()> {
  let runtime = Builder::new_multi_thread().enable_all().build();
  match runtime {
    Ok(runtime) => {
      runtime.block_on(async move { Interactive::new(alias)?.execute().await })?;
      Ok(())
    }
    Err(err) => Err(err.into()),
  }
}
