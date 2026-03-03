use crate::mcps::RegistrationType;
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[case::pre_registered(RegistrationType::PreRegistered, "\"pre_registered\"")]
#[case::dynamic_registration(RegistrationType::DynamicRegistration, "\"dynamic_registration\"")]
fn test_registration_type_serde_roundtrip(
  #[case] variant: RegistrationType,
  #[case] expected_json: &str,
) {
  let json = serde_json::to_string(&variant).unwrap();
  assert_eq!(expected_json, json);
  let back: RegistrationType = serde_json::from_str(&json).unwrap();
  assert_eq!(variant, back);
}

#[rstest]
#[case::pre_registered(RegistrationType::PreRegistered, "pre_registered")]
#[case::dynamic_registration(RegistrationType::DynamicRegistration, "dynamic_registration")]
fn test_registration_type_display(#[case] variant: RegistrationType, #[case] expected: &str) {
  assert_eq!(expected, variant.to_string());
}

#[rstest]
#[case::pre_registered("pre_registered", Ok(RegistrationType::PreRegistered))]
#[case::dynamic_registration("dynamic_registration", Ok(RegistrationType::DynamicRegistration))]
#[case::invalid("invalid", Err(()))]
fn test_registration_type_from_str(
  #[case] input: &str,
  #[case] expected: Result<RegistrationType, ()>,
) {
  assert_eq!(expected, input.parse::<RegistrationType>().map_err(|_| ()));
}

#[test]
fn test_registration_type_default() {
  assert_eq!(RegistrationType::PreRegistered, RegistrationType::default());
}
