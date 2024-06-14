use std::sync::Arc;

use prettytable::{format::FormatBuilder, row, Cell, Row, Table};

use crate::service::AppServiceFn;

#[derive(Debug, derive_new::new)]
pub struct EnvCommand {
  service: Arc<dyn AppServiceFn>,
}

impl EnvCommand {
  pub fn execute(&self) -> crate::error::Result<()> {
    let envs = self.service.env_service().list();
    // println!("List of current environment/config variables:");
    // println!();
    let mut table = Table::new();
    table.add_row(row!["ENV VARIABLE", "VALUE"]);
    let mut keys = envs.keys().collect::<Vec<_>>();
    keys.sort();
    for key in keys {
      table.add_row(Row::from(vec![
        Cell::new(key),
        Cell::new(envs.get(key).expect("should be present")),
      ]));
    }
    table.set_format(FormatBuilder::default().padding(2, 2).build());
    table.printstd();
    println!();
    Ok(())
  }
}
