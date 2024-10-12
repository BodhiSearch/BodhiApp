use objs::FluentLocalizationService;
use routes_app::test_utils::setup_l10n_routes_app;
use rstest::fixture;
use std::sync::Arc;

#[fixture]
pub fn setup_l10n_server_app(
  #[from(setup_l10n_routes_app)] localization_service: Arc<FluentLocalizationService>,
) -> Arc<FluentLocalizationService> {
  localization_service
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/tests/resources"
    ))
    .unwrap();
  localization_service
}
