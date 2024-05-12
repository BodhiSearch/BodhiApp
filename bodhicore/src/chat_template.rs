use crate::hf_tokenizer::HubTokenizerConfig;
use minijinja::{Environment, ErrorKind, Template};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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

  pub(crate) fn apply(&self, messages: Value) -> anyhow::Result<(String, Value)> {
    match self {
      ChatTemplate::Empty => Ok(("".to_string(), json! {{"messages": messages}})),
      ChatTemplate::LlamaCpp { id } => Ok((id.to_string(), json! {{"messages": messages}})),
      ChatTemplate::Jinja(template) => {
        let prompt = template.apply(messages)?;
        Ok(("".to_string(), json! {{"prompt": prompt}}))
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_chat_template_empty() -> anyhow::Result<()> {
    let template = ChatTemplate::new(HubTokenizerConfig::from_json_str("{}")?)?;
    assert_eq!(ChatTemplate::Empty, template);
    Ok(())
  }

  #[test]
  fn test_chat_template_llama_cpp() -> anyhow::Result<()> {
    let template = ChatTemplate::new(HubTokenizerConfig::from_json_str(
      r#"{"chat_template": "llama.cpp:gemma"}"#,
    )?)?;
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
    let template = r#"{% for message in messages %}{{ message['role'] + ':' + message['content'] }}{% endfor %}"#.to_string();
    let result = ChatTemplate::new(HubTokenizerConfig::from_json_str(&format!(
      "{{\"chat_template\": \"{template}\"}}"
    ))?)?;
    let expected = ChatTemplate::Jinja(JinjaTemplate::new(template, None, None)?);
    assert_eq!(expected, result);
    Ok(())
  }
}
