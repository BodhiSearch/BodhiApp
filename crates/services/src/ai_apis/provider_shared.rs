use super::error::{AiApiClientFactoryError, Result};
use crate::models::ApiModel;
use crate::SafeReqwest;
use async_openai::types::models::Model as OpenAIModel;
use axum::body::Body;
use axum::http::Method;
use axum::response::Response;
use serde::Deserialize;
use serde_json::Value;

// OpenRouter (and other OpenAI-compatible providers) omit `object`/`owned_by`/`created`,
// which `async_openai::Model` requires — parse leniently with defaults, then convert.
#[derive(Debug, Deserialize)]
struct LenientOpenAIModel {
  id: String,
  #[serde(default = "default_object")]
  object: String,
  #[serde(default)]
  created: u32,
  #[serde(default)]
  owned_by: String,
}

fn default_object() -> String {
  "model".to_string()
}

impl From<LenientOpenAIModel> for OpenAIModel {
  fn from(m: LenientOpenAIModel) -> Self {
    OpenAIModel {
      id: m.id,
      object: m.object,
      created: m.created,
      owned_by: m.owned_by,
    }
  }
}

fn is_hop_by_hop(name: &str) -> bool {
  matches!(
    name.to_ascii_lowercase().as_str(),
    "connection"
      | "keep-alive"
      | "proxy-authenticate"
      | "proxy-authorization"
      | "te"
      | "trailers"
      | "transfer-encoding"
      | "upgrade"
      | "set-cookie" // upstream infrastructure cookies (e.g. Cloudflare) must not leak to clients
  )
}

pub(crate) fn convert_reqwest_to_axum(reqwest_response: reqwest::Response) -> Result<Response> {
  let status = reqwest_response.status();
  let headers = reqwest_response.headers().clone();

  let mut builder = Response::builder().status(status.as_u16());
  for (key, value) in &headers {
    if is_hop_by_hop(key.as_str()) {
      continue;
    }
    if let Ok(value_str) = value.to_str() {
      builder = builder.header(key.as_str(), value_str);
    }
  }

  let body = Body::from_stream(reqwest_response.bytes_stream());

  builder
    .body(body)
    .map_err(|e| AiApiClientFactoryError::ApiError(format!("Failed to build axum response: {}", e)))
}

pub(crate) async fn fetch_openai_models(
  client: &SafeReqwest,
  api_key: Option<&str>,
  base_url: &str,
) -> Result<Vec<ApiModel>> {
  let url = format!("{}/models", base_url);
  let mut request = client.get(&url)?;

  if let Some(key) = api_key {
    request = request.header("Authorization", format!("Bearer {}", key));
  }

  let response = request.send().await?;
  let status = response.status();
  if !status.is_success() {
    let body = response.text().await.unwrap_or_default();
    return Err(AiApiClientFactoryError::status_to_error(status, body));
  }

  let body: Value = response.json().await?;
  let models: Vec<ApiModel> = body
    .get("data")
    .and_then(|d| d.as_array())
    .map(|arr| {
      arr
        .iter()
        .filter_map(|v| serde_json::from_value::<LenientOpenAIModel>(v.clone()).ok())
        .map(|m| ApiModel::OpenAI(m.into()))
        .collect()
    })
    .unwrap_or_default();

  Ok(models)
}

// Merges `config` extra_body into `incoming` request body.
// `system` arrays: config items prepend incoming. All other keys: incoming wins.
pub(crate) fn merge_extra_body(mut incoming: Value, config: &Value) -> Value {
  let config_obj = match config.as_object() {
    Some(obj) => obj,
    None => return incoming,
  };

  for (key, config_val) in config_obj {
    if key == "system" {
      match incoming.get("system") {
        Some(Value::Array(incoming_arr)) => {
          if let Some(config_arr) = config_val.as_array() {
            let mut merged = config_arr.clone();
            merged.extend(incoming_arr.clone());
            if let Some(obj) = incoming.as_object_mut() {
              obj.insert("system".to_string(), Value::Array(merged));
            }
          }
        }
        None => {
          if let Some(obj) = incoming.as_object_mut() {
            obj.insert("system".to_string(), config_val.clone());
          }
        }
        Some(_) => {}
      }
    } else if incoming.get(key.as_str()).is_none() {
      if let Some(obj) = incoming.as_object_mut() {
        obj.insert(key.clone(), config_val.clone());
      }
    }
  }

  incoming
}

