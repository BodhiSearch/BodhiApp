use objs::AppError;
use prettytable::{format::FormatBuilder, row, Cell, Row, Table};
use services::AppService;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, derive_new::new)]
pub struct EnvCommand {
  service: Arc<dyn AppService>,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum EnvCommandError {}

type Result<T> = std::result::Result<T, EnvCommandError>;

impl EnvCommand {
  pub fn execute(&self) -> Result<()> {
    let envs = self.service.env_service().list();
    // println!("List of current environment/config variables:");
    // println!();
    let mut table = Table::new();
    table.add_row(row!["ENV VARIABLE", "VALUE"]);
    let envs = envs
      .into_iter()
      .map(|s| (s.key.clone(), s))
      .collect::<HashMap<_, _>>();
    let mut keys = envs.keys().collect::<Vec<_>>();
    keys.sort();
    for key in keys {
      table.add_row(Row::from(vec![
        Cell::new(key),
        Cell::new(
          serde_yaml::to_string(&envs.get(key).expect("should be present").current_value)
            .unwrap()
            .trim(),
        ),
      ]));
    }
    table.set_format(FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    println!();
    Ok(())
  }
}
