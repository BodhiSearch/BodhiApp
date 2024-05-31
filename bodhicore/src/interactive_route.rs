use axum::Router;

fn build_routes() -> Router {
  todo!()
}

#[cfg(test)]
mod test {
  use super::build_routes;
  use crate::{
    test_utils::{RequestTestExt, ResponseTestExt},
  };
  use anyhow::anyhow;
  use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
  };
  use axum::http::Request;
  use rstest::rstest;
  use tower::ServiceExt;

  #[rstest]
  #[tokio::test]
  async fn test_interactive_routes_chat() -> anyhow::Result<()> {
    let user_message = ChatCompletionRequestUserMessageArgs::default()
      .content("What day comes after Monday?")
      .build()
      .unwrap();
    let request = CreateChatCompletionRequestArgs::default()
      .messages(vec![ChatCompletionRequestMessage::User(user_message)])
      .build()
      .unwrap();
    let response = build_routes()
      .oneshot(Request::post("/v1/chat/completions").json(request)?)
      .await
      .unwrap();
    let response = response.json::<CreateChatCompletionResponse>().await?;
    assert_eq!(
      "Tuesday",
      response.choices[0]
        .message
        .content
        .as_ref()
        .ok_or(anyhow!("expecting content"))?
    );
    Ok(())
  }
}
