use crate::home::configs_dir;
use anyhow::anyhow;
use async_openai::types::{
  ChatCompletionRequestMessage,
  ChatCompletionRequestUserMessageContent::{Array, Text},
};
use minijinja::{Environment, ErrorKind};
use serde::{
  de::{self, MapAccess, Visitor},
  Deserialize, Deserializer, Serialize,
};
use std::{fmt, fs};

pub fn raise_exception(err_text: String) -> Result<String, minijinja::Error> {
  Err(minijinja::Error::new(ErrorKind::SyntaxError, err_text))
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ChatMessage {
  role: Option<String>,
  content: Option<String>,
}

impl<'a> From<&'a ChatMessage> for ChatMessage {
  fn from(value: &'a ChatMessage) -> Self {
    value.clone()
  }
}

impl<'a> From<&'a ChatCompletionRequestMessage> for ChatMessage {
  fn from(value: &'a ChatCompletionRequestMessage) -> Self {
    let (role, content) = match value {
      ChatCompletionRequestMessage::System(m) => (m.role.to_string(), Some(m.content.clone())),
      ChatCompletionRequestMessage::User(m) => match &m.content {
        Text(content) => (m.role.to_string(), Some(content.clone())),
        Array(content) => {
          let fold = content.clone().into_iter().fold(String::new(), |mut f, i| {
            match i {
              async_openai::types::ChatCompletionRequestMessageContentPart::Text(t) => {
                f.push_str(&t.text);
              }
              async_openai::types::ChatCompletionRequestMessageContentPart::Image(_) => {
                unimplemented!()
              }
            };
            f
          });
          (m.role.to_string().clone(), Some(fold))
        }
      },
      ChatCompletionRequestMessage::Assistant(m) => (m.role.to_string().clone(), m.content.clone()),
      ChatCompletionRequestMessage::Tool(_) => unimplemented!(),
      ChatCompletionRequestMessage::Function(_) => unimplemented!(),
    };
    ChatMessage {
      role: Some(role),
      content,
    }
  }
}

impl ChatMessage {
  pub fn new(role: String, content: String) -> Self {
    Self {
      role: Some(role),
      content: Some(content),
    }
  }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub(crate) struct ChatTemplateInputs {
  messages: Vec<ChatMessage>,
  bos_token: Option<String>,
  eos_token: Option<String>,
  add_generation_prompt: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ChatTemplate {
  name: String,
  template: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ChatTemplateVersions {
  Single(String),
  Multiple(Vec<ChatTemplate>),
}

impl ChatTemplateVersions {
  pub fn is_empty(&self) -> bool {
    match self {
      ChatTemplateVersions::Single(template) => template.is_empty(),
      ChatTemplateVersions::Multiple(templates) => templates
        .iter()
        .find(|t| t.name == "default")
        .map(|t| t.template.is_empty())
        .unwrap_or(true),
    }
  }
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct HubTokenizerConfig {
  pub chat_template: Option<ChatTemplateVersions>,
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
      chat_template: chat_template.map(ChatTemplateVersions::Single),
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

  pub fn apply_chat_template<T>(&self, messages: &[T]) -> anyhow::Result<String>
  where
    for<'a> &'a T: Into<ChatMessage>,
  {
    let chat_template = self
      .chat_template
      .clone() // TODO: do not clone
      .and_then(|t| match t {
        ChatTemplateVersions::Single(template) => Some(template),
        ChatTemplateVersions::Multiple(templates) => templates
          .into_iter()
          .find(|t| t.name == "default")
          .map(|t| t.template),
      })
      .ok_or(anyhow!("chat_template not found in tokenizer_config.json"))?
      .replace(".strip()", " | trim")
      .replace(".title()", " | title");
    let mut env = Box::new(Environment::new());
    let template_str = chat_template.into_boxed_str();
    eprintln!("{template_str}");
    env.add_function("raise_exception", raise_exception);
    let template = Box::leak(env).template_from_str(Box::leak(template_str))?;
    let messages: Vec<ChatMessage> = messages.iter().map(Into::into).collect();

    let inputs = ChatTemplateInputs {
      messages,
      bos_token: self.bos_token.clone(),
      eos_token: self.eos_token.clone(),
      add_generation_prompt: true,
    };
    let result = template.render(inputs)?;
    Ok(result)
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
      while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
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
  use anyhow::anyhow;
  use anyhow_trace::anyhow_trace;
  use rstest::rstest;
  use tempfile::NamedTempFile;

  #[test]
  fn test_hf_tokenizer_from_json_str_empty() -> anyhow::Result<()> {
    let empty = HubTokenizerConfig::from_json_str("{}")?;
    assert_eq!(HubTokenizerConfig::default(), empty);
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[case("llama3", "meta-llama/Meta-Llama-3-8B-Instruct")]
  #[case("llama2", "meta-llama/Llama-2-13b-chat-hf")]
  #[case("phi3", "microsoft/Phi-3-mini-4k-instruct")]
  #[case("llama2-legacy", "mistralai/Mixtral-8x7B-Instruct-v0.1")]
  #[case("gemma", "google/gemma-7b-it")]
  // #[case("zephyr", "HuggingFaceH4/zephyr-7b-beta")]
  #[case("deepseek", "deepseek-ai/deepseek-llm-67b-chat")]
  #[case("command-r", "CohereForAI/c4ai-command-r-plus")]
  #[case("openchat", "openchat/openchat-3.6-8b-20240522")]
  fn test_hf_tokenizer_apply_chat_template(
    #[case] format: String,
    #[case] model: String,
    #[values(
      "simple",
      "assistant",
      "system",
      "convo",
      "unknown-role",
      "error-user-at-even-no-system",
      "error-user-at-even-with-system"
    )]
    case: String,
  ) -> anyhow::Result<()> {
    let filename = format!("tests/data/tokenizers/{}/tokenizer_config.json", model);
    let content = std::fs::read_to_string(filename)?;
    let tokenizer = HubTokenizerConfig::from_json_str(&content)?;

    let inputs = std::fs::read_to_string("chat-template-compat/tests/data/inputs.yaml")?;
    let inputs: serde_yaml::Value = serde_yaml::from_str(&inputs)?;
    let input = inputs
      .as_sequence()
      .ok_or(anyhow!("should be an array of test cases"))?
      .iter()
      .find(|item| item["id"] == case)
      .ok_or(anyhow!(
        "test case with id: {case} not found for model: {model}"
      ))?;
    let messages: Vec<ChatMessage> = serde_yaml::from_value(input["messages"].clone())?;
    let expected = &input[&format];

    #[allow(clippy::blocks_in_conditions)]
    if expected.is_string() {
      let prompt = tokenizer.apply_chat_template(&messages)?;
      let expected = expected
        .as_str()
        .ok_or(anyhow!(
          "expected value for key: {format}, for case {case} to be string"
        ))?
        .trim_end_matches('\n')
        .replace("\\n", "\n");
      assert_eq!(expected, prompt);
    } else if expected["exception"]
      .as_bool()
      .ok_or(anyhow!("exception should be bool"))?
    {
      let message = expected["message"]
        .as_str()
        .ok_or(anyhow!("error message should be str"))?;
      let prompt = tokenizer.apply_chat_template(&messages);
      assert!(prompt.is_err());
      assert!(prompt
        .unwrap_err()
        .to_string()
        .starts_with(&format!("syntax error: {message} (in <string>:")));
    } else {
      unreachable!("expected should be either string, or exception")
    }
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

  #[test]
  fn test_hf_tokenizer_fails_on_invalid_json() -> anyhow::Result<()> {
    let config = HubTokenizerConfig::from_json_str(r#"{"eos_token": true}"#);
    assert!(config.is_err());
    let error = config.unwrap_err();
    assert!(error.is::<serde_json::Error>());
    assert_eq!("invalid type: boolean `true`, expected a string or a map with a 'content' key at line 1 column 18", format!("{error}"));
    Ok(())
  }
}
