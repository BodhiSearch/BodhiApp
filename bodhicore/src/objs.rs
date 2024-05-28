use derive_new::new;
use prettytable::{Cell, Row};
use serde::{Deserialize, Serialize};

use crate::list::RemoteModel;

#[derive(Debug, Serialize, Deserialize, PartialEq, new)]
pub struct Alias {
  pub alias: String,
  family: Option<String>,
  repo: Option<String>,
  filename: Option<String>,
  features: Vec<String>,
}

impl From<RemoteModel> for Alias {
  fn from(value: RemoteModel) -> Self {
    Alias::new(
      value.alias,
      Some(value.family),
      Some(value.repo),
      Some(value.filename),
      value.features,
    )
  }
}

impl From<Alias> for Row {
  fn from(value: Alias) -> Self {
    Row::from(vec![
      Cell::new(&value.alias),
      Cell::new(&value.family.unwrap_or_default()),
      Cell::new(&value.repo.unwrap_or_default()),
      Cell::new(&value.filename.unwrap_or_default()),
      Cell::new(&value.features.join(",")),
    ])
  }
}
