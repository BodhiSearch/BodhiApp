use async_openai::types::CreateChatCompletionStreamResponse;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{BasicHistory, Input};
use indicatif::{ProgressBar, ProgressStyle};
use llama_server_bindings::{disable_llama_log, BodhiServerContext, GptParams};
use serde_json::json;
use std::io::Write;
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

pub(super) fn launch_interactive(model_path: &Path) -> anyhow::Result<()> {
  let runtime = Builder::new_multi_thread().enable_all().build();
  match runtime {
    Ok(runtime) => {
      runtime.block_on(async move { _launch_interactive(model_path).await })?;
      Ok(())
    }
    Err(err) => Err(err.into()),
  }
}

fn process_input(ctx: &BodhiServerContext, input: &str) -> anyhow::Result<()> {
  let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
  let request =
    json! {{"model": "", "stream": true, "messages": [{"role": "user", "content": input}]}};
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
    "",
    Some(callback_stream),
    &tx as *const _ as *mut _,
  )?;
  println!();
  Ok(())
}

async fn _launch_interactive(model_path: &Path) -> anyhow::Result<()> {
  let pb = infinite_loading(String::from("Loading..."));
  let gpt_params = GptParams {
    model: Some(model_path.to_string_lossy().into_owned()),
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
      process_input(&ctx, &cmd)?;
    }
  }
  let pb = infinite_loading(String::from("Stopping..."));
  ctx.stop()?;
  pb.finish_and_clear();
  Ok(())
}
