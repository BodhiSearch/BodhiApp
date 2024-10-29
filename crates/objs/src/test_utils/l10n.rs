use crate::{l10n::L10N_RESOURCES, FluentLocalizationService};
use rstest::fixture;
use std::sync::{Arc, RwLock};

impl FluentLocalizationService {
  pub fn new_standalone() -> Self {
    Self::new()
  }

  pub fn get_instance() -> Arc<FluentLocalizationService> {
    MOCK_LOCALIZATION_SERVICE
      .read()
      .unwrap()
      .as_ref()
      .unwrap()
      .clone()
  }
}

#[fixture]
pub fn localization_service() -> Arc<FluentLocalizationService> {
  Arc::new(FluentLocalizationService::new_standalone())
}

lazy_static::lazy_static! {
  pub static ref MOCK_LOCALIZATION_SERVICE: Arc<RwLock<Option<Arc<FluentLocalizationService>>>> = Arc::new(RwLock::new(None));
}

pub fn set_mock_localization_service(service: Arc<FluentLocalizationService>) {
  let mut mock = MOCK_LOCALIZATION_SERVICE.write().unwrap();
  *mock = Some(service);
}

pub fn clear_mock_localization_service() {
  let mut mock = MOCK_LOCALIZATION_SERVICE.write().unwrap();
  *mock = None;
}

#[fixture]
#[once]
pub fn setup_l10n(
  localization_service: Arc<FluentLocalizationService>,
) -> Arc<FluentLocalizationService> {
  localization_service
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../objs/src/resources"
    ))
    .unwrap()
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../llamacpp_rs/src/resources"
    ))
    .unwrap()
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../services/src/resources"
    ))
    .unwrap()
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../commands/src/resources"
    ))
    .unwrap()
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../server_core/src/resources"
    ))
    .unwrap()
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../auth_middleware/src/resources"
    ))
    .unwrap()
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../routes_oai/src/resources"
    ))
    .unwrap()
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../routes_app/src/resources"
    ))
    .unwrap()
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../routes_all/src/resources"
    ))
    .unwrap()
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../server_app/src/resources"
    ))
    .unwrap()
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/../bodhiui/src-tauri/src/resources"
    ))
    .unwrap();
  set_mock_localization_service(localization_service.clone());
  localization_service
}
