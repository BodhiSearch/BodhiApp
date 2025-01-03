use crate::{
  is_default, to_safe_filename, ChatTemplateType, GptContextParams, OAIRequestParams, Repo,
};
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
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  pub chat_template: ChatTemplateType,
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

impl std::fmt::Display for Alias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Alias {{ alias: {}, repo: {}, filename: {}, snapshot: {} }}",
      self.alias, self.repo, self.filename, self.snapshot
    )
  }
}

#[cfg(test)]
mod test {
  use crate::{
    Alias, AliasBuilder, ChatTemplateId, ChatTemplateType, GptContextParamsBuilder,
    OAIRequestParamsBuilder, Repo,
  };
  use rstest::rstest;

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
  #[case::full(
    AliasBuilder::tinyllama()
      .chat_template(ChatTemplateType::tinyllama())
      .request_params(OAIRequestParamsBuilder::default()
        .temperature(0.7)
        .top_p(0.95)
        .build()
        .unwrap())
      .context_params(
        GptContextParamsBuilder::default()
          .n_ctx(2048)
          .n_parallel(4u8)
          .n_predict(256)
          .build()
          .unwrap(),
      )
      .build()
      .unwrap(),
    r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
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
  #[case::chat_template_id(
    AliasBuilder::tinyllama()
      .chat_template(ChatTemplateType::Id(ChatTemplateId::Tinyllama))
      .build()
      .unwrap(),
    r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
chat_template: tinyllama
"#)]
  fn test_alias_serialize(#[case] alias: Alias, #[case] expected: &str) -> anyhow::Result<()> {
    let actual = serde_yaml::to_string(&alias)?;
    assert_eq!(expected, actual);
    Ok(())
  }

  #[rstest]
  #[case::request_ctx_params(
    r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
chat_template: TinyLlama/TinyLlama-1.1B-Chat-v1.0
request_params:
  temperature: 0.7
  top_p: 0.95
context_params:
  n_ctx: 2048
  n_parallel: 4
  n_predict: 256
"#,
  AliasBuilder::tinyllama()
    .chat_template(ChatTemplateType::tinyllama())
  .request_params(OAIRequestParamsBuilder::default()
  .temperature(0.7)
  .top_p(0.95)
  .build()
  .unwrap())
  .context_params(
  GptContextParamsBuilder::default()
    .n_ctx(2048)
    .n_parallel(4u8)
    .n_predict(256)
    .build()
    .unwrap(),
  )
  .build()
  .unwrap()
  )]
  #[case::chat_template_id(r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
chat_template: tinyllama
"#, AliasBuilder::tinyllama()
.chat_template(ChatTemplateType::Id(ChatTemplateId::Tinyllama))
.build()
.unwrap())]
  fn test_alias_deserialized(
    #[case] serialized: &str,
    #[case] expected: Alias,
  ) -> anyhow::Result<()> {
    let actual = serde_yaml::from_str(serialized)?;
    assert_eq!(expected, actual);
    Ok(())
  }

  #[test]
  fn test_alias_display() {
    let alias = Alias {
      alias: "test:alias".to_string(),
      repo: Repo::try_from("test/repo").unwrap(),
      filename: "test.gguf".to_string(),
      snapshot: "main".to_string(),
      ..Default::default()
    };
    assert_eq!(
      format!("{}", alias),
      "Alias { alias: test:alias, repo: test/repo, filename: test.gguf, snapshot: main }"
    );
  }
}
