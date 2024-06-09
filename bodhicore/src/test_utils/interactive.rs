use crate::{error::Result, objs::Alias, service::AppServiceFn};
use std::sync::Arc;

mockall::mock! {
  pub InteractiveRuntime {
    pub fn new() -> Self;

    pub fn execute(&self, alias: Alias, service: Arc<dyn AppServiceFn>) -> Result<()>;
  }
}
