//! Tool execution delegation to external services (Exa, MCP, etc.).

use super::ToolsetError;
use crate::exa_service::ExaService;
use objs::{ToolsetExecutionRequest, ToolsetExecutionResponse};
use serde_json::json;

/// Execute an Exa toolset method (search, findSimilar, contents, answer).
pub(super) async fn execute_exa(
  exa_service: &dyn ExaService,
  api_key: &str,
  method: &str,
  request: ToolsetExecutionRequest,
) -> Result<ToolsetExecutionResponse, ToolsetError> {
  match method {
    "search" => {
      let query = request.params["query"]
        .as_str()
        .ok_or_else(|| ToolsetError::ExecutionFailed("Missing 'query' parameter".to_string()))?;
      let num_results = request.params["num_results"].as_u64().map(|n| n as u32);

      match exa_service.search(api_key, query, num_results).await {
        Ok(response) => {
          let results: Vec<_> = response
            .results
            .into_iter()
            .map(|r| {
              json!({
                "title": r.title,
                "url": r.url,
                "snippet": r.highlights.join(" ... "),
                "published_date": r.published_date,
                "score": r.score,
              })
            })
            .collect();

          Ok(ToolsetExecutionResponse {
            result: Some(json!({
              "results": results,
              "query_used": response.autoprompt_string,
            })),
            error: None,
          })
        }
        Err(e) => Ok(ToolsetExecutionResponse {
          result: None,
          error: Some(e.to_string()),
        }),
      }
    }
    "findSimilar" => {
      let url = request.params["url"]
        .as_str()
        .ok_or_else(|| ToolsetError::ExecutionFailed("Missing 'url' parameter".to_string()))?;
      let num_results = request.params["num_results"].as_u64().map(|n| n as u32);

      match exa_service.find_similar(api_key, url, num_results).await {
        Ok(response) => {
          let results: Vec<_> = response
            .results
            .into_iter()
            .map(|r| {
              json!({
                "title": r.title,
                "url": r.url,
                "snippet": r.highlights.join(" ... "),
                "published_date": r.published_date,
                "score": r.score,
              })
            })
            .collect();

          Ok(ToolsetExecutionResponse {
            result: Some(json!({"results": results})),
            error: None,
          })
        }
        Err(e) => Ok(ToolsetExecutionResponse {
          result: None,
          error: Some(e.to_string()),
        }),
      }
    }
    "contents" => {
      let urls_array = request.params["urls"]
        .as_array()
        .ok_or_else(|| ToolsetError::ExecutionFailed("Missing 'urls' parameter".to_string()))?;
      let urls: Vec<String> = urls_array
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
      let text = request
        .params
        .get("text")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

      match exa_service.get_contents(api_key, urls, text).await {
        Ok(response) => {
          let results: Vec<_> = response
            .results
            .into_iter()
            .map(|r| {
              json!({
                "url": r.url,
                "title": r.title,
                "text": r.text,
              })
            })
            .collect();

          Ok(ToolsetExecutionResponse {
            result: Some(json!({"results": results})),
            error: None,
          })
        }
        Err(e) => Ok(ToolsetExecutionResponse {
          result: None,
          error: Some(e.to_string()),
        }),
      }
    }
    "answer" => {
      let query = request.params["query"]
        .as_str()
        .ok_or_else(|| ToolsetError::ExecutionFailed("Missing 'query' parameter".to_string()))?;
      let text = request
        .params
        .get("text")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

      match exa_service.answer(api_key, query, text).await {
        Ok(response) => {
          let sources: Vec<_> = response
            .results
            .into_iter()
            .map(|r| {
              json!({
                "title": r.title,
                "url": r.url,
                "snippet": r.highlights.join(" ... "),
              })
            })
            .collect();

          Ok(ToolsetExecutionResponse {
            result: Some(json!({
              "answer": response.answer,
              "sources": sources,
            })),
            error: None,
          })
        }
        Err(e) => Ok(ToolsetExecutionResponse {
          result: None,
          error: Some(e.to_string()),
        }),
      }
    }
    _ => Err(ToolsetError::MethodNotFound(format!(
      "Unknown method: {}",
      method
    ))),
  }
}
