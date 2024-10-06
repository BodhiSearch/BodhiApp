use commands::{Command, CreateCommand, ListCommand, ManageAliasCommand, PullCommand};
use objs::{ChatTemplate, Repo};
use server::{RunCommand, ServeCommand};

#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
  #[error("Command '{input}' cannot be converted into command '{output}'")]
  Convert { input: String, output: String },
  #[error("Command '{input}' cannot be converted into command '{output}', error: '{error}'")]
  BadRequest {
    input: String,
    output: String,
    error: String,
  },
}

pub fn build_list_command(remote: bool, models: bool) -> Result<ListCommand, ConvertError> {
  match (remote, models) {
    (true, false) => Ok(ListCommand::Remote),
    (false, true) => Ok(ListCommand::Models),
    (false, false) => Ok(ListCommand::Local),
    (true, true) => Err(ConvertError::BadRequest {
      input: "list".to_string(),
      output: "ListCommand".to_string(),
      error: "cannot initialize list command with invalid state. --remote: true, --models: true"
        .to_string(),
    }),
  }
}

pub fn build_run_command(alias: String) -> Result<RunCommand, ConvertError> {
  Ok(RunCommand::WithAlias { alias })
}

pub fn build_serve_command(host: String, port: u16) -> Result<ServeCommand, ConvertError> {
  Ok(ServeCommand::ByParams { host, port })
}

#[allow(clippy::too_many_arguments)]
pub fn build_create_command(command: Command) -> Result<CreateCommand, ConvertError> {
  match command {
    Command::Create {
      alias,
      repo,
      filename,
      snapshot,
      chat_template,
      tokenizer_config,
      family,
      update,
      oai_request_params,
      context_params,
    } => {
      let chat_template = match (chat_template, tokenizer_config) {
        (Some(chat_template), None) => ChatTemplate::Id(chat_template),
        (None, Some(tokenizer_config)) => {
          let repo = Repo::try_from(tokenizer_config).map_err(|err| ConvertError::BadRequest {
            input: "create".to_string(),
            output: "CreateCommand".to_string(),
            error: format!("tokenizer_config repo {err}"),
          })?;
          ChatTemplate::Repo(repo)
        }
        _ => {
          return Err(ConvertError::BadRequest {
            input: "create".to_string(),
            output: "CreateCommand".to_string(),
            error: "one of chat_template and tokenizer_config must be provided".to_string(),
          })
        }
      };
      let repo = Repo::try_from(repo).map_err(|err| ConvertError::BadRequest {
        input: "create".to_string(),
        output: "CreateCommand".to_string(),
        error: format!("model repo {err}"),
      })?;
      Ok(CreateCommand {
        alias,
        repo,
        filename,
        snapshot,
        chat_template,
        family,
        auto_download: true,
        update,
        oai_request_params,
        context_params,
      })
    }
    _ => Err(ConvertError::Convert {
      input: command.to_string(),
      output: "CreateCommand".to_string(),
    }),
  }
}

pub fn build_manage_alias_command(command: Command) -> Result<ManageAliasCommand, ConvertError> {
  match command {
    Command::Show { alias } => Ok(ManageAliasCommand::Show { alias }),
    Command::Cp { alias, new_alias } => Ok(ManageAliasCommand::Copy { alias, new_alias }),
    Command::Edit { alias } => Ok(ManageAliasCommand::Edit { alias }),
    Command::Rm { alias } => Ok(ManageAliasCommand::Delete { alias }),
    _ => Err(ConvertError::Convert {
      input: command.to_string(),
      output: "ManageAliasCommand".to_string(),
    }),
  }
}

