use std::collections::HashMap;

pub fn merge_server_args(
  setting_args: &[String],
  setting_variant_args: &[String],
  alias_args: &[String],
) -> Vec<String> {
  let mut arg_map: HashMap<String, Option<String>> = HashMap::new();

  // Parse and add base args first (lowest precedence)
  let parsed_setting_args = parse_args_from_strings(setting_args);
  for (key, value) in parsed_setting_args {
    arg_map.insert(key, value);
  }

  // Parse and add variant-specific args (medium precedence)
  let parsed_variant_args = parse_args_from_strings(setting_variant_args);
  for (key, value) in parsed_variant_args {
    arg_map.insert(key, value);
  }

  // Parse and add alias args (highest precedence)
  let parsed_alias_args = parse_args_from_strings(alias_args);
  for (key, value) in parsed_alias_args {
    arg_map.insert(key, value);
  }

  // Convert back to Vec<String> maintaining flag format
  arg_map
    .into_iter()
    .map(|(key, value)| match value {
      Some(val) => format!("{} {}", key, val),
      None => key,
    })
    .collect()
}

/// Parse arguments from a collection of strings by joining them and parsing as a single stream
fn parse_args_from_strings(args: &[String]) -> Vec<(String, Option<String>)> {
  if args.is_empty() {
    return Vec::new();
  }

  // Join all strings with whitespace and parse as a single stream
  let combined = args.join(" ");
  parse_args_from_string(&combined)
}

/// Parse arguments from a single string that may contain multiple flags and values
fn parse_args_from_string(arg_string: &str) -> Vec<(String, Option<String>)> {
  let tokens: Vec<&str> = arg_string.split_whitespace().collect();
  let mut result = Vec::new();
  let mut i = 0;

  while i < tokens.len() {
    let token = tokens[i];

    // Skip non-flag tokens (but not negative numbers)
    if !is_flag(token) {
      i += 1;
      continue;
    }

    let flag = token.to_string();
    let mut values = Vec::new();

    // Look ahead to collect values until next flag or end
    i += 1;
    while i < tokens.len() && !is_flag(tokens[i]) {
      values.push(tokens[i]);
      i += 1;
    }

    // Create value: None for standalone flags, Some(value) for flags with values
    let value = if values.is_empty() {
      None
    } else {
      Some(values.join(" "))
    };

    result.push((flag, value));
  }

  result
}

/// Check if a token is a flag (starts with - but is not a negative number)
fn is_flag(token: &str) -> bool {
  if !token.starts_with('-') {
    return false;
  }

  // Handle edge cases
  if token.len() == 1 {
    return false; // Just a single "-" is not a flag
  }

  // If it starts with -- it's definitely a flag
  if token.starts_with("--") {
    return true;
  }

  // If it starts with a single - and the second character is not a digit,
  // it's a flag (like -t, -v, etc.)
  // If the second character is a digit, it might be a negative number
  let second_char = token.chars().nth(1).unwrap();
  if second_char.is_ascii_digit() {
    // Check if the entire string (after the -) is a valid number
    let number_part = &token[1..];
    // Try to parse as integer or float
    number_part.parse::<i64>().is_err() && number_part.parse::<f64>().is_err()
  } else {
    true
  }
}

#[cfg(test)]
mod tests {
  use super::{is_flag, merge_server_args, parse_args_from_string, parse_args_from_strings};
  use rstest::rstest;

