use super::error::OAIRouteError;
use super::{ENDPOINT_OAI_CHAT_COMPLETIONS, ENDPOINT_OAI_EMBEDDINGS};
use crate::shared::AuthScope;
use crate::API_TAG_OPENAI;
use crate::{ApiError, JsonRejectionError};
use async_openai::types::{
  chat::{
    CreateChatCompletionRequest, CreateChatCompletionResponse, CreateChatCompletionStreamResponse,
  },
  embeddings::{CreateEmbeddingRequest, CreateEmbeddingResponse},
};
use axum::{response::Response, Json};
use axum_extra::extract::WithRejection;
use services::inference::LlmEndpoint;

/// Validates basic structure of chat completion request
fn validate_chat_completion_request(request: &serde_json::Value) -> Result<(), OAIRouteError> {
  // Validate model field exists and is a string
  if request.get("model").and_then(|v| v.as_str()).is_none() {
    return Err(OAIRouteError::InvalidRequest(
      "Field 'model' is required and must be a string.".to_string(),
    ));
  }

  // Validate messages field exists and is an array
  if !request
    .get("messages")
    .map(|v| v.is_array())
    .unwrap_or(false)
  {
    return Err(OAIRouteError::InvalidRequest(
      "Field 'messages' is required and must be an array.".to_string(),
    ));
  }

  // Validate stream field is boolean if present
  if let Some(stream) = request.get("stream") {
    if !stream.is_boolean() {
      return Err(OAIRouteError::InvalidRequest(
        "Field 'stream' must be a boolean.".to_string(),
      ));
    }
  }

  Ok(())
}

/// Create a chat completion
#[utoipa::path(
    post,
    path = ENDPOINT_OAI_CHAT_COMPLETIONS,
    tag = API_TAG_OPENAI,
    operation_id = "createChatCompletion",
    summary = "Create Chat Completion (OpenAI Compatible)",
    description = "Creates a chat completion response using the specified model. Supports both streaming and non-streaming responses. Fully compatible with OpenAI's chat completions API format.",
    request_body(
        content = CreateChatCompletionRequest,
        example = json!({
            "model": "llama2:chat",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant."
                },
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ],
            "temperature": 0.7,
            "max_tokens": 100,
            "stream": false
        })
    ),
    responses(
        (status = 200, description = "Chat completion response",
         content_type = "application/json",
         body = CreateChatCompletionResponse,
         example = json!({
             "id": "chatcmpl-123",
             "object": "chat.completion",
             "created": 1677610602,
             "model": "llama2:chat",
             "choices": [
                 {
                     "index": 0,
                     "message": {
                         "role": "assistant",
                         "content": "Hello! How can I help you today?"
                     },
                     "finish_reason": "stop"
                 }
             ],
             "usage": {
                 "prompt_tokens": 20,
                 "completion_tokens": 10,
                 "total_tokens": 30
             }
         })),
         (status = 201, description = "Chat completion stream, the status is 200, using 201 to avoid OpenAPI format limitation.",
         content_type = "text/event-stream",
         headers(
             ("Cache-Control" = String, description = "No-cache directive")
         ),
         body = CreateChatCompletionStreamResponse,
         example = json!({
             "id": "chatcmpl-123",
             "object": "chat.completion.chunk",
             "created": 1694268190,
             "model": "llama2:chat",
             "choices": [{
                 "delta": {"content": "Hello"},
                 "index": 0,
                 "finish_reason": null
             }]
         })
        ),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn chat_completions_handler(
  auth_scope: AuthScope,
  WithRejection(Json(request), _): WithRejection<Json<serde_json::Value>, JsonRejectionError>,
) -> Result<Response, ApiError> {
  validate_chat_completion_request(&request)?;

  let model = request
    .get("model")
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();

  let alias = auth_scope
    .data()
    .find_alias(&model)
    .await
    .ok_or_else(|| ApiError::from(services::DataServiceError::AliasNotFound(model)))?;

  let inference = auth_scope.inference();

  use services::{Alias, ApiFormat};
  let response = match alias {
    Alias::User(_) | Alias::Model(_) => inference
      .forward_local(LlmEndpoint::ChatCompletions, request, alias)
      .await
      .map_err(ApiError::from)?,
    Alias::Api(ref api_alias) if api_alias.api_format != ApiFormat::OpenAIResponses => {
      let api_key = super::resolve_api_key_for_alias(&auth_scope, &api_alias.id).await;
      inference
        .forward_remote(LlmEndpoint::ChatCompletions, request, api_alias, api_key)
        .await
        .map_err(ApiError::from)?
    }
    Alias::Api(ref api_alias) => {
      return Err(ApiError::from(OAIRouteError::InvalidRequest(format!(
        "Model is configured with '{}' format which does not support the chat completions endpoint. Use the responses API endpoint instead.",
        api_alias.api_format
      ))));
    }
  };

  Ok(response)
}

/// Create embeddings
#[utoipa::path(
    post,
    path = ENDPOINT_OAI_EMBEDDINGS,
    tag = API_TAG_OPENAI,
    operation_id = "createEmbedding",
    summary = "Create Embeddings (OpenAI Compatible)",
    description = "Creates embeddings for the input text using the specified model. Fully compatible with OpenAI's embeddings API format.",
    request_body(
        content = CreateEmbeddingRequest,
        example = json!({
            "model": "text-embedding-model",
            "input": "The quick brown fox jumps over the lazy dog"
        })
    ),
    responses(
        (status = 200, description = "Embedding response",
         content_type = "application/json",
         body = CreateEmbeddingResponse,
         example = json!({
             "object": "list",
             "data": [
                 {
                     "object": "embedding",
                     "index": 0,
                     "embedding": [0.1, 0.2, 0.3]
                 }
             ],
             "model": "text-embedding-model",
             "usage": {
                 "prompt_tokens": 8,
                 "total_tokens": 8
             }
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn embeddings_handler(
  auth_scope: AuthScope,
  WithRejection(Json(request), _): WithRejection<Json<CreateEmbeddingRequest>, JsonRejectionError>,
) -> Result<Response, ApiError> {
  let model = request.model.clone();
  let request_value = serde_json::to_value(request).map_err(OAIRouteError::Serialization)?;

  let alias = auth_scope
    .data()
    .find_alias(&model)
    .await
    .ok_or_else(|| ApiError::from(services::DataServiceError::AliasNotFound(model)))?;

  let inference = auth_scope.inference();

  use services::{Alias, ApiFormat};
  let response = match alias {
    Alias::User(_) | Alias::Model(_) => inference
      .forward_local(LlmEndpoint::Embeddings, request_value, alias)
      .await
      .map_err(ApiError::from)?,
    Alias::Api(ref api_alias) if api_alias.api_format != ApiFormat::OpenAIResponses => {
      let api_key = super::resolve_api_key_for_alias(&auth_scope, &api_alias.id).await;
      inference
        .forward_remote(LlmEndpoint::Embeddings, request_value, api_alias, api_key)
        .await
        .map_err(ApiError::from)?
    }
    Alias::Api(ref api_alias) => {
      return Err(ApiError::from(OAIRouteError::InvalidRequest(format!(
        "Model is configured with '{}' format which does not support the embeddings endpoint.",
        api_alias.api_format
      ))));
    }
  };

  Ok(response)
}

#[cfg(test)]
#[path = "test_chat.rs"]
mod test_chat;

#[cfg(test)]
#[path = "test_live_chat.rs"]
mod test_live_chat;
