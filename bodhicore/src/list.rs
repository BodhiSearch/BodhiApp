use crate::{
  objs::{LocalModelFile, RemoteModel},
  service::AppServiceFn,
};
use prettytable::{
  format::{self},
  row, Cell, Row, Table,
};

impl From<LocalModelFile> for Row {
  fn from(model: LocalModelFile) -> Self {
    let LocalModelFile {
      repo,
      filename,
      snapshot,
      size,
      ..
    } = model;
    let human_size = size
      .map(|size| format!("{:.2} GB", size as f64 / 2_f64.powf(30.0)))
      .unwrap_or_else(|| String::from("Unknown"));
    Row::from(vec![
      Cell::new(&filename),
      Cell::new(&repo),
      Cell::new(&snapshot[..8]),
      Cell::new(&human_size),
    ])
  }
}

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

#[cfg(test)]
mod test {
  use crate::{objs::LocalModelFile, Repo};
  use prettytable::{Cell, Row};
  use std::path::PathBuf;

  #[test]
  fn test_list_model_item_to_row() -> anyhow::Result<()> {
    let model = LocalModelFile::new(
      PathBuf::from("."),
      Repo::try_new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string())?,
      "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      "1234567890".to_string(),
      Some(1024 * 1024 * 1024 * 10),
    );
    let row = model.into();
    let expected = Row::from(vec![
      Cell::new("Meta-Llama-3-8B-Instruct.Q8_0.gguf"),
      Cell::new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF"),
      Cell::new("12345678"),
      Cell::new("10.00 GB"),
    ]);
    assert_eq!(expected, row);
    Ok(())
  }
}
