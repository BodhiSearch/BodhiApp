use crate::{error::Result, objs::Alias, service::AppServiceFn};

mockall::mock! {
  pub InteractiveRuntime {
    pub fn new() -> Self;

    pub fn execute(&self, alias: Alias, service: &dyn AppServiceFn) -> Result<()>;
  }
}
