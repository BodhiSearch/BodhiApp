use super::*;
use rstest::rstest;
use std::path::Path;

fn load_test_gguf(filename: &str) -> GGUFMetadata {
  let path = Path::new("tests/data/gguf-capabilities").join(filename);
  GGUFMetadata::new(&path).unwrap_or_else(|_| panic!("Failed to load test file: {}", filename))
}

#[test]
fn test_llama_plain_no_capabilities() {
  let metadata = load_test_gguf("llama-plain.gguf");
  let capabilities = extract_capabilities(&metadata);

  assert_eq!(Some(false), capabilities.vision);
  assert_eq!(Some(false), capabilities.audio);
  assert_eq!(Some(false), capabilities.thinking);
  assert_eq!(Some(false), capabilities.tools.function_calling);
  assert_eq!(Some(false), capabilities.tools.structured_output);
}

#[test]
fn test_qwen_vision_capability() {
  let metadata = load_test_gguf("qwen-vision.gguf");
  let capabilities = extract_capabilities(&metadata);

  assert_eq!(Some(true), capabilities.vision);
  assert_eq!(Some(false), capabilities.audio);
  assert_eq!(Some(false), capabilities.thinking);
  assert_eq!(Some(false), capabilities.tools.function_calling);
  assert_eq!(Some(false), capabilities.tools.structured_output);
}

#[test]
fn test_phi_tools_capabilities() {
  let metadata = load_test_gguf("phi-tools.gguf");
  let capabilities = extract_capabilities(&metadata);

  assert_eq!(Some(false), capabilities.vision);
  assert_eq!(Some(false), capabilities.audio);
  assert_eq!(Some(false), capabilities.thinking);
  assert_eq!(Some(true), capabilities.tools.function_calling);
  assert_eq!(Some(false), capabilities.tools.structured_output);
}

#[test]
fn test_deepseek_thinking_capability() {
  let metadata = load_test_gguf("deepseek-thinking.gguf");
  let capabilities = extract_capabilities(&metadata);

  assert_eq!(Some(false), capabilities.vision);
  assert_eq!(Some(false), capabilities.audio);
  assert_eq!(Some(true), capabilities.thinking);
  assert_eq!(Some(false), capabilities.tools.function_calling);
  assert_eq!(Some(false), capabilities.tools.structured_output);
}

#[test]
fn test_mistral_audio_capability() {
  let metadata = load_test_gguf("mistral-audio.gguf");
  let capabilities = extract_capabilities(&metadata);

  assert_eq!(Some(false), capabilities.vision);
  assert_eq!(Some(true), capabilities.audio);
  assert_eq!(Some(false), capabilities.thinking);
  assert_eq!(Some(false), capabilities.tools.function_calling);
  assert_eq!(Some(false), capabilities.tools.structured_output);
}

#[test]
fn test_llava_multimodal_capabilities() {
  let metadata = load_test_gguf("llava-multimodal.gguf");
  let capabilities = extract_capabilities(&metadata);

  assert_eq!(Some(true), capabilities.vision);
  assert_eq!(Some(false), capabilities.audio);
  assert_eq!(Some(false), capabilities.thinking);
  assert_eq!(Some(true), capabilities.tools.function_calling);
  assert_eq!(Some(false), capabilities.tools.structured_output);
}

#[rstest]
#[case::jinja_if_tools("{% if tools %}")]
#[case::tool_call_tag("Use <tool_call> to invoke functions")]
#[case::mistral_tool_calls("[TOOL_CALLS]")]
#[case::ollama_tools("{{ .Tools }}")]
#[case::builtin_tools("builtin_tools available")]
fn test_detect_tool_calling_success(#[case] template: &str) {
  assert_eq!(detect_tool_calling(Some(template)), Some(true));
}

#[rstest]
#[case::regular_template("regular chat template")]
fn test_detect_tool_calling_failure(#[case] template: &str) {
  assert_eq!(detect_tool_calling(Some(template)), Some(false));
}

#[test]
fn test_detect_tool_calling_none() {
  assert_eq!(detect_tool_calling(None), Some(false));
}

