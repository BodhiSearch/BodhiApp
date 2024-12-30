use crate::ContextError;
use objs::{ChatTemplate, ChatTemplateType, TOKENIZER_CONFIG_JSON};
use services::HubService;
use std::sync::Arc;
use validator::Validate;

pub trait IntoChatTemplate {
  fn into_chat_template(
    &self,
    hub_service: Arc<dyn HubService>,
  ) -> Result<ChatTemplate, ContextError>;
}

impl IntoChatTemplate for ChatTemplateType {
  fn into_chat_template(
    &self,
    hub_service: Arc<dyn HubService>,
  ) -> Result<ChatTemplate, ContextError> {
    let repo = match self {
      ChatTemplateType::Id(id) => (*id).into(),
      ChatTemplateType::Repo(repo) => repo.clone(),
    };
    let file = hub_service.find_local_file(&repo, TOKENIZER_CONFIG_JSON, None)?;
    let chat_template: ChatTemplate = ChatTemplate::try_from(file)?;
    chat_template.validate()?;
    Ok(chat_template)
  }
}
