use crate::{FluentLocalizationService, LocalizationService, EN_US};
use fluent::{FluentBundle, FluentResource};
use rstest::fixture;
use std::{collections::HashMap, fs, sync::Arc};

pub fn assert_error_message(
  service: &Arc<FluentLocalizationService>,
  code: &str,
  args: HashMap<String, String>,
  expected: &str,
) {
  let message = service.get_message(&EN_US, code, Some(args)).unwrap();
  assert_eq!(
    message
      .to_string()
      .replace("\u{2068}", "")
      .replace("\u{2069}", ""),
    expected
  );
}

#[fixture]
pub fn fluent_bundle(
  #[default("tests/resources/en-US/test.ftl")] path: &str,
) -> FluentBundle<FluentResource> {
  let ftl_string = fs::read_to_string(path).expect("Failed to read FTL file");
  let res = FluentResource::try_new(ftl_string).expect("Failed to parse FTL resource");
  let mut bundle = FluentBundle::default();
  bundle
    .add_resource(res)
    .expect("Failed to add FTL resource to bundle");
  bundle
}
