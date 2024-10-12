use crate::interactive::InteractiveError;
use objs::Alias;
use services::AppService;
use std::sync::Arc;

mockall::mock! {
  #[async_trait::async_trait]
  pub InteractiveRuntime {
    pub fn new() -> Self;

    pub async fn execute(&self, alias: Alias, service: Arc<dyn AppService>) -> Result<(), InteractiveError>;
  }
}
