use crate::FluentLocalizationService;
use rstest::fixture;
use std::sync::Arc;

impl FluentLocalizationService {
  pub fn new_standalone() -> Self {
    Self::new()
  }
}

#[fixture]
pub fn localization_service() -> Arc<FluentLocalizationService> {
  Arc::new(FluentLocalizationService::new_standalone())
}
