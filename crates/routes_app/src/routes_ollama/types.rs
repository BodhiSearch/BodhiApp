use async_openai::types::chat::{
  ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
  ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent,
  ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
  ChatCompletionResponseMessage, ChatCompletionStreamResponseDelta, CreateChatCompletionRequest,
  CreateChatCompletionResponse, CreateChatCompletionStreamResponse, FinishReason, ResponseFormat,
  Role, StopConfiguration,
};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ModelsResponse {
  pub models: Vec<Model>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(as = OllamaModel)]
#[serde(rename_all = "snake_case")]
pub struct Model {
  pub model: String,
  #[serde(serialize_with = "serialize_datetime")]
  pub modified_at: u32,
  pub size: i64,
  pub digest: String,
  pub details: ModelDetails,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ModelDetails {
  pub parent_model: Option<String>,
  pub format: String,
  pub family: String,
  pub families: Option<Vec<String>>,
  pub parameter_size: String,
  pub quantization_level: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct OllamaError {
  pub error: String,
}

pub fn serialize_datetime<S>(timestamp: &u32, serializer: S) -> Result<S::Ok, S::Error>
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

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ShowRequest {
  pub name: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ChatRequest {
  pub model: String,
  pub messages: Vec<Message>,
  pub stream: Option<bool>,
  pub format: Option<String>,
  pub keep_alive: Option<Duration>,
  pub options: Option<Options>,
}

fn response_format(input: Option<String>) -> Option<ResponseFormat> {
  input.map(|i| match i.as_str() {
    "json_object" => ResponseFormat::JsonObject,
    _ => ResponseFormat::Text,
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
      max_completion_tokens: options.num_predict,
      n: Some(1),
      presence_penalty: options.presence_penalty,
      response_format: response_format(val.format),
      seed: options.seed,
      stop: options.stop.map(StopConfiguration::StringArray),
      stream: val.stream,
      temperature: options.temperature,
      top_p: options.top_p,
      ..Default::default()
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
  pub model: String,
  #[serde(serialize_with = "serialize_datetime")]
  pub created_at: u32,
  pub message: Message,
  pub done: bool,
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
        content: ChatCompletionRequestSystemMessageContent::Text(val.content),
        ..Default::default()
      }),
      _ => ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(val.content),
        ..Default::default()
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Duration(String);

#[derive(Debug, Serialize, Deserialize, Default, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Options {
  pub num_keep: Option<i32>,
  pub seed: Option<i64>,
  pub num_predict: Option<u32>,
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
