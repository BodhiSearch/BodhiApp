use crate::builder::BuilderError;
use async_openai::types::{CreateChatCompletionRequest, Stop};
use clap::Args;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default, PartialOrd, Args, Builder)]
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
  pub max_tokens: Option<u16>,

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
  let lower = 0.0;
  let upper = 2.0;
  validate_range(s, lower, upper)
}

fn validate_range_0_to_1(s: &str) -> Result<f32, String> {
  let lower = 0.0;
  let upper = 1.0;
  validate_range(s, lower, upper)
}

fn validate_range(s: &str, lower: f32, upper: f32) -> Result<f32, String> {
  match s.parse::<f32>() {
    Ok(val) if (lower..=upper).contains(&val) => Ok(val),
    Ok(_) => Err(format!(
      "The value must be between {lower} and {upper} inclusive."
    )),
    Err(_) => Err(String::from(
      "The value must be a valid floating point number.",
    )),
  }
}

impl OAIRequestParams {
  pub fn update(&self, request: &mut CreateChatCompletionRequest) {
    update_if_none(&self.frequency_penalty, &mut request.frequency_penalty);
    update_if_none(&self.max_tokens, &mut request.max_tokens);
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
