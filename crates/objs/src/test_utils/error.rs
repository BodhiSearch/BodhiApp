use fluent::{FluentBundle, FluentResource};
use rstest::fixture;
use std::{collections::HashMap, fs};

pub fn assert_error_message(
  bundle: &FluentBundle<FluentResource>,
  code: &str,
  args: HashMap<String, String>,
  expected: &str,
) {
  let message = bundle
    .get_message(code)
    .expect(&format!("Message not found, code: {code}"))
    .value()
    .expect(&format!("Message has no value, code: {code}"));
  let mut errors = Vec::new();
  let args = args
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
  let formatted = bundle.format_pattern(message, Some(&args), &mut errors);
  assert_eq!(
    errors
      .first()
      .map(|err| err.to_string())
      .unwrap_or_default(),
    "",
    "formatting errors occurred"
  );
  assert!(errors.is_empty(), "formatting errors occurred");
  assert_eq!(
    formatted
      .to_string()
      .replace("\u{2068}", "")
      .replace("\u{2069}", ""),
    expected
  );
}

#[fixture]
pub fn fluent_bundle(
  #[default("tests/messages/test.ftl")] path: &str,
) -> FluentBundle<FluentResource> {
  let ftl_string = fs::read_to_string(path).expect("Failed to read FTL file");
  let res = FluentResource::try_new(ftl_string).expect("Failed to parse FTL resource");
  let mut bundle = FluentBundle::default();
  bundle
    .add_resource(res)
    .expect("Failed to add FTL resource to bundle");
  bundle
}
