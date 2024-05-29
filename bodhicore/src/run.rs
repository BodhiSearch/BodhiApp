use crate::{error::AppError, service::DataService, AppService, Command};

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
  pub fn execute(self, service: &AppService) -> crate::error::Result<()> {
    match self {
      RunCommand::WithAlias { alias } => {
        let Some(model) = service.find_alias(&alias) else {
          return Err(AppError::AliasNotFound(alias));
        };
        // launch_interactive(alias)?;
      }
    };
    Ok(())
  }
}
