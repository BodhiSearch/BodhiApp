use crate::{CreateCommand, CreateCommandBuilder};
use objs::{ChatTemplate, ChatTemplateId, GptContextParams, OAIRequestParams, Repo};

impl CreateCommand {
  pub fn testalias() -> CreateCommand {
    CreateCommand::testalias_builder().build().unwrap()
  }

  pub fn testalias_builder() -> CreateCommandBuilder {
    CreateCommandBuilder::default()
      .alias("testalias:instruct".to_string())
      .repo(Repo::try_from("MyFactory/testalias-gguf").unwrap())
      .filename("testalias.Q8_0.gguf".to_string())
      .chat_template(ChatTemplate::Id(ChatTemplateId::Llama3))
      .family(Some("testalias".to_string()))
      .oai_request_params(OAIRequestParams::default())
      .context_params(GptContextParams::default())
      .to_owned()
  }
}
