use crate::BuilderError;
use async_openai::types::{CreateChatCompletionRequest, Stop};
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
  pub fn update(&self, request: &mut CreateChatCompletionRequest) {
    update_if_none(&self.frequency_penalty, &mut request.frequency_penalty);
    update_if_none(&self.max_tokens, &mut request.max_completion_tokens);
    update_if_none(&self.presence_penalty, &mut request.presence_penalty);
    update_if_none(&self.seed, &mut request.seed);
    update_if_none(&self.temperature, &mut request.temperature);
    update_if_none(&self.top_p, &mut request.top_p);
    update_if_none(&self.user, &mut request.user);
    if !self.stop.is_empty() && request.stop.is_none() {
      request.stop = Some(Stop::StringArray(self.stop.clone()));
    }
  }
}

fn update_if_none<T: Clone>(self_param: &Option<T>, request_param: &mut Option<T>) {
  if self_param.is_some() && request_param.is_none() {
    request_param.clone_from(self_param);
  }
}

#[cfg(test)]
mod tests {
  use async_openai::types::CreateChatCompletionRequestArgs;

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
  fn test_oai_request_params_update() {
    let mut request = CreateChatCompletionRequest::default();
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

    params.update(&mut request);

    assert_eq!(Some(0.5), request.frequency_penalty);
    assert_eq!(Some(100), request.max_completion_tokens);
    assert_eq!(Some(0.2), request.presence_penalty);
    assert_eq!(Some(42), request.seed);
    assert_eq!(
      Some(Stop::StringArray(vec!["END".to_string()])),
      request.stop
    );
    assert_eq!(Some(0.7), request.temperature);
    assert_eq!(Some(0.9), request.top_p);
    assert_eq!(Some("test_user".to_string()), request.user);
  }

  #[test]
  fn test_oai_request_params_update_partial() {
    let mut request = CreateChatCompletionRequestArgs::default()
      .temperature(0.5)
      .max_completion_tokens(50_u32)
      .build()
      .unwrap();

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

    params.update(&mut request);

    assert_eq!(Some(0.5), request.frequency_penalty);
    assert_eq!(Some(50), request.max_completion_tokens);
    assert_eq!(None, request.presence_penalty);
    assert_eq!(None, request.seed);
    assert_eq!(None, request.stop);
    assert_eq!(Some(0.5), request.temperature);
    assert_eq!(Some(0.9), request.top_p);
    assert_eq!(None, request.user);
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
