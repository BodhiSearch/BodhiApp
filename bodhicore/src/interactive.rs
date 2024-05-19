use crate::chat_template::ChatTemplate;
use crate::hf_tokenizer::HubTokenizerConfig;
use async_openai::types::CreateChatCompletionStreamResponse;
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

struct Interactive {
  model_path: PathBuf,
  chat_template: ChatTemplate,
}

impl Interactive {
  fn new(repo: &str, model_path: &Path) -> anyhow::Result<Self> {
    let config = HubTokenizerConfig::for_repo(repo).ok().unwrap_or_default();
    let chat_template = ChatTemplate::new(config)?;
    Ok(Self {
      model_path: model_path.to_path_buf(),
      chat_template,
    })
  }

  async fn execute(&self) -> anyhow::Result<()> {
    let pb = infinite_loading(String::from("Loading..."));
    let gpt_params = GptParams {
      model: self.model_path.to_string_lossy().into_owned(),
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
    let messages = json! {[{"role": "user", "content": input}]};
    let (chat_template, mut request) = self.chat_template.apply(messages)?;
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
      &chat_template,
      Some(callback_stream),
      &tx as *const _ as *mut _,
    )?;
    println!();
    Ok(())
  }
}

pub(super) fn launch_interactive(repo: &str, model_path: &Path) -> anyhow::Result<()> {
  let runtime = Builder::new_multi_thread().enable_all().build();
  match runtime {
    Ok(runtime) => {
      runtime.block_on(async move { Interactive::new(repo, model_path)?.execute().await })?;
      Ok(())
    }
    Err(err) => Err(err.into()),
  }
}
