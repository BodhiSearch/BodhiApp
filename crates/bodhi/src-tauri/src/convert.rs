use commands::{Command, CreateCommand, ListCommand, ManageAliasCommand, PullCommand};
use objs::{AppError, ChatTemplateType, ErrorType, Repo};
use server_app::{RunCommand, ServeCommand};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("convert_error_bad_request")]
#[error_meta(error_type = ErrorType::BadRequest, code = self.error_code)]
pub struct ConvertBadRequestError {
  input: String,
  output: String,
  error_code: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ConvertError {
  #[error("convert")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Convert { input: String, output: String },
  #[error(transparent)]
  ConvertBadRequest(#[from] ConvertBadRequestError),
}

pub fn build_list_command(remote: bool, models: bool) -> Result<ListCommand, ConvertError> {
  match (remote, models) {
    (true, false) => Ok(ListCommand::Remote),
    (false, true) => Ok(ListCommand::Models),
    (false, false) => Ok(ListCommand::Local),
    (true, true) => Err(ConvertBadRequestError::new(
      "list".to_string(),
      "ListCommand".to_string(),
      "convert_bad_request-list_command".to_string(),
    ))?,
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
      update,
      oai_request_params,
      context_params,
    } => {
      let chat_template = match (chat_template, tokenizer_config) {
        (Some(chat_template), None) => ChatTemplateType::Id(chat_template),
        (None, Some(tokenizer_config)) => ChatTemplateType::Repo(tokenizer_config),
        _ => {
          return Err(ConvertBadRequestError::new(
            "create".to_string(),
            "CreateCommand".to_string(),
            "convert_bad_request-create_command".to_string(),
          ))?;
        }
      };
      Ok(CreateCommand {
        alias,
        repo,
        filename,
        snapshot,
        chat_template,
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
  repo: Option<Repo>,
  filename: Option<String>,
  snapshot: Option<String>,
) -> Result<PullCommand, ConvertError> {
  match (alias, repo, filename) {
    (Some(alias), None, None) => Ok(PullCommand::ByAlias { alias }),
    (None, Some(repo), Some(filename)) => Ok(PullCommand::ByRepoFile {
      repo,
      filename,
      snapshot,
    }),
    _ => Err(ConvertBadRequestError::new(
      "pull".to_string(),
      "PullCommand".to_string(),
      "convert_bad_request-pull_command".to_string(),
    ))?,
  }
}

#[cfg(test)]
mod tests {
  use crate::convert::{
    build_create_command, build_list_command, build_manage_alias_command, build_pull_command,
    build_serve_command,
  };
  use commands::{Command, CreateCommand, ListCommand, ManageAliasCommand, PullCommand};
  use objs::{
    test_utils::{assert_error_message, setup_l10n},
    AppError, ChatTemplateId, ChatTemplateType, FluentLocalizationService, GptContextParams,
    OAIRequestParams, Repo,
  };
  use rstest::rstest;
  use server_app::ServeCommand;
  use std::sync::Arc;

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

  #[rstest]
  fn test_build_manage_alias_command_invalid(
    #[from(setup_l10n)] service: &Arc<FluentLocalizationService>,
  ) {
    let invalid_cmd = Command::List {
      remote: false,
      models: false,
    };
    let result = build_manage_alias_command(invalid_cmd);
    assert!(result.is_err());
    let app_error: &dyn AppError = &result.unwrap_err();
    assert_error_message(
      service,
      &app_error.code(),
      app_error.args(),
      "Command 'list' cannot be converted into command 'ManageAliasCommand'",
    );
  }

  #[rstest]
  #[case(
  Command::Create {
    alias: "testalias:instruct".to_string(),
    repo: Repo::testalias(),
    filename: Repo::testalias_model_q8(),
    snapshot: Some("main".to_string()),
    chat_template: Some(ChatTemplateId::Llama3),
    tokenizer_config: None,
    update: true,
    oai_request_params: OAIRequestParams::default(),
    context_params: GptContextParams::default(),
  },
  CreateCommand {
    alias: "testalias:instruct".to_string(),
    repo: Repo::testalias(),
    filename: Repo::testalias_model_q8(),
    snapshot: Some("main".to_string()),
    chat_template: ChatTemplateType::Id(ChatTemplateId::Llama3),
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
      repo: Repo::testalias(),
      filename: Repo::testalias_model_q8(),
      snapshot: None,
      chat_template: None,
      tokenizer_config: None,
      update: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    },
    "Command 'create' cannot be converted into command 'CreateCommand', one of chat_template and tokenizer_config must be provided"
  )]
  #[anyhow_trace::anyhow_trace]
  fn test_create_try_from_invalid(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[case] input: Command,
    #[case] message: String,
  ) -> anyhow::Result<()> {
    let actual = build_create_command(input);
    assert!(actual.is_err());
    let api_error: &dyn AppError = &actual.unwrap_err();
    assert_error_message(
      _localization_service,
      &api_error.code(),
      api_error.args(),
      &message,
    );
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
  #[case(true, true, "Command 'list' cannot be converted into command 'ListCommand', cannot initialize list command with invalid state. --remote: true, --models: true")]
  fn test_list_invalid_try_from(
    #[from(setup_l10n)] service: &Arc<FluentLocalizationService>,
    #[case] remote: bool,
    #[case] models: bool,
    #[case] expected: String,
  ) {
    let result = build_list_command(remote, models);
    assert!(result.is_err());
    let app_error: &dyn AppError = &result.unwrap_err();
    assert_error_message(service, &app_error.code(), app_error.args(), &expected);
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
  #[case((None, Some(Repo::llama3()), Some(Repo::LLAMA3_Q8.to_string()), None) , PullCommand::ByRepoFile { repo: Repo::llama3(),
    filename: Repo::LLAMA3_Q8.to_string(),
    snapshot: None,
  })]
  #[case((None, Some(Repo::llama3()), Some(Repo::LLAMA3_Q8.to_string()), Some("main".to_string())) , PullCommand::ByRepoFile { repo: Repo::llama3(),
    filename: Repo::LLAMA3_Q8.to_string(),
    snapshot: Some("main".to_string()),
  })]
  #[case((None, Some(Repo::llama3()), Some(Repo::LLAMA3_Q8.to_string()), Some("b32046744d93031a26c8e925de2c8932c305f7b9".to_string())) , PullCommand::ByRepoFile { repo: Repo::llama3(),
    filename: Repo::LLAMA3_Q8.to_string(),
    snapshot: Some("b32046744d93031a26c8e925de2c8932c305f7b9".to_string()),
  })]
  fn test_pull_command_try_from_command(
    #[case] input: (Option<String>, Option<Repo>, Option<String>, Option<String>),
    #[case] expected: PullCommand,
  ) -> anyhow::Result<()> {
    let pull_command: PullCommand = build_pull_command(input.0, input.1, input.2, input.3)?;
    assert_eq!(expected, pull_command);
    Ok(())
  }
}
