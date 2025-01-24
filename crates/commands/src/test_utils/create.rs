use crate::{CreateCommand, CreateCommandBuilder};
use objs::{ChatTemplateId, ChatTemplateType, GptContextParams, OAIRequestParams, Repo};

impl CreateCommand {
  pub fn testalias() -> CreateCommand {
    CreateCommandBuilder::testalias().build().unwrap()
  }
}

impl CreateCommandBuilder {
  pub fn testalias() -> CreateCommandBuilder {
    CreateCommandBuilder::default()
      .alias("testalias:instruct".to_string())
      .repo(Repo::testalias())
      .filename(Repo::testalias_model_q8())
      .snapshot(None)
      .chat_template(ChatTemplateType::Id(ChatTemplateId::Llama3))
      .oai_request_params(OAIRequestParams::default())
      .context_params(GptContextParams::default())
      .to_owned()
  }
}
