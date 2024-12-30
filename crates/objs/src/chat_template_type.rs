use crate::Repo;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::{AsRefStr, EnumIter};

#[derive(
  clap::ValueEnum,
  Clone,
  Debug,
  Serialize,
  Deserialize,
  PartialEq,
  EnumIter,
  AsRefStr,
  strum::Display,
  Eq,
  Hash,
  Copy,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
// TODO: have other, so can take input from command line as pre-defined chat template ids, or the repo
// easier for user to provide input as --chat-template llama3 or --chat-template meta-llama/llama3
pub enum ChatTemplateId {
  Llama3,
  Llama2,
  Llama2Legacy,
  Phi3,
  Gemma,
  Deepseek,
  CommandR,
  Openchat,
  Tinyllama,
}

impl PartialOrd for ChatTemplateId {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.as_ref().partial_cmp(other.as_ref())
  }
}

impl From<ChatTemplateId> for Repo {
  fn from(id: ChatTemplateId) -> Self {
    let repo = match id {
      ChatTemplateId::Llama3 => "meta-llama/Meta-Llama-3-8B-Instruct",
      ChatTemplateId::Llama2 => "meta-llama/Llama-2-13b-chat-hf",
      ChatTemplateId::Llama2Legacy => "mistralai/Mixtral-8x7B-Instruct-v0.1",
      ChatTemplateId::Phi3 => "microsoft/Phi-3-mini-4k-instruct",
      ChatTemplateId::Gemma => "google/gemma-7b-it",
      ChatTemplateId::Deepseek => "deepseek-ai/deepseek-llm-67b-chat",
      ChatTemplateId::CommandR => "CohereForAI/c4ai-command-r-plus",
      ChatTemplateId::Openchat => "openchat/openchat-3.6-8b-20240522",
      ChatTemplateId::Tinyllama => "TinyLlama/TinyLlama-1.1B-Chat-v1.0",
    };
    Repo::try_from(repo).unwrap()
  }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub enum ChatTemplateType {
  Embedded,
  Id(ChatTemplateId),
  Repo(Repo),
}

impl<'de> Deserialize<'de> for ChatTemplateType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    if s == "embedded" {
      return Ok(ChatTemplateType::Embedded);
    }

    // Try parsing as ChatTemplateId first
    if let Ok(id) = serde_json::from_value(serde_json::Value::String(s.clone())) {
      return Ok(ChatTemplateType::Id(id));
    }

    // Finally try parsing as Repo
    match Repo::try_from(s) {
      Ok(repo) => Ok(ChatTemplateType::Repo(repo)),
      Err(e) => Err(serde::de::Error::custom(format!(
        "Invalid chat template: {}",
        e
      ))),
    }
  }
}

impl Serialize for ChatTemplateType {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      ChatTemplateType::Embedded => serializer.serialize_str("embedded"),
      ChatTemplateType::Id(id) => id.serialize(serializer),
      ChatTemplateType::Repo(repo) => repo.serialize(serializer),
    }
  }
}

impl Display for ChatTemplateType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ChatTemplateType::Id(id) => write!(f, "{}", id),
      ChatTemplateType::Repo(repo) => write!(f, "{}", repo),
      ChatTemplateType::Embedded => write!(f, "embedded"),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{ChatTemplateId, ChatTemplateType, Repo};
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use std::collections::HashSet;

  #[rstest]
  fn test_chat_template_type_id_partial_ord() {
    assert!(ChatTemplateId::Llama3.gt(&ChatTemplateId::Llama2));
    assert!(ChatTemplateId::Openchat.gt(&ChatTemplateId::CommandR));
  }

  #[rstest]
  #[case(ChatTemplateId::Llama3, "meta-llama/Meta-Llama-3-8B-Instruct")]
  #[case(ChatTemplateId::Llama2, "meta-llama/Llama-2-13b-chat-hf")]
  #[case(ChatTemplateId::Llama2Legacy, "mistralai/Mixtral-8x7B-Instruct-v0.1")]
  #[case(ChatTemplateId::Phi3, "microsoft/Phi-3-mini-4k-instruct")]
  #[case(ChatTemplateId::Gemma, "google/gemma-7b-it")]
  #[case(ChatTemplateId::Deepseek, "deepseek-ai/deepseek-llm-67b-chat")]
  #[case(ChatTemplateId::CommandR, "CohereForAI/c4ai-command-r-plus")]
  #[case(ChatTemplateId::Openchat, "openchat/openchat-3.6-8b-20240522")]
  fn test_chat_template_type_to_repo_for_chat_template_with_id(
    #[case] id: ChatTemplateId,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    assert_eq!(Repo::try_from(expected)?, Repo::from(id));
    Ok(())
  }

  #[test]
  fn test_chat_template_type_eq_and_hash() {
    let template1 = ChatTemplateType::Id(ChatTemplateId::Llama3);
    let template2 = ChatTemplateType::Id(ChatTemplateId::Llama3);
    let template3 = ChatTemplateType::Id(ChatTemplateId::Llama2);

    assert_eq!(template1, template2);
    assert_ne!(template1, template3);

    let mut set = HashSet::new();
    set.insert(template1);
    assert!(set.contains(&template2));
    assert!(!set.contains(&template3));
  }

  #[rstest]
  #[case("llama3", ChatTemplateType::Id(ChatTemplateId::Llama3))]
  #[case(
    "meta-llama/Meta-Llama-3-8B-Instruct",
    ChatTemplateType::Repo(Repo::llama3())
  )]
  #[case("embedded", ChatTemplateType::Embedded)]
  fn test_chat_template_deser(
    #[case] input: &str,
    #[case] expected: ChatTemplateType,
  ) -> anyhow::Result<()> {
    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Test {
      template: ChatTemplateType,
    }
    let input = format!("{{\"template\": \"{input}\"}}");
    let deser: Test = serde_json::from_str(&input)?;
    assert_eq!(deser, Test { template: expected });
    Ok(())
  }

  #[rstest]
  #[case(ChatTemplateType::Id(ChatTemplateId::Llama3), "\"llama3\"")]
  #[case(
    ChatTemplateType::Repo(Repo::llama3()),
    "\"meta-llama/Meta-Llama-3-8B-Instruct\""
  )]
  #[case(ChatTemplateType::Embedded, "\"embedded\"")]
  fn test_chat_template_ser(
    #[case] input: ChatTemplateType,
    #[case] expected: &str,
  ) -> anyhow::Result<()> {
    #[derive(Debug, Serialize)]
    struct Test {
      template: ChatTemplateType,
    }
    let ser = serde_json::to_string(&Test { template: input })?;
    assert_eq!(ser, format!("{{\"template\":{}}}", expected));
    Ok(())
  }
}
