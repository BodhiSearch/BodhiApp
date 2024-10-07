use crate::objs_ext::IntoRow;
use objs::RemoteModel;
use prettytable::{
  format::{self},
  row, Table,
};
use services::{AppService, DataServiceError};
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub enum ListCommand {
  Local,
  Remote,
  Models,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
pub enum ListCommandError {
  #[error(transparent)]
  DataServiceError(#[from] DataServiceError),
}

type Result<T> = std::result::Result<T, ListCommandError>;

impl ListCommand {
  #[allow(clippy::result_large_err)]
  pub fn execute(self, service: Arc<dyn AppService>) -> Result<()> {
    match self {
      ListCommand::Local => self.list_local_model_alias(service)?,
      ListCommand::Remote => self.list_remote_models(service)?,
      ListCommand::Models => self.list_local_models(service)?,
    }
    Ok(())
  }

  fn list_local_model_alias(self, service: Arc<dyn AppService>) -> Result<()> {
    let mut table = Table::new();
    table.add_row(row![
      "ALIAS",
      "FAMILY",
      "REPO",
      "FILENAME",
      "FEATURES",
      "CHAT TEMPLATE"
    ]);
    let aliases = service.data_service().list_aliases()?;
    for row in aliases.into_iter().map(IntoRow::into_row) {
      table.add_row(row);
    }
    table.set_format(format::FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    println!();
    println!("To run a model alias, run `bodhi run <ALIAS>`");
    Ok(())
  }

  fn list_local_models(self, service: Arc<dyn AppService>) -> Result<()> {
    let mut table = Table::new();
    table.add_row(row!["REPO", "FILENAME", "SNAPSHOT", "SIZE"]);
    let mut models = service.hub_service().list_local_models();
    models.sort_by(|a, b| a.repo.cmp(&b.repo));
    for row in models.into_iter().map(IntoRow::into_row) {
      table.add_row(row);
    }
    table.set_format(format::FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    Ok(())
  }

  fn list_remote_models(self, service: Arc<dyn AppService>) -> Result<()> {
    let models: Vec<RemoteModel> = service.data_service().list_remote_models()?;
    let mut table = Table::new();
    table.add_row(row![
      "ALIAS",
      "FAMILY",
      "REPO",
      "FILENAME",
      "FEATURES",
      "CHAT TEMPLATE"
    ]);
    for row in models.into_iter().map(IntoRow::into_row) {
      table.add_row(row);
    }
    table.set_format(format::FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    println!();
    println!("To download and configure the model alias, run `bodhi pull <ALIAS>`");
    Ok(())
  }
}
