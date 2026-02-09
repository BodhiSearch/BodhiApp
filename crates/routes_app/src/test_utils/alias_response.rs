use crate::{UserAliasResponse, UserAliasResponseBuilder};
use chrono::{DateTime, Utc};
use objs::{OAIRequestParamsBuilder, Repo};
use std::collections::HashMap;

impl UserAliasResponse {
  pub fn llama3_with_time(now: DateTime<Utc>) -> Self {
    UserAliasResponseBuilder::default()
      .id("test-llama3-instruct")
      .alias("llama3:instruct")
      .repo(Repo::LLAMA3)
      .filename(Repo::LLAMA3_Q8)
      .snapshot("5007652f7a641fe7170e0bad4f63839419bd9213")
      .source("user")
      .model_params(HashMap::new())
      .request_params(
        OAIRequestParamsBuilder::default()
          .stop(vec![
            "<|start_header_id|>".to_string(),
            "<|end_header_id|>".to_string(),
            "<|eot_id|>".to_string(),
          ])
          .build()
          .unwrap(),
      )
      .context_params(vec!["--n-keep 24".to_string()])
      .created_at(now)
      .updated_at(now)
      .build()
      .unwrap()
  }

  pub fn tinyllama_with_time(now: DateTime<Utc>) -> Self {
    UserAliasResponseBuilder::tinyllama_builder(now)
      .build()
      .unwrap()
  }
}

impl UserAliasResponseBuilder {
  pub fn tinyllama_builder(now: DateTime<Utc>) -> Self {
    UserAliasResponseBuilder::default()
      .id("test-tinyllama-instruct")
      .alias("tinyllama:instruct".to_string())
      .repo(Repo::TINYLLAMA)
      .filename(Repo::TINYLLAMA_FILENAME)
      .source("user")
      .snapshot("b32046744d93031a26c8e925de2c8932c305f7b9".to_string())
      .model_params(HashMap::new())
      .request_params(OAIRequestParamsBuilder::default().build().unwrap())
      .context_params(Vec::<String>::new())
      .created_at(now)
      .updated_at(now)
      .to_owned()
  }
}
