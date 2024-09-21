use crate::{DirectEvent, DirectSse, RouterStateFn};
use async_openai::types::{
  ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
  ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
  ChatCompletionRequestUserMessageContent, ChatCompletionResponseFormat,
  ChatCompletionResponseFormatType, ChatCompletionResponseMessage,
  ChatCompletionStreamResponseDelta, CreateChatCompletionRequest, CreateChatCompletionResponse,
  CreateChatCompletionStreamResponse, FinishReason, Role, Stop,
};
use axum::{
  extract::State,
  http::{header, HeaderMap, HeaderValue, StatusCode},
  response::{IntoResponse, Response},
  Json,
};
use chrono::{TimeZone, Utc};
use futures_util::StreamExt;
use objs::{Alias, GGUF};
use serde::{Deserialize, Serialize, Serializer};
use std::{collections::HashMap, fs, sync::Arc, time::UNIX_EPOCH};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Serialize, Deserialize)]
pub struct ModelsResponse {
  models: Vec<Model>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Model {
  model: String,
  #[serde(serialize_with = "serialize_datetime")]
  modified_at: u32,
  size: i64,
  digest: String,
  details: ModelDetails,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModelDetails {
  parent_model: Option<String>,
  format: String,
  family: String,
  families: Option<Vec<String>>,
  parameter_size: String,
  quantization_level: String,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaError {
  error: String,
}

pub async fn ollama_models_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
) -> Result<Json<ModelsResponse>, Json<OllamaError>> {
  let models = state
    .app_service()
    .data_service()
    .list_aliases()
    .map_err(|err| {
      Json(OllamaError {
        error: err.to_string(),
      })
    })?
    .into_iter()
    .map(|alias| to_ollama_model(state.clone(), alias))
    .collect::<Vec<_>>();
  Ok(Json(ModelsResponse { models }))
}

fn to_ollama_model(state: Arc<dyn RouterStateFn>, alias: Alias) -> Model {
  let bodhi_home = &state.app_service().env_service().bodhi_home();
  let path = bodhi_home.join("aliases").join(alias.config_filename());
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
      family: alias.family.unwrap_or_else(|| "unknown".to_string()),
      families: None,
      // TODO: have alias contain parameter size and quantizaiton level
      parameter_size: "".to_string(),
      quantization_level: "".to_string(),
    },
  }
}

fn serialize_datetime<S>(timestamp: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let datetime = Utc
    .timestamp_opt(*timestamp as i64, 0)
    .single()
    .unwrap_or_default();
  let formatted = datetime.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
  serializer.serialize_str(&formatted)
}

#[derive(Serialize, Deserialize)]
pub struct ShowRequest {
  pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ShowResponse {
  pub details: ModelDetails,
  pub license: String,
  pub model_info: HashMap<String, serde_json::Value>,
  pub modelfile: String,
  #[serde(serialize_with = "serialize_datetime")]
  pub modified_at: u32,
  pub parameters: String,
  pub template: String,
}

pub async fn ollama_model_show_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
  Json(request): Json<ShowRequest>,
) -> Result<Json<ShowResponse>, Json<OllamaError>> {
  let alias = state
    .app_service()
    .data_service()
    .find_alias(&request.name)
    .ok_or_else(|| {
      Json(OllamaError {
        error: "model not found".to_string(),
      })
    })?;
  let model = to_ollama_model_show(state, alias);
  Ok(Json(model))
}

