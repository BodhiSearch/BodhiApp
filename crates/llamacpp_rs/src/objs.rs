use derive_builder::Builder;

#[derive(Debug, PartialEq, Default, Clone, Builder)]
#[builder(default, setter(into, strip_option))]
pub struct CommonParams {
  pub model: String,
  pub embedding: Option<bool>,
  pub n_ctx: Option<i32>,
  pub n_keep: Option<i32>,
  pub n_parallel: Option<i32>,
  pub n_predict: Option<i32>,
  pub seed: Option<u32>,
}
impl CommonParams {
  pub(crate) fn as_args(&self) -> Vec<String> {
    let mut args = Vec::<String>::new();
    args.push("bodhi".to_string()); // program name
    args.push("--model".to_string());
    args.push(self.model.clone());
    if let Some(true) = self.embedding {
      args.push("--embedding".to_string());
    }
    if let Some(n_ctx) = self.n_ctx {
      args.push("--ctx-size".to_string());
      args.push(n_ctx.to_string());
    }
    if let Some(n_keep) = self.n_keep {
      args.push("--keep".to_string());
      args.push(n_keep.to_string());
    }
    if let Some(n_parallel) = self.n_parallel {
      args.push("--parallel".to_string());
      args.push(n_parallel.to_string());
    }
    if let Some(n_predict) = self.n_predict {
      args.push("--predict".to_string());
      args.push(n_predict.to_string());
    }
    if let Some(seed) = self.seed {
      args.push("--seed".to_string());
      args.push(seed.to_string());
    }
    args
  }
}
