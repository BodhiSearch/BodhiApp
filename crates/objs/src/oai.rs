use crate::BuilderError;
use clap::Args;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(
  Deserialize, Serialize, Debug, Clone, PartialEq, Default, PartialOrd, Args, Builder, ToSchema,
)]
#[
  builder(
    default,
    setter(into, strip_option),
    build_fn(error = BuilderError))]
pub struct OAIRequestParams {
  #[clap(long, value_parser = validate_range_neg_to_pos_2, help=r#"Number between -2.0 and 2.0. 
Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
default: 0.0 (disabled)"#)]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub frequency_penalty: Option<f32>,

  #[arg(
    long,
    help = r#"The maximum number of tokens that can be generated in the completion.
The token count of your prompt plus `max_tokens` cannot exceed the model's context length.
default: -1 (disabled)"#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_tokens: Option<u32>,

  #[arg(long, value_parser = validate_range_neg_to_pos_2, help=r#"Number between -2.0 and 2.0.
Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
default: 0.0 (disabled)"#)]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub presence_penalty: Option<f32>,

  #[arg(long, value_parser = clap::value_parser!(i64).range(i64::MIN..=i64::MAX),
  help=r#"If specified, our system will make a best effort to sample deterministically, such that repeated requests with the same `seed` and parameters should return the same result."#)]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub seed: Option<i64>,

  #[arg(
    long,
    number_of_values = 1,
    help = r#"Up to 4 sequences where the API will stop generating further tokens."#
  )]
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub stop: Vec<String>,

  #[arg(long, value_parser = validate_range_0_to_2, help=r#"Number between 0.0 and 2.0.
Higher values like will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
default: 0.0 (disabled)"#)]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub temperature: Option<f32>,

  #[arg(long, value_parser = validate_range_0_to_1, help=r#"Number between 0.0 and 1.0.
An alternative to sampling with temperature, called nucleus sampling.
The model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered.
Alter this or `temperature` but not both.
default: 1.0 (disabled)"#)]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub top_p: Option<f32>,

  #[arg(
    long,
    help = r#"A unique identifier representing your end-user, which can help to monitor and detect abuse."#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub user: Option<String>,
}

fn validate_range_neg_to_pos_2(s: &str) -> Result<f32, String> {
  let lower = -2.0;
  let upper = 2.0;
  validate_range(s, lower, upper)
}

fn validate_range_0_to_2(s: &str) -> Result<f32, String> {
  validate_range(s, 0.0, 2.0)
}

fn validate_range_0_to_1(s: &str) -> Result<f32, String> {
  validate_range(s, 0.0, 1.0)
}

fn validate_range<T: PartialOrd + FromStr + std::fmt::Debug + std::fmt::Display>(
  s: &str,
  lower: T,
  upper: T,
) -> Result<T, String> {
  match s.parse::<T>() {
    Ok(val) if lower <= val && val <= upper => Ok(val),
    Ok(val) => Err(format!(
      "The value {} is out of range. It must be between {:?} and {:?} inclusive.",
      val, lower, upper
    )),
    Err(_) => Err(format!(
      "'{}' is not a valid number. Please enter a number between {:?} and {:?}.",
      s, lower, upper
    )),
  }
}

