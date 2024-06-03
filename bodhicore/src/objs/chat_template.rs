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
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ChatTemplateId {
  Llama3,
  Llama2,
  Llama2Legacy,
  Phi3,
  Gemma,
  Deepseek,
  CommandR,
  Openchat,
}

impl PartialOrd for ChatTemplateId {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.as_ref().partial_cmp(other.as_ref())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
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
        };
        Repo::try_new(repo.to_string())?
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

  #[rstest]
  fn test_chat_template_id_partial_ord() -> anyhow::Result<()> {
    assert!(ChatTemplateId::Llama3.gt(&ChatTemplateId::Llama2));
    assert!(ChatTemplateId::Openchat.gt(&ChatTemplateId::CommandR));
    Ok(())
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
    ChatTemplate::Repo(Repo::try_new("foo/bar".to_string()).unwrap()),
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
}
