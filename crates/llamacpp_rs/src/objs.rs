use derive_builder::Builder;

#[derive(Debug, PartialEq, Default, Clone, Builder)]
#[builder(default, setter(into, strip_option))]
pub struct CommonParams {
  pub seed: Option<u32>,
  pub n_predict: Option<i32>,
  pub n_ctx: Option<i32>,
  pub model: String,
  pub embedding: Option<bool>,
  pub n_parallel: Option<i32>,
  pub n_keep: Option<i32>,
}
