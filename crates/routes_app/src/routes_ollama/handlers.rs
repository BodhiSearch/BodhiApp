use super::types::*;
use super::{ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS};
use async_openai::types::chat::{
  CreateChatCompletionRequest, CreateChatCompletionResponse, CreateChatCompletionStreamResponse,
};
use axum::{
  body::Body,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use futures_util::StreamExt;
use objs::{Alias, ModelAlias, UserAlias, API_TAG_OLLAMA, GGUF};
use server_core::RouterState;
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
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<ModelsResponse>, Json<OllamaError>> {
  let aliases = state
    .app_service()
    .data_service()
    .list_aliases()
    .await
    .map_err(|err| {
      Json(OllamaError {
        error: err.to_string(),
      })
    })?;

  let models = aliases
    .into_iter()
    .filter_map(|alias| {
      // Only include User and Model aliases for Ollama
      match alias {
        Alias::User(user) => Some(user_alias_to_ollama_model(state.clone(), user)),
        Alias::Model(model) => Some(model_alias_to_ollama_model(state.clone(), model)),
        Alias::Api(_) => None, // Skip API aliases for Ollama
      }
    })
    .collect::<Vec<_>>();

  Ok(Json(ModelsResponse { models }))
}

pub fn user_alias_to_ollama_model(_state: Arc<dyn RouterState>, alias: UserAlias) -> Model {
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

pub fn model_alias_to_ollama_model(state: Arc<dyn RouterState>, alias: ModelAlias) -> Model {
  // Construct path from HF cache structure
  let hf_cache = state.app_service().setting_service().hf_cache();
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
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<ShowRequest>,
) -> Result<Json<ShowResponse>, Json<OllamaError>> {
  let alias = state
    .app_service()
    .data_service()
    .find_alias(&request.name)
    .await
    .ok_or_else(|| {
      Json(OllamaError {
        error: "model not found".to_string(),
      })
    })?;
  let model = alias_to_ollama_model_show(state, alias);
  Ok(Json(model))
}

pub fn alias_to_ollama_model_show(state: Arc<dyn RouterState>, alias: Alias) -> ShowResponse {
  match alias {
    Alias::User(user_alias) => {
      let request_params = serde_yaml::to_string(&user_alias.request_params).unwrap_or_default();
      let context_params = serde_yaml::to_string(&user_alias.context_params).unwrap_or_default();
      let parameters = format!("{context_params}{request_params}");
      let template = "".to_string(); // Chat template removed since llama.cpp now handles this
      let model = user_alias_to_ollama_model(state, user_alias);

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
      let model = model_alias_to_ollama_model(state, model_alias.clone());
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
  State(state): State<Arc<dyn RouterState>>,
  Json(ollama_request): Json<ChatRequest>,
) -> Result<Response, Json<OllamaError>> {
  let request: CreateChatCompletionRequest = ollama_request.into();
  let stream = request.stream.unwrap_or(true);

  // Get raw response from forward_request
  let request_value = serde_json::to_value(&request).map_err(|e| {
    Json(OllamaError {
      error: format!("failed to serialize request: {e}"),
    })
  })?;
  let response = state
    .forward_request(server_core::LlmEndpoint::ChatCompletions, request_value)
    .await
    .map_err(|e| {
      Json(OllamaError {
        error: format!("chat completion error: {e}"),
      })
    })?;

  let mut response_builder = Response::builder().status(response.status());
  if let Some(headers) = response_builder.headers_mut() {
    *headers = response.headers().clone();
  }

  // For non-streaming responses, we need to convert the entire response
  if !stream {
    let bytes = response.bytes().await.map_err(|e| {
      Json(OllamaError {
        error: format!("failed to read response bytes: {e}"),
      })
    })?;

    let oai_response: CreateChatCompletionResponse =
      serde_json::from_slice(&bytes).map_err(|e| {
        Json(OllamaError {
          error: format!("failed to parse response: {e}"),
        })
      })?;

    let ollama_response: ChatResponse = oai_response.into();
    return Ok((StatusCode::OK, Json(ollama_response)).into_response());
  }

  // For streaming, transform each SSE chunk into Ollama format
  let stream = response.bytes_stream().map(move |chunk| {
    let chunk = chunk.map_err(|e| format!("error reading chunk: {e}"))?;
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

  let body = Body::from_stream(stream);
  response_builder.body(body).map_err(|e| {
    Json(OllamaError {
      error: format!("failed to build response: {e}"),
    })
  })
}
