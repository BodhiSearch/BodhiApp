use super::ollama_api_schemas::*;
use super::{ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS};
use crate::shared::AuthScope;
use crate::API_TAG_OLLAMA;
use async_openai::types::chat::{
  CreateChatCompletionRequest, CreateChatCompletionResponse, CreateChatCompletionStreamResponse,
};
use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use futures_util::StreamExt;
use services::{inference::LlmEndpoint, Alias, ModelAlias, SettingService, UserAlias, GGUF};
use std::{fs, sync::Arc, time::UNIX_EPOCH};

/// List available models in Ollama format
#[utoipa::path(
    get,
    path = ENDPOINT_OLLAMA_TAGS,
    tag = API_TAG_OLLAMA,
    operation_id = "listOllamaModels",
    summary = "List Available Models (Ollama Compatible)",
    description = "Returns a list of all available models in Ollama API compatible format. Includes model metadata such as size, modification time, and format details.",
    responses(
        (status = 200, description = "List of available models", body = ModelsResponse,
         example = json!({
             "models": [
                 {
                     "model": "llama2:chat",
                     "modified_at": "2024-01-20T12:00:00.000000000Z",
                     "size": 0,
                     "digest": "sha256:abc123",
                     "details": {
                         "parent_model": null,
                         "format": "gguf",
                         "family": "unknown",
                         "families": null,
                         "parameter_size": "",
                         "quantization_level": ""
                     }
                 }
             ]
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn ollama_models_handler(
  auth_scope: AuthScope,
) -> Result<Json<ModelsResponse>, Json<OllamaError>> {
  let setting_service = auth_scope.setting_service();
  let aliases = auth_scope.data().list_aliases().await.map_err(|err| {
    Json(OllamaError {
      error: err.to_string(),
    })
  })?;

  let mut models = Vec::new();
  for alias in aliases {
    match alias {
      Alias::User(user) => models.push(user_alias_to_ollama_model(user)),
      Alias::Model(model) => {
        models.push(model_alias_to_ollama_model(&setting_service, model).await)
      }
      Alias::Api(_) => {}
    }
  }

  Ok(Json(ModelsResponse { models }))
}

pub fn user_alias_to_ollama_model(alias: UserAlias) -> Model {
  Model {
    model: alias.alias,
    modified_at: alias.created_at.timestamp() as u32,
    size: 0,
    digest: alias.snapshot,
    details: ModelDetails {
      parent_model: None,
      format: GGUF.to_string(),
      family: "unknown".to_string(),
      families: None,
      parameter_size: "".to_string(),
      quantization_level: "".to_string(),
    },
  }
}

pub async fn model_alias_to_ollama_model(
  setting_service: &Arc<dyn SettingService>,
  alias: ModelAlias,
) -> Model {
  // Construct path from HF cache structure
  let hf_cache = setting_service.hf_cache().await;
  let path = hf_cache
    .join(alias.repo.path())
    .join("snapshots")
    .join(&alias.snapshot)
    .join(&alias.filename);
  let created = fs::metadata(path)
    .map_err(|e| e.to_string())
    .and_then(|m| m.created().map_err(|e| e.to_string()))
    .and_then(|t| t.duration_since(UNIX_EPOCH).map_err(|e| e.to_string()))
    .unwrap_or_default()
    .as_secs() as u32;
  Model {
    model: alias.alias,
    modified_at: created,
    size: 0,
    digest: alias.snapshot,
    details: ModelDetails {
      parent_model: None,
      format: GGUF.to_string(),
      family: "unknown".to_string(),
      families: None,
      parameter_size: "".to_string(),
      quantization_level: "".to_string(),
    },
  }
}

/// Get detailed information about a model in Ollama format
#[utoipa::path(
    post,
    path = ENDPOINT_OLLAMA_SHOW,
    tag = API_TAG_OLLAMA,
    operation_id = "showOllamaModel",
    summary = "Show Model Details (Ollama Compatible)",
    description = "Retrieves detailed information about a specific model in Ollama API compatible format. Includes model parameters, template, license, and configuration details.",
    request_body(
        content = ShowRequest,
        description = "Model name to get details for",
        example = json!({
            "name": "llama2:chat"
        })
    ),
    responses(
        (status = 200, description = "Model details", body = ShowResponse,
         example = json!({
             "details": {
                 "parent_model": null,
                 "format": "gguf",
                 "family": "unknown",
                 "families": null,
                 "parameter_size": "",
                 "quantization_level": ""
             },
             "license": "",
             "model_info": {},
             "modelfile": "",
             "modified_at": "2024-01-20T12:00:00.000000000Z",
             "parameters": "n_keep: 24\nstop:\n- <|start_header_id|>\n- <|end_header_id|>\n- <|eot_id|>\n",
             "template": "llama2"
         })),
        (status = 404, description = "Model not found", body = OllamaError,
         example = json!({
             "error": "model not found"
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn ollama_model_show_handler(
  auth_scope: AuthScope,
  Json(request): Json<ShowRequest>,
) -> Result<Json<ShowResponse>, Json<OllamaError>> {
  let setting_service = auth_scope.setting_service();
  let alias = auth_scope
    .data()
    .find_alias(&request.name)
    .await
    .ok_or_else(|| {
      Json(OllamaError {
        error: "model not found".to_string(),
      })
    })?;
  let model = alias_to_ollama_model_show(&setting_service, alias).await;
  Ok(Json(model))
}

pub async fn alias_to_ollama_model_show(
  setting_service: &Arc<dyn SettingService>,
  alias: Alias,
) -> ShowResponse {
  match alias {
    Alias::User(user_alias) => {
      let request_params = serde_yaml::to_string(&user_alias.request_params).unwrap_or_default();
      let context_params = serde_yaml::to_string(&user_alias.context_params).unwrap_or_default();
      let parameters = format!("{context_params}{request_params}");
      let template = "".to_string(); // Chat template removed since llama.cpp now handles this
      let model = user_alias_to_ollama_model(user_alias);

      ShowResponse {
        details: model.details,
        license: "".to_string(),
        model_info: std::collections::HashMap::new(),
        modelfile: "".to_string(),
        modified_at: model.modified_at,
        parameters,
        template,
      }
    }
    Alias::Model(model_alias) => {
      // Create a minimal ShowResponse for auto-discovered models
      let model = model_alias_to_ollama_model(setting_service, model_alias.clone()).await;
      ShowResponse {
        details: model.details,
        license: "".to_string(),
        model_info: std::collections::HashMap::new(),
        modelfile: "".to_string(),
        modified_at: model.modified_at,
        parameters: "".to_string(),
        template: "".to_string(), // ModelAlias doesn't have chat_template
      }
    }
    Alias::Api(_) => {
      // API aliases don't have Ollama-style details, this shouldn't happen
      // since we filter them out in the find_alias call, but handle it anyway
      ShowResponse {
        details: ModelDetails {
          parent_model: None,
          format: GGUF.to_string(),
          family: "unknown".to_string(),
          families: None,
          parameter_size: "".to_string(),
          quantization_level: "".to_string(),
        },
        license: "".to_string(),
        model_info: std::collections::HashMap::new(),
        modelfile: "".to_string(),
        modified_at: 0,
        parameters: "".to_string(),
        template: "".to_string(),
      }
    }
  }
}

/// Chat with a model using Ollama format
#[utoipa::path(
    post,
    path = ENDPOINT_OLLAMA_CHAT,
    tag = API_TAG_OLLAMA,
    operation_id = "chatOllamaModel",
    summary = "Chat with Model (Ollama Compatible)",
    description = "Creates a chat completion using Ollama API format. Supports both streaming and non-streaming responses with Ollama-specific options and response format.",
    request_body(
        content = ChatRequest,
        description = "Chat request in Ollama format",
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
            "stream": true,
            "options": {
                "temperature": 0.7,
                "num_predict": 100
            }
        })
    ),
    responses(
        (status = 200, description = "Chat response", body = serde_json::Value,
         example = json!({
             "model": "llama2:chat",
             "created_at": "2024-01-20T12:00:00.000000000Z",
             "message": {
                 "role": "assistant",
                 "content": "Hello! How can I help you today?",
                 "images": null
             },
             "done": true,
             "done_reason": "stop",
             "total_duration": 0.0,
             "load_duration": "-1",
             "prompt_eval_count": 20,
             "prompt_eval_duration": "-1",
             "eval_count": 10,
             "eval_duration": "-1"
         })),
        (status = 404, description = "Model not found", body = OllamaError,
         example = json!({
             "error": "model not found"
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn ollama_model_chat_handler(
  auth_scope: AuthScope,
  Json(ollama_request): Json<ChatRequest>,
) -> Result<Response, Json<OllamaError>> {
  let request: CreateChatCompletionRequest = ollama_request.into();
  let stream = request.stream.unwrap_or(true);

  let request_value = serde_json::to_value(&request).map_err(|e| {
    Json(OllamaError {
      error: format!("failed to serialize request: {e}"),
    })
  })?;

  let model = request_value
    .get("model")
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();

  let alias = auth_scope.data().find_alias(&model).await.ok_or_else(|| {
    Json(OllamaError {
      error: format!("model not found: {model}"),
    })
  })?;

  let inference = auth_scope.inference();

  use services::Alias as AliasType;
  let oai_response = match alias {
    AliasType::User(_) | AliasType::Model(_) => inference
      .forward_local(LlmEndpoint::ChatCompletions, request_value, alias)
      .await
      .map_err(|e| {
        Json(OllamaError {
          error: format!("chat completion error: {e}"),
        })
      })?,
    AliasType::Api(ref api_alias) => {
      let tenant_id = auth_scope.tenant_id().unwrap_or("").to_string();
      let user_id = auth_scope
        .auth_context()
        .user_id()
        .unwrap_or("")
        .to_string();
      let api_key = auth_scope
        .db_service()
        .get_api_key_for_alias(&tenant_id, &user_id, &api_alias.id)
        .await
        .ok()
        .flatten();
      inference
        .forward_remote(
          LlmEndpoint::ChatCompletions,
          request_value,
          api_alias,
          api_key,
        )
        .await
        .map_err(|e| {
          Json(OllamaError {
            error: format!("chat completion error: {e}"),
          })
        })?
    }
  };

  // InferenceService returns axum::response::Response directly
  // For non-streaming responses, we need to convert the entire response body
  if !stream {
    use axum::body::to_bytes;
    let (_parts, body) = oai_response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.map_err(|e| {
      Json(OllamaError {
        error: format!("failed to read response bytes: {e}"),
      })
    })?;

    let oai_chat_response: CreateChatCompletionResponse =
      serde_json::from_slice(&bytes).map_err(|e| {
        Json(OllamaError {
          error: format!("failed to parse response: {e}"),
        })
      })?;

    let ollama_response: ChatResponse = oai_chat_response.into();
    return Ok((StatusCode::OK, Json(ollama_response)).into_response());
  }

  // For streaming, transform each SSE chunk into Ollama format
  // The response body is an axum::body::Body stream
  let body_stream = oai_response.into_body().into_data_stream();
  let transformed_stream = body_stream.map(move |chunk| {
    let chunk: axum::body::Bytes = chunk.map_err(|e| format!("error reading chunk: {e}"))?;
    let text = String::from_utf8_lossy(&chunk);

    if text.starts_with("data: ") {
      let msg = text
        .strip_prefix("data: ")
        .unwrap()
        .strip_suffix("\n\n")
        .unwrap();

      if msg.is_empty() {
        return Ok(String::new());
      }

      let oai_chunk: CreateChatCompletionStreamResponse =
        serde_json::from_str(msg).map_err(|e| format!("error parsing chunk: {e}"))?;

      let data: ResponseStream = oai_chunk.into();
      serde_json::to_string(&data)
        .map(|s| format!("data: {s}\n\n"))
        .map_err(|e| format!("error serializing chunk: {e}"))
    } else {
      Ok(text.into_owned())
    }
  });

  use axum::body::Body;
  let body = Body::from_stream(transformed_stream);
  Response::builder()
    .status(StatusCode::OK)
    .body(body)
    .map_err(|e| {
      Json(OllamaError {
        error: format!("failed to build response: {e}"),
      })
    })
}

#[cfg(test)]
#[path = "test_handlers.rs"]
mod test_handlers;
