use crate::{is_default, to_safe_filename, ChatTemplate, GptContextParams, OAIRequestParams, Repo};
use derive_new::new;
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
    build_fn(error = crate::BuilderError)))]
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

// TODO: hard coding for time being
pub fn default_features() -> Vec<String> {
  vec!["chat".to_string()]
}

impl std::fmt::Display for Alias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Alias {{ alias: {}, repo: {}, filename: {} }}",
      self.alias, self.repo, self.filename
    )
  }
}

#[cfg(test)]
mod test {
  use crate::{
    Alias, AliasBuilder, ChatTemplate, ChatTemplateId, GptContextParamsBuilder,
    OAIRequestParamsBuilder, Repo,
  };
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
    tinyllama_builder()
      .chat_template(ChatTemplate::Repo(
        Repo::try_from("TinyLlama/TinyLlama-1.1B-Chat-v1.0").unwrap(),
      ))
      .build()
      .unwrap(),
    r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF
filename: tinyllama-1.1b-chat-v1.0.Q4_0.gguf
snapshot: 52e7645ba7c309695bec7ac98f4f005b139cf465
features:
- chat
chat_template: TinyLlama/TinyLlama-1.1B-Chat-v1.0
request_params:
  temperature: 0.7
  top_p: 0.95
context_params:
  n_ctx: 2048
  n_parallel: 4
  n_predict: 256
"#
  )]
  #[case(
    tinyllama_builder()
      .chat_template(ChatTemplate::Id(ChatTemplateId::Tinyllama))
      .build()
      .unwrap(),
    r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF
filename: tinyllama-1.1b-chat-v1.0.Q4_0.gguf
snapshot: 52e7645ba7c309695bec7ac98f4f005b139cf465
features:
- chat
chat_template: tinyllama
request_params:
  temperature: 0.7
  top_p: 0.95
context_params:
  n_ctx: 2048
  n_parallel: 4
  n_predict: 256
"#)]
  fn test_alias_serialize(#[case] alias: Alias, #[case] expected: &str) -> anyhow::Result<()> {
    let actual = serde_yaml::to_string(&alias)?;
    assert_eq!(expected, actual);
    Ok(())
  }

  #[rstest]
  #[case(
    r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF
filename: tinyllama-1.1b-chat-v1.0.Q4_0.gguf
snapshot: 52e7645ba7c309695bec7ac98f4f005b139cf465
features:
- chat
chat_template: TinyLlama/TinyLlama-1.1B-Chat-v1.0
request_params:
  temperature: 0.7
  top_p: 0.95
context_params:
  n_ctx: 2048
  n_parallel: 4
  n_predict: 256
"#,
tinyllama_builder()
.chat_template(ChatTemplate::Repo(
  Repo::try_from("TinyLlama/TinyLlama-1.1B-Chat-v1.0").unwrap(),
))
.build()
.unwrap()
  )]
  #[case(r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF
filename: tinyllama-1.1b-chat-v1.0.Q4_0.gguf
snapshot: 52e7645ba7c309695bec7ac98f4f005b139cf465
features:
- chat
chat_template: tinyllama
request_params:
  temperature: 0.7
  top_p: 0.95
context_params:
  n_ctx: 2048
  n_parallel: 4
  n_predict: 256
"#, tinyllama_builder()
.chat_template(ChatTemplate::Id(ChatTemplateId::Tinyllama))
.build()
.unwrap())]
  fn test_alias_deserialized(
    #[case] serialized: &str,
    #[case] expected: Alias,
  ) -> anyhow::Result<()> {
    let actual = serde_yaml::from_str(&serialized)?;
    assert_eq!(expected, actual);
    Ok(())
  }

  #[test]
  fn test_alias_display() {
    let alias = Alias {
      alias: "test:alias".to_string(),
      repo: Repo::try_from("test/repo").unwrap(),
      filename: "test.gguf".to_string(),
      ..Default::default()
    };
    assert_eq!(
      format!("{}", alias),
      "Alias { alias: test:alias, repo: test/repo, filename: test.gguf }"
    );
  }
}