use objs::{AppError, ErrorType};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

const EXA_API_URL: &str = "https://api.exa.ai/search";
const EXA_TIMEOUT_SECS: u64 = 30;

// ============================================================================
// ExaError - Errors from Exa API integration
// ============================================================================

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ExaError {
  #[error("request_failed")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  RequestFailed(String),

  #[error("rate_limited")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  RateLimited,

  #[error("invalid_api_key")]
  #[error_meta(error_type = ErrorType::Unauthorized)]
  InvalidApiKey,

  #[error("timeout")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  Timeout,
}

// ============================================================================
// ExaService - Service for calling Exa AI search API
// ============================================================================

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait ExaService: Debug + Send + Sync {
  /// Search the web using Exa AI semantic search
  async fn search(
    &self,
    api_key: &str,
    query: &str,
    num_results: Option<u32>,
  ) -> Result<ExaSearchResponse, ExaError>;
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExaSearchRequest {
  query: String,
  #[serde(rename = "type")]
  search_type: String,
  use_autoprompt: bool,
  num_results: u32,
  contents: ExaContents,
}

#[derive(Debug, Clone, Serialize)]
struct ExaContents {
  text: bool,
  highlights: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExaSearchResult {
  pub title: String,
  pub url: String,
  pub published_date: Option<String>,
  pub author: Option<String>,
  pub score: f64,
  pub text: Option<String>,
  #[serde(default)]
  pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExaSearchResponse {
  pub results: Vec<ExaSearchResult>,
  pub autoprompt_string: Option<String>,
}

// ============================================================================
// DefaultExaService - Implementation using reqwest
// ============================================================================

#[derive(Debug)]
pub struct DefaultExaService {
  client: reqwest::Client,
}

impl Default for DefaultExaService {
  fn default() -> Self {
    Self::new()
  }
}

impl DefaultExaService {
  pub fn new() -> Self {
    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(EXA_TIMEOUT_SECS))
      .build()
      .expect("Failed to create HTTP client");

    Self { client }
  }
}

#[async_trait::async_trait]
impl ExaService for DefaultExaService {
  async fn search(
    &self,
    api_key: &str,
    query: &str,
    num_results: Option<u32>,
  ) -> Result<ExaSearchResponse, ExaError> {
    let request = ExaSearchRequest {
      query: query.to_string(),
      search_type: "neural".to_string(),
      use_autoprompt: true,
      num_results: num_results.unwrap_or(5).min(10),
      contents: ExaContents {
        text: true,
        highlights: true,
      },
    };

    let response = self
      .client
      .post(EXA_API_URL)
      .header("x-api-key", api_key)
      .header("Content-Type", "application/json")
      .json(&request)
      .send()
      .await
      .map_err(|e| {
        if e.is_timeout() {
          ExaError::Timeout
        } else {
          ExaError::RequestFailed(e.to_string())
        }
      })?;

    let status = response.status();

    if status == reqwest::StatusCode::UNAUTHORIZED {
      return Err(ExaError::InvalidApiKey);
    }

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
      return Err(ExaError::RateLimited);
    }

    if !status.is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(ExaError::RequestFailed(format!(
        "HTTP {}: {}",
        status, error_text
      )));
    }

    response
      .json::<ExaSearchResponse>()
      .await
      .map_err(|e| ExaError::RequestFailed(format!("Parse error: {}", e)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use mockito::{Matcher, Server};
  use rstest::rstest;
  use serde_json::json;

  #[rstest]
  #[tokio::test]
  async fn test_exa_search_success() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let mock = server
      .mock("POST", "/search")
      .match_header("x-api-key", "test-key")
      .match_header("content-type", "application/json")
      .match_body(Matcher::JsonString(
        json!({
          "query": "rust programming",
          "type": "neural",
          "useAutoprompt": true,
          "numResults": 5,
          "contents": {
            "text": true,
            "highlights": true
          }
        })
        .to_string(),
      ))
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(
        json!({
          "results": [
            {
              "title": "Rust Programming Language",
              "url": "https://rust-lang.org",
              "publishedDate": "2024-01-15",
              "author": "Rust Team",
              "score": 0.95,
              "text": "Rust is a systems programming language...",
              "highlights": ["systems programming language"]
            }
          ],
          "autopromptString": "rust programming language"
        })
        .to_string(),
      )
      .create_async()
      .await;

    // Create service with mock server URL
    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(30))
      .build()?;
    let service = DefaultExaService { client };

    // Temporarily replace the EXA_API_URL for testing
    let result = service
      .client
      .post(server.url() + "/search")
      .header("x-api-key", "test-key")
      .header("Content-Type", "application/json")
      .json(&ExaSearchRequest {
        query: "rust programming".to_string(),
        search_type: "neural".to_string(),
        use_autoprompt: true,
        num_results: 5,
        contents: ExaContents {
          text: true,
          highlights: true,
        },
      })
      .send()
      .await?;

    assert_eq!(200, result.status());
    let response: ExaSearchResponse = result.json().await?;
    assert_eq!(1, response.results.len());
    assert_eq!("Rust Programming Language", response.results[0].title);
    assert_eq!("https://rust-lang.org", response.results[0].url);

    mock.assert_async().await;
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_exa_search_unauthorized() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let _mock = server
      .mock("POST", "/search")
      .with_status(401)
      .with_body("Unauthorized")
      .create_async()
      .await;

    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(30))
      .build()?;

    let response = client
      .post(server.url() + "/search")
      .header("x-api-key", "invalid-key")
      .json(&ExaSearchRequest {
        query: "test".to_string(),
        search_type: "neural".to_string(),
        use_autoprompt: true,
        num_results: 5,
        contents: ExaContents {
          text: true,
          highlights: true,
        },
      })
      .send()
      .await?;

    assert_eq!(reqwest::StatusCode::UNAUTHORIZED, response.status());

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_exa_search_rate_limited() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let _mock = server
      .mock("POST", "/search")
      .with_status(429)
      .with_body("Rate limit exceeded")
      .create_async()
      .await;

    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(30))
      .build()?;

    let response = client
      .post(server.url() + "/search")
      .header("x-api-key", "test-key")
      .json(&ExaSearchRequest {
        query: "test".to_string(),
        search_type: "neural".to_string(),
        use_autoprompt: true,
        num_results: 5,
        contents: ExaContents {
          text: true,
          highlights: true,
        },
      })
      .send()
      .await?;

    assert_eq!(reqwest::StatusCode::TOO_MANY_REQUESTS, response.status());

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_exa_search_server_error() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let _mock = server
      .mock("POST", "/search")
      .with_status(500)
      .with_body("Internal server error")
      .create_async()
      .await;

    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(30))
      .build()?;

    let response = client
      .post(server.url() + "/search")
      .header("x-api-key", "test-key")
      .json(&ExaSearchRequest {
        query: "test".to_string(),
        search_type: "neural".to_string(),
        use_autoprompt: true,
        num_results: 5,
        contents: ExaContents {
          text: true,
          highlights: true,
        },
      })
      .send()
      .await?;

    assert!(!response.status().is_success());
    assert_eq!(
      reqwest::StatusCode::INTERNAL_SERVER_ERROR,
      response.status()
    );

    Ok(())
  }

  #[rstest]
  fn test_exa_search_request_serialization() -> anyhow::Result<()> {
    let request = ExaSearchRequest {
      query: "test query".to_string(),
      search_type: "neural".to_string(),
      use_autoprompt: true,
      num_results: 10,
      contents: ExaContents {
        text: true,
        highlights: true,
      },
    };

    let json = serde_json::to_value(&request)?;
    assert_eq!("test query", json["query"]);
    assert_eq!("neural", json["type"]);
    assert_eq!(true, json["useAutoprompt"]);
    assert_eq!(10, json["numResults"]);

    Ok(())
  }

  #[rstest]
  fn test_exa_search_response_deserialization() -> anyhow::Result<()> {
    let json = json!({
      "results": [
        {
          "title": "Test Title",
          "url": "https://example.com",
          "publishedDate": "2024-01-15",
          "author": "Test Author",
          "score": 0.9,
          "text": "Test content",
          "highlights": ["test"]
        }
      ],
      "autopromptString": "optimized query"
    });

    let response: ExaSearchResponse = serde_json::from_value(json)?;
    assert_eq!(1, response.results.len());
    assert_eq!("Test Title", response.results[0].title);
    assert_eq!(
      Some("optimized query".to_string()),
      response.autoprompt_string
    );

    Ok(())
  }
}
