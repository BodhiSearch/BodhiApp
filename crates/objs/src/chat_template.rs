use crate::{
  gguf::{GGUFMetadata, GGUFMetadataError, GGUFValue},
  impl_error_from, validation_errors, AppError, ErrorType, HubFile, IoWithPathError,
  ObjValidationError, SerdeJsonError,
};
use async_openai::types::{
  ChatCompletionRequestMessage,
  ChatCompletionRequestUserMessageContent::{Array, Text},
  Role,
};
use derive_new::new;
use minijinja::{Environment, ErrorKind};
use serde::{
  de::{self, MapAccess, Visitor},
  Deserialize, Deserializer, Serialize,
};
use std::{fmt, ops::Deref, path::Path};
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
      ChatCompletionRequestMessage::System(m) => {
        (Role::System.to_string(), Some(m.content.clone()))
      }
      ChatCompletionRequestMessage::User(m) => match &m.content {
        Text(content) => (Role::User.to_string(), Some(content.clone())),
        Array(content) => {
          let fold = content.clone().into_iter().fold(String::new(), |mut f, i| {
            match i {
              async_openai::types::ChatCompletionRequestMessageContentPart::Text(t) => {
                f.push_str(&t.text);
              }
              async_openai::types::ChatCompletionRequestMessageContentPart::ImageUrl(_) => {
                unimplemented!()
              }
            };
            f
          });
          (Role::User.to_string(), Some(fold))
        }
      },
      ChatCompletionRequestMessage::Assistant(m) => {
        (Role::Assistant.to_string(), m.content.clone())
      }
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
pub struct ChatTemplate {
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

impl ChatTemplate {
  #[allow(clippy::result_large_err)]
  pub fn apply_chat_template<T>(&self, messages: &[T]) -> Result<String, ChatTemplateError>
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

