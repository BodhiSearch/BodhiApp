use crate::home::configs_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatTemplate {
  name: String,
  template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ChatTemplateVersions {
  Single(String),
  Multiple(Vec<ChatTemplate>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HubTokenizerConfig {
  pub chat_template: Option<ChatTemplateVersions>,
  pub completion_template: Option<String>,
  #[serde(deserialize_with = "token_serde::deserialize")]
  pub bos_token: Option<String>,
  #[serde(deserialize_with = "token_serde::deserialize")]
  pub eos_token: Option<String>,
}

pub(crate) static TOKENIZER_CONFIG_FILENAME: &str = "tokenizer_config.json";

impl HubTokenizerConfig {
  pub fn from_file<P: AsRef<std::path::Path>>(filename: P) -> Option<Self> {
    let content = std::fs::read_to_string(filename).ok()?;
    serde_json::from_str(&content).ok()
  }

  pub fn save(&self, repo: &str) -> anyhow::Result<PathBuf> {
    let contents = serde_json::to_string_pretty(&self)?;
    let configs_dir = configs_dir(repo)?;
    let config_file = configs_dir.join(TOKENIZER_CONFIG_FILENAME);
    fs::write(&config_file, contents)?;
    Ok(config_file)
  }

  pub fn load(repo: &str) -> anyhow::Result<Self> {
    let configs_dir = configs_dir(repo)?;
    let config_file = configs_dir.join(TOKENIZER_CONFIG_FILENAME);
    let contents = fs::read_to_string(config_file)?;
    let config: Self = serde_json::from_str(&contents)?;
    Ok(config)
  }
}

mod token_serde {
  use super::*;
  use serde::de;
  use serde::Deserializer;
  use serde_json::Value;

  pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
  where
    D: Deserializer<'de>,
  {
    let value = Value::deserialize(deserializer)?;

    match value {
      Value::String(s) => Ok(Some(s)),
      Value::Object(map) => {
        if let Some(content) = map.get("content").and_then(|v| v.as_str()) {
          Ok(Some(content.to_string()))
        } else {
          Err(de::Error::custom(
            "content key not found in structured token",
          ))
        }
      }
      Value::Null => Ok(None),
      _ => Err(de::Error::custom("invalid token format")),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{hf::HF_TOKEN, server::BODHI_HOME};
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_hub_tokenizer_config_save() -> anyhow::Result<()> {
    std::env::set_var(HF_TOKEN, "");
    let bodhi_home = tempfile::tempdir().unwrap();
    std::env::set_var(BODHI_HOME, bodhi_home.path().to_string_lossy().into_owned());

    let chat_template =
      r#"{% for message in messages %}<|{{ message['role'] }}|> {{ message['content'] }}\n"#;
    let config = HubTokenizerConfig {
      chat_template: Some(ChatTemplateVersions::Single(chat_template.to_string())),
      completion_template: Some(chat_template.to_string()),
      bos_token: Some("<s>".to_string()),
      eos_token: Some("</s>".to_string()),
    };

    let repo = "meta-llama/Meta-Llama-3-8B";
    let config_path = config.save(repo)?;
    let expected = bodhi_home
      .path()
      .to_path_buf()
      .join("configs--meta-llama--Meta-Llama-3-8B")
      .join(TOKENIZER_CONFIG_FILENAME)
      .to_string_lossy()
      .into_owned();
    assert_eq!(expected, config_path.to_string_lossy().into_owned());
    let config = HubTokenizerConfig::load(repo)?;
    assert_eq!(
      ChatTemplateVersions::Single(chat_template.to_string()),
      config.chat_template.unwrap()
    );
    Ok(())
  }
}
