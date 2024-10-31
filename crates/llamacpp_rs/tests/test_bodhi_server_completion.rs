mod utils;

use anyhow::Context;
use async_openai::types::CreateChatCompletionResponse;
use llamacpp_rs::{BodhiServerContext, ServerContext};
use rstest::rstest;
use utils::{bodhi_server_ctx, chat_completion_request, test_callback};

#[rstest]
#[ignore]
pub fn test_bodhi_server_completion(
  #[from(bodhi_server_ctx)] ctx: BodhiServerContext,
  chat_completion_request: String,
) -> anyhow::Result<()> {
  ctx.init()?;
  ctx.start_event_loop()?;
  let userdata = String::with_capacity(1024);
  ctx.completions(
    &chat_completion_request,
    "",
    Some(test_callback),
    &userdata as *const _ as *mut _,
  )?;
  let response: CreateChatCompletionResponse =
    serde_json::from_str(&userdata).context("parse as chat completion response json")?;
  assert_eq!(
    "Tuesday comes after Monday.",
    response.choices[0]
      .message
      .content
      .as_ref()
      .expect("content does not exists")
  );
  Ok(())
}
