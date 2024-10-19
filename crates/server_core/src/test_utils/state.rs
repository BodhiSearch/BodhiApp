use crate::{DefaultRouterState, MockSharedContextRw, ServerContextFactory};
use llamacpp_rs::ServerContext;
use rstest::fixture;
use services::test_utils::{app_service_stub, AppServiceStub};
use std::sync::{Arc, Mutex};

#[fixture]
#[awt]
pub async fn router_state_stub(#[future] app_service_stub: AppServiceStub) -> DefaultRouterState {
  DefaultRouterState::new(
    Arc::new(MockSharedContextRw::default()),
    Arc::new(app_service_stub),
  )
}

#[derive(Debug)]
pub struct BodhiServerFactoryStub {
  pub bodhi_server: Mutex<Vec<Box<dyn ServerContext>>>,
}

impl BodhiServerFactoryStub {
  pub fn new(instance: Box<dyn ServerContext>) -> Self {
    Self {
      bodhi_server: Mutex::new(vec![instance]),
    }
  }

  pub fn new_with_instances(instances: Vec<Box<dyn ServerContext>>) -> Self {
    Self {
      bodhi_server: Mutex::new(instances),
    }
  }
}

impl ServerContextFactory for BodhiServerFactoryStub {
  fn create_server_context(&self) -> Box<dyn ServerContext> {
    self.bodhi_server.lock().unwrap().pop().unwrap()
  }
}