fn to_ollama_model_show(state: Arc<dyn RouterStateFn>, alias: Alias) -> ShowResponse {
  let request_params = serde_yaml::to_string(&alias.request_params).unwrap_or_default();
  let context_params = serde_yaml::to_string(&alias.context_params).unwrap_or_default();
  let parameters = format!("{context_params}{request_params}");
  let template = alias.chat_template.to_string();
  let model = to_ollama_model(state, alias);

  ShowResponse {
    details: model.details,
    license: "".to_string(),
    model_info: HashMap::new(),
    modelfile: "".to_string(),
    modified_at: model.modified_at,
    parameters,
    template,
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChatRequest {
  pub model: String,
  pub messages: Vec<Message>,
  pub stream: Option<bool>,
  pub format: Option<String>,
  pub keep_alive: Option<Duration>,
  pub options: Option<Options>,
}

fn response_format(input: Option<String>) -> Option<ChatCompletionResponseFormat> {
  input.map(|i| {
    let format_type = match i.as_str() {
      "json_object" => ChatCompletionResponseFormatType::JsonObject,
      _ => ChatCompletionResponseFormatType::Text,
    };
    ChatCompletionResponseFormat {
      r#type: format_type,
    }
  })
}

#[allow(deprecated)]
impl From<ChatRequest> for CreateChatCompletionRequest {
  fn from(val: ChatRequest) -> Self {
    let options = val.options.unwrap_or_default();
    CreateChatCompletionRequest {
      messages: val
        .messages
        .into_iter()
        .map(|i| i.into())
        .collect::<Vec<_>>(),
      model: val.model,
      frequency_penalty: options.frequency_penalty,
      logit_bias: None,
      logprobs: None,
      top_logprobs: None,
      max_tokens: options.num_predict,
      n: Some(1),
      presence_penalty: options.presence_penalty,
      response_format: response_format(val.format),
      seed: options.seed,
      stop: options.stop.map(Stop::StringArray),
      stream: val.stream,
      temperature: options.temperature,
      top_p: options.top_p,
      tools: None,
      tool_choice: None,
      user: None,
      function_call: None,
      functions: None,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChatResponse {
  pub model: String,
  #[serde(serialize_with = "serialize_datetime")]
  pub created_at: u32,
  pub message: Message,
  pub done_reason: Option<FinishReason>,
  pub done: bool,
  pub total_duration: f64,
  pub load_duration: String,
  pub prompt_eval_count: i32,
  pub prompt_eval_duration: String,
  pub eval_count: i32,
  pub eval_duration: String,
}

impl From<CreateChatCompletionResponse> for ChatResponse {
  fn from(response: CreateChatCompletionResponse) -> Self {
    let first = response.choices.first();
    let message = first
      .map(|choice| choice.message.clone().into())
      .unwrap_or_default();
    let done_reason = first.map(|choice| choice.finish_reason).unwrap_or(None);
    let done = done_reason.is_some();
    let usage = response.usage;

    ChatResponse {
      model: response.model,
      created_at: response.created,
      message,
      done_reason,
      done,
      // TODO: send back load analytics
      total_duration: 0 as f64,
      load_duration: "-1".to_string(),
      prompt_eval_count: usage.as_ref().map(|u| u.prompt_tokens as i32).unwrap_or(-1),
      prompt_eval_duration: "-1".to_string(),
      eval_count: usage.map(|u| u.completion_tokens as i32).unwrap_or(-1),
      eval_duration: "-1".to_string(),
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct ResponseStream {
  model: String,
  #[serde(serialize_with = "serialize_datetime")]
  pub created_at: u32,
  message: Message,
  done: bool,
}

impl From<CreateChatCompletionStreamResponse> for ResponseStream {
  fn from(val: CreateChatCompletionStreamResponse) -> Self {
    let first = val.choices.first();
    let message: Message = first
      .map(|c| c.delta.clone().into())
      .unwrap_or_else(|| Message {
        role: Role::Assistant.to_string(),
        ..Default::default()
      });
    let done = first.map(|c| c.finish_reason.is_some()).unwrap_or(false);
    ResponseStream {
      model: val.model,
      created_at: val.created,
      message,
      done,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Message {
  pub role: String,
  pub content: String,
  pub images: Option<Vec<String>>,
}

impl From<Message> for ChatCompletionRequestMessage {
  fn from(val: Message) -> Self {
    match val.role.as_str() {
      "assistant" => {
        let message = ChatCompletionRequestAssistantMessageArgs::default()
          .content(val.content)
          .build()
          .unwrap();
        ChatCompletionRequestMessage::Assistant(message)
      }
      "system" => ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: val.content,
        role: Role::System,
        name: None,
      }),
      _ => ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(val.content),
        role: Role::User,
        name: None,
      }),
    }
  }
}

impl From<ChatCompletionStreamResponseDelta> for Message {
  fn from(val: ChatCompletionStreamResponseDelta) -> Self {
    Message {
      role: val.role.unwrap_or(Role::Assistant).to_string(),
      content: val.content.unwrap_or_default(),
      images: None,
    }
  }
}

impl From<ChatCompletionResponseMessage> for Message {
  fn from(message: ChatCompletionResponseMessage) -> Self {
    Message {
      role: message.role.to_string(),
      content: message.content.unwrap_or_default(),
      images: None,
    }
  }
}

impl Default for Message {
  fn default() -> Self {
    Self {
      role: Role::Assistant.to_string(),
      content: Default::default(),
      images: Default::default(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Duration(String);

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct Options {
  pub num_keep: Option<i32>,
  pub seed: Option<i64>,
  pub num_predict: Option<u16>,
  pub top_k: Option<i32>,
  pub top_p: Option<f32>,
  pub tfs_z: Option<f32>,
  pub typical_p: Option<f32>,
  pub repeat_last_n: Option<i32>,
  pub temperature: Option<f32>,
  pub repeat_penalty: Option<f32>,
  pub presence_penalty: Option<f32>,
  pub frequency_penalty: Option<f32>,
  pub mirostat: Option<f32>,
  pub mirostat_tau: Option<f32>,
  pub mirostat_eta: Option<f32>,
  pub penalize_newline: Option<bool>,
  pub stop: Option<Vec<String>>,
  pub numa: Option<bool>,
  pub num_ctx: Option<i32>,
  pub num_batch: Option<i32>,
  pub num_gpu: Option<i32>,
  pub main_gpu: Option<i32>,
  pub low_vram: Option<bool>,
  pub f16_kv: Option<bool>,
  pub logits_all: Option<bool>,
  pub vocab_only: Option<bool>,
  pub use_mmap: Option<bool>,
  pub use_mlock: Option<bool>,
  pub num_thread: Option<i32>,
}

pub async fn ollama_model_chat_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
  Json(ollama_request): Json<ChatRequest>,
) -> Result<Response, Json<OllamaError>> {
  let request: CreateChatCompletionRequest = ollama_request.into();
  let stream = request.stream.unwrap_or(true);
  let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
  let handle = tokio::spawn(async move { state.chat_completions(request, tx).await });
  if !stream {
    if let Some(message) = rx.recv().await {
      drop(rx);
      let response: CreateChatCompletionResponse =
        serde_json::from_str(&message).map_err(|err| {
          Json(OllamaError {
            error: format!("error serializing response {err}"),
          })
        })?;
      let ollama_response: ChatResponse = response.into();
      let mut headers = HeaderMap::new();
      headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
      );
      Ok((StatusCode::OK, headers, Json(ollama_response)).into_response())
    } else if let Ok(Err(e)) = handle.await {
      tracing::warn!(?e, "error while processing reqeust");
      Err(Json(OllamaError {
        error: format!("internal server error {e}"),
      }))
    } else {
      Err(Json(OllamaError {
        error: "receiver stream abruptly closed".to_string(),
      }))
    }
  } else {
    let stream = ReceiverStream::new(rx).map::<Result<DirectEvent, String>, _>(move |msg| {
      if msg.starts_with("data: ") {
        let msg = msg
          .strip_prefix("data: ")
          .unwrap()
          .strip_suffix("\n\n")
          .unwrap();
        let oai_chunk = serde_json::from_str::<CreateChatCompletionStreamResponse>(msg);
        match oai_chunk {
          Ok(oai_chunk) => {
            let data: ResponseStream = oai_chunk.into();
            let data = serde_json::to_string(&data)
              .unwrap_or_else(|err| format!("{{\"error\": \"error serializing delta {err}\"}}"));
            Ok(DirectEvent::default().data(data.trim_end_matches('\n')))
          }
          Err(err) => Ok(DirectEvent::default().data(format!(
            "{{\"error\": \"error serializing delta '{msg}', {err}\"}}"
          ))),
        }
      } else if msg.starts_with("error: ") {
        let msg = msg
          .strip_prefix("error: ")
          .unwrap()
          .strip_suffix("\n\n")
          .unwrap();
        Ok(DirectEvent::new().data(msg))
      } else {
        tracing::error!(msg, "unknown event type raised from bodhi_server");
        Ok(DirectEvent::new().data(msg))
      }
    });
    Ok(DirectSse::new(stream).into_response())
  }
}

#[cfg(test)]
mod test {
  use crate::{
    ollama_model_show_handler, ollama_models_handler,
    test_utils::{MockRouterState, RequestTestExt, ResponseTestExt},
  };
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    http::Request,
    routing::{get, post},
    Router,
  };
  use rstest::rstest;
  use serde_json::{json, Value};
  use services::test_utils::AppServiceStubBuilder;
  use std::sync::Arc;
  use tower::ServiceExt;
  use validator::ValidateLength;

  #[rstest]
  #[tokio::test]
  async fn test_ollama_routes_models_list() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()?;
    let service = Arc::new(service);
    let mut router_state = MockRouterState::new();
    router_state
      .expect_app_service()
      .returning(move || service.clone());
    let app = Router::new()
      .route("/api/tags", get(ollama_models_handler))
      .with_state(Arc::new(router_state));
    let response = app
      .oneshot(Request::get("/api/tags").body(Body::empty()).unwrap())
      .await?
      .json::<Value>()
      .await?;
    assert_eq!(3, response["models"].as_array().length().unwrap());
    let llama3 = response["models"]
      .as_array()
      .unwrap()
      .iter()
      .find(|item| item["model"] == "llama3:instruct")
      .unwrap();
    assert_eq!(llama3["digest"], "5007652f7a641fe7170e0bad4f63839419bd9213");
    Ok(())
  }

  #[rstest]
  #[anyhow_trace]
  #[tokio::test]
  async fn test_ollama_model_show() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()?;
    let service = Arc::new(service);
    let mut router_state = MockRouterState::new();
    router_state
      .expect_app_service()
      .returning(move || service.clone());
    let app = Router::new()
      .route("/api/show", post(ollama_model_show_handler))
      .with_state(Arc::new(router_state));
    let response = app
      .oneshot(Request::post("/api/show").json(json! {{"name": "llama3:instruct"}})?)
      .await?
      .json::<Value>()
      .await?;
    assert_eq!(
      json! {
      {
        "families": null,
        "family": "llama3",
        "format": "gguf",
        "parameter_size": "",
        "parent_model": null,
        "quantization_level": ""
      }},
      response["details"]
    );
    assert_eq!("llama3", response["template"]);
    assert_eq!(
      r#"n_keep: 24
stop:
- <|start_header_id|>
- <|end_header_id|>
- <|eot_id|>
"#,
      response["parameters"].as_str().unwrap()
    );
    Ok(())
  }
}
