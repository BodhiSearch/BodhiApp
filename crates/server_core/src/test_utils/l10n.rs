use commands::test_utils::setup_l10n_commands;
use objs::FluentLocalizationService;
use rstest::fixture;
use std::sync::Arc;

#[fixture]
pub fn setup_l10n_server_core(
  #[from(setup_l10n_commands)] localization_service: Arc<FluentLocalizationService>,
) -> Arc<FluentLocalizationService> {
  localization_service
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/tests/resources"
    ))
    .unwrap();
  localization_service
}
