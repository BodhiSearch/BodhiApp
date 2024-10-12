use objs::FluentLocalizationService;
use rstest::fixture;
use server_app::test_utils::setup_l10n_server_app;
use std::sync::Arc;

#[fixture]
pub fn setup_l10n_bodhi(
  #[from(setup_l10n_server_app)] localization_service: Arc<FluentLocalizationService>,
) -> Arc<FluentLocalizationService> {
  localization_service
    .load_resource(crate::l10n::L10N_RESOURCES)
    .unwrap();
  localization_service
}
