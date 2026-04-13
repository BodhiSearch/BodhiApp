use super::super::provider_shared::merge_extra_body;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};

// =============================================================================
// merge_extra_body unit tests
// =============================================================================

#[rstest]
#[case::empty_config(
  json!({"model": "claude-3", "messages": []}),
  json!({}),
  json!({"model": "claude-3", "messages": []})
)]
#[case::scalar_absent_in_incoming(
  json!({"model": "claude-3", "messages": []}),
  json!({"max_tokens": 32000}),
  json!({"model": "claude-3", "messages": [], "max_tokens": 32000})
)]
#[case::scalar_present_in_incoming_wins(
  json!({"model": "claude-3", "max_tokens": 10}),
  json!({"max_tokens": 32000}),
  json!({"model": "claude-3", "max_tokens": 10})
)]
#[case::system_absent_config_applied(
  json!({"model": "claude-3", "messages": []}),
  json!({"system": [{"type": "text", "text": "You are Claude Code"}]}),
  json!({"model": "claude-3", "messages": [], "system": [{"type": "text", "text": "You are Claude Code"}]})
)]
#[case::system_arrays_config_prepended(
  json!({"model": "claude-3", "system": [{"type": "text", "text": "USER"}]}),
  json!({"system": [{"type": "text", "text": "CONFIG"}]}),
  json!({"model": "claude-3", "system": [{"type": "text", "text": "CONFIG"}, {"type": "text", "text": "USER"}]})
)]
#[case::null_config_passthrough(
  json!({"model": "claude-3"}),
  json!(null),
  json!({"model": "claude-3"})
)]
#[case::non_object_config_passthrough(
  json!({"model": "claude-3"}),
  json!("not-an-object"),
  json!({"model": "claude-3"})
)]
#[case::system_incoming_string_incoming_wins(
  json!({"model": "claude-3", "system": "plain string"}),
  json!({"system": [{"type": "text", "text": "CONFIG"}]}),
  json!({"model": "claude-3", "system": "plain string"})
)]
fn test_merge_extra_body(#[case] incoming: Value, #[case] config: Value, #[case] expected: Value) {
  let result = merge_extra_body(incoming, &config);
  assert_eq!(expected, result);
}