pub fn build_pull_command(
  alias: Option<String>,
  repo: Option<String>,
  filename: Option<String>,
  snapshot: Option<String>,
) -> Result<PullCommand, ConvertError> {
  match (alias, repo, filename) {
    (Some(alias), None, None) => Ok(PullCommand::ByAlias { alias }),
    (None, Some(repo), Some(filename)) => {
      let repo = Repo::try_from(repo).map_err(|err| ConvertError::BadRequest {
        input: "pull".to_string(),
        output: "PullCommand".to_string(),
        error: format!("invalid repo {err}"),
      })?;
      Ok(PullCommand::ByRepoFile {
        repo,
        filename,
        snapshot,
      })
    }
    _ => Err(ConvertError::BadRequest {
      input: "pull".to_string(),
      output: "PullCommand".to_string(),
      error: "invalid pull command".to_string(),
    }),
  }
}

#[cfg(test)]
mod tests {
  use crate::convert::{
    build_create_command, build_list_command, build_manage_alias_command, build_pull_command,
    build_serve_command,
  };
  use commands::{Command, CreateCommand, ListCommand, ManageAliasCommand, PullCommand};
  use objs::{ChatTemplate, ChatTemplateId, GptContextParams, OAIRequestParams, Repo};
  use rstest::rstest;
  use server::ServeCommand;