  fn extract_token(metadata: &GGUFMetadata, tokens: &[GGUFValue], key: &str) -> Option<String> {
    let token_id = metadata.metadata().get(key).map(|eos| eos.as_u32());
    if let Some(Ok(token_id)) = token_id {
      let token_val = tokens.get(token_id as usize).map(|token| token.as_str());
      if let Some(Ok(token_val)) = token_val {
        Some(token_val.to_string())
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn extract_embed_chat_template(path: &Path) -> Result<ChatTemplate, ChatTemplateError> {
    let metadata = GGUFMetadata::new(path)?;
    let chat_template = metadata
      .metadata()
      .get("tokenizer.chat_template")
      .ok_or(ChatTemplateError::EmbedChatTemplateNotFound)?
      .as_str()?;
    let tokens = metadata
      .metadata()
      .get("tokenizer.ggml.tokens")
      .map(|tokens| tokens.as_array());
    let (bos_token, eos_token) = if let Some(Ok(tokens)) = tokens {
      (
        Self::extract_token(&metadata, tokens, "tokenizer.ggml.bos_token_id"),
        Self::extract_token(&metadata, tokens, "tokenizer.ggml.eos_token_id"),
      )
    } else {
      (None, None)
    };
    Ok(ChatTemplate {
      chat_template: ChatTemplateVersions::Single(chat_template.to_string()),
      bos_token,
      eos_token,
    })
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

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ChatTemplateError {
  #[error("unknown_file_extension")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  UnknownFileExtension(String),
  #[error(transparent)]
  GGUFMetadata(#[from] GGUFMetadataError),
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
  #[error(transparent)]
  IoWithPathError(#[from] IoWithPathError),
  #[error(transparent)]
  ObjValidationError(#[from] ObjValidationError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "chat_template_error-minijina_error", args_delegate = false)]
  Minijina(#[from] minijinja::Error),
  #[error("embed_chat_template_not_found")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  EmbedChatTemplateNotFound,
}

impl_error_from!(
  ::serde_json::Error,
  ChatTemplateError::SerdeJson,
  crate::SerdeJsonError
);

impl_error_from!(
  ::validator::ValidationErrors,
  ChatTemplateError::ObjValidationError,
  crate::ObjValidationError
);

impl TryFrom<HubFile> for ChatTemplate {
  type Error = ChatTemplateError;

  fn try_from(value: HubFile) -> Result<Self, Self::Error> {
    let path = value.path();
    match path.extension() {
      Some(ext) if ext == "json" => {
        let content = std::fs::read_to_string(path.clone())
          .map_err(|err| IoWithPathError::new(err, path.display().to_string()))?;
        let chat_template: ChatTemplate = serde_json::from_str(&content)?;
        Ok(chat_template)
      }
      Some(ext) if ext == "gguf" => Self::extract_embed_chat_template(&path),
      _ => Err(ChatTemplateError::UnknownFileExtension(
        path
          .extension()
          .unwrap_or_default()
          .to_string_lossy()
          .to_string(),
      )),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{
    test_utils::{
      assert_error_message, generate_test_data_chat_template, setup_l10n, temp_hf_home,
    },
    AppError, ChatMessage, ChatTemplate, ChatTemplateError, ChatTemplateVersions,
    FluentLocalizationService, HubFileBuilder, Repo,
  };
  use anyhow::anyhow;
  use anyhow_trace::anyhow_trace;
  use dirs::home_dir;
  use rstest::rstest;
  use std::{path::PathBuf, sync::Arc};
  use tempfile::TempDir;

  #[rstest]
  #[case(&ChatTemplateError::Minijina(minijinja::Error::new(minijinja::ErrorKind::NonKey, "error")), "error rendering template: not a key type: error")]
  #[case(&ChatTemplateError::EmbedChatTemplateNotFound, "chat template not found in gguf file")]
  fn test_error_messages_chat_template(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected_message: &str,
  ) {
    assert_error_message(
      localization_service,
      &error.code(),
      error.args(),
      expected_message,
    );
  }

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
  fn test_chat_template_apply_chat_template(
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
    let config = serde_json::from_str::<ChatTemplate>(&content)?;

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
  ChatTemplate::new(
    ChatTemplateVersions::Single("{{ bos_token }} {%- for message in messages %} message['role']: {{ message['content'] }} {% endfor %} {{ eos_token }}".to_string()),
    Some("<s>".to_string()),
    Some("</s>".to_string()),
  ))]
  #[case("bos_eos_objs.json", ChatTemplate::new(
    ChatTemplateVersions::Single("{{ bos_token }} {% for message in messages %}{{ message['role'] }}: {{ message['content'] }}{% endfor %} {{ eos_token }}".to_string()),
    Some("<s>".to_string()),
    Some("</s>".to_string()),
  ))]
  fn test_chat_template_from_json_str_empty(
    #[case] input: String,
    #[case] expected: ChatTemplate,
  ) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(format!("tests/data/tokenizer_configs/{}", input))?;
    let empty = serde_json::from_str::<ChatTemplate>(&content)?;
    assert_eq!(expected, empty);
    Ok(())
  }

  #[rstest]
  #[case("invalid.json", "invalid type: boolean `true`, expected a string or a map with a 'content' key at line 2 column 19")]
  fn test_chat_template_invalid(
    #[case] input: String,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(format!("tests/data/tokenizer_configs/{}", input))?;
    let config = serde_json::from_str::<ChatTemplate>(&content);
    assert!(config.is_err());
    assert_eq!(expected, config.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  fn test_chat_template_from_hub_file(temp_hf_home: TempDir) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface").join("hub");
    let tokenizer_file = HubFileBuilder::testalias_tokenizer()
      .hf_cache(hf_cache)
      .build()
      .unwrap();
    let chat_template = ChatTemplate::try_from(tokenizer_file)?;
    let expected = ChatTemplate::new(
      ChatTemplateVersions::Single("{% set loop_messages = messages %}{% for message in loop_messages %}{% set content = '<|start_header_id|>' + message['role'] + '<|end_header_id|>\n\n'+ message['content'] | trim + '<|eot_id|>' %}{% if loop.index0 == 0 %}{% set content = bos_token + content %}{% endif %}{{ content }}{% endfor %}{% if add_generation_prompt %}{{ '<|start_header_id|>assistant<|end_header_id|>\n\n' }}{% endif %}".to_string()),
      Some("<|begin_of_text|>".to_string()),
      Some("<|eot_id|>".to_string()),
    );
    assert_eq!(expected, chat_template);
    Ok(())
  }

  #[rstest]
  fn test_chat_template_parse_from_unknown_extension(temp_hf_home: TempDir) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface").join("hub");
    let tokenizer_model = HubFileBuilder::testalias()
      .repo(Repo::llama2_70b_chat())
      .filename("tokenizer.model".to_string())
      .snapshot("e9149a12809580e8602995856f8098ce973d1080".to_string())
      .hf_cache(hf_cache)
      .size(Some(1000))
      .build()
      .unwrap();
    let chat_template = ChatTemplate::try_from(tokenizer_model);
    assert!(chat_template.is_err());
    assert!(matches!(
      chat_template.unwrap_err(),
      ChatTemplateError::UnknownFileExtension(ext) if ext == "model"
    ));
    Ok(())
  }

  fn chat_template_version() -> ChatTemplateVersions {
    ChatTemplateVersions::Single(
      "{% for message in messages %}{{ message.role }}: {{ message.content }}{% endfor %}"
        .to_string(),
    )
  }

  #[anyhow_trace]
  #[rstest]
  fn test_chat_template_from_valid_gguf(
    #[from(generate_test_data_chat_template)] _setup: &(),
  ) -> anyhow::Result<()> {
    let file = HubFileBuilder::fakemodel()
      .hf_cache(PathBuf::from("tests/data/gguf-chat-template"))
      .filename("valid_complete.gguf".to_string())
      .build()
      .unwrap();
    let chat_template = ChatTemplate::try_from(file)?;

    let expected = ChatTemplate::new(
      chat_template_version(),
      Some("<s>".to_string()),
      Some("</s>".to_string()),
    );
    assert_eq!(expected, chat_template);
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_chat_template_from_missing_template(
    #[from(generate_test_data_chat_template)] _setup: &(),
  ) -> anyhow::Result<()> {
    let file = HubFileBuilder::fakemodel()
      .hf_cache(PathBuf::from("tests/data/gguf-chat-template"))
      .filename("missing_chat_template.gguf".to_string())
      .build()
      .unwrap();

    let result = ChatTemplate::try_from(file);

    assert!(matches!(
      result.unwrap_err(),
      ChatTemplateError::EmbedChatTemplateNotFound
    ));

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_chat_template_from_missing_tokens(
    #[from(generate_test_data_chat_template)] _setup: &(),
  ) -> anyhow::Result<()> {
    let file = HubFileBuilder::fakemodel()
      .hf_cache(PathBuf::from("tests/data/gguf-chat-template"))
      .filename("missing_tokens.gguf".to_string())
      .build()
      .unwrap();
    let chat_template = ChatTemplate::try_from(file)?;

    let expected = ChatTemplate::new(chat_template_version(), None, None);
    assert_eq!(expected, chat_template);
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_chat_template_from_invalid_token_ids(
    #[from(generate_test_data_chat_template)] _setup: &(),
  ) -> anyhow::Result<()> {
    let file = HubFileBuilder::fakemodel()
      .hf_cache(PathBuf::from("tests/data/gguf-chat-template"))
      .filename("invalid_token_ids.gguf".to_string())
      .build()
      .unwrap();
    let chat_template = ChatTemplate::try_from(file)?;
    let expected = ChatTemplate::new(chat_template_version(), None, None);
    assert_eq!(expected, chat_template);
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_chat_template_from_empty_template(
    #[from(generate_test_data_chat_template)] _setup: &(),
  ) -> anyhow::Result<()> {
    let file = HubFileBuilder::fakemodel()
      .hf_cache(PathBuf::from("tests/data/gguf-chat-template"))
      .filename("empty_template.gguf".to_string())
      .build()
      .unwrap();
    let chat_template = ChatTemplate::try_from(file);

    assert!(chat_template.is_err());
    assert!(matches!(
      chat_template.unwrap_err(),
      ChatTemplateError::EmbedChatTemplateNotFound
    ));
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  fn test_chat_template_from_unicode_tokens(
    #[from(generate_test_data_chat_template)] _setup: &(),
  ) -> anyhow::Result<()> {
    let file = HubFileBuilder::fakemodel()
      .hf_cache(PathBuf::from("tests/data/gguf-chat-template"))
      .filename("unicode_tokens.gguf".to_string())
      .build()
      .unwrap();
    let chat_template = ChatTemplate::try_from(file)?;

    let expected = ChatTemplate::new(
      chat_template_version(),
      Some("<s>".to_string()),
      Some("</s>".to_string()),
    );
    assert_eq!(expected, chat_template);
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[ignore]
  fn test_chat_template_llama3() -> anyhow::Result<()> {
    let hf_cache = home_dir()
      .unwrap()
      .join(".cache")
      .join("huggingface")
      .join("hub");
    let file = HubFileBuilder::default()
      .hf_cache(hf_cache)
      .repo(Repo::try_from("unsloth/Llama-3.2-1B-Instruct-GGUF")?)
      .snapshot("952f952526368e61a6393b1a65684e0824fba86c".to_string())
      .filename("Llama-3.2-1B-Instruct-Q4_K_M.gguf".to_string())
      .size(Some(1000))
      .build()
      .unwrap();
    let chat_template = ChatTemplate::try_from(file)?;
    let expected = ChatTemplate::new(
      ChatTemplateVersions::Single(
        "{% for message in messages %}{{ message.role }}: {{ message.content }}{% endfor %}"
          .to_string(),
      ),
      Some("<s>".to_string()),
      Some("</s>".to_string()),
    );
    assert_eq!(expected, chat_template);
    Ok(())
  }
}
