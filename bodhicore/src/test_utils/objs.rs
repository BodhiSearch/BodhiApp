use super::SNAPSHOT;
use crate::{
  create::CreateCommandBuilder,
  objs::{
    Alias, AliasBuilder, ChatTemplate, ChatTemplateId, GptContextParams, LocalModelFile,
    LocalModelFileBuilder, OAIRequestParams, RemoteModel, Repo, TOKENIZER_CONFIG_JSON,
  },
  CreateCommand,
};
use rstest::fixture;
use std::path::PathBuf;

impl Default for ChatTemplate {
  fn default() -> Self {
    ChatTemplate::Id(ChatTemplateId::Llama3)
  }
}

impl LocalModelFile {
  pub fn never_download() -> LocalModelFile {
    LocalModelFile::never_download_builder().build().unwrap()
  }

  pub fn never_download_builder() -> LocalModelFileBuilder {
    LocalModelFileBuilder::default()
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .repo(Repo::try_new("MyFactory/testalias-neverdownload-gguf".to_string()).unwrap())
      .filename("testalias-neverdownload.Q8_0.gguf".to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }

  pub fn never_download_tokenizer_builder() -> LocalModelFileBuilder {
    LocalModelFileBuilder::default()
      .hf_cache(PathBuf::from("/tmp/ignored/huggingface/hub"))
      .repo(Repo::try_new("MyFactory/testalias-neverdownload-gguf".to_string()).unwrap())
      .filename(TOKENIZER_CONFIG_JSON.to_string())
      .snapshot(SNAPSHOT.to_string())
      .size(Some(22))
      .to_owned()
  }

  pub fn testalias() -> LocalModelFile {
    LocalModelFile::new(
      PathBuf::from("/tmp/ignored/huggingface/hub"),
      Repo::try_new("MyFactory/testalias-gguf".to_string()).unwrap(),
      "testalias.Q8_0.gguf".to_string(),
      SNAPSHOT.to_string(),
      Some(22),
    )
  }

  pub fn testalias_tokenizer() -> LocalModelFile {
    LocalModelFile::new(
      PathBuf::from("/tmp/ignored/huggingface/hub"),
      Repo::try_new("MyFactory/testalias-gguf".to_string()).unwrap(),
      "tokenizer_config.json".to_string(),
      SNAPSHOT.to_string(),
      Some(22),
    )
  }

  pub fn llama3_tokenizer() -> LocalModelFile {
    LocalModelFile::new(
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
      Repo::try_new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string()).unwrap(),
      "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      vec!["chat".to_string()],
      ChatTemplate::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }

  pub fn test_alias() -> RemoteModel {
    RemoteModel::new(
      "testalias:instruct".to_string(),
      "testalias".to_string(),
      Repo::try_new("MyFactory/testalias-gguf".to_string()).unwrap(),
      "testalias.Q8_0.gguf".to_string(),
      vec!["chat".to_string()],
      ChatTemplate::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }

  pub fn never_download() -> RemoteModel {
    RemoteModel::new(
      String::from("testalias-neverdownload:instruct"),
      String::from("testalias"),
      Repo::try_new(String::from("MyFactory/testalias-neverdownload-gguf")).unwrap(),
      String::from("testalias-neverdownload.Q8_0.gguf"),
      vec![String::from("chat")],
      ChatTemplate::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }
}

impl CreateCommand {
  pub fn testalias() -> CreateCommand {
    CreateCommand::testalias_builder().build().unwrap()
  }

  pub fn testalias_builder() -> CreateCommandBuilder {
    CreateCommandBuilder::default()
      .alias("testalias:instruct".to_string())
      .repo(Repo::try_new("MyFactory/testalias-gguf".to_string()).unwrap())
      .filename("testalias.Q8_0.gguf".to_string())
      .chat_template(ChatTemplate::Id(ChatTemplateId::Llama3))
      .family(Some("testalias".to_string()))
      .force(false)
      .oai_request_params(OAIRequestParams::default())
      .context_params(GptContextParams::default())
      .to_owned()
  }
}

impl Alias {
  pub fn test_alias() -> Alias {
    Alias::test_alias_instruct_builder().build().unwrap()
  }

  pub fn test_alias_instruct_builder() -> AliasBuilder {
    AliasBuilder::default()
      .alias("testalias:instruct".to_string())
      .family(Some("testalias".to_string()))
      .repo(Repo::try_new("MyFactory/testalias-gguf".to_string()).unwrap())
      .filename("testalias.Q8_0.gguf".to_string())
      .snapshot(SNAPSHOT.to_string())
      .features(vec!["chat".to_string()])
      .chat_template(ChatTemplate::Id(ChatTemplateId::Llama3))
      .request_params(OAIRequestParams::default())
      .context_params(GptContextParams::default())
      .to_owned()
  }

  pub fn never_download() -> Alias {
    Alias::new(
      "testalias-neverdownload:instruct".to_string(),
      Some("testalias".to_string()),
      Repo::try_new("MyFactory/testalias-neverdownload-gguf".to_string()).unwrap(),
      "testalias-neverdownload.Q8_0.gguf".to_string(),
      SNAPSHOT.to_string(),
      vec!["chat".to_string()],
      ChatTemplate::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }

  pub fn test_alias_exists() -> Alias {
    Alias::new(
      String::from("testalias-exists:instruct"),
      Some(String::from("testalias")),
      Repo::try_new(String::from("MyFactory/testalias-exists-instruct-gguf")).unwrap(),
      String::from("testalias-exists-instruct.Q8_0.gguf"),
      SNAPSHOT.to_string(),
      vec![String::from("chat")],
      ChatTemplate::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }

  pub fn llama3() -> Alias {
    Alias::new(
      String::from("llama3:instruct"),
      Some(String::from("llama3")),
      Repo::try_new(String::from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF")).unwrap(),
      String::from("Meta-Llama-3-8B-Instruct.Q8_0.gguf"),
      SNAPSHOT.to_string(),
      vec![String::from("chat")],
      ChatTemplate::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }

  pub fn tinyllama() -> Alias {
    Alias::new(
      "tinyllama:instruct".to_string(),
      None,
      Repo::try_new("TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF".to_string()).unwrap(),
      "tinyllama-1.1b-chat-v0.3.Q2_K.gguf".to_string(),
      "b32046744d93031a26c8e925de2c8932c305f7b9".to_string(),
      vec!["chat".to_string()],
      ChatTemplate::Repo(Repo::try_new("TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string()).unwrap()),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }
}

#[fixture]
pub fn tinyllama() -> Alias {
  Alias::tinyllama()
}

pub fn llama3() -> Repo {
  Repo::try_new("meta-llama/Meta-Llama-3-8B-Instruct".to_string()).unwrap()
}
