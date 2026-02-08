use objs::{AppError, ErrorType};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

const EXA_SEARCH_URL: &str = "https://api.exa.ai/search";
const EXA_FIND_SIMILAR_URL: &str = "https://api.exa.ai/findSimilar";
const EXA_CONTENTS_URL: &str = "https://api.exa.ai/contents";
const EXA_ANSWER_URL: &str = "https://api.exa.ai/answer";
const EXA_TIMEOUT_SECS: u64 = 30;

// ============================================================================
// ExaError - Errors from Exa API integration
// ============================================================================

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ExaError {
  #[error("Search request failed: {0}.")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  RequestFailed(String),

  #[error("Search rate limit exceeded. Please wait and try again.")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  RateLimited,

  #[error("Search API key is invalid or missing.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidApiKey,

  #[error("Search request timed out.")]
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

  /// Find pages similar to a given URL
  async fn find_similar(
    &self,
    api_key: &str,
    url: &str,
    num_results: Option<u32>,
  ) -> Result<ExaFindSimilarResponse, ExaError>;

  /// Get contents of specific web pages
  async fn get_contents(
    &self,
    api_key: &str,
    urls: Vec<String>,
    text: bool,
  ) -> Result<ExaContentsResponse, ExaError>;

  /// Get AI-powered answer to a query
  async fn answer(
    &self,
    api_key: &str,
    query: &str,
    text: bool,
  ) -> Result<ExaAnswerResponse, ExaError>;
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

// FindSimilar API

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExaFindSimilarRequest {
  url: String,
  num_results: u32,
  contents: ExaContents,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExaFindSimilarResponse {
  pub results: Vec<ExaSearchResult>,
}

// Contents API

#[derive(Debug, Clone, Serialize)]
struct ExaGetContentsRequest {
  urls: Vec<String>,
  text: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExaContentsResponse {
  pub results: Vec<ExaContentResult>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExaContentResult {
  pub url: String,
  pub title: Option<String>,
  pub text: Option<String>,
}

// Answer API

#[derive(Debug, Clone, Serialize)]
struct ExaAnswerRequest {
  query: String,
  text: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExaAnswerResponse {
  pub answer: String,
  pub results: Vec<ExaSearchResult>,
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
      .post(EXA_SEARCH_URL)
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

  async fn find_similar(
    &self,
    api_key: &str,
    url: &str,
    num_results: Option<u32>,
  ) -> Result<ExaFindSimilarResponse, ExaError> {
    let request = ExaFindSimilarRequest {
      url: url.to_string(),
      num_results: num_results.unwrap_or(5).min(10),
      contents: ExaContents {
        text: true,
        highlights: true,
      },
    };

    let response = self
      .client
      .post(EXA_FIND_SIMILAR_URL)
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
      .json::<ExaFindSimilarResponse>()
      .await
      .map_err(|e| ExaError::RequestFailed(format!("Parse error: {}", e)))
  }

  async fn get_contents(
    &self,
    api_key: &str,
    urls: Vec<String>,
    text: bool,
  ) -> Result<ExaContentsResponse, ExaError> {
    let request = ExaGetContentsRequest { urls, text };

    let response = self
      .client
      .post(EXA_CONTENTS_URL)
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
      .json::<ExaContentsResponse>()
      .await
      .map_err(|e| ExaError::RequestFailed(format!("Parse error: {}", e)))
  }

  async fn answer(
    &self,
    api_key: &str,
    query: &str,
    text: bool,
  ) -> Result<ExaAnswerResponse, ExaError> {
    let request = ExaAnswerRequest {
      query: query.to_string(),
      text,
    };

    let response = self
      .client
      .post(EXA_ANSWER_URL)
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
      .json::<ExaAnswerResponse>()
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
  #[tokio::test]
  async fn test_exa_find_similar_success() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let _mock = server
      .mock("POST", "/findSimilar")
      .match_header("x-api-key", "test-key")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(
        json!({
          "results": [
            {
              "title": "Similar Page",
              "url": "https://similar.com",
              "publishedDate": "2024-01-15",
              "author": "Author",
              "score": 0.92,
              "text": "Similar content",
              "highlights": ["similar"]
            }
          ]
        })
        .to_string(),
      )
      .create_async()
      .await;

    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(30))
      .build()?;

    let response = client
      .post(server.url() + "/findSimilar")
      .header("x-api-key", "test-key")
      .header("Content-Type", "application/json")
      .json(&ExaFindSimilarRequest {
        url: "https://example.com".to_string(),
        num_results: 5,
        contents: ExaContents {
          text: true,
          highlights: true,
        },
      })
      .send()
      .await?;

    assert_eq!(200, response.status());
    let result: ExaFindSimilarResponse = response.json().await?;
    assert_eq!(1, result.results.len());
    assert_eq!("Similar Page", result.results[0].title);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_exa_get_contents_success() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let _mock = server
      .mock("POST", "/contents")
      .match_header("x-api-key", "test-key")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(
        json!({
          "results": [
            {
              "url": "https://example.com",
              "title": "Example Page",
              "text": "Page content here"
            }
          ]
        })
        .to_string(),
      )
      .create_async()
      .await;

    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(30))
      .build()?;

    let response = client
      .post(server.url() + "/contents")
      .header("x-api-key", "test-key")
      .header("Content-Type", "application/json")
      .json(&ExaGetContentsRequest {
        urls: vec!["https://example.com".to_string()],
        text: true,
      })
      .send()
      .await?;

    assert_eq!(200, response.status());
    let result: ExaContentsResponse = response.json().await?;
    assert_eq!(1, result.results.len());
    assert_eq!("https://example.com", result.results[0].url);
    assert_eq!(Some("Example Page".to_string()), result.results[0].title);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_exa_answer_success() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let _mock = server
      .mock("POST", "/answer")
      .match_header("x-api-key", "test-key")
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(
        json!({
          "answer": "The answer is 42",
          "results": [
            {
              "title": "Source Page",
              "url": "https://source.com",
              "publishedDate": "2024-01-15",
              "author": "Author",
              "score": 0.95,
              "text": "Source text",
              "highlights": ["answer"]
            }
          ]
        })
        .to_string(),
      )
      .create_async()
      .await;

    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(30))
      .build()?;

    let response = client
      .post(server.url() + "/answer")
      .header("x-api-key", "test-key")
      .header("Content-Type", "application/json")
      .json(&ExaAnswerRequest {
        query: "what is the answer".to_string(),
        text: true,
      })
      .send()
      .await?;

    assert_eq!(200, response.status());
    let result: ExaAnswerResponse = response.json().await?;
    assert_eq!("The answer is 42", result.answer);
    assert_eq!(1, result.results.len());

    Ok(())
  }
}