impl OAIRequestParams {
  /// Apply request parameters directly to a JSON Value without deserializing.
  /// This preserves any non-standard fields that may be present in the request.
  pub fn apply_to_value(&self, request: &mut serde_json::Value) {
    if let Some(obj) = request.as_object_mut() {
      // Only set if not already present in request
      if let Some(val) = &self.frequency_penalty {
        if !obj.contains_key("frequency_penalty") {
          obj.insert("frequency_penalty".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.max_tokens {
        if !obj.contains_key("max_completion_tokens") && !obj.contains_key("max_tokens") {
          obj.insert("max_completion_tokens".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.presence_penalty {
        if !obj.contains_key("presence_penalty") {
          obj.insert("presence_penalty".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.seed {
        if !obj.contains_key("seed") {
          obj.insert("seed".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.temperature {
        if !obj.contains_key("temperature") {
          obj.insert("temperature".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.top_p {
        if !obj.contains_key("top_p") {
          obj.insert("top_p".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.user {
        if !obj.contains_key("user") {
          obj.insert("user".to_string(), serde_json::json!(val));
        }
      }
      if !self.stop.is_empty() && !obj.contains_key("stop") {
        obj.insert("stop".to_string(), serde_json::json!(self.stop));
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_validate_range_neg_to_pos_2() {
    assert!(validate_range_neg_to_pos_2("-2.0").is_ok());
    assert!(validate_range_neg_to_pos_2("0").is_ok());
    assert!(validate_range_neg_to_pos_2("2.0").is_ok());
    assert!(validate_range_neg_to_pos_2("-2.1").is_err());
    assert!(validate_range_neg_to_pos_2("2.1").is_err());
    assert!(validate_range_neg_to_pos_2("invalid").is_err());
  }

  #[test]
  fn test_validate_range_0_to_2() {
    assert!(validate_range_0_to_2("0").is_ok());
    assert!(validate_range_0_to_2("1.5").is_ok());
    assert!(validate_range_0_to_2("2.0").is_ok());
    assert!(validate_range_0_to_2("-0.1").is_err());
    assert!(validate_range_0_to_2("2.1").is_err());
    assert!(validate_range_0_to_2("invalid").is_err());
  }

  #[test]
  fn test_validate_range_0_to_1() {
    assert!(validate_range_0_to_1("0").is_ok());
    assert!(validate_range_0_to_1("0.5").is_ok());
    assert!(validate_range_0_to_1("1.0").is_ok());
    assert!(validate_range_0_to_1("-0.1").is_err());
    assert!(validate_range_0_to_1("1.1").is_err());
    assert!(validate_range_0_to_1("invalid").is_err());
  }

  #[test]
  fn test_validate_range() {
    assert!(validate_range("5", 0, 10).is_ok());
    assert!(validate_range("0", 0, 10).is_ok());
    assert!(validate_range("10", 0, 10).is_ok());
    assert!(validate_range("-1", 0, 10).is_err());
    assert!(validate_range("11", 0, 10).is_err());
    assert!(validate_range("invalid", 0, 10).is_err());
  }

  #[test]
  fn test_oai_request_params_apply_to_value() {
    let mut request = serde_json::json!({});
    let params = OAIRequestParams {
      frequency_penalty: Some(0.5),
      max_tokens: Some(100),
      presence_penalty: Some(0.2),
      seed: Some(42),
      stop: vec!["END".to_string()],
      temperature: Some(0.7),
      top_p: Some(0.9),
      user: Some("test_user".to_string()),
    };

    params.apply_to_value(&mut request);

    assert_eq!(
      Some(0.5),
      request
        .get("frequency_penalty")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
    );
    assert_eq!(
      Some(100),
      request
        .get("max_completion_tokens")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
    );
    assert_eq!(
      Some(0.2),
      request
        .get("presence_penalty")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
    );
    assert_eq!(Some(42), request.get("seed").and_then(|v| v.as_i64()));
    assert_eq!(Some(&serde_json::json!(["END"])), request.get("stop"));
    assert_eq!(
      Some(0.7),
      request
        .get("temperature")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
    );
    assert_eq!(
      Some(0.9),
      request
        .get("top_p")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
    );
    assert_eq!(
      Some("test_user"),
      request.get("user").and_then(|v| v.as_str())
    );
  }

  #[test]
  fn test_oai_request_params_apply_to_value_partial() {
    let mut request = serde_json::json!({
      "temperature": 0.5,
      "max_completion_tokens": 50
    });

    let params = OAIRequestParams {
      frequency_penalty: Some(0.5),
      max_tokens: Some(100),
      presence_penalty: None,
      seed: None,
      stop: vec![],
      temperature: None,
      top_p: Some(0.9),
      user: None,
    };

    params.apply_to_value(&mut request);

    assert_eq!(
      Some(0.5),
      request
        .get("frequency_penalty")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
    );
    assert_eq!(
      Some(50),
      request
        .get("max_completion_tokens")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
    );
    assert_eq!(None, request.get("presence_penalty"));
    assert_eq!(None, request.get("seed"));
    assert_eq!(None, request.get("stop"));
    assert_eq!(
      Some(0.5),
      request
        .get("temperature")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
    );
    assert_eq!(
      Some(0.9),
      request
        .get("top_p")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
    );
    assert_eq!(None, request.get("user"));
  }

  #[test]
  fn test_validate_range_error_messages() {
    let result = validate_range("2.5", 0.0, 2.0);
    assert_eq!(
      "The value 2.5 is out of range. It must be between 0.0 and 2.0 inclusive.",
      result.unwrap_err()
    );

    let result = validate_range("invalid", 0, 10);
    assert_eq!(
      "'invalid' is not a valid number. Please enter a number between 0 and 10.",
      result.unwrap_err(),
    );
  }
}
