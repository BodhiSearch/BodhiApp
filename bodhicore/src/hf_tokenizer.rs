use serde::{
  de::{self, MapAccess, Visitor},
  Deserialize, Deserializer, Serialize,
};
use std::{fmt, fs};

use crate::home::configs_dir;

pub(crate) static TOKENIZER_CONFIG_FILENAME: &str = "tokenizer_config.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct HubTokenizerConfig {
  pub chat_template: Option<String>,
  #[serde(deserialize_with = "deserialize_token", default)]
  pub bos_token: Option<String>,
  #[serde(deserialize_with = "deserialize_token", default)]
  pub eos_token: Option<String>,
}

impl HubTokenizerConfig {
  pub fn from_json_file<P: AsRef<std::path::Path>>(filename: P) -> anyhow::Result<Self> {
    let content = std::fs::read_to_string(filename)?;
    HubTokenizerConfig::from_json_str(&content)
  }

  pub fn from_json_str(content: &str) -> anyhow::Result<Self> {
    let config = serde_json::from_str::<HubTokenizerConfig>(content)?;
    Ok(config)
  }

  pub fn for_repo(repo: &str) -> anyhow::Result<Self> {
    let config_file = configs_dir(repo)?.join("default.yaml");
    let content = fs::read_to_string(config_file)?;
    let config = serde_yaml::from_str::<Self>(&content)?;
    Ok(config)
  }
}

fn deserialize_token<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  struct StringOrMap;

  impl<'de> Visitor<'de> for StringOrMap {
    type Value = Option<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("a string or a map with a 'content' key")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: de::Error,
    {
      Ok(Some(v.to_owned()))
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
      M: MapAccess<'de>,
    {
      let mut content = None;
      while let Some((key, value)) = map.next_entry::<String, String>()? {
        if key == "content" {
          content = Some(value);
        }
      }
      Ok(content)
    }
  }

  deserializer.deserialize_any(StringOrMap)
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_parse_hub_tokenizer_config_load_empty() -> anyhow::Result<()> {
    let empty = HubTokenizerConfig::from_json_str("{}")?;
    assert_eq!(HubTokenizerConfig::default(), empty);
    Ok(())
  }

  #[test]
  fn test_parse_hub_tokenizer_config_load_chat_template() -> anyhow::Result<()> {
    let chat_template =
      HubTokenizerConfig::from_json_str("{\n \"chat_template\": \"llama.cpp:gemma\"\n}\n")?;
    let expected = HubTokenizerConfig {
      chat_template: Some("llama.cpp:gemma".to_string()),
      ..Default::default()
    };
    assert_eq!(expected, chat_template);
    Ok(())
  }
}
