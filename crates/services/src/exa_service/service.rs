use objs::{AppError, ErrorType};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
  base_url: String,
}

impl Default for DefaultExaService {
  fn default() -> Self {
    Self::new()
  }
}

impl DefaultExaService {
  pub fn new() -> Self {
    Self::with_base_url("https://api.exa.ai".to_string())
  }

  pub fn with_base_url(base_url: String) -> Self {
    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(EXA_TIMEOUT_SECS))
      .build()
      .expect("Failed to create HTTP client");

    Self { client, base_url }
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
      .post(format!("{}/search", self.base_url))
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
      .post(format!("{}/findSimilar", self.base_url))
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
      .post(format!("{}/contents", self.base_url))
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
      .post(format!("{}/answer", self.base_url))
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