pub(crate) async fn forward_to_upstream(
  client: &SafeReqwest,
  base_url: &str,
  method: &Method,
  api_path: &str,
  prefix: Option<&str>,
  mut request: Option<Value>,
  query_params: Option<&[(String, String)]>,
  apply_auth: impl FnOnce(reqwest::RequestBuilder) -> reqwest::RequestBuilder,
  client_headers: Option<&[(String, String)]>,
) -> Result<Response> {
  let url = format!("{}{}", base_url, api_path);

  if let Some(ref req) = request {
    if let Some(prefix) = prefix {
      if let Some(model_str) = req.get("model").and_then(|v| v.as_str()) {
        if model_str.starts_with(prefix) {
          let stripped_model = model_str
            .strip_prefix(prefix)
            .unwrap_or(model_str)
            .to_string();
          if let Some(obj) = request.as_mut().and_then(|r| r.as_object_mut()) {
            obj.insert(
              "model".to_string(),
              serde_json::Value::String(stripped_model),
            );
          }
        }
      }
    }
  }

  let mut http_request = client.request(method.clone(), &url)?;
  if method == Method::POST {
    http_request = http_request.header("Content-Type", "application/json");
  }

  if let Some(params) = query_params {
    http_request = http_request.query(params);
  }

  http_request = apply_auth(http_request);

  if let Some(hdrs) = client_headers {
    for (k, v) in hdrs {
      http_request = http_request.header(k.as_str(), v.as_str());
    }
  }

  let response = if let Some(body) = request {
    http_request.json(&body).send().await?
  } else {
    http_request.send().await?
  };

  let status = response.status();
  let mut builder = Response::builder().status(status.as_u16());

  for (key, value) in response.headers() {
    if is_hop_by_hop(key.as_str()) {
      continue;
    }
    if let Ok(value_str) = value.to_str() {
      builder = builder.header(key.as_str(), value_str);
    }
  }

  let body_stream = response.bytes_stream();
  let body = Body::from_stream(body_stream);

  let axum_response = builder
    .body(body)
    .map_err(|e| AiApiClientFactoryError::ApiError(e.to_string()))?;

  Ok(axum_response)
}

#[cfg(test)]
mod tests {
  use super::LenientOpenAIModel;
  use crate::models::ApiModel;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  fn parse_data(data: serde_json::Value) -> Vec<ApiModel> {
    data
      .as_array()
      .unwrap()
      .iter()
      .filter_map(|v| serde_json::from_value::<LenientOpenAIModel>(v.clone()).ok())
      .map(|m| ApiModel::OpenAI(m.into()))
      .collect()
  }

  #[test]
  fn parses_openrouter_entries_missing_object_and_owned_by() {
    let data = json!([
      {
        "id": "openai/gpt-4o",
        "canonical_slug": "openai/gpt-4o",
        "created": 1715367049u32,
        "pricing": {"prompt": "0.0000025"}
      },
      {"id": "anthropic/claude-3.5-sonnet"}
    ]);
    let models = parse_data(data);
    let ids: Vec<&str> = models.iter().map(|m| m.id()).collect();
    assert_eq!(vec!["openai/gpt-4o", "anthropic/claude-3.5-sonnet"], ids);
    match &models[0] {
      ApiModel::OpenAI(m) => {
        assert_eq!("model", m.object);
        assert_eq!("", m.owned_by);
        assert_eq!(1715367049, m.created);
      }
      other => panic!("expected OpenAI variant, got {other:?}"),
    }
  }

  #[test]
  fn parses_strict_openai_shape_without_regression() {
    let data = json!([
      {"id": "gpt-4", "object": "model", "created": 1687882411u32, "owned_by": "openai"}
    ]);
    let models = parse_data(data);
    match &models[0] {
      ApiModel::OpenAI(m) => {
        assert_eq!("gpt-4", m.id);
        assert_eq!("model", m.object);
        assert_eq!("openai", m.owned_by);
      }
      other => panic!("expected OpenAI variant, got {other:?}"),
    }
  }

  #[test]
  fn skips_entries_without_id() {
    let data = json!([{"object": "model"}, {"id": "valid"}]);
    let models = parse_data(data);
    let ids: Vec<&str> = models.iter().map(|m| m.id()).collect();
    assert_eq!(vec!["valid"], ids);
  }
}
