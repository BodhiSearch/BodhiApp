use crate::hf::{list_models, LocalModel};
use derive_new::new;
use prettytable::{
  format::{self},
  row, Cell, Row, Table,
};
use regex::Regex;
use serde::Deserialize;

pub(super) const MODELS_YAML: &str = include_str!("models.yaml");

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Deserialize, Default, PartialEq, Clone, new)]
pub(super) struct RemoteModel {
  pub(super) display_name: String,
  pub(super) family: Option<String>,
  pub(super) repo: String,
  pub(super) base_model: Option<String>,
  pub(super) tokenizer_config: String,
  pub(super) features: Vec<String>,
  pub(super) files: Vec<String>,
  #[serde(rename = "default")]
  pub(super) default_variant: String,
}

impl RemoteModel {
  fn variants(&self) -> Vec<String> {
    let re = Regex::new(r".*\.(?P<variant>[^\.]*)\.gguf").unwrap();
    self
      .files
      .iter()
      .map(|f| match re.captures(f) {
        Some(captures) => captures["variant"].to_string(),
        None => f.to_string(),
      })
      .collect::<Vec<String>>()
  }

  fn default_variant(&self) -> String {
    let re = Regex::new(r".*\.(?P<variant>[^\.]*)\.gguf").unwrap();
    let Some(cap) = re.captures(&self.default_variant) else {
      return self.default_variant.to_string();
    };
    cap["variant"].to_string()
  }
}

impl From<RemoteModel> for Row {
  fn from(model: RemoteModel) -> Self {
    let variants = model
      .variants()
      .into_iter()
      .fold(vec![String::from("")], |mut fold, item| {
        if fold.last().unwrap().len() > 24 {
          fold.push(String::new());
        }
        let last = fold.last_mut().unwrap();
        if !last.is_empty() {
          last.push_str(", ");
        }
        last.push_str(item.as_str());
        fold
      })
      .join(",\n");

    Row::from(vec![
      &model.display_name,
      &model.repo,
      model.family.as_deref().unwrap_or(""),
      &model.features.join(","),
      &variants,
      &model.default_variant(),
    ])
  }
}

impl From<LocalModel> for Row {
  fn from(model: LocalModel) -> Self {
    let LocalModel {
      name,
      repo,
      sha,
      size,
      updated,
      ..
    } = model;
    let human_size = size
      .map(|size| format!("{:.2} GB", size as f64 / 2_f64.powf(30.0)))
      .unwrap_or_else(|| String::from("Unknown"));
    let humantime = || -> Option<String> {
      let updated = updated?;
      let duration = chrono::Utc::now()
        .signed_duration_since(updated)
        .to_std()
        .ok()?;
      let formatted = humantime::format_duration(duration)
        .to_string()
        .split(' ')
        .take(2)
        .collect::<Vec<_>>()
        .join(" ");
      Some(formatted)
    }();
    let humantime = humantime.unwrap_or_else(|| String::from("Unknown"));
    Row::from(vec![
      Cell::new(&name),
      Cell::new(&repo),
      Cell::new(&sha[..8]),
      Cell::new(&human_size),
      Cell::new(&humantime),
    ])
  }
}

pub(crate) fn find_remote_model(id: &str) -> Option<RemoteModel> {
  let models: Vec<RemoteModel> = serde_yaml::from_str(MODELS_YAML).ok()?;
  _find_remote_model(models, id)
}

fn _find_remote_model(models: Vec<RemoteModel>, id: &str) -> Option<RemoteModel> {
  models.into_iter().find(|model| model.display_name.eq(id))
}

pub struct List {
  remote: bool,
}

impl List {
  pub fn new(remote: bool) -> Self {
    Self { remote }
  }

  pub fn execute(self) -> anyhow::Result<()> {
    if self.remote {
      self.list_remote_models()?;
    } else {
      self.list_local_models()?;
    }
    Ok(())
  }

  fn list_local_models(self) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.add_row(row!["NAME", "REPO ID", "SHA", "SIZE", "MODIFIED"]);
    let models = list_models();
    for row in models.into_iter().map(Row::from) {
      table.add_row(row);
    }
    table.set_format(format::FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    Ok(())
  }

  fn list_remote_models(self) -> anyhow::Result<()> {
    let models: Vec<RemoteModel> = serde_yaml::from_str(MODELS_YAML)?;
    let mut table = Table::new();
    table.add_row(row![
      "ID", "REPO ID", "FAMILY", "FEATURES", "VARIANTS", "DEFAULT"
    ]);
    for row in models.into_iter().map(Row::from) {
      table.add_row(row);
    }
    table.set_format(format::FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    println!();
    println!("To download the model, run `bodhi pull <ID>1");
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::RemoteModel;
  use crate::{hf::LocalModel, list::_find_remote_model, test_utils::TEST_MODELS_YAML};
  use chrono::Utc;
  use prettytable::{Cell, Row};

  #[test]
  fn test_list_remote_model_variants_default() -> anyhow::Result<()> {
    let model = RemoteModel {
      files: vec![
        "Meta-Llama-3-8B-Instruct.Q4_0.gguf".to_string(),
        "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      ],
      default_variant: "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      ..Default::default()
    };
    let expected = vec!["Q4_0".to_string(), "Q8_0".to_string()];
    assert_eq!(expected, model.variants());
    assert_eq!("Q8_0", model.default_variant());
    Ok(())
  }

  #[test]
  fn test_list_find_remote_model_by_id() -> anyhow::Result<()> {
    let llama3_instruct = RemoteModel {
      display_name: "meta-llama3-8b-instruct".to_string(),
      ..Default::default()
    };
    let llama3 = RemoteModel {
      display_name: "meta-llama3-8b".to_string(),
      ..Default::default()
    };
    let models = vec![llama3_instruct, llama3.clone()];
    let model = _find_remote_model(models, "meta-llama3-8b").unwrap();
    assert_eq!(llama3, model);
    Ok(())
  }

  #[test]
  fn test_list_remote_model_to_row() -> anyhow::Result<()> {
    let model = serde_yaml::from_str::<Vec<RemoteModel>>(TEST_MODELS_YAML)?
      .first()
      .unwrap()
      .to_owned();
    let row: Row = model.into();
    let expected = Row::from(vec![
      Cell::new("meta-llama3-8b-instruct"),
      Cell::new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF"),
      Cell::new("llama3"),
      Cell::new("chat"),
      Cell::new("Q2_K, Q4_0, Q8_0"),
      Cell::new("Q8_0"),
    ]);
    assert_eq!(expected, row);
    Ok(())
  }

  #[test]
  fn test_list_model_item_to_row() -> anyhow::Result<()> {
    let three_days_back = Utc::now() - chrono::Duration::days(3) - chrono::Duration::hours(1);
    let model = LocalModel {
      name: "Meta-Llama-3-8B-Instruct-GGUF".to_string(),
      repo: "QuantFactory".to_string(),
      path: "models--QuantFactory--Meta-Llama-3-8B-Instruct-GGUF".to_string(),
      sha: "1234567890".to_string(),
      size: Some(1024 * 1024 * 1024 * 10),
      updated: Some(three_days_back),
    };
    let row = model.into();
    let expected = Row::from(vec![
      Cell::new("Meta-Llama-3-8B-Instruct-GGUF"),
      Cell::new("QuantFactory"),
      Cell::new("12345678"),
      Cell::new("10.00 GB"),
      Cell::new("3days 1h"),
    ]);
    assert_eq!(expected, row);
    Ok(())
  }
}
