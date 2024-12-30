use crate::Repo;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
#[serde(untagged)]
pub enum ChatTemplateType {
  Id(ChatTemplateId),
  Repo(Repo),
}

impl Display for ChatTemplateType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ChatTemplateType::Id(id) => write!(f, "{}", id),
      ChatTemplateType::Repo(repo) => write!(f, "{}", repo),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{ChatTemplateId, ChatTemplateType, Repo};
  use rstest::rstest;
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
}
