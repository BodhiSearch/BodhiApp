use crate::{objs::RemoteModel, service::AppServiceFn};
use prettytable::{
  format::{self},
  row, Row, Table,
};

pub enum List {
  Local,
  Remote,
  Models,
}

impl List {
  pub fn new(remote: bool, models: bool) -> Self {
    match (remote, models) {
      (true, false) => List::Remote,
      (false, true) => List::Models,
      (false, false) => List::Local,
      (true, true) => unreachable!("both remote and models cannot be true"),
    }
  }

  pub fn execute(self, service: &dyn AppServiceFn) -> anyhow::Result<()> {
    match self {
      List::Local => self.list_local_model_alias(service)?,
      List::Remote => self.list_remote_models(service)?,
      List::Models => self.list_local_models(service)?,
    }
    Ok(())
  }

  fn list_local_model_alias(self, service: &dyn AppServiceFn) -> anyhow::Result<()> {
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

  fn list_local_models(self, service: &dyn AppServiceFn) -> anyhow::Result<()> {
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

  fn list_remote_models(self, service: &dyn AppServiceFn) -> anyhow::Result<()> {
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
