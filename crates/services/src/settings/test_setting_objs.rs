use super::SettingMetadata;
use crate::{ApiError, OpenAIApiError};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_yaml::Number;

#[rstest]
// String metadata tests
#[case::string_to_string(
  SettingMetadata::String,
  serde_yaml::Value::String("test".to_string()),
  serde_yaml::Value::String("test".to_string())
)]
#[case::number_to_string(
  SettingMetadata::String,
  serde_yaml::Value::Number(Number::from(123)),
  serde_yaml::Value::String("123".to_string())
)]
#[case::boolean_to_string(
  SettingMetadata::String,
  serde_yaml::Value::Bool(true),
  serde_yaml::Value::String("true".to_string())
)]
// Number metadata tests
#[case::number_to_number(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_yaml::Value::Number(Number::from(50)),
  serde_yaml::Value::Number(Number::from(50))
)]
#[case::string_to_number(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_yaml::Value::String("42".to_string()),
  serde_yaml::Value::Number(Number::from(42))
)]
#[case(
  SettingMetadata::Number { min: -100, max: 100 },
  serde_yaml::Value::String("-42".to_string()),
  serde_yaml::Value::Number(Number::from(-42))
)]
// Boolean metadata tests
#[case(
  SettingMetadata::Boolean,
  serde_yaml::Value::Bool(true),
  serde_yaml::Value::Bool(true)
)]
#[case(
  SettingMetadata::Boolean,
  serde_yaml::Value::Bool(false),
  serde_yaml::Value::Bool(false)
)]
#[case(
  SettingMetadata::Boolean,
  serde_yaml::Value::String("true".to_string()),
  serde_yaml::Value::Bool(true)
)]
#[case(
  SettingMetadata::Boolean,
  serde_yaml::Value::String("false".to_string()),
  serde_yaml::Value::Bool(false)
)]
// Option metadata tests
#[case(
  SettingMetadata::Option { options: vec!["a".to_string(), "b".to_string()] },
  serde_yaml::Value::String("a".to_string()),
  serde_yaml::Value::String("a".to_string())
)]
#[case(
  SettingMetadata::Option { options: vec!["a".to_string(), "b".to_string()] },
  serde_yaml::Value::Number(Number::from(123)),
  serde_yaml::Value::String("123".to_string())
)]
#[case(
  SettingMetadata::Option { options: vec!["true".to_string(), "false".to_string()] },
  serde_yaml::Value::Bool(true),
  serde_yaml::Value::String("true".to_string())
)]
#[case(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_yaml::Value::String("12.34".to_string()),
  serde_yaml::Value::Number(Number::from(12.34))
)]
fn test_setting_metadata_parse(
  #[case] metadata: SettingMetadata,
  #[case] input: serde_yaml::Value,
  #[case] expected: serde_yaml::Value,
) {
  assert_eq!(expected, metadata.parse(input));
}

#[rstest]
// Invalid number strings
#[case(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_yaml::Value::String("not_a_number".to_string())
)]
#[case(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_yaml::Value::String("".to_string())
)]
// Invalid boolean strings
#[case(
  SettingMetadata::Boolean,
  serde_yaml::Value::String("not_a_bool".to_string())
)]
#[case(
  SettingMetadata::Boolean,
  serde_yaml::Value::String("1".to_string())
)]
#[case(
  SettingMetadata::Boolean,
  serde_yaml::Value::String("".to_string())
)]
fn test_setting_metadata_parse_invalid_values(
  #[case] metadata: SettingMetadata,
  #[case] input: serde_yaml::Value,
) {
  // When parsing fails, the original value should be returned
  assert_eq!(input, metadata.parse(input.clone()));
}

#[rstest]
// String metadata tests
#[case::convert_string_to_string(
  SettingMetadata::String,
  serde_json::json!("test"),
  serde_yaml::Value::String("test".to_string())
)]
#[case::convert_number_to_string(
  SettingMetadata::String,
  serde_json::json!(42),
  serde_yaml::Value::String("42".to_string())
)]
#[case::convert_option_to_string(
  SettingMetadata::Option { options: vec!["a".to_string(), "b".to_string()] },
  serde_json::json!("a"),
  serde_yaml::Value::String("a".to_string())
)]
// Number metadata tests
#[case::convert_number_to_number(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_json::json!(42),
  serde_yaml::Value::Number(serde_yaml::Number::from(42))
)]
#[case::convert_string_to_number(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_json::json!("42"),
  serde_yaml::Value::Number(serde_yaml::Number::from(42))
)]
#[case::convert_string_to_number(
  SettingMetadata::Number { min: -100, max: 100 },
  serde_json::json!(-42),
  serde_yaml::Value::Number(serde_yaml::Number::from(-42))
)]
// Boolean metadata tests
#[case::convert_boolean_to_boolean(
  SettingMetadata::Boolean,
  serde_json::json!(true),
  serde_yaml::Value::Bool(true)
)]
#[case::convert_string_to_boolean(
  SettingMetadata::Boolean,
  serde_json::json!("true"),
  serde_yaml::Value::Bool(true)
)]
fn test_setting_metadata_convert_success(
  #[case] metadata: SettingMetadata,
  #[case] input: serde_json::Value,
  #[case] expected: serde_yaml::Value,
) {
  let result = metadata.convert(input);
  assert!(result.is_ok());
  assert_eq!(expected, result.unwrap());
}

#[rstest]
// Invalid string for boolean
#[case::invalid_string_for_boolean(
  SettingMetadata::Boolean,
  serde_json::json!("not_a_bool"),
  "cannot parse \"not_a_bool\" as Boolean"
)]
// Invalid type combinations
#[case(
  SettingMetadata::Boolean,
  serde_json::json!(42),
  "cannot parse 42 as Boolean"
)]
#[case(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_json::json!(true),
  "cannot parse true as Number"
)]
// Number range validation
#[case::number_range_validation_lower(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_json::json!(-1),
  "passed value is not a valid value: -1"
)]
#[case::number_range_validation_upper(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_json::json!(101),
  "passed value is not a valid value: 101"
)]
#[case::number_range_validation_string(
  SettingMetadata::Number { min: 0, max: 100 },
  serde_json::json!("101"),
  "passed value is not a valid value: \"101\""
)]
// Option validation
#[case(
  SettingMetadata::Option { options: vec!["a".to_string(), "b".to_string()] },
  serde_json::json!("c"),
  "passed value is not a valid value: \"c\""
)]
// Null test
#[case(
  SettingMetadata::String,
  serde_json::json!(null),
  "value is null"
)]
fn test_setting_metadata_convert_error(
  #[case] metadata: SettingMetadata,
  #[case] input: serde_json::Value,
  #[case] expected_error: &str,
) {
  let app_error = metadata.convert(input).unwrap_err();
  let message = OpenAIApiError::from(ApiError::from(app_error))
    .error
    .message;
  assert_eq!(expected_error, message.trim());
}
