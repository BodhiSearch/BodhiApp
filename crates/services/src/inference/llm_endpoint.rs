use axum::http::Method;

/// Clone (not Copy) because response ID variants contain owned String.
#[derive(Debug, Clone, PartialEq)]
pub enum LlmEndpoint {
  ChatCompletions,
  Embeddings,
  Responses,
  ResponsesGet(String),
  ResponsesDelete(String),
  ResponsesInputItems(String),
  ResponsesCancel(String),
  AnthropicMessages,
  AnthropicModels,
  AnthropicModel(String),
  GeminiModels,
  GeminiModel(String),
  GeminiGenerateContent(String),
  GeminiStreamGenerateContent(String),
  GeminiEmbedContent(String),
  GeminiBatchEmbedContents(String),
}

impl LlmEndpoint {
  pub fn api_path(&self) -> String {
    match self {
      Self::ChatCompletions => "/chat/completions".to_string(),
      Self::Embeddings => "/embeddings".to_string(),
      Self::Responses => "/responses".to_string(),
      Self::ResponsesGet(id) | Self::ResponsesDelete(id) => format!("/responses/{}", id),
      Self::ResponsesInputItems(id) => format!("/responses/{}/input_items", id),
      Self::ResponsesCancel(id) => format!("/responses/{}/cancel", id),
      Self::AnthropicMessages => "/messages".to_string(),
      Self::AnthropicModels => "/models".to_string(),
      Self::AnthropicModel(id) => format!("/models/{}", id),
      Self::GeminiModels => "/models".to_string(),
      Self::GeminiModel(id) => format!("/models/{}", id),
      Self::GeminiGenerateContent(id) => format!("/models/{}:generateContent", id),
      Self::GeminiStreamGenerateContent(id) => format!("/models/{}:streamGenerateContent", id),
      Self::GeminiEmbedContent(id) => format!("/models/{}:embedContent", id),
      Self::GeminiBatchEmbedContents(id) => format!("/models/{}:batchEmbedContents", id),
    }
  }

  pub fn http_method(&self) -> &'static Method {
    match self {
      Self::ResponsesGet(_) | Self::ResponsesInputItems(_) => &Method::GET,
      Self::AnthropicModels | Self::AnthropicModel(_) => &Method::GET,
      Self::GeminiModels | Self::GeminiModel(_) => &Method::GET,
      Self::ResponsesDelete(_) => &Method::DELETE,
      _ => &Method::POST,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case(LlmEndpoint::AnthropicMessages, "/messages", &Method::POST)]
  #[case(LlmEndpoint::AnthropicModels, "/models", &Method::GET)]
  #[case(
    LlmEndpoint::AnthropicModel("claude-3-5-sonnet".to_string()),
    "/models/claude-3-5-sonnet",
    &Method::GET
  )]
  #[case(LlmEndpoint::GeminiModels, "/models", &Method::GET)]
  #[case(
    LlmEndpoint::GeminiModel("gemini-2.5-flash".to_string()),
    "/models/gemini-2.5-flash",
    &Method::GET
  )]
  #[case(
    LlmEndpoint::GeminiGenerateContent("gemini-2.5-flash".to_string()),
    "/models/gemini-2.5-flash:generateContent",
    &Method::POST
  )]
  #[case(
    LlmEndpoint::GeminiStreamGenerateContent("gemini-2.5-flash".to_string()),
    "/models/gemini-2.5-flash:streamGenerateContent",
    &Method::POST
  )]
  #[case(
    LlmEndpoint::GeminiEmbedContent("gemini-2.5-flash".to_string()),
    "/models/gemini-2.5-flash:embedContent",
    &Method::POST
  )]
  #[case(
    LlmEndpoint::GeminiBatchEmbedContents("gemini-2.5-flash".to_string()),
    "/models/gemini-2.5-flash:batchEmbedContents",
    &Method::POST
  )]
  fn test_endpoint_paths(
    #[case] endpoint: LlmEndpoint,
    #[case] expected_path: &str,
    #[case] expected_method: &Method,
  ) {
    assert_eq!(expected_path, endpoint.api_path());
    assert_eq!(expected_method, endpoint.http_method());
  }
}
