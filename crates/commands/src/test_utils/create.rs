use crate::{CreateCommand, CreateCommandBuilder};
use objs::{ChatTemplate, ChatTemplateId, GptContextParams, OAIRequestParams, Repo};

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
      .filename(Repo::testalias_filename())
      .snapshot(None)
      .chat_template(ChatTemplate::Id(ChatTemplateId::Llama3))
      .family(Some("testalias".to_string()))
      .oai_request_params(OAIRequestParams::default())
      .context_params(GptContextParams::default())
      .to_owned()
  }
}
