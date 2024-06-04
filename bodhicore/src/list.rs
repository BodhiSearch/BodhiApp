use crate::{error::AppError, objs::RemoteModel, service::AppServiceFn, Command};
use prettytable::{
  format::{self},
  row, Row, Table,
};

#[derive(Debug, PartialEq)]
pub enum ListCommand {
  Local,
  Remote,
  Models,
}

impl TryFrom<Command> for ListCommand {
  type Error = AppError;

  fn try_from(value: Command) -> Result<Self, Self::Error> {
    match value {
      Command::List { remote, models } => match (remote, models) {
        (true, false) => Ok(ListCommand::Remote),
        (false, true) => Ok(ListCommand::Models),
        (false, false) => Ok(ListCommand::Local),
        (true, true) => Err(AppError::BadRequest(format!(
          "cannot initialize list command with invalid state. --remote: {remote}, --models: {models}"
        ))),
      },
      cmd => Err(AppError::ConvertCommand(cmd, "list".to_string())),
    }
  }
}

impl ListCommand {
  #[allow(clippy::result_large_err)]
  pub fn execute(self, service: &dyn AppServiceFn) -> crate::error::Result<()> {
    match self {
      ListCommand::Local => self.list_local_model_alias(service)?,
      ListCommand::Remote => self.list_remote_models(service)?,
      ListCommand::Models => self.list_local_models(service)?,
    }
    Ok(())
  }

  fn list_local_model_alias(self, service: &dyn AppServiceFn) -> crate::error::Result<()> {
    let mut table = Table::new();
    table.add_row(row![
      "ALIAS",
      "FAMILY",
      "REPO",
      "FILENAME",
      "FEATURES",
      "CHAT TEMPLATE"
    ]);
    let aliases = service.list_aliases()?;
    for row in aliases.into_iter().map(Row::from) {
      table.add_row(row);
    }
    table.set_format(format::FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    println!();
    println!("To run a model alias, run `bodhi run <ALIAS>`");
    Ok(())
  }

  fn list_local_models(self, service: &dyn AppServiceFn) -> crate::error::Result<()> {
    let mut table = Table::new();
    table.add_row(row!["FILENAME", "REPO", "SNAPSHOT", "SIZE"]);
    let models = service.list_local_models();
    for row in models.into_iter().map(Row::from) {
      table.add_row(row);
    }
    table.set_format(format::FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    Ok(())
  }

  fn list_remote_models(self, service: &dyn AppServiceFn) -> crate::error::Result<()> {
    let models: Vec<RemoteModel> = service.list_remote_models()?;
    let mut table = Table::new();
    table.add_row(row![
      "ALIAS",
      "FAMILY",
      "REPO",
      "FILENAME",
      "FEATURES",
      "CHAT TEMPLATE"
    ]);
    for row in models.into_iter().map(Row::from) {
      table.add_row(row);
    }
    table.set_format(format::FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    println!();
    println!("To download and configure the model alias, run `bodhi pull <ALIAS>`");
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::{Command, ListCommand};
  use rstest::rstest;

  #[rstest]
  #[case(Command::App {}, "Command 'app' cannot be converted into command 'list'")]
  #[case(Command::List {remote: true, models: true}, "cannot initialize list command with invalid state. --remote: true, --models: true")]
  fn test_list_invalid_try_from(#[case] input: Command, #[case] expected: String) {
    let result = ListCommand::try_from(input);
    assert!(result.is_err());
    assert_eq!(expected, result.unwrap_err().to_string());
  }

  #[rstest]
  #[case(Command::List {
    remote: false,
    models: false,
  }, ListCommand::Local)]
  #[case(Command::List {
    remote: true,
    models: false,
  }, ListCommand::Remote)]
  #[case(Command::List {
    remote: false,
    models: true,
  }, ListCommand::Models)]
  fn test_list_valid_try_from(
    #[case] input: Command,
    #[case] expected: ListCommand,
  ) -> anyhow::Result<()> {
    let result = ListCommand::try_from(input)?;
    assert_eq!(expected, result);
    Ok(())
  }
}
