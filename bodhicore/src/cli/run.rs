use crate::{error::AppError, interactive::launch_interactive, service::AppServiceFn, Command};

pub enum RunCommand {
  WithAlias { alias: String },
}

impl TryFrom<Command> for RunCommand {
  type Error = AppError;

  fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
    match value {
      Command::Run { alias } => Ok(RunCommand::WithAlias { alias }),
      cmd => Err(AppError::ConvertCommand(cmd, "run".to_string())),
    }
  }
}

impl RunCommand {
  #[allow(clippy::result_large_err)]
  pub fn execute(self, service: &dyn AppServiceFn) -> crate::error::Result<()> {
    match self {
      RunCommand::WithAlias { alias } => {
        let Some(alias) = service.find_alias(&alias) else {
          return Err(AppError::AliasNotFound(alias));
        };
        // TODO: after removing anyhow::Error from launch_interactive, replace with direct call
        launch_interactive(alias, service)?;
        Ok(())
      }
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{test_utils::MockAppService, RunCommand};
  use mockall::predicate::eq;
  use rstest::rstest;

  #[rstest]
  fn test_run_with_alias_return_error_if_alias_not_found() -> anyhow::Result<()> {
    let run_command = RunCommand::WithAlias {
      alias: "testalias".to_string(),
    };
    let mut mock = MockAppService::default();
    mock
      .expect_find_alias()
      .with(eq("testalias".to_string()))
      .return_once(|_| None);
    let result = run_command.execute(&mock);
    assert!(result.is_err());
    assert_eq!(
      r#"model alias 'testalias' not found in pre-configured model aliases.
Run `bodhi list -r` to see list of pre-configured model aliases
"#,
      result.unwrap_err().to_string()
    );
    Ok(())
  }
}
