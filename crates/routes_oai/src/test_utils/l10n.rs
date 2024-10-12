use auth_middleware::test_utils::setup_l10n_middleware;
use objs::FluentLocalizationService;
use rstest::fixture;
use std::sync::Arc;

#[fixture]
pub fn setup_l10n_routes_oai(
  #[from(setup_l10n_middleware)] localization_service: Arc<FluentLocalizationService>,
) -> Arc<FluentLocalizationService> {
  localization_service
    .load_resource(&crate::l10n::L10N_RESOURCES)
    .unwrap();
  localization_service
}
