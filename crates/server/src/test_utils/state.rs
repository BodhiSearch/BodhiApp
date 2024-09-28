use crate::{DefaultRouterState, MockSharedContextRw};
use rstest::fixture;
use services::test_utils::{app_service_stub, AppServiceStub};
use std::sync::Arc;

#[fixture]
#[awt]
pub async fn router_state_stub(#[future] app_service_stub: AppServiceStub) -> DefaultRouterState {
  DefaultRouterState::new(
    Arc::new(MockSharedContextRw::default()),
    Arc::new(app_service_stub),
  )
}
