use crate::mcps::{validate_oauth_endpoint_url, RegistrationType, MAX_MCP_SERVER_URL_LEN};
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

#[rstest]
#[case::https_url("https://auth.example.com/authorize", "authorization_endpoint")]
#[case::localhost_url("http://localhost:8080/token", "token_endpoint")]
fn test_validate_oauth_endpoint_url_accepts_valid(#[case] url: &str, #[case] field_name: &str) {
  assert!(validate_oauth_endpoint_url(url, field_name).is_ok());
}

#[rstest]
#[case::empty("", "authorization_endpoint", "cannot be empty")]
#[case::invalid("not-a-url", "token_endpoint", "not a valid URL")]
fn test_validate_oauth_endpoint_url_rejects_invalid(
  #[case] url: &str,
  #[case] field_name: &str,
  #[case] expected_msg: &str,
) {
  let result = validate_oauth_endpoint_url(url, field_name);
  assert!(result.is_err());
  assert!(result.unwrap_err().contains(expected_msg));
}

#[test]
fn test_validate_oauth_endpoint_url_rejects_too_long() {
  let long_url = format!("https://example.com/{}", "a".repeat(MAX_MCP_SERVER_URL_LEN));
  assert!(validate_oauth_endpoint_url(&long_url, "authorization_endpoint").is_err());
}
