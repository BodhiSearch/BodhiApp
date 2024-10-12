use objs::{
  test_utils::{enable_tracing, setup_l10n_objs},
  FluentLocalizationService,
};
use rstest::fixture;
use std::sync::Arc;

#[fixture]
pub fn setup_l10n_services(
  #[from(enable_tracing)] _enable_tracing: &(),
  #[from(setup_l10n_objs)] localization_service: Arc<FluentLocalizationService>,
) -> Arc<FluentLocalizationService> {
  localization_service
    .load_resource(&crate::l10n::L10N_RESOURCES)
    .unwrap();
  localization_service
}
