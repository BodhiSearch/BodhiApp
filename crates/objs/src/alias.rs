use super::{
  chat_template::ChatTemplate, gpt_params::GptContextParams, oai::OAIRequestParams, repo::Repo,
  utils::is_default, utils::to_safe_filename,
};
use derive_new::new;
use prettytable::{Cell, Row};
use serde::{Deserialize, Serialize};

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, new)]
#[cfg_attr(
  any(test, feature = "test-utils"),
  derive(Default, derive_builder::Builder)
)]
#[cfg_attr(
  any(test, feature = "test-utils"),
  builder(
    default,
    setter(into, strip_option),
    build_fn(error = super::builder::BuilderError)))]
pub struct Alias {
  pub alias: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub family: Option<String>,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  pub features: Vec<String>,
  pub chat_template: ChatTemplate,
  #[serde(default, skip_serializing_if = "is_default")]
  pub request_params: OAIRequestParams,
  #[serde(default, skip_serializing_if = "is_default")]
  pub context_params: GptContextParams,
}

impl Alias {
  pub fn config_filename(&self) -> String {
    let filename = self.alias.replace(':', "--");
    let filename = to_safe_filename(&filename);
    format!("{}.yaml", filename)
  }
}

impl From<Alias> for Row {
  fn from(value: Alias) -> Self {
    Row::from(vec![
      Cell::new(&value.alias),
      Cell::new(&value.family.unwrap_or_default()),
      Cell::new(&value.repo.to_string()),
      Cell::new(&value.filename),
      Cell::new(&value.features.join(",")),
      Cell::new(&value.chat_template.to_string()),
    ])
  }
}

// TODO: hard coding for time being
pub fn default_features() -> Vec<String> {
  vec!["chat".to_string()]
}

#[cfg(test)]
mod test {
  use super::Alias;
  use crate::{
    alias::AliasBuilder,
    chat_template::{ChatTemplate, ChatTemplateId},
    gpt_params::GptContextParamsBuilder,
    oai::OAIRequestParamsBuilder,
    repo::Repo,
  };
  use prettytable::{Cell, Row};
  use rstest::rstest;

  fn tinyllama_builder() -> AliasBuilder {
    AliasBuilder::default()
      .alias("tinyllama:instruct")
      .repo(Repo::try_from("TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF").unwrap())
      .filename("tinyllama-1.1b-chat-v1.0.Q4_0.gguf")
      .snapshot("52e7645ba7c309695bec7ac98f4f005b139cf465")
      .features(vec!["chat".to_string()])
      .request_params(
        OAIRequestParamsBuilder::default()
          .temperature(0.7)
          .top_p(0.95)
          .build()
          .unwrap(),
      )
      .context_params(
        GptContextParamsBuilder::default()
          .n_ctx(2048)
          .n_parallel(4u8)
          .n_predict(256)
          .build()
          .unwrap(),
      )
      .to_owned()
  }

  fn tinyllama_chat_template_repo() -> Alias {
    tinyllama_builder()
      .chat_template(ChatTemplate::Repo(
        Repo::try_from("TinyLlama/TinyLlama-1.1B-Chat-v1.0").unwrap(),
      ))
      .build()
      .unwrap()
  }

  fn tinyllama_chat_template_id() -> Alias {
    tinyllama_builder()
      .chat_template(ChatTemplate::Id(ChatTemplateId::Tinyllama))
      .build()
      .unwrap()
  }

  fn tiny_llama_serialized() -> String {
    let result = r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF
filename: tinyllama-1.1b-chat-v1.0.Q4_0.gguf
snapshot: 52e7645ba7c309695bec7ac98f4f005b139cf465
features:
- chat
chat_template: {chat_template}
request_params:
  temperature: 0.7
  top_p: 0.95
context_params:
  n_ctx: 2048
  n_parallel: 4
  n_predict: 256
"#;
    result.to_string()
  }

  fn tinyllama_chat_template_repo_serialized() -> String {
    tiny_llama_serialized().replace("{chat_template}", "TinyLlama/TinyLlama-1.1B-Chat-v1.0")
  }

  fn tinyllama_chat_template_id_serialized() -> String {
    tiny_llama_serialized().replace("{chat_template}", "tinyllama")
  }

  #[rstest]
  #[case("llama3:instruct", "llama3--instruct.yaml")]
  #[case("llama3/instruct", "llama3--instruct.yaml")]
  fn test_alias_config_filename(#[case] input: String, #[case] expected: String) {
    let alias = Alias {
      alias: input,
      ..Default::default()
    };
    assert_eq!(expected, alias.config_filename());
  }

  #[rstest]
  #[case(
    Alias::default(),
    r#"alias: ''
repo: ''
filename: ''
snapshot: ''
features: []
chat_template: llama3
"#
  )]
  #[case(
    tinyllama_chat_template_repo(),
    tinyllama_chat_template_repo_serialized()
  )]
  #[case(tinyllama_chat_template_id(), tinyllama_chat_template_id_serialized())]
  fn test_alias_serialize(#[case] alias: Alias, #[case] expected: String) -> anyhow::Result<()> {
    let actual = serde_yaml::to_string(&alias)?;
    assert_eq!(expected, actual);
    Ok(())
  }

  #[rstest]
  #[case(
    tinyllama_chat_template_repo_serialized(),
    tinyllama_chat_template_repo()
  )]
  #[case(tinyllama_chat_template_id_serialized(), tinyllama_chat_template_id())]
  fn test_alias_deserialized(
    #[case] serialized: String,
    #[case] expected: Alias,
  ) -> anyhow::Result<()> {
    let actual = serde_yaml::from_str(&serialized)?;
    assert_eq!(expected, actual);
    Ok(())
  }

  #[test]
  fn test_alias_to_row() -> anyhow::Result<()> {
    let alias = Alias::testalias();
    let row = Row::from(alias);
    assert_eq!(
      Row::from(vec![
        Cell::new("testalias:instruct"),
        Cell::new("testalias"),
        Cell::new("MyFactory/testalias-gguf"),
        Cell::new("testalias.Q8_0.gguf"),
        Cell::new("chat"),
        Cell::new("llama3"),
      ]),
      row
    );
    Ok(())
  }
}
