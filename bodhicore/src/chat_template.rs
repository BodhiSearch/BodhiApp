use crate::hf_tokenizer::HubTokenizerConfig;
use minijinja::{Environment, ErrorKind, Template};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// https://github.com/huggingface/text-generation-inference/tree/main/router/src/infer.rs
/// Raise a exception (custom function) used in the chat templates
fn raise_exception(err_text: String) -> Result<String, minijinja::Error> {
  Err(minijinja::Error::new(ErrorKind::SyntaxError, err_text))
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub(crate) struct ChatTemplateInputs<'a> {
  messages: Value,
  bos_token: Option<&'a str>,
  eos_token: Option<&'a str>,
  add_generation_prompt: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct JinjaTemplate {
  template: Template<'static, 'static>,
  bos_token: Option<String>,
  eos_token: Option<String>,
}

impl PartialEq for JinjaTemplate {
  fn eq(&self, other: &Self) -> bool {
    self.bos_token == other.bos_token
      && self.eos_token == other.eos_token
      && self.template.source() == other.template.source()
  }
}

impl JinjaTemplate {
  fn new(
    template: String,
    bos_token: Option<String>,
    eos_token: Option<String>,
  ) -> anyhow::Result<Self> {
    let mut env = Box::new(Environment::new());
    let template_str = template.into_boxed_str();
    env.add_function("raise_exception", raise_exception);
    let template = Box::leak(env).template_from_str(Box::leak(template_str))?;
    Ok(Self {
      template,
      bos_token,
      eos_token,
    })
  }

  fn apply(&self, messages: Value) -> anyhow::Result<String> {
    let result = self.template.render(ChatTemplateInputs {
      messages,
      bos_token: self.bos_token.as_deref(),
      eos_token: self.eos_token.as_deref(),
      add_generation_prompt: true,
    })?;
    Ok(result)
  }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ChatTemplate {
  Empty,
  LlamaCpp { id: String },
  Jinja(JinjaTemplate),
}

impl ChatTemplate {
  pub(crate) fn new(config: HubTokenizerConfig) -> anyhow::Result<Self> {
    match config.chat_template {
      Some(chat_template) if chat_template.starts_with("llama.cpp:") => {
        let id = chat_template.replace("llama.cpp:", "");
        Ok(ChatTemplate::LlamaCpp { id })
      }
      Some(template) => {
        let template = template.replace(".strip()", " | trim");
        let jinja_template = JinjaTemplate::new(template, config.bos_token, config.eos_token)?;
        Ok(ChatTemplate::Jinja(jinja_template))
      }
      None => Ok(ChatTemplate::Empty),
    }
  }

  pub(crate) fn apply(&self, mut input: Value) -> anyhow::Result<(String, Value)> {
    match self {
      ChatTemplate::Empty => Ok(("".to_string(), input)),
      ChatTemplate::LlamaCpp { id } => Ok((id.to_string(), input)),
      ChatTemplate::Jinja(template) => {
        let input_obj = input.as_object_mut().unwrap();
        let messages = input_obj.remove("messages").unwrap();
        let prompt = template.apply(messages)?;
        input_obj.insert("prompt".to_string(), Value::String(prompt));
        Ok(("".to_string(), input))
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test_utils::LLAMA2_CHAT_TEMPLATE;
  use serde_json::json;

  #[test]
  fn test_chat_template_empty() -> anyhow::Result<()> {
    let config = HubTokenizerConfig::new(None, None, None);
    let template = ChatTemplate::new(config)?;
    assert_eq!(ChatTemplate::Empty, template);
    Ok(())
  }

  #[test]
  fn test_chat_template_llama_cpp() -> anyhow::Result<()> {
    let config = HubTokenizerConfig::new(Some("llama.cpp:gemma".to_string()), None, None);
    let template = ChatTemplate::new(config)?;
    assert_eq!(
      ChatTemplate::LlamaCpp {
        id: "gemma".to_string()
      },
      template
    );
    Ok(())
  }

  #[test]
  fn test_chat_template_jinja() -> anyhow::Result<()> {
    let chat_template = r#"{% for message in messages %}{{ message['role'] + ':' + message['content'] }}{% endfor %}"#.to_string();
    let config = HubTokenizerConfig::new(Some(chat_template.clone()), None, None);
    let result = ChatTemplate::new(config)?;
    let expected = ChatTemplate::Jinja(JinjaTemplate::new(chat_template, None, None)?);
    assert_eq!(expected, result);
    Ok(())
  }

  #[test]
  fn test_chat_template_apply_empty_does_not_change_messages() -> anyhow::Result<()> {
    let config = HubTokenizerConfig::default();
    let template = ChatTemplate::new(config)?;
    let input = json! {{"messsages": [{"role": "system", "content": "you are a helpful ai assistant."}, {"role": "user", "content": "what day comes after Monday?"}]}};
    let (chat_template, output) = template.apply(input.clone())?;
    assert_eq!(input, output);
    assert_eq!("", chat_template);
    Ok(())
  }

  #[test]
  fn test_chat_template_apply_llama_cpp_not_change_messages() -> anyhow::Result<()> {
    let config = HubTokenizerConfig::new(Some("llama.cpp:gemma".to_string()), None, None);
    let template = ChatTemplate::new(config)?;
    let input = json! {{"messsages": [{"role": "system", "content": "you are a helpful ai assistant."}, {"role": "user", "content": "what day comes after Monday?"}]}};
    let (chat_template, output) = template.apply(input.clone())?;
    assert_eq!("gemma", chat_template);
    assert_eq!(input, output);
    Ok(())
  }

  #[test]
  fn test_chat_template_apply_jinja_replaces_messages_with_prompt() -> anyhow::Result<()> {
    let config = HubTokenizerConfig::new(
      Some(LLAMA2_CHAT_TEMPLATE.to_string()),
      Some("<s>".to_string()),
      Some("</s>".to_string()),
    );
    let template = ChatTemplate::new(config)?;
    let input =
      json! {{ "messages": [{"role": "user", "content": "What day comes after Monday?"}] }};
    let (chat_template, output) = template.apply(input)?;
    assert_eq!("", chat_template);
    let expected = json! {{ "prompt": "<s>[INST] What day comes after Monday? [/INST]" }};
    assert_eq!(expected, output);
    Ok(())
  }

  #[test]
  fn test_chat_template_apply_jinja_raises_exception() -> anyhow::Result<()> {
    let config = HubTokenizerConfig::new(
      Some(LLAMA2_CHAT_TEMPLATE.to_string()),
      Some("<s>".to_string()),
      Some("</s>".to_string()),
    );
    let template = ChatTemplate::new(config)?;
    let input =
      json! {{ "messages": [{"role": "assistant", "content": "What day comes after Monday?"}] }};
    let result = template.apply(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().starts_with(
      "syntax error: Conversation roles must alternate user/assistant/user/assistant/..."
    ));
    Ok(())
  }
}
