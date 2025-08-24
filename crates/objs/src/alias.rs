use crate::{is_default, to_safe_filename, OAIRequestParams, Repo};
use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(
  Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, strum::Display,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum AliasSource {
  #[default]
  User,
  Model,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, derive_builder::Builder, new)]
#[builder(
  setter(into, strip_option),
  build_fn(error = crate::BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct Alias {
  pub alias: String,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  #[serde(default, skip_serializing_if = "is_default")]
  #[builder(default)]
  pub source: AliasSource,
  #[serde(default, skip_serializing_if = "is_default")]
  #[builder(default)]
  pub request_params: OAIRequestParams,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  #[builder(default)]
  pub context_params: Vec<String>,
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
  use crate::{Alias, AliasBuilder, OAIRequestParamsBuilder, Repo};
  use anyhow_trace::anyhow_trace;
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

  #[anyhow_trace]
  #[rstest]
  #[case::full(
    AliasBuilder::tinyllama()
      .request_params(OAIRequestParamsBuilder::default()
        .temperature(0.7)
        .top_p(0.95)
        .build()
        .unwrap())
      .context_params(vec![
        "--ctx-size 2048".to_string(),
        "--parallel 4".to_string(),
        "--n-predict 256".to_string(),
      ])
      .build()
      .unwrap(),
    r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
request_params:
  temperature: 0.7
  top_p: 0.95
context_params:
- --ctx-size 2048
- --parallel 4
- --n-predict 256
"#
  )]
  #[case::minimal(
    AliasBuilder::tinyllama()
      .build()
      .unwrap(),
    r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
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
request_params:
  temperature: 0.7
  top_p: 0.95
context_params:
- --ctx-size 2048
- --parallel 4
- --n-predict 256
"#,
  AliasBuilder::tinyllama()
  .request_params(OAIRequestParamsBuilder::default()
  .temperature(0.7)
  .top_p(0.95)
  .build()
  .unwrap())
  .context_params(vec![
    "--ctx-size 2048".to_string(),
    "--parallel 4".to_string(),
    "--n-predict 256".to_string(),
  ])
  .build()
  .unwrap()
  )]
  #[case::minimal(r#"alias: tinyllama:instruct
repo: TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF
filename: tinyllama-1.1b-chat-v0.3.Q2_K.gguf
snapshot: b32046744d93031a26c8e925de2c8932c305f7b9
"#, AliasBuilder::tinyllama()
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
