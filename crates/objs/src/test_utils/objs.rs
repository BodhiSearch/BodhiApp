use super::SNAPSHOT;
use crate::{
  alias::{Alias, AliasBuilder},
  chat_template::{ChatTemplate, ChatTemplateId},
  gpt_params::{GptContextParams, GptContextParamsBuilder},
  hub_file::{HubFile, HubFileBuilder},
  oai::{OAIRequestParams, OAIRequestParamsBuilder},
  remote_file::RemoteModel,
  repo::{Repo, TOKENIZER_CONFIG_JSON},
};
use std::path::PathBuf;

impl Default for ChatTemplate {
  fn default() -> Self {
    ChatTemplate::Id(ChatTemplateId::Llama3)
  }
}

impl Repo {
  pub fn llama3() -> Repo {
    Repo::try_from("meta-llama/Meta-Llama-3-8B-Instruct").unwrap()
  }

  pub fn testalias() -> Repo {
    Repo::try_from("MyFactory/testalias-gguf").unwrap()
  }

  pub fn fakemodel() -> Repo {
    Repo::try_from("FakeFactory/fakemodel-gguf").unwrap()
  }
}

impl HubFile {
  pub fn testalias_builder() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::testalias())
      .filename("testalias.Q8_0.gguf".to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }

  pub fn fakemodel_builder() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::fakemodel())
      .filename("fakemodel.Q4_0.gguf".to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }

  pub fn testalias() -> HubFile {
    HubFile::testalias_builder()
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .build()
      .unwrap()
  }

  pub fn testalias_tokenizer_builder() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::testalias())
      .filename(TOKENIZER_CONFIG_JSON.to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }

  pub fn testalias_tokenizer() -> HubFile {
    HubFile::testalias_tokenizer_builder()
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .build()
      .unwrap()
  }

  pub fn fakemodel_tokenizer_builder() -> HubFileBuilder {
    HubFileBuilder::default()
      .repo(Repo::fakemodel())
      .filename(TOKENIZER_CONFIG_JSON.to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }

  pub fn llama3_tokenizer() -> HubFile {
    HubFile::new(
      PathBuf::from("/tmp/ignored/huggingface/hub"),
      Repo::llama3(),
      TOKENIZER_CONFIG_JSON.to_string(),
      SNAPSHOT.to_string(),
      Some(33),
    )
  }
}

impl RemoteModel {
  pub fn llama3() -> RemoteModel {
    RemoteModel::new(
      "llama3:instruct".to_string(),
      "llama3".to_string(),
      Repo::try_from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF").unwrap(),
      "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      vec!["chat".to_string()],
      ChatTemplate::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }

  pub fn testalias() -> RemoteModel {
    RemoteModel::new(
      String::from("testalias:instruct"),
      String::from("testalias"),
      Repo::try_from("MyFactory/testalias-gguf").unwrap(),
      String::from("testalias.Q8_0.gguf"),
      vec![String::from("chat")],
      ChatTemplate::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }
}

impl Alias {
  pub fn testalias() -> Alias {
    Alias::test_alias_instruct_builder().build().unwrap()
  }

  pub fn test_alias_instruct_builder() -> AliasBuilder {
    AliasBuilder::default()
      .alias("testalias:instruct".to_string())
      .family("testalias")
      .repo(Repo::testalias())
      .filename("testalias.Q8_0.gguf".to_string())
      .snapshot(SNAPSHOT.to_string())
      .features(vec!["chat".to_string()])
      .chat_template(ChatTemplate::Id(ChatTemplateId::Llama3))
      .request_params(OAIRequestParams::default())
      .context_params(GptContextParams::default())
      .to_owned()
  }

  pub fn test_alias_exists() -> Alias {
    Alias::new(
      String::from("testalias-exists:instruct"),
      Some(String::from("testalias")),
      Repo::try_from("MyFactory/testalias-exists-instruct-gguf").unwrap(),
      String::from("testalias-exists-instruct.Q8_0.gguf"),
      SNAPSHOT.to_string(),
      vec![String::from("chat")],
      ChatTemplate::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }

  pub fn llama3() -> Alias {
    let request_params = OAIRequestParamsBuilder::default()
      .stop(vec![
        "<|start_header_id|>".to_string(),
        "<|end_header_id|>".to_string(),
        "<|eot_id|>".to_string(),
      ])
      .build()
      .unwrap();
    let gpt_params = GptContextParamsBuilder::default()
      .n_keep(24)
      .build()
      .unwrap();
    Alias::new(
      String::from("llama3:instruct"),
      Some(String::from("llama3")),
      Repo::try_from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF").unwrap(),
      String::from("Meta-Llama-3-8B-Instruct.Q8_0.gguf"),
      SNAPSHOT.to_string(),
      vec![String::from("chat")],
      ChatTemplate::Id(ChatTemplateId::Llama3),
      request_params,
      gpt_params,
    )
  }

  pub fn tinyllama() -> Alias {
    Alias::new(
      "tinyllama:instruct".to_string(),
      None,
      Repo::try_from("TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF").unwrap(),
      "tinyllama-1.1b-chat-v0.3.Q2_K.gguf".to_string(),
      "b32046744d93031a26c8e925de2c8932c305f7b9".to_string(),
      vec!["chat".to_string()],
      ChatTemplate::Repo(Repo::try_from("TinyLlama/TinyLlama-1.1B-Chat-v1.0").unwrap()),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }
}
