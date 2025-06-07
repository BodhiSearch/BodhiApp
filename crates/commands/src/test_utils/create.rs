use crate::{CreateCommand, CreateCommandBuilder};
use objs::{GptContextParams, OAIRequestParams, Repo};

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
      // chat_template removed since llama.cpp now handles chat templates
      .oai_request_params(OAIRequestParams::default())
      .context_params(GptContextParams::default())
      .to_owned()
  }
}
