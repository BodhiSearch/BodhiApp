use crate::{
  test_utils::SNAPSHOT, HubFile, HubFileBuilder, OAIRequestParams, OAIRequestParamsBuilder, Repo,
  UserAlias, TOKENIZER_CONFIG_JSON,
};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use std::{path::PathBuf, str::FromStr};

// Type alias for backward compatibility
pub type AliasBuilder = crate::UserAliasBuilder;

// Chat template related code removed since llama.cpp now handles this

impl Repo {
  pub const LLAMA3: &str = "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF";
  pub const LLAMA3_Q8: &str = "Meta-Llama-3-8B-Instruct.Q8_0.gguf";
  pub const LLAMA3_TOKENIZER: &str = "meta-llama/Meta-Llama-3-8B-Instruct";
  pub const LLAMA2: &str = "TheBloke/Llama-2-7B-Chat-GGUF";
  pub const QWEN3_1_7B_INSTRUCT: &str = "ggml-org/Qwen3-1.7B-GGUF";
  pub const QWEN3_1_7B_Q8_0: &str = "Qwen3-1.7B-Q8_0.gguf";
  pub const LLAMA2_TOKENIZER: &str = "meta-llama/Llama-2-70b-chat-hf";
  pub const LLAMA2_FILENAME: &str = "llama-2-7b-chat.Q4_K_M.gguf";
  pub const LLAMA2_Q8: &str = "llama-2-7b-chat.Q8_0.gguf";
  pub const TINYLLAMA: &str = "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF";
  pub const TINYLLAMA_TOKENIZER: &str = "TinyLlama/TinyLlama-1.1B-Chat-v1.0";
  pub const TINYLLAMA_FILENAME: &str = "tinyllama-1.1b-chat-v0.3.Q2_K.gguf";
  pub const TESTALIAS_FILENAME: &str = "testalias.Q8_0.gguf";
  pub const TESTALIAS_Q4_FILENAME: &str = "testalias.Q4_0.gguf";
  pub const TESTALIAS: &str = "MyFactory/testalias-gguf";
  pub const TESTALIAS_TOKENIZER: &str = "MyFactory/testalias";
  pub const FAKEMODEL: &str = "FakeFactory/fakemodel-gguf";

  pub const SNAPSHOT_LATEST: &str = "b32046744d93031a26c8e925de2c8932c305f7b9";

  pub fn llama3() -> Repo {
    Repo::from_str(Self::LLAMA3).unwrap()
  }

  pub fn llama3_tokenizer() -> Repo {
    Repo::from_str(Self::LLAMA3_TOKENIZER).unwrap()
  }

  pub fn llama2() -> Repo {
    Repo::from_str(Self::LLAMA2).unwrap()
  }

  pub fn qwen3_1_7b_instruct() -> Repo {
    Repo::from_str(Self::QWEN3_1_7B_INSTRUCT).unwrap()
  }

  pub fn llama2_tokenizer() -> Repo {
    Repo::from_str(Self::LLAMA2_TOKENIZER).unwrap()
  }

  pub fn testalias() -> Repo {
    Repo::from_str(Self::TESTALIAS).unwrap()
  }

  pub fn testalias_tokenizer() -> Repo {
    Repo::from_str(Self::TESTALIAS_TOKENIZER).unwrap()
  }

  pub fn testalias_model_q8() -> String {
    Self::TESTALIAS_FILENAME.to_string()
  }

  pub fn fakemodel() -> Repo {
    Repo::from_str(Self::FAKEMODEL).unwrap()
  }

  pub fn testalias_model_q4() -> String {
    Self::TESTALIAS_Q4_FILENAME.to_string()
  }

  pub fn tinyllama() -> Repo {
    Repo::from_str(Self::TINYLLAMA).unwrap()
  }

  pub fn tinyllama_tokenizer() -> Repo {
    Repo::from_str(Self::TINYLLAMA_TOKENIZER).unwrap()
  }
}

impl HubFileBuilder {
  pub fn testalias() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::testalias())
      .filename("testalias.Q8_0.gguf".to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }

  pub fn testalias_exists() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::testalias())
      .filename(Repo::testalias_model_q8())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(21))
      .to_owned()
  }

  pub fn fakemodel() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::fakemodel())
      .filename("fakemodel.Q4_0.gguf".to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }

  pub fn testalias_tokenizer() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::testalias())
      .filename(TOKENIZER_CONFIG_JSON.to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }

  pub fn llama3_tokenizer() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::llama3_tokenizer())
      .filename(TOKENIZER_CONFIG_JSON.to_string())
      .snapshot("c4a54320a52ed5f88b7a2f84496903ea4ff07b45".to_string())
      .size(Some(50977))
      .to_owned()
  }

  pub fn live_llama2_7b_chat() -> HubFileBuilder {
    let hf_cache = dirs::home_dir()
      .unwrap()
      .join(".cache")
      .join("huggingface")
      .join("hub");
    HubFileBuilder::default()
      .hf_cache(hf_cache)
      .repo(Repo::llama2())
      .filename(Repo::LLAMA2_FILENAME.to_string())
      .snapshot("191239b3e26b2882fb562ffccdd1cf0f65402adb".to_string())
      .size(Some(1000))
      .to_owned()
  }
}

impl HubFile {
  pub fn testalias() -> HubFile {
    HubFileBuilder::testalias()
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .build()
      .unwrap()
  }

