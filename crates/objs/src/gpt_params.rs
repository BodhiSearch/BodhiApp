use crate::BuilderError;
use clap::Args;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default, PartialOrd, Args, Builder)]
#[builder(
  default,
  setter(into, strip_option),
  build_fn(error = BuilderError))]
pub struct GptContextParams {
  #[arg(
    long,
    help = r#"seed to initialize the llamacpp context.
default: 0xFFFFFFFF"#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub n_seed: Option<u32>,

  #[arg(
    long,
    help = r#"number of threads to use during computation
default: num_cpus()"#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub n_threads: Option<u32>,

  #[arg(
    long,
    help = r#"size of the prompt context
default: 512"#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub n_ctx: Option<i32>,

  #[arg(
    long,
    help = r#"number of parallel sequences to decode/number of parallel requests served concurrently
default: 1"#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub n_parallel: Option<i32>,

  #[arg(
    long,
    help = r#"new tokens to predict
default: -1 (unbounded)"#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub n_predict: Option<i32>,

  #[arg(
    long,
    help = r#"number of tokens to keep from the initial prompt
default: 0"#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub n_keep: Option<i32>,
}