  #[rstest]
  #[case::show(
      Command::Show { alias: "test_alias".to_string() },
      ManageAliasCommand::Show { alias: "test_alias".to_string() }
  )]
  #[case::copy(
      Command::Cp {
          alias: "old_alias".to_string(), 
          new_alias: "new_alias".to_string() 
      },
      ManageAliasCommand::Copy {
          alias: "old_alias".to_string(), 
          new_alias: "new_alias".to_string() 
      }
  )]
  #[case::edit(
      Command::Edit { alias: "edit_alias".to_string() },
      ManageAliasCommand::Edit { alias: "edit_alias".to_string() }
  )]
  #[case::delete(
      Command::Rm { alias: "delete_alias".to_string() },
      ManageAliasCommand::Delete { alias: "delete_alias".to_string() }
  )]
  fn test_build_manage_alias_command(
    #[case] input: Command,
    #[case] expected: ManageAliasCommand,
  ) -> anyhow::Result<()> {
    let result = build_manage_alias_command(input)?;
    assert_eq!(expected, result);
    Ok(())
  }

  #[test]
  fn test_build_manage_alias_command_invalid() {
    let invalid_cmd = Command::List {
      remote: false,
      models: false,
    };
    let result = build_manage_alias_command(invalid_cmd);
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().to_string(),
      "Command 'list' cannot be converted into command 'ManageAliasCommand'"
    );
  }

  #[rstest]
  #[case(
  Command::Create {
    alias: "testalias:instruct".to_string(),
    repo: "MyFactory/testalias-gguf".to_string(),
    filename: "testalias.Q8_0.gguf".to_string(),
    snapshot: Some("main".to_string()),
    chat_template: Some(ChatTemplateId::Llama3),
    tokenizer_config: None,
    family: Some("testalias".to_string()),
    update: true,
    oai_request_params: OAIRequestParams::default(),
    context_params: GptContextParams::default(),
  },
  CreateCommand {
    alias: "testalias:instruct".to_string(),
    repo: Repo::try_from("MyFactory/testalias-gguf".to_string())?,
    filename: "testalias.Q8_0.gguf".to_string(),
    snapshot: Some("main".to_string()),
    chat_template: ChatTemplate::Id(ChatTemplateId::Llama3),
    family: Some("testalias".to_string()),
    auto_download: true,
    update: true,
    oai_request_params: OAIRequestParams::default(),
    context_params: GptContextParams::default(),
  })]
  fn test_create_try_from_valid(
    #[case] input: Command,
    #[case] expected: CreateCommand,
  ) -> anyhow::Result<()> {
    let command = build_create_command(input)?;
    assert_eq!(expected, command);
    Ok(())
  }

  #[rstest]
  #[case(
    Command::App {ui: false},
    "Command 'app' cannot be converted into command 'CreateCommand'"
  )]
  #[case(
    Command::Create {
      alias: "test".to_string(),
      repo: "valid/repo".to_string(),
      filename: "model.gguf".to_string(),
      snapshot: None,
      chat_template: None,
      tokenizer_config: Some("invalid-chat/repo/pattern".to_string()),
      family: None,
      update: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    },
    "Command 'create' cannot be converted into command 'CreateCommand', error: 'tokenizer_config repo Validation failed: value: does not match the huggingface repo pattern 'username/repo''"
  )]
  #[case(
    Command::Create {
      alias: "test".to_string(),
      repo: "invalid-repo".to_string(),
      filename: "model.gguf".to_string(),
      snapshot: None,
      chat_template: Some(ChatTemplateId::Llama3),
      tokenizer_config: None,
      family: None,
      update: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    },
    "Command 'create' cannot be converted into command 'CreateCommand', error: 'model repo Validation failed: value: does not match the huggingface repo pattern 'username/repo''"
  )]
  #[case(
    Command::Create {
      alias: "test".to_string(),
      repo: "invalid-repo".to_string(),
      filename: "model.gguf".to_string(),
      snapshot: None,
      chat_template: None,
      tokenizer_config: None,
      family: None,
      update: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    },
    "Command 'create' cannot be converted into command 'CreateCommand', error: 'one of chat_template and tokenizer_config must be provided'"
  )]
  #[anyhow_trace::anyhow_trace]
  fn test_create_try_from_invalid(
    #[case] input: Command,
    #[case] message: String,
  ) -> anyhow::Result<()> {
    let actual = build_create_command(input);
    assert!(actual.is_err());
    assert_eq!(message, actual.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[case("localhost", 1135, ServeCommand::ByParams {
    host: "localhost".to_string(),
    port: 1135,
  })]
  fn test_build_serve_command(
    #[case] host: String,
    #[case] port: u16,
    #[case] expected: ServeCommand,
  ) -> anyhow::Result<()> {
    let result = build_serve_command(host, port)?;
    assert_eq!(expected, result);
    Ok(())
  }

  #[rstest]
  #[case(true, true, "Command 'list' cannot be converted into command 'ListCommand', error: 'cannot initialize list command with invalid state. --remote: true, --models: true'")]
  fn test_list_invalid_try_from(
    #[case] remote: bool,
    #[case] models: bool,
    #[case] expected: String,
  ) {
    let result = build_list_command(remote, models);
    assert!(result.is_err());
    assert_eq!(expected, result.unwrap_err().to_string());
  }

  #[rstest]
  #[case(false, false, ListCommand::Local)]
  #[case(true, false, ListCommand::Remote)]
  #[case(false, true, ListCommand::Models)]
  fn test_list_valid_try_from(
    #[case] remote: bool,
    #[case] models: bool,
    #[case] expected: ListCommand,
  ) -> anyhow::Result<()> {
    let result = build_list_command(remote, models)?;
    assert_eq!(expected, result);
    Ok(())
  }

  #[rstest]
  #[case((Some("llama3:instruct".to_string()), None, None, None) , PullCommand::ByAlias { alias: "llama3:instruct".to_string(), })]
  #[case((None, Some("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string()), Some("Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string()), None) , PullCommand::ByRepoFile { repo: Repo::try_from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF").unwrap(),
    filename: "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
    snapshot: None,
  })]
  #[case((None, Some("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string()), Some("Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string()), Some("main".to_string())) , PullCommand::ByRepoFile { repo: Repo::try_from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF").unwrap(),
    filename: "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
    snapshot: Some("main".to_string()),
  })]
  #[case((None, Some("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string()), Some("Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string()), Some("191239b3e26b2882fb562ffccdd1cf0f65402adb".to_string())) , PullCommand::ByRepoFile { repo: Repo::try_from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF").unwrap(),
    filename: "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
    snapshot: Some("191239b3e26b2882fb562ffccdd1cf0f65402adb".to_string()),
  })]
  fn test_pull_command_try_from_command(
    #[case] input: (
      Option<String>,
      Option<String>,
      Option<String>,
      Option<String>,
    ),
    #[case] expected: PullCommand,
  ) -> anyhow::Result<()> {
    let pull_command: PullCommand = build_pull_command(input.0, input.1, input.2, input.3)?;
    assert_eq!(expected, pull_command);
    Ok(())
  }
}
