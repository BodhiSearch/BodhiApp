use super::error::{AiApiServiceError, Result};
use crate::models::ApiModel;
use crate::SafeReqwest;
use async_openai::types::models::Model as OpenAIModel;
use axum::body::Body;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

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
    return Err(AiApiServiceError::status_to_error(status, body));
  }

  let body: Value = response.json().await?;
  let models: Vec<ApiModel> = body
    .get("data")
    .and_then(|d| d.as_array())
    .map(|arr| {
      arr
        .iter()
        .filter_map(|v| serde_json::from_value::<OpenAIModel>(v.clone()).ok())
        .map(ApiModel::OpenAI)
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
    if let Ok(value_str) = value.to_str() {
      builder = builder.header(key.as_str(), value_str);
    }
  }

  let body_stream = response.bytes_stream();
  let body = Body::from_stream(body_stream);

  let axum_response = builder
    .body(body)
    .map_err(|e| AiApiServiceError::ApiError(e.to_string()))?;

  Ok(axum_response)
}
