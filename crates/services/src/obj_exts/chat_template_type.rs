use crate::{HubService, HubServiceError};
use objs::{
  impl_error_from, AppError, ChatTemplate, ChatTemplateError, ChatTemplateType, HubFile,
  ObjValidationError, TOKENIZER_CONFIG_JSON,
};
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ObjExtsError {
  #[error(transparent)]
  HubService(#[from] HubServiceError),
  #[error(transparent)]
  ChatTemplate(#[from] ChatTemplateError),
  #[error(transparent)]
  ObjValidationError(#[from] ObjValidationError),
}

impl_error_from!(
  ::validator::ValidationErrors,
  ObjExtsError::ObjValidationError,
  objs::ObjValidationError
);

pub trait IntoChatTemplate {
  #[allow(clippy::wrong_self_convention)]
  fn into_chat_template(
    &self,
    hub_service: Arc<dyn HubService>,
  ) -> Result<ChatTemplate, ObjExtsError>;
}

impl IntoChatTemplate for ChatTemplateType {
  fn into_chat_template(
    &self,
    hub_service: Arc<dyn HubService>,
  ) -> Result<ChatTemplate, ObjExtsError> {
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

pub trait HubDownloadable {
  fn download(&self, hub_service: Arc<dyn HubService>) -> Result<HubFile, ObjExtsError>;
}

impl HubDownloadable for ChatTemplateType {
  fn download(&self, hub_service: Arc<dyn HubService>) -> Result<HubFile, ObjExtsError> {
    let repo = match self {
      ChatTemplateType::Id(id) => (*id).into(),
      ChatTemplateType::Repo(repo) => repo.clone(),
    };
    let hub_file = hub_service.download(&repo, TOKENIZER_CONFIG_JSON, None)?;
    Ok(hub_file)
  }
}