#[rstest]
#[case::think_tag("<think>reasoning</think>")]
#[case::reasoning_tag("<reasoning>process</reasoning>")]
#[case::reasoning_content("reasoning_content goes here")]
fn test_detect_thinking_template_success(#[case] template: &str) {
  assert_eq!(detect_thinking(Some(template), None), Some(true));
}

#[rstest]
#[case::regular_template("regular template")]
fn test_detect_thinking_template_failure(#[case] template: &str) {
  assert_eq!(detect_thinking(Some(template), None), Some(false));
}

#[rstest]
#[case::deepseek_r1("deepseek-r1")]
#[case::qwen_qwq("Qwen-QwQ-32B")]
#[case::model_reasoning("model-reasoning")]
#[case::deepseek_r("deepseek-r7")]
fn test_detect_thinking_model_name_success(#[case] model_name: &str) {
  assert_eq!(detect_thinking(None, Some(model_name)), Some(true));
}

#[rstest]
#[case::llama("llama-3")]
fn test_detect_thinking_model_name_failure(#[case] model_name: &str) {
  assert_eq!(detect_thinking(None, Some(model_name)), Some(false));
}

#[test]
fn test_detect_thinking_template_takes_precedence() {
  assert_eq!(detect_thinking(Some("<think>"), Some("llama")), Some(true));
}

#[rstest]
#[case::response_format("response_format")]
#[case::json_schema("json_schema")]
#[case::json_mode("json_mode enabled")]
#[case::schema_tags("<schema></schema>")]
fn test_detect_structured_output_success(#[case] template: &str) {
  assert_eq!(detect_structured_output(Some(template)), Some(true));
}

#[rstest]
#[case::regular_template("regular template")]
fn test_detect_structured_output_failure(#[case] template: &str) {
  assert_eq!(detect_structured_output(Some(template)), Some(false));
}

#[test]
fn test_detect_structured_output_none() {
  assert_eq!(detect_structured_output(None), Some(false));
}

#[rstest]
#[case::q4_k_m("model-Q4_K_M.gguf", "Q4_K_M")]
#[case::q8_0("llama-3.2-Q8_0.gguf", "Q8_0")]
#[case::f16("model-f16.gguf", "F16")]
#[case::q5_k_m("phi-3-mini-4k-instruct-q5_k_m.gguf", "Q5_K_M")]
fn test_parse_quantization_from_filename_success(#[case] filename: &str, #[case] expected: &str) {
  assert_eq!(
    parse_quantization_from_filename(filename),
    Some(expected.to_string())
  );
}

#[rstest]
#[case::no_quantization("model.gguf")]
fn test_parse_quantization_from_filename_failure(#[case] filename: &str) {
  assert_eq!(parse_quantization_from_filename(filename), None);
}

#[test]
fn test_extract_metadata_returns_complete_struct() {
  let metadata = load_test_gguf("llama-plain.gguf");
  let model_metadata = extract_metadata(&metadata, "model-Q4_K_M.gguf");

  assert_eq!(Some(false), model_metadata.capabilities.vision);
  assert!(model_metadata.context.max_input_tokens.is_some());
  assert_eq!(
    Some("Q4_K_M".to_string()),
    model_metadata.architecture.quantization
  );
  assert_eq!("gguf", model_metadata.architecture.format);
}

#[test]
fn test_extract_context_from_gguf() {
  let metadata = load_test_gguf("llama-plain.gguf");
  let context = extract_context(&metadata);

  assert_eq!(Some(4096), context.max_input_tokens);
  assert_eq!(None, context.max_output_tokens);
}

#[test]
fn test_extract_architecture_from_gguf() {
  let metadata = load_test_gguf("llama-plain.gguf");
  let arch = extract_architecture(&metadata, "model-Q8_0.gguf");

  assert_eq!(Some("llama".to_string()), arch.family);
  assert_eq!(Some(7_000_000_000), arch.parameter_count);
  assert_eq!(Some("Q8_0".to_string()), arch.quantization);
  assert_eq!("gguf", arch.format);
}

#[test]
fn test_extract_architecture_no_quantization_in_filename() {
  let metadata = load_test_gguf("llama-plain.gguf");
  let arch = extract_architecture(&metadata, "model.gguf");

  assert_eq!(Some("llama".to_string()), arch.family);
  assert_eq!(None, arch.quantization);
}
