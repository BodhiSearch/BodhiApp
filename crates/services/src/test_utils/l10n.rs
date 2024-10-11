use objs::{
  test_utils::{enable_tracing, setup_l10n_objs},
  FluentLocalizationService,
};
use rstest::fixture;
use std::sync::Arc;

#[fixture]
pub fn setup_l10n_services(
  #[from(enable_tracing)] _enable_tracing: &(),
  setup_l10n_objs: Arc<FluentLocalizationService>,
) -> Arc<FluentLocalizationService> {
  setup_l10n_objs
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/tests/resources"
    ))
    .unwrap();
  setup_l10n_objs
}
