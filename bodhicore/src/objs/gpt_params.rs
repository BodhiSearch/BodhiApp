#[allow(unused_imports)]
use crate::objs::BuilderError;
use clap::Args;
use llama_server_bindings::GptParams;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default, PartialOrd, Args)]
#[cfg_attr(test, derive(derive_builder::Builder))]
#[cfg_attr(test,
  builder(
    default,
    setter(into, strip_option),
    build_fn(error = BuilderError)))]
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

impl GptContextParams {
  pub fn update(&self, gpt_params: &mut GptParams) {
    // gpt_params.n_threads = self.n_threads;
    gpt_params.seed = self.n_seed;
    gpt_params.n_ctx = self.n_ctx;
    gpt_params.n_predict = self.n_predict;
    gpt_params.n_parallel = self.n_parallel;
    gpt_params.n_keep = self.n_keep;
  }
}
