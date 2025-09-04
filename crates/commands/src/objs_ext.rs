use objs::{HubFile, RemoteModel, UserAlias};
use prettytable::{Cell, Row};

pub trait IntoRow {
  fn into_row(self) -> Row;
}

impl IntoRow for UserAlias {
  fn into_row(self) -> Row {
    Row::from(vec![
      Cell::new(&self.alias),
      Cell::new(&self.repo.to_string()),
      Cell::new(&self.filename),
      Cell::new(&self.snapshot[..8]),
      // Chat template column removed since llama.cpp now handles chat templates
    ])
  }
}

impl IntoRow for HubFile {
  fn into_row(self) -> Row {
    let HubFile {
      repo,
      filename,
      snapshot,
      size,
      ..
    } = self;
    let human_size = size
      .map(|size| format!("{:.2} GB", size as f64 / 2_f64.powf(30.0)))
      .unwrap_or_else(|| String::from("Unknown"));
    Row::from(vec![
      Cell::new(&repo.to_string()),
      Cell::new(&filename),
      Cell::new(&snapshot[..8]),
      Cell::new(&human_size),
    ])
  }
}

impl IntoRow for RemoteModel {
  fn into_row(self) -> Row {
    Row::from(vec![
      &self.alias,
      &self.repo.to_string(),
      &self.filename,
      // Chat template column removed since llama.cpp now handles chat templates
    ])
  }
}

#[cfg(test)]
mod test {
  use crate::objs_ext::IntoRow;
  use objs::{HubFile, RemoteModel, Repo, UserAlias};
  use pretty_assertions::assert_eq;
  use prettytable::{Cell, Row};
  use rstest::rstest;
  use std::path::PathBuf;

  #[test]
  fn test_alias_to_row() -> anyhow::Result<()> {
    let alias = UserAlias::testalias();
    let row = alias.into_row();
    assert_eq!(
      Row::from(vec![
        Cell::new("testalias:instruct"),
        Cell::new("MyFactory/testalias-gguf"),
        Cell::new("testalias.Q8_0.gguf"),
        Cell::new("5007652f"),
        // Chat template column removed since llama.cpp now handles chat templates
      ]),
      row
    );
    Ok(())
  }

  #[test]
  fn test_local_model_to_row() -> anyhow::Result<()> {
    let local_model = HubFile::new(
      PathBuf::from("."),
      Repo::llama3(),
      Repo::LLAMA3_Q8.to_string(),
      "1234567890".to_string(),
      Some(1024 * 1024 * 1024 * 10),
    );
    let row = local_model.into_row();
    let expected = Row::from(vec![
      Cell::new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF"),
      Cell::new("Meta-Llama-3-8B-Instruct.Q8_0.gguf"),
      Cell::new("12345678"),
      Cell::new("10.00 GB"),
    ]);
    assert_eq!(expected, row);
    Ok(())
  }

  #[rstest]
  fn test_remote_model_to_row() -> anyhow::Result<()> {
    let model = RemoteModel::llama3();
    let row: Row = model.into_row();
    let expected = Row::from(vec![
      Cell::new("llama3:instruct"),
      Cell::new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF"),
      Cell::new("Meta-Llama-3-8B-Instruct.Q8_0.gguf"),
      // Chat template column removed since llama.cpp now handles chat templates
    ]);
    assert_eq!(expected, row);
    Ok(())
  }
}