  #[rstest]
  #[case::basic_precedence(
    vec!["--verbose --threads 4".to_string()],
    vec!["--n-gpu-layers 999".to_string()],
    vec!["--threads 8".to_string(), "--batch-size 512".to_string()],
    vec!["--verbose", "--n-gpu-layers 999", "--threads 8", "--batch-size 512"],
    vec!["--threads 4"]
  )]
  #[case::empty_inputs(
    vec![],
    vec![],
    vec![],
    vec![],
    vec![]
  )]
  #[case::only_base_args(
    vec!["--verbose --threads 4".to_string()],
    vec![],
    vec![],
    vec!["--verbose", "--threads 4"],
    vec![]
  )]
  fn test_merge_server_args(
    #[case] setting_args: Vec<String>,
    #[case] variant_args: Vec<String>,
    #[case] alias_args: Vec<String>,
    #[case] should_contain: Vec<&str>,
    #[case] should_not_contain: Vec<&str>,
  ) {
    let result = merge_server_args(&setting_args, &variant_args, &alias_args);
    let result_str = result.join(" ");
    for expected in should_contain {
      assert!(
        result_str.contains(expected),
        "Result should contain '{}', but got: {}",
        expected,
        result_str
      );
    }
    for not_expected in should_not_contain {
      assert!(
        !result_str.contains(not_expected),
        "Result should not contain '{}', but got: {}",
        not_expected,
        result_str
      );
    }
  }

  #[rstest]
  #[case::single_flag("--verbose", vec![("--verbose".to_string(), None)])]
  #[case::flag_with_single_value("--threads 4", vec![("--threads".to_string(), Some("4".to_string()))])]
  #[case::flag_with_comma_separated_values("--tensor-split 0.7,0.3", vec![("--tensor-split".to_string(), Some("0.7,0.3".to_string()))])]
  #[case::flag_with_space_separated_values("--lora-scaled /path/to/lora.bin 0.5", vec![("--lora-scaled".to_string(), Some("/path/to/lora.bin 0.5".to_string()))])]
  #[case::complex_key_value_pairs("--override-kv tokenizer.ggml.add_bos_token=bool:false", vec![("--override-kv".to_string(), Some("tokenizer.ggml.add_bos_token=bool:false".to_string()))])]
  #[case::quoted_values("--dry-sequence-breaker none", vec![("--dry-sequence-breaker".to_string(), Some("none".to_string()))])]
  #[case::comma_separated_devices("--device cuda:0,cuda:1", vec![("--device".to_string(), Some("cuda:0,cuda:1".to_string()))])]
  #[case::range_values("--cpu-range 0-7", vec![("--cpu-range".to_string(), Some("0-7".to_string()))])]
  #[case::file_paths("--model /path/to/model.gguf", vec![("--model".to_string(), Some("/path/to/model.gguf".to_string()))])]
  #[case::urls("--model-url https://example.com/model.gguf", vec![("--model-url".to_string(), Some("https://example.com/model.gguf".to_string()))])]
  #[case::negative_integer("-1", vec![])]
  #[case::negative_float("-0.5", vec![])]
  #[case::empty_string("", vec![])]
  #[case::whitespace_only("   \t\n  ", vec![])]
  #[case::non_flag_text("not a flag", vec![])]
  fn test_parse_args_from_string_single_cases(
    #[case] input: &str,
    #[case] expected: Vec<(String, Option<String>)>,
  ) {
    let result = parse_args_from_string(input);
    assert_eq!(result, expected);
  }

  #[rstest]
  #[case::multiple_long_flags(
    "--verbose --threads 4 --batch-size 512",
    vec![
      ("--verbose".to_string(), None),
      ("--threads".to_string(), Some("4".to_string())),
      ("--batch-size".to_string(), Some("512".to_string())),
    ]
  )]
  #[case::mixed_short_long_flags(
    "-t 8 --verbose -b 256",
    vec![
      ("-t".to_string(), Some("8".to_string())),
      ("--verbose".to_string(), None),
      ("-b".to_string(), Some("256".to_string())),
    ]
  )]
  #[case::comprehensive_mix(
    "--verbose --threads 8 --temp 0.7 --no-mmap --ctx-size 4096",
    vec![
      ("--verbose".to_string(), None),
      ("--threads".to_string(), Some("8".to_string())),
      ("--temp".to_string(), Some("0.7".to_string())),
      ("--no-mmap".to_string(), None),
      ("--ctx-size".to_string(), Some("4096".to_string())),
    ]
  )]
  fn test_parse_args_from_string_multiple_flags(
    #[case] input: &str,
    #[case] expected: Vec<(String, Option<String>)>,
  ) {
    let result = parse_args_from_string(input);
    assert_eq!(result, expected);
  }

  #[rstest]
  #[case::negative_values(
    "--seed -1 --predict -1",
    vec![
      ("--seed".to_string(), Some("-1".to_string())),
      ("--predict".to_string(), Some("-1".to_string())),
    ]
  )]
  #[case::floating_point_values(
    "--temp 0.8 --top-p 0.9",
    vec![
      ("--temp".to_string(), Some("0.8".to_string())),
      ("--top-p".to_string(), Some("0.9".to_string())),
    ]
  )]
  #[case::file_paths(
    "--model /path/to/model.gguf --lora /path/to/lora.bin",
    vec![
      ("--model".to_string(), Some("/path/to/model.gguf".to_string())),
      ("--lora".to_string(), Some("/path/to/lora.bin".to_string())),
    ]
  )]
  fn test_parse_args_from_string_special_values(
    #[case] input: &str,
    #[case] expected: Vec<(String, Option<String>)>,
  ) {
    let result = parse_args_from_string(input);
    assert_eq!(result, expected);
  }

  #[rstest]
  #[case::multiple_string_inputs(
    vec![
      "--verbose --threads 4".to_string(),
      "--batch-size 512".to_string(),
      "-ngl 32".to_string(),
    ],
    vec![
      ("--verbose".to_string(), None),
      ("--threads".to_string(), Some("4".to_string())),
      ("--batch-size".to_string(), Some("512".to_string())),
      ("-ngl".to_string(), Some("32".to_string())),
    ]
  )]
  #[case::single_string_multiple_args(
    vec!["--verbose".to_string(), "--threads 4".to_string(), "--temp 0.8".to_string(), "-b 512".to_string()],
    vec![
      ("--verbose".to_string(), None),
      ("--threads".to_string(), Some("4".to_string())),
      ("--temp".to_string(), Some("0.8".to_string())),
      ("-b".to_string(), Some("512".to_string())),
    ]
  )]
  #[case::cross_string_boundary(
    vec![
      "--verbose --threads".to_string(),
      "8 --temp".to_string(),
      "0.7".to_string(),
    ],
    vec![
      ("--verbose".to_string(), None),
      ("--threads".to_string(), Some("8".to_string())),
      ("--temp".to_string(), Some("0.7".to_string())),
    ]
  )]
  fn test_parse_args_from_strings(
    #[case] args: Vec<String>,
    #[case] expected: Vec<(String, Option<String>)>,
  ) {
    let result = parse_args_from_strings(&args);
    assert_eq!(result, expected);
  }

  #[rstest]
  #[case::complex_precedence(
    vec!["--verbose --threads 4 --batch-size 256".to_string(), "--temp 0.7".to_string()],
    vec!["--n-gpu-layers 999 --threads 8".to_string()],
    vec!["--threads 16".to_string(), "--top-p 0.9".to_string()],
    vec!["--threads 16", "--verbose", "--batch-size 256", "--temp 0.7", "--n-gpu-layers 999", "--top-p 0.9"],
    vec!["--threads 4", "--threads 8"]
  )]
  #[case::same_flag_different_sources(
    vec!["--threads 4".to_string()],
    vec!["--threads 8".to_string()],
    vec!["--threads 12".to_string()],
    vec!["--threads 12"],
    vec!["--threads 4", "--threads 8"]
  )]
  fn test_deduplication_precedence(
    #[case] setting_args: Vec<String>,
    #[case] variant_args: Vec<String>,
    #[case] alias_args: Vec<String>,
    #[case] should_contain: Vec<&str>,
    #[case] should_not_contain: Vec<&str>,
  ) {
    let result = merge_server_args(&setting_args, &variant_args, &alias_args);
    let result_str = result.join(" ");

    for expected in should_contain {
      assert!(
        result_str.contains(expected),
        "Result should contain '{}', but got: {}",
        expected,
        result_str
      );
    }

    for not_expected in should_not_contain {
      assert!(
        !result_str.contains(not_expected),
        "Result should not contain '{}', but got: {}",
        not_expected,
        result_str
      );
    }

    // For same flag test, ensure only one occurrence
    if setting_args.len() == 1 && variant_args.len() == 1 && alias_args.len() == 1 {
      let threads_count = result.iter().filter(|s| s.contains("--threads")).count();
      assert_eq!(threads_count, 1, "Should have exactly one --threads entry");
    }
  }

  #[test]
  fn test_output_format_consistency() {
    // Ensure the output format is consistent and usable
    let args = vec!["--verbose --threads 8 --temp 0.7".to_string()];
    let result = merge_server_args(&args, &[], &[]);

    // Each element should be a properly formatted flag
    for arg in &result {
      if arg.contains(' ') {
        // Should be "--flag value" format
        let parts: Vec<&str> = arg.split_whitespace().collect();
        assert!(parts.len() >= 2);
        assert!(parts[0].starts_with('-'));
      } else {
        // Should be just "--flag" format
        assert!(arg.starts_with('-'));
      }
    }

    // Should be able to join back into a command line
    let command_line = result.join(" ");
    assert!(command_line.contains("--verbose"));
    assert!(command_line.contains("--threads 8"));
    assert!(command_line.contains("--temp 0.7"));
  }

  #[rstest]
  #[case::realistic_server_args(
    vec!["--host 0.0.0.0 --port 8080 --ctx-size 4096".to_string()],
    vec!["--n-gpu-layers 35 --gpu-layers-draft 10".to_string()],
    vec!["--model /path/to/model.gguf".to_string(), "--threads 12 --batch-size 1024".to_string()],
    vec![
      "--host 0.0.0.0", "--port 8080", "--ctx-size 4096",
      "--n-gpu-layers 35", "--gpu-layers-draft 10",
      "--model /path/to/model.gguf", "--threads 12", "--batch-size 1024"
    ],
    vec![]
  )]
  #[case::complex_real_world_scenario(
    vec!["--verbose --host 127.0.0.1 --port 8080".to_string(), "--ctx-size 2048 --threads 4".to_string()],
    vec!["--n-gpu-layers 32 --tensor-split 0.8,0.2".to_string(), "--temp 0.7 --top-p 0.9".to_string()],
    vec![
      "--model /models/llama-7b.gguf".to_string(),
      "--threads 8 --batch-size 512".to_string(),
      "--override-kv tokenizer.add_bos=bool:true".to_string()
    ],
    vec![
      "--verbose", "--host 127.0.0.1", "--port 8080", "--ctx-size 2048",
      "--n-gpu-layers 32", "--tensor-split 0.8,0.2", "--temp 0.7", "--top-p 0.9",
      "--model /models/llama-7b.gguf", "--threads 8", "--batch-size 512",
      "--override-kv tokenizer.add_bos=bool:true"
    ],
    vec!["--threads 4"]
  )]
  fn test_real_world_scenarios(
    #[case] setting_args: Vec<String>,
    #[case] variant_args: Vec<String>,
    #[case] alias_args: Vec<String>,
    #[case] should_contain: Vec<&str>,
    #[case] should_not_contain: Vec<&str>,
  ) {
    let result = merge_server_args(&setting_args, &variant_args, &alias_args);
    let result_str = result.join(" ");

    for expected in should_contain {
      assert!(
        result_str.contains(expected),
        "Result should contain '{}', but got: {}",
        expected,
        result_str
      );
    }

    for not_expected in should_not_contain {
      assert!(
        !result_str.contains(not_expected),
        "Result should not contain '{}', but got: {}",
        not_expected,
        result_str
      );
    }
  }

  #[test]
  fn test_special_llama_server_flags() {
    // Test various special flag patterns from llama-server
    let args = vec![
      "--logit-bias 15043+1 --logit-bias 15044-1".to_string(),
      "--override-kv tokenizer.ggml.add_bos_token=bool:false".to_string(),
      "--lora-scaled /path/to/lora.bin 0.5".to_string(),
    ];

    let result = parse_args_from_strings(&args);

    // Should handle parsing correctly (last occurrence will win in dedup)
    assert!(result.iter().any(|(k, v)| k == "--logit-bias"
      && v.as_ref().is_some_and(|val| val.contains("15043+1")
        || val.contains("15044-1"))));
    assert!(result.iter().any(|(k, v)| k == "--override-kv"
      && v.as_ref().is_some_and(|val| val
        .contains("tokenizer.ggml.add_bos_token=bool:false"))));
    assert!(result.iter().any(|(k, v)| k == "--lora-scaled"
      && v
        .as_ref()
        .is_some_and(|val| val.contains("/path/to/lora.bin 0.5"))));
  }

  #[rstest]
  #[case::boolean_verbose("--verbose", vec![("--verbose".to_string(), None)])]
  #[case::boolean_help("--help", vec![("--help".to_string(), None)])]
  #[case::boolean_no_mmap("--no-mmap", vec![("--no-mmap".to_string(), None)])]
  #[case::single_value_threads("--threads 8", vec![("--threads".to_string(), Some("8".to_string()))])]
  #[case::single_value_ctx_size("--ctx-size 4096", vec![("--ctx-size".to_string(), Some("4096".to_string()))])]
  #[case::single_value_temp("--temp 0.8", vec![("--temp".to_string(), Some("0.8".to_string()))])]
  #[case::negative_seed("--seed -1", vec![("--seed".to_string(), Some("-1".to_string()))])]
  #[case::negative_predict("--predict -1", vec![("--predict".to_string(), Some("-1".to_string()))])]
  #[case::comma_separated_device("--device cuda:0,cuda:1", vec![("--device".to_string(), Some("cuda:0,cuda:1".to_string()))])]
  #[case::comma_separated_tensor_split("--tensor-split 0.7,0.3", vec![("--tensor-split".to_string(), Some("0.7,0.3".to_string()))])]
  #[case::range_cpu_range("--cpu-range 0-7", vec![("--cpu-range".to_string(), Some("0-7".to_string()))])]
  #[case::file_path_model("--model /path/to/model.gguf", vec![("--model".to_string(), Some("/path/to/model.gguf".to_string()))])]
  #[case::file_path_lora("--lora /path/to/lora.bin", vec![("--lora".to_string(), Some("/path/to/lora.bin".to_string()))])]
  #[case::url_model("--model-url https://example.com/model.gguf", vec![("--model-url".to_string(), Some("https://example.com/model.gguf".to_string()))])]
  #[case::complex_key_value("--override-kv tokenizer.ggml.add_bos_token=bool:false", vec![("--override-kv".to_string(), Some("tokenizer.ggml.add_bos_token=bool:false".to_string()))])]
  #[case::space_separated_lora_scaled("--lora-scaled /path/to/lora.bin 0.5", vec![("--lora-scaled".to_string(), Some("/path/to/lora.bin 0.5".to_string()))])]
  #[case::space_separated_control_vector("--control-vector-layer-range 10 20", vec![("--control-vector-layer-range".to_string(), Some("10 20".to_string()))])]
  #[case::short_flag_t("-t 8", vec![("-t".to_string(), Some("8".to_string()))])]
  #[case::short_flag_b("-b 512", vec![("-b".to_string(), Some("512".to_string()))])]
  #[case::short_flag_v("-v", vec![("-v".to_string(), None)])]
  fn test_comprehensive_llama_server_patterns(
    #[case] input: &str,
    #[case] expected: Vec<(String, Option<String>)>,
  ) {
    let result = parse_args_from_string(input);
    assert_eq!(result, expected);
  }

  #[rstest]
  #[case::empty_string("", vec![])]
  #[case::only_whitespace("   \t\n  ", vec![])]
  #[case::non_flag_tokens_only("not a flag", vec![])]
  #[case::mixed_non_flag_and_flag_tokens(
    "not --threads 8 also not",
    vec![("--threads".to_string(), Some("8 also not".to_string()))]
  )]
  #[case::flag_at_end_with_no_value(
    "--threads 8 --verbose",
    vec![
      ("--threads".to_string(), Some("8".to_string())),
      ("--verbose".to_string(), None),
    ]
  )]
  fn test_edge_cases(#[case] input: &str, #[case] expected: Vec<(String, Option<String>)>) {
    let result = parse_args_from_string(input);
    assert_eq!(result, expected);
  }

  #[rstest]
  #[case::long_verbose("--verbose", true)]
  #[case::long_threads("--threads", true)]
  #[case::short_t("-t", true)]
  #[case::short_v("-v", true)]
  #[case::long_ctx_size("--ctx-size", true)]
  #[case::short_ngl("-ngl", true)]
  #[case::negative_int("-1", false)]
  #[case::negative_large_int("-123", false)]
  #[case::negative_float("-0.5", false)]
  #[case::negative_large_float("-999.99", false)]
  #[case::just_dash("-", false)]
  #[case::empty_string("", false)]
  #[case::regular_text("regular_text", false)]
  #[case::positive_number("123", false)]
  #[case::alpha_after_dash("-abc", true)]
  #[case::digit_then_alpha("-1abc", true)]
  fn test_is_flag_function(#[case] input: &str, #[case] expected: bool) {
    assert_eq!(is_flag(input), expected);
  }
}
