use crate::FluentLocalizationService;
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
pub fn setup_l10n_objs(
  localization_service: Arc<FluentLocalizationService>,
) -> Arc<FluentLocalizationService> {
  localization_service
    .load_resource(&include_dir::include_dir!(
      "$CARGO_MANIFEST_DIR/tests/resources"
    ))
    .unwrap();
  set_mock_localization_service(localization_service.clone());
  localization_service
}
