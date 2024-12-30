use crate::{
  test_utils::SNAPSHOT, Alias, AliasBuilder, ChatTemplateType, ChatTemplateId, GptContextParams,
  GptContextParamsBuilder, HubFile, HubFileBuilder, OAIRequestParams, OAIRequestParamsBuilder,
  RemoteModel, Repo, TOKENIZER_CONFIG_JSON,
};
use std::path::PathBuf;

const DEFAULT_CHAT_TEMPLATE: ChatTemplateId = ChatTemplateId::Llama3;

impl Default for ChatTemplateType {
  fn default() -> Self {
    ChatTemplateType::Id(DEFAULT_CHAT_TEMPLATE)
  }
}

impl Repo {
  pub fn llama3() -> Repo {
    Repo::try_from("meta-llama/Meta-Llama-3-8B-Instruct").unwrap()
  }

  pub fn testalias() -> Repo {
    Repo::try_from("MyFactory/testalias-gguf").unwrap()
  }

  pub fn testalias_filename() -> String {
    "testalias.Q8_0.gguf".to_string()
  }

  pub fn testalias_exists() -> Repo {
    Repo::try_from("MyFactory/testalias-gguf").unwrap()
  }

  pub fn fakemodel() -> Repo {
    Repo::try_from("FakeFactory/fakemodel-gguf").unwrap()
  }

  pub fn testalias_exists_filename() -> String {
    "testalias.Q8_0.gguf".to_string()
  }

  pub fn testalias_q4() -> String {
    "testalias.Q4_0.gguf".to_string()
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
      .repo(Repo::testalias_exists())
      .filename(Repo::testalias_exists_filename())
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
      .repo(Repo::llama3())
      .filename(TOKENIZER_CONFIG_JSON.to_string())
      .snapshot("c4a54320a52ed5f88b7a2f84496903ea4ff07b45".to_string())
      .size(Some(50977))
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
      .filename(Repo::testalias_q4())
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

impl RemoteModel {
  pub fn llama3() -> RemoteModel {
    RemoteModel::new(
      "llama3:instruct".to_string(),
      "llama3".to_string(),
      Repo::try_from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF").unwrap(),
      "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      None,
      vec!["chat".to_string()],
      ChatTemplateType::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }

  pub fn testalias() -> RemoteModel {
    RemoteModel::new(
      String::from("testalias:instruct"),
      String::from("testalias"),
      Repo::testalias(),
      Repo::testalias_filename(),
      None,
      vec![String::from("chat")],
      ChatTemplateType::Id(ChatTemplateId::Llama3),
      OAIRequestParams::default(),
      GptContextParams::default(),
    )
  }
}

impl AliasBuilder {
  pub fn testalias() -> AliasBuilder {
    AliasBuilder::default()
      .alias("testalias:instruct".to_string())
      .family("testalias")
      .repo(Repo::testalias())
      .filename(Repo::testalias_filename())
      .snapshot(SNAPSHOT.to_string())
      .features(vec!["chat".to_string()])
      .chat_template(ChatTemplateType::Id(ChatTemplateId::Llama3))
      .request_params(OAIRequestParams::default())
      .context_params(GptContextParams::default())
      .to_owned()
  }

  pub fn testalias_q4() -> AliasBuilder {
    AliasBuilder::testalias()
      .alias("testalias:q4_instruct")
      .filename(Repo::testalias_q4())
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
    let gpt_params = GptContextParamsBuilder::default()
      .n_keep(24)
      .build()
      .unwrap();
    AliasBuilder::default()
      .alias("llama3:instruct".to_string())
      .family("llama3")
      .repo(Repo::try_from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF").unwrap())
      .filename("Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string())
      .snapshot(SNAPSHOT.to_string())
      .features(vec!["chat".to_string()])
      .chat_template(ChatTemplateType::Id(ChatTemplateId::Llama3))
      .request_params(request_params)
      .context_params(gpt_params)
      .to_owned()
  }

  pub fn tinyllama() -> AliasBuilder {
    AliasBuilder::default()
      .alias("tinyllama:instruct".to_string())
      .repo(Repo::try_from("TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF").unwrap())
      .filename("tinyllama-1.1b-chat-v0.3.Q2_K.gguf".to_string())
      .snapshot("b32046744d93031a26c8e925de2c8932c305f7b9".to_string())
      .features(vec!["chat".to_string()])
      .chat_template(ChatTemplateType::Repo(
        Repo::try_from("TinyLlama/TinyLlama-1.1B-Chat-v1.0").unwrap(),
      ))
      .request_params(OAIRequestParams::default())
      .context_params(GptContextParams::default())
      .to_owned()
  }
}

impl Alias {
  pub fn testalias() -> Alias {
    AliasBuilder::testalias().build().unwrap()
  }

  pub fn testalias_q4() -> Alias {
    AliasBuilder::testalias_q4().build().unwrap()
  }

  pub fn testalias_exists() -> Alias {
    AliasBuilder::testalias_exists().build().unwrap()
  }

  pub fn llama3() -> Alias {
    AliasBuilder::llama3().build().unwrap()
  }

  pub fn tinyllama() -> Alias {
    AliasBuilder::tinyllama().build().unwrap()
  }
}
