use crate::{error::Result, service::AppServiceFn};
use objs::Alias;
use std::sync::Arc;

mockall::mock! {
  #[async_trait::async_trait]
  pub InteractiveRuntime {
    pub fn new() -> Self;

    pub async fn execute(&self, alias: Alias, service: Arc<dyn AppServiceFn>) -> Result<()>;
  }
}
