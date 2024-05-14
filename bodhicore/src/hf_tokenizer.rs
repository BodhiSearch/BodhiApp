use crate::home::configs_dir;
use serde::{
  de::{self, MapAccess, Visitor},
  Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
use std::{fmt, fs};

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
  pub fn new(
    chat_template: Option<String>,
    bos_token: Option<String>,
    eos_token: Option<String>,
  ) -> Self {
    Self {
      chat_template,
      bos_token,
      eos_token,
    }
  }
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
      let mut content: Option<String> = None;
      while let Some((key, value)) = map.next_entry::<String, Value>()? {
        if key == "content" {
          content = value.as_str().map(|str| str.to_string());
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
  use crate::test_utils::{config_dirs, ConfigDirs};
  use rstest::rstest;
  use tempfile::{tempfile_in, NamedTempFile};

  #[test]
  fn test_hf_tokenizer_from_json_str_empty() -> anyhow::Result<()> {
    let empty = HubTokenizerConfig::from_json_str("{}")?;
    assert_eq!(HubTokenizerConfig::default(), empty);
    Ok(())
  }

  #[test]
  fn test_hf_tokenizer_from_json_str_chat_template() -> anyhow::Result<()> {
    let chat_template =
      HubTokenizerConfig::from_json_str(r#"{"chat_template": "llama.cpp:gemma"}"#)?;
    let expected = HubTokenizerConfig {
      chat_template: Some("llama.cpp:gemma".to_string()),
      ..Default::default()
    };
    assert_eq!(expected, chat_template);
    Ok(())
  }

  #[test]
  fn test_hf_tokenizer_from_json_str_bos_eos_token() -> anyhow::Result<()> {
    let chat_template = r#"{{ bos_token }} {%- for message in messages %} message['role']: {{ message['content'] }} {% endfor %} {{ eos_token }}"#;
    let hf_tokenizer = HubTokenizerConfig::from_json_str(&format!(
      r#"{{
        "chat_template": "{chat_template}",
        "bos_token": "<s>",
        "eos_token": "</s>"
      }}"#
    ))?;
    let expected = HubTokenizerConfig::new(
      Some(chat_template.to_string()),
      Some("<s>".to_string()),
      Some("</s>".to_string()),
    );
    assert_eq!(expected, hf_tokenizer);
    Ok(())
  }

  #[test]
  fn test_hf_tokenizer_from_json_file() -> anyhow::Result<()> {
    let chat_template = "{{ bos_token }} {% for message in messages %}{{ message['role'] }}: {{ message['content'] }}{% endfor %} {{ eos_token }}";
    let tokenizer_json = format!(
      r#"{{
      "bos_token": "<s>",
      "chat_template": "{chat_template}",
      "eos_token": "</s>"
    }}"#
    );
    let tempdir = tempfile::tempdir()?;
    let json_file = NamedTempFile::new_in(&tempdir)?;
    fs::write(&json_file, tokenizer_json)?;
    let config = HubTokenizerConfig::from_json_file(&json_file)?;
    let expected = HubTokenizerConfig::new(
      Some(chat_template.to_string()),
      Some("<s>".to_string()),
      Some("</s>".to_string()),
    );
    assert_eq!(expected, config);
    Ok(())
  }

  #[rstest]
  fn test_hf_tokenizer_for_repo(config_dirs: ConfigDirs) -> anyhow::Result<()> {
    let ConfigDirs(_home_dir, config_dir, repo) = config_dirs;
    let default_config_file = config_dir.join("default.yaml");
    fs::write(
      default_config_file,
      r#"
chat_template: |
  {{ bos_token }} {% for message in messages -%}
  message['role']: message['content']
  {% endfor %} {{ eos_token }}
bos_token: <s>
eos_token: </s>
"#,
    )?;
    let expected = HubTokenizerConfig::new(
      Some(
        r#"{{ bos_token }} {% for message in messages -%}
message['role']: message['content']
{% endfor %} {{ eos_token }}
"#
        .to_string(),
      ),
      Some("<s>".to_string()),
      Some("</s>".to_string()),
    );
    let config = HubTokenizerConfig::for_repo(repo)?;
    assert_eq!(expected, config);
    Ok(())
  }

  #[test]
  fn test_hf_tokenizer_parses_eos_token_as_obj() -> anyhow::Result<()> {
    let tokenizer_json = r#"{
      "bos_token": {
        "__type": "AddedToken",
        "content": "<s>",
        "lstrip": false,
        "normalized": false,
        "rstrip": false,
        "single_word": false
      },
      "chat_template": "{{ bos_token }} {% for message in messages %}{{ message['role'] }}: {{ message['content'] }}{% endfor %} {{ eos_token }}",
      "eos_token": {
        "__type": "AddedToken",
        "content": "</s>",
        "lstrip": false,
        "normalized": false,
        "rstrip": false,
        "single_word": false
      }
    }
    "#;
    let chat_template = "{{ bos_token }} {% for message in messages %}{{ message['role'] }}: {{ message['content'] }}{% endfor %} {{ eos_token }}";
    let config = HubTokenizerConfig::from_json_str(tokenizer_json)?;
    let expected = HubTokenizerConfig::new(
      Some(chat_template.to_string()),
      Some("<s>".to_string()),
      Some("</s>".to_string()),
    );
    assert_eq!(expected, config);
    Ok(())
  }
}