  pub fn testalias_q4() -> HubFile {
    HubFileBuilder::testalias()
      .filename(Repo::testalias_model_q4())
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .build()
      .unwrap()
  }

  pub fn testalias_exists() -> HubFile {
    HubFileBuilder::testalias_exists()
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .build()
      .unwrap()
  }

  pub fn testalias_tokenizer() -> HubFile {
    HubFileBuilder::testalias_tokenizer()
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .build()
      .unwrap()
  }

  pub fn llama3_tokenizer() -> HubFile {
    HubFileBuilder::llama3_tokenizer()
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .build()
      .unwrap()
  }
}

impl AliasBuilder {
  pub fn testalias() -> AliasBuilder {
    AliasBuilder::default()
      .alias("testalias:instruct")
      .repo(Repo::testalias())
      .filename(Repo::testalias_model_q8())
      .snapshot(SNAPSHOT)
      .request_params(OAIRequestParams::default())
      .context_params(Vec::default())
      .to_owned()
  }

  pub fn testalias_q4() -> AliasBuilder {
    AliasBuilder::testalias()
      .alias("testalias:q4_instruct")
      .filename(Repo::testalias_model_q4())
      .to_owned()
  }

  pub fn testalias_exists() -> AliasBuilder {
    AliasBuilder::testalias()
      .alias("testalias-exists:instruct".to_string())
      .to_owned()
  }

  pub fn llama3() -> AliasBuilder {
    let request_params = OAIRequestParamsBuilder::default()
      .stop(vec![
        "<|start_header_id|>".to_string(),
        "<|end_header_id|>".to_string(),
        "<|eot_id|>".to_string(),
      ])
      .build()
      .unwrap();
    let gpt_params = vec!["--n-keep 24".to_string()];
    AliasBuilder::default()
      .alias("llama3:instruct".to_string())
      .repo(Repo::llama3())
      .filename(Repo::LLAMA3_Q8.to_string())
      .snapshot(SNAPSHOT.to_string())
      .request_params(request_params)
      .context_params(gpt_params)
      .to_owned()
  }

  pub fn tinyllama() -> AliasBuilder {
    AliasBuilder::default()
      .alias("tinyllama:instruct".to_string())
      .repo(Repo::tinyllama())
      .filename(Repo::TINYLLAMA_FILENAME)
      .snapshot(Repo::SNAPSHOT_LATEST)
      .request_params(OAIRequestParams::default())
      .context_params(Vec::default())
      .to_owned()
  }
}

impl UserAlias {
  pub fn testalias() -> UserAlias {
    AliasBuilder::testalias().build_test().unwrap()
  }

  pub fn testalias_q4() -> UserAlias {
    AliasBuilder::testalias_q4().build_test().unwrap()
  }

  pub fn testalias_exists() -> UserAlias {
    AliasBuilder::testalias_exists().build_test().unwrap()
  }

  pub fn llama3() -> UserAlias {
    AliasBuilder::llama3().build_test().unwrap()
  }

  pub fn tinyllama() -> UserAlias {
    AliasBuilder::tinyllama().build_test().unwrap()
  }

  pub fn tinyllama_model() -> UserAlias {
    AliasBuilder::default()
      .alias("TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF:Q2_K")
      .repo(Repo::tinyllama())
      .filename(Repo::TINYLLAMA_FILENAME)
      .snapshot(Repo::SNAPSHOT_LATEST)
      .request_params(OAIRequestParams::default())
      .context_params(Vec::default())
      .build_test()
      .unwrap()
  }

  pub fn llama2_model() -> UserAlias {
    AliasBuilder::default()
      .alias("TheBloke/Llama-2-7B-Chat-GGUF:Q8_0")
      .repo(Repo::llama2())
      .filename(Repo::LLAMA2_Q8)
      .snapshot("191239b3e26b2882fb562ffccdd1cf0f65402adb")
      .request_params(OAIRequestParams::default())
      .context_params(Vec::default())
      .build_test()
      .unwrap()
  }

  pub fn fakefactory_model() -> UserAlias {
    AliasBuilder::default()
      .alias("FakeFactory/fakemodel-gguf:Q4_0")
      .repo(Repo::fakemodel())
      .filename("fakemodel.Q4_0.gguf")
      .snapshot("9ca625120374ddaae21f067cb006517d14dc91a6")
      .request_params(OAIRequestParams::default())
      .context_params(Vec::default())
      .build_test()
      .unwrap()
  }

  pub fn llama3_with_time(now: DateTime<Utc>) -> UserAlias {
    AliasBuilder::llama3().build_with_id("test-llama3-instruct", now)
  }

  pub fn testalias_exists_with_time(now: DateTime<Utc>) -> UserAlias {
    AliasBuilder::testalias_exists().build_with_id("test-testalias-exists-instruct", now)
  }

  pub fn tinyllama_with_time(now: DateTime<Utc>) -> UserAlias {
    AliasBuilder::tinyllama().build_with_id("test-tinyllama-instruct", now)
  }
}

// ChatTemplateType implementations removed since llama.cpp now handles chat templates

/// Fixed deterministic timestamp matching `FrozenTimeService` default (2025-01-01T00:00:00Z).
pub fn fixed_dt() -> DateTime<Utc> {
  chrono::TimeZone::with_ymd_and_hms(&Utc, 2025, 1, 1, 0, 0, 0).unwrap()
}
