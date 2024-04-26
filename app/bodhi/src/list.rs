use crate::hf::{list_models, ModelItem};
use hf_hub::Cache;
use prettytable::{
  format::{self},
  row, Cell, Row, Table,
};

pub struct List;

impl List {
  pub fn execute() -> anyhow::Result<()> {
    let cache = Cache::default();
    let cache = cache.path();
    let mut table = Table::new();
    table.add_row(row!["NAME", "REPO ID", "SHA", "SIZE", "MODIFIED"]);
    let models = list_models(cache);

    table = models.into_iter().fold(table, |mut table, model| {
      let ModelItem {
        name,
        owner,
        repo,
        sha,
        size,
        updated,
        ..
      } = model;
      let human_size = size
        .map(|size| format!("{:.2} GB", size as f64 / 1e9))
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
      table.add_row(Row::from(vec![
        Cell::new(&name),
        Cell::new(&format!("{}/{}", owner, repo)),
        Cell::new(&sha[..8]),
        Cell::new(&human_size),
        Cell::new(&humantime),
      ]));
      table
    });
    table.set_format(format::FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    Ok(())
  }
}
