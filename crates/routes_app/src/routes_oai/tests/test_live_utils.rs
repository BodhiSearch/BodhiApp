#![allow(unused)]
use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use serde_json::{json, Value};

/// Get weather tool for testing tool calling functionality
pub fn get_weather_tool() -> Vec<ChatCompletionTools> {
  vec![ChatCompletionTools::Function(ChatCompletionTool {
    function: FunctionObjectArgs::default()
      .name("get_current_temperature")
      .description("Gets the current temperature for a given location.")
      .parameters(json!({
        "type": "object",
        "properties": {
          "location": {
            "type": "string",
            "description": "The city name, e.g. San Francisco"
          }
        },
        "required": ["location"]
      }))
      .build()
      .unwrap(),
  })]
}

/// Parse SSE stream and extract tool calls from streaming response
pub fn parse_streaming_tool_calls(response_text: &str) -> (Vec<Value>, String) {
  let mut tool_call_map: std::collections::HashMap<i64, Value> = std::collections::HashMap::new();
  let mut finish_reason = String::new();

  for line in response_text.lines() {
    if line.is_empty() || line == "data: [DONE]" {
      continue;
    }
    if let Some(data) = line.strip_prefix("data: ") {
      if let Ok(chunk) = serde_json::from_str::<Value>(data) {
        if let Some(choices) = chunk["choices"].as_array() {
          for choice in choices {
            // Check for finish_reason
            if let Some(reason) = choice["finish_reason"].as_str() {
              finish_reason = reason.to_string();
            }

            // Check for tool_calls delta
            if let Some(delta_tool_calls) = choice["delta"]["tool_calls"].as_array() {
              for tc in delta_tool_calls {
                let index = tc["index"].as_i64().unwrap_or(0);

                if let std::collections::hash_map::Entry::Vacant(e) = tool_call_map.entry(index) {
                  // Initialize new tool call
                  e.insert(json!({
                      "id": tc["id"].as_str().unwrap_or(""),
                      "type": "function",
                      "function": {
                        "name": tc["function"]["name"].as_str().unwrap_or(""),
                        "arguments": tc["function"]["arguments"].as_str().unwrap_or("")
                      }
                    }));
                } else {
                  // Accumulate arguments
                  let existing = tool_call_map.get_mut(&index).unwrap();
                  if let Some(args) = tc["function"]["arguments"].as_str() {
                    let current_args = existing["function"]["arguments"].as_str().unwrap_or("");
                    existing["function"]["arguments"] =
                      Value::String(format!("{}{}", current_args, args));
                  }
                  // Update id if present
                  if let Some(id) = tc["id"].as_str() {
                    if !id.is_empty() {
                      existing["id"] = Value::String(id.to_string());
                    }
                  }
                  // Update name if present
                  if let Some(name) = tc["function"]["name"].as_str() {
                    if !name.is_empty() {
                      existing["function"]["name"] = Value::String(name.to_string());
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }

  // Convert map to vector, sorted by index
  let mut tool_calls: Vec<Value> = Vec::new();
  let mut indices: Vec<i64> = tool_call_map.keys().cloned().collect();
  indices.sort();
  for idx in indices {
    if let Some(tc) = tool_call_map.remove(&idx) {
      tool_calls.push(tc);
    }
  }

  (tool_calls, finish_reason)
}

/// Parse SSE stream and extract content from streaming response
pub fn parse_streaming_content(response_text: &str) -> (String, String) {
  let mut content = String::new();
  let mut finish_reason = String::new();

  for line in response_text.lines() {
    if line.is_empty() || line == "data: [DONE]" {
      continue;
    }
    if let Some(data) = line.strip_prefix("data: ") {
      if let Ok(chunk) = serde_json::from_str::<Value>(data) {
        if let Some(choices) = chunk["choices"].as_array() {
          for choice in choices {
            if let Some(reason) = choice["finish_reason"].as_str() {
              finish_reason = reason.to_string();
            }
            if let Some(delta_content) = choice["delta"]["content"].as_str() {
              content.push_str(delta_content);
            }
          }
        }
      }
    }
  }

  (content, finish_reason)
}
