use crate::{AliasResponse, AliasResponseBuilder};
use objs::{GptContextParamsBuilder, OAIRequestParamsBuilder, Repo};
use std::collections::HashMap;

impl AliasResponse {
  pub fn llama3() -> Self {
    AliasResponse::new(
      "llama3:instruct".to_string(),
      "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string(),
      "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
      "llama3".to_string(),
      HashMap::new(),
      OAIRequestParamsBuilder::default()
        .stop(vec![
          "<|start_header_id|>".to_string(),
          "<|end_header_id|>".to_string(),
          "<|eot_id|>".to_string(),
        ])
        .build()
        .unwrap(),
      GptContextParamsBuilder::default()
        .n_keep(24)
        .build()
        .unwrap(),
    )
  }

  pub fn tinyllama() -> Self {
    AliasResponseBuilder::tinyllama_builder().build().unwrap()
  }
}

impl AliasResponseBuilder {
  pub fn tinyllama_builder() -> Self {
    AliasResponseBuilder::default()
      .alias("tinyllama:instruct".to_string())
      .repo(Repo::TINYLLAMA)
      .filename(Repo::TINYLLAMA_FILENAME)
      .snapshot("b32046744d93031a26c8e925de2c8932c305f7b9".to_string())
      .chat_template(Repo::TINYLLAMA_TOKENIZER)
      .model_params(HashMap::new())
      .request_params(OAIRequestParamsBuilder::default().build().unwrap())
      .context_params(GptContextParamsBuilder::default().build().unwrap())
      .to_owned()
  }
}
