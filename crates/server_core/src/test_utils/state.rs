use crate::{ContextError, DefaultRouterState, MockSharedContext, ServerFactory};
use llama_server_proc::{LlamaServerArgs, Server};
use rstest::fixture;
use services::test_utils::{app_service_stub, AppServiceStub};
use std::{
  path::Path,
  sync::{Arc, Mutex},
};

#[fixture]
#[awt]
pub async fn router_state_stub(#[future] app_service_stub: AppServiceStub) -> DefaultRouterState {
  DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service_stub),
  )
}

#[derive(Debug)]
pub struct ServerFactoryStub {
  pub servers: Mutex<Vec<Box<dyn Server>>>,
}

impl ServerFactoryStub {
  pub fn new(instance: Box<dyn Server>) -> Self {
    Self {
      servers: Mutex::new(vec![instance]),
    }
  }

  pub fn new_with_instances(instances: Vec<Box<dyn Server>>) -> Self {
    Self {
      servers: Mutex::new(instances),
    }
  }
}

impl ServerFactory for ServerFactoryStub {
  fn create_server(
    &self,
    _executable_path: &Path,
    _server_args: &LlamaServerArgs,
  ) -> Result<Box<dyn Server>, ContextError> {
    Ok(self.servers.lock().unwrap().pop().unwrap())
  }
}
