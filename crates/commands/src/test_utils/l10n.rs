use objs::FluentLocalizationService;
use rstest::fixture;
use services::test_utils::setup_l10n_services;
use std::sync::Arc;

#[fixture]
pub fn setup_l10n_commands(
  #[from(setup_l10n_services)] localization_service: Arc<FluentLocalizationService>,
) -> Arc<FluentLocalizationService> {
  localization_service
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/tests/resources"
    ))
    .unwrap();
  localization_service
}
