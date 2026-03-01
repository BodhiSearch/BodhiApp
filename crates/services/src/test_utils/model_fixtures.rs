use std::str::FromStr;

use crate::models::model_objs::{
  ApiAliasBuilder, ApiFormat, HubFile, HubFileBuilder, OAIRequestParams, OAIRequestParamsBuilder,
  Repo, UserAlias, UserAliasBuilder,
};
use crate::models::BuilderError;

impl Repo {
  pub const TESTALIAS: &str = "MyFactory/testalias-gguf";
  pub const FAKEMODEL: &str = "FakeFactory/fakemodel-gguf";
  pub const TESTALIAS_FILENAME: &str = "testalias.Q8_0.gguf";
  pub const TESTALIAS_Q4_FILENAME: &str = "testalias.Q4_0.gguf";
  pub const LLAMA3: &str = "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF";
  pub const LLAMA3_Q8: &str = "Meta-Llama-3-8B-Instruct.Q8_0.gguf";
  pub const TINYLLAMA: &str = "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF";
  pub const TINYLLAMA_FILENAME: &str = "tinyllama-1.1b-chat-v0.3.Q2_K.gguf";
  pub const SNAPSHOT_LATEST: &str = "b32046744d93031a26c8e925de2c8932c305f7b9";

  pub fn testalias() -> Repo {
    Repo::from_str(Self::TESTALIAS).unwrap()
  }

  pub fn fakemodel() -> Repo {
    Repo::from_str(Self::FAKEMODEL).unwrap()
  }

  pub fn llama3() -> Repo {
    Repo::from_str(Self::LLAMA3).unwrap()
  }

  pub fn tinyllama() -> Repo {
    Repo::from_str(Self::TINYLLAMA).unwrap()
  }

  pub fn testalias_model_q8() -> String {
    Self::TESTALIAS_FILENAME.to_string()
  }

  pub fn testalias_model_q4() -> String {
    Self::TESTALIAS_Q4_FILENAME.to_string()
  }
}

impl HubFileBuilder {
  pub fn testalias() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::testalias())
      .filename("testalias.Q8_0.gguf".to_string())
      .snapshot(crate::test_utils::SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }

  pub fn fakemodel() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::fakemodel())
      .filename("fakemodel.Q4_0.gguf".to_string())
      .snapshot(crate::test_utils::SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }
}

impl HubFile {
  pub fn testalias() -> HubFile {
    use std::path::PathBuf;
    HubFileBuilder::testalias()
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .build()
      .unwrap()
  }

  pub fn testalias_exists() -> HubFile {
    use std::path::PathBuf;
    HubFileBuilder::testalias()
      .size(Some(21))
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .build()
      .unwrap()
  }
}

impl UserAlias {
  pub fn testalias_exists() -> UserAlias {
    UserAliasBuilder::testalias_exists().build_test().unwrap()
  }

  pub fn testalias() -> UserAlias {
    UserAliasBuilder::testalias().build_test().unwrap()
  }
}

impl UserAliasBuilder {
  pub fn build_test(&self) -> Result<UserAlias, BuilderError> {
    use chrono::TimeZone;
    let fixed_time = chrono::Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let mut alias = self.build_with_time(fixed_time)?;
    alias.id = format!("test-{}", alias.alias.replace(':', "-"));
    Ok(alias)
  }

  pub fn build_with_id(&self, id: &str, now: chrono::DateTime<chrono::Utc>) -> UserAlias {
    let mut alias = self
      .build_with_time(now)
      .expect("build_with_id requires all builder fields to be set");
    alias.id = id.to_string();
    alias
  }

  pub fn testalias() -> UserAliasBuilder {
    UserAliasBuilder::default()
      .alias("testalias:instruct")
      .repo(Repo::testalias())
      .filename(Repo::testalias_model_q8())
      .snapshot(crate::test_utils::SNAPSHOT)
      .request_params(OAIRequestParams::default())
      .context_params(Vec::<String>::default())
      .to_owned()
  }

  pub fn testalias_exists() -> UserAliasBuilder {
    UserAliasBuilder::testalias()
      .alias("testalias-exists:instruct".to_string())
      .to_owned()
  }

  pub fn llama3() -> UserAliasBuilder {
    let request_params = OAIRequestParamsBuilder::default()
      .stop(vec![
        "<|start_header_id|>".to_string(),
        "<|end_header_id|>".to_string(),
        "<|eot_id|>".to_string(),
      ])
      .build()
      .unwrap();
    let gpt_params = vec!["--n-keep 24".to_string()];
    UserAliasBuilder::default()
      .alias("llama3:instruct".to_string())
      .repo(Repo::llama3())
      .filename(Repo::LLAMA3_Q8.to_string())
      .snapshot(crate::test_utils::SNAPSHOT.to_string())
      .request_params(request_params)
      .context_params(gpt_params)
      .to_owned()
  }

  pub fn tinyllama() -> UserAliasBuilder {
    UserAliasBuilder::default()
      .alias("tinyllama:instruct".to_string())
      .repo(Repo::tinyllama())
      .filename(Repo::TINYLLAMA_FILENAME)
      .snapshot(Repo::SNAPSHOT_LATEST)
      .request_params(OAIRequestParams::default())
      .context_params(Vec::<String>::default())
      .to_owned()
  }
}

impl UserAlias {
  pub fn llama3_with_time(now: chrono::DateTime<chrono::Utc>) -> UserAlias {
    UserAliasBuilder::llama3().build_with_id("test-llama3-instruct", now)
  }

  pub fn testalias_exists_with_time(now: chrono::DateTime<chrono::Utc>) -> UserAlias {
    UserAliasBuilder::testalias_exists().build_with_id("test-testalias-exists-instruct", now)
  }

  pub fn tinyllama_with_time(now: chrono::DateTime<chrono::Utc>) -> UserAlias {
    UserAliasBuilder::tinyllama().build_with_id("test-tinyllama-instruct", now)
  }
}

impl ApiAliasBuilder {
  /// Create a builder pre-configured with test defaults.
  ///
  /// This convenience method is useful in tests to create builders with sensible defaults
  /// that can be customized as needed.
  pub fn test_default() -> Self {
    let mut builder = Self::default();
    builder
      .id("test-id")
      .api_format(ApiFormat::OpenAI)
      .base_url("http://localhost:8080");
    builder
  }
}
