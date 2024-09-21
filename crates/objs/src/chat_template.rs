use super::{ObjError, Repo};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
#[serde(untagged)]
pub enum ChatTemplate {
  Id(ChatTemplateId),
  Repo(Repo),
}

impl TryFrom<ChatTemplate> for Repo {
  type Error = ObjError;

  fn try_from(value: ChatTemplate) -> Result<Self, Self::Error> {
    let repo = match value {
      ChatTemplate::Id(id) => {
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
        Repo::try_from(repo)?
      }
      ChatTemplate::Repo(repo) => repo,
    };
    Ok(repo)
  }
}

impl Display for ChatTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ChatTemplate::Id(id) => write!(f, "{}", id),
      ChatTemplate::Repo(repo) => write!(f, "{}", repo),
    }
  }
}

#[cfg(test)]
mod test {
  use super::{ChatTemplate, ChatTemplateId, Repo};
  use rstest::rstest;
  use std::collections::HashSet;

  #[rstest]
  fn test_chat_template_id_partial_ord() {
    assert!(ChatTemplateId::Llama3.gt(&ChatTemplateId::Llama2));
    assert!(ChatTemplateId::Openchat.gt(&ChatTemplateId::CommandR));
  }

  #[rstest]
  #[case(
    ChatTemplate::Id(ChatTemplateId::Llama3),
    "meta-llama/Meta-Llama-3-8B-Instruct"
  )]
  #[rstest]
  #[case(
    ChatTemplate::Id(ChatTemplateId::Llama2),
    "meta-llama/Llama-2-13b-chat-hf"
  )]
  #[rstest]
  #[case(
    ChatTemplate::Id(ChatTemplateId::Llama2Legacy),
    "mistralai/Mixtral-8x7B-Instruct-v0.1"
  )]
  #[rstest]
  #[case(
    ChatTemplate::Id(ChatTemplateId::Phi3),
    "microsoft/Phi-3-mini-4k-instruct"
  )]
  #[rstest]
  #[case(ChatTemplate::Id(ChatTemplateId::Gemma), "google/gemma-7b-it")]
  #[rstest]
  #[case(
    ChatTemplate::Id(ChatTemplateId::Deepseek),
    "deepseek-ai/deepseek-llm-67b-chat"
  )]
  #[rstest]
  #[case(
    ChatTemplate::Id(ChatTemplateId::CommandR),
    "CohereForAI/c4ai-command-r-plus"
  )]
  #[rstest]
  #[case(
    ChatTemplate::Id(ChatTemplateId::Openchat),
    "openchat/openchat-3.6-8b-20240522"
  )]
  #[rstest]
  #[case(
    ChatTemplate::Repo(Repo::try_from("foo/bar").unwrap()),
    "foo/bar"
  )]
  fn test_chat_template_to_repo_for_chat_template(
    #[case] input: ChatTemplate,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    let repo: Repo = Repo::try_from(input)?;
    assert_eq!(expected, repo.to_string());
    Ok(())
  }

  #[test]
  fn test_chat_template_eq_and_hash() {
    let template1 = ChatTemplate::Id(ChatTemplateId::Llama3);
    let template2 = ChatTemplate::Id(ChatTemplateId::Llama3);
    let template3 = ChatTemplate::Id(ChatTemplateId::Llama2);

    assert_eq!(template1, template2);
    assert_ne!(template1, template3);

    let mut set = HashSet::new();
    set.insert(template1);
    assert!(set.contains(&template2));
    assert!(!set.contains(&template3));
  }
}
