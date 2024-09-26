use crate::ContextError;
use async_openai::types::{
  ChatCompletionRequestMessage,
  ChatCompletionRequestUserMessageContent::{Array, Text},
};
use derive_new::new;
use minijinja::{Environment, ErrorKind};
use objs::{validation_errors, HubFile, ObjError};
use serde::{
  de::{self, MapAccess, Visitor},
  Deserialize, Deserializer, Serialize,
};
use std::{fmt, ops::Deref};
use validator::{Validate, ValidationError};

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

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ChatTemplateInputs {
  messages: Vec<ChatMessage>,
  bos_token: Option<String>,
  eos_token: Option<String>,
  add_generation_prompt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatTemplateEntry {
  name: String,
  template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ChatTemplateVersions {
  Single(String),
  Multiple(Vec<ChatTemplateEntry>),
}

impl ChatTemplateVersions {
  pub fn chat_template(&self) -> Option<String> {
    match self {
      ChatTemplateVersions::Single(template) => Some(template.clone()),
      ChatTemplateVersions::Multiple(templates) => templates
        .deref()
        .iter()
        .find(|t| t.name == "default")
        .map(|t| t.template.clone()),
    }
  }
}

#[derive(Debug, Clone, Deserialize, PartialEq, new, Validate)]
pub struct TokenizerConfig {
  #[validate(custom(function = "validate_chat_template"))]
  pub chat_template: ChatTemplateVersions,
  #[serde(deserialize_with = "deserialize_token", default)]
  pub bos_token: Option<String>,
  #[serde(deserialize_with = "deserialize_token", default)]
  pub eos_token: Option<String>,
}

fn validate_chat_template(chat_template: &ChatTemplateVersions) -> Result<(), ValidationError> {
  match chat_template.chat_template() {
    Some(_) => Ok(()),
    None => Err(ValidationError::new(
      "chat_template missing in tokenizer_config.json",
    )),
  }
}

impl TokenizerConfig {
  #[allow(clippy::result_large_err)]
  pub fn apply_chat_template<T>(&self, messages: &[T]) -> Result<String, ContextError>
  where
    for<'a> &'a T: Into<ChatMessage>,
  {
    let chat_template = self
      .chat_template
      .chat_template()
      .ok_or_else(|| {
        let error = ValidationError::new("chat_template missing in tokenizer_config.json");
        validation_errors("chat_template", error)
      })?
      .replace(".strip()", " | trim")
      .replace(".title()", " | title");
    let mut env = Box::new(Environment::new());
    let template_str = chat_template.into_boxed_str();
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

impl TryFrom<HubFile> for TokenizerConfig {
  type Error = ObjError;

  fn try_from(value: HubFile) -> Result<Self, Self::Error> {
    let path = value.path();
    let content = std::fs::read_to_string(path.clone())
      .map_err(move |source| ObjError::IoWithDetail { source, path })?;
    let tokenizer_config: TokenizerConfig = serde_json::from_str(&content)?;
    Ok(tokenizer_config)
  }
}

#[cfg(test)]
mod test {
  use crate::{ChatMessage, ChatTemplateVersions, TokenizerConfig};
  use anyhow::anyhow;
  use anyhow_trace::anyhow_trace;
  use objs::{test_utils::temp_hf_home, HubFileBuilder};
  use rstest::rstest;
  use tempfile::TempDir;

  #[anyhow_trace]
  #[rstest]
  #[case("llama3", "meta-llama/Meta-Llama-3-8B-Instruct")]
  #[case("llama2", "meta-llama/Llama-2-13b-chat-hf")]
  #[case("phi3", "microsoft/Phi-3-mini-4k-instruct")]
  #[case("llama2-legacy", "mistralai/Mixtral-8x7B-Instruct-v0.1")]
  #[case("gemma", "google/gemma-7b-it")]
  #[case("deepseek", "deepseek-ai/deepseek-llm-67b-chat")]
  #[case("command-r", "CohereForAI/c4ai-command-r-plus")]
  #[case("openchat", "openchat/openchat-3.6-8b-20240522")]
  #[case("tinyllama", "TinyLlama/TinyLlama-1.1B-Chat-v1.0")]
  // #[case("zephyr", "HuggingFaceH4/zephyr-7b-beta")]
  fn test_tokenizer_config_apply_chat_template(
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
    let config = serde_json::from_str::<TokenizerConfig>(&content)?;

    let input_filename = concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/../../chat-template-compat/tests/data/inputs.yaml"
    );
    let inputs = std::fs::read_to_string(input_filename).map_err(|source| {
      anyhow!("failed to read inputs file on path  {input_filename}: {source}")
    })?;
    let inputs: serde_yaml::Value = serde_yaml::from_str(&inputs)?;
    let input = inputs
      .as_sequence()
      .ok_or_else(|| anyhow!("should be an array of test cases"))?
      .iter()
      .find(|item| item["id"] == case)
      .ok_or_else(|| anyhow!("test case with id: {case} not found for model: {model}"))?;
    let messages: Vec<ChatMessage> = serde_yaml::from_value(input["messages"].clone())?;
    let expected = &input[&format];

    #[allow(clippy::blocks_in_conditions)]
    if expected.is_string() {
      let prompt = config.apply_chat_template(&messages)?;
      let expected = expected
        .as_str()
        .ok_or_else(|| anyhow!("expected value for key: {format}, for case {case} to be string"))?
        .trim_end_matches('\n')
        .replace("\\n", "\n");
      assert_eq!(expected, prompt);
    } else if expected["exception"]
      .as_bool()
      .ok_or_else(|| anyhow!("exception should be bool"))?
    {
      let message = expected["message"]
        .as_str()
        .ok_or_else(|| anyhow!("error message should be str"))?;
      let prompt = config.apply_chat_template(&messages);
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

  #[rstest]
  #[case("simple.json", 
  TokenizerConfig::new(
    ChatTemplateVersions::Single("{{ bos_token }} {%- for message in messages %} message['role']: {{ message['content'] }} {% endfor %} {{ eos_token }}".to_string()),
    Some("<s>".to_string()),
    Some("</s>".to_string()),
  ))]
  #[case("bos_eos_objs.json", TokenizerConfig::new(
    ChatTemplateVersions::Single("{{ bos_token }} {% for message in messages %}{{ message['role'] }}: {{ message['content'] }}{% endfor %} {{ eos_token }}".to_string()),
    Some("<s>".to_string()),
    Some("</s>".to_string()),
  ))]
  fn test_tokenizer_config_from_json_str_empty(
    #[case] input: String,
    #[case] expected: TokenizerConfig,
  ) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(format!("tests/data/tokenizer_configs/{}", input))?;
    let empty = serde_json::from_str::<TokenizerConfig>(&content)?;
    assert_eq!(expected, empty);
    Ok(())
  }

  #[rstest]
  #[case("invalid.json", "invalid type: boolean `true`, expected a string or a map with a 'content' key at line 2 column 19")]
  fn test_tokenizer_config_invalid(
    #[case] input: String,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(format!("tests/data/tokenizer_configs/{}", input))?;
    let config = serde_json::from_str::<TokenizerConfig>(&content);
    assert!(config.is_err());
    assert_eq!(expected, config.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  fn test_tokenizer_config_from_hub_file(temp_hf_home: TempDir) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface/hub");
    let tokenizer_file = HubFileBuilder::testalias_tokenizer()
      .hf_cache(hf_cache)
      .build()
      .unwrap();
    let tokenizer_config = TokenizerConfig::try_from(tokenizer_file)?;
    let expected = TokenizerConfig::new(
      ChatTemplateVersions::Single("{% set loop_messages = messages %}{% for message in loop_messages %}{% set content = '<|start_header_id|>' + message['role'] + '<|end_header_id|>\n\n'+ message['content'] | trim + '<|eot_id|>' %}{% if loop.index0 == 0 %}{% set content = bos_token + content %}{% endif %}{{ content }}{% endfor %}{% if add_generation_prompt %}{{ '<|start_header_id|>assistant<|end_header_id|>\n\n' }}{% endif %}".to_string()),
      Some("<|begin_of_text|>".to_string()),
      Some("<|eot_id|>".to_string()),
    );
    assert_eq!(expected, tokenizer_config);
    Ok(())
  }
}
