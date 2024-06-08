#[allow(unused_imports)]
use crate::objs::BuilderError;
use clap::Args;
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
  pub n_ctx: Option<u32>,

  #[arg(
    long,
    help = r#"number of parallel sequences to decode
default: 1"#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub n_parallel: Option<u8>,

  #[arg(
    long,
    help = r#"new tokens to predict
default: -1 (unbounded)"#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub n_predict: Option<u32>,
}
