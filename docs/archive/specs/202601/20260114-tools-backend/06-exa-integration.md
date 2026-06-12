# Exa API Integration

> Layer: `services` crate | Status: ✅ Complete

**File**: `crates/services/src/exa_service.rs`

## Exa Toolset Overview

The `builtin-exa-web-search` toolset provides 4 tools powered by the Exa AI API:

| Tool | Exa Endpoint | Description |
|------|--------------|-------------|
| `search` | `POST /search` | Semantic web search |
| `find_similar` | `POST /findSimilar` | Find pages similar to URL |
| `get_contents` | `POST /contents` | Get full page contents |
| `answer` | `POST /answer` | AI-generated answer from search |

## API Configuration

**Base URL**: `https://api.exa.ai`
**Timeout**: 30 seconds
**Authentication**: `x-api-key` header

## Tool Definitions (OpenAI Format)

### search

```json
{
  "type": "function",
  "function": {
    "name": "toolset__builtin-exa-web-search__search",
    "description": "Search the web for current information using Exa AI semantic search. Returns relevant web pages with titles, URLs, and content snippets.",
    "parameters": {
      "type": "object",
      "properties": {
        "query": {
          "type": "string",
          "description": "The search query to find relevant web pages"
        },
        "num_results": {
          "type": "integer",
          "description": "Number of results to return (default: 5, max: 10)",
          "minimum": 1,
          "maximum": 10,
          "default": 5
        }
      },
      "required": ["query"]
    }
  }
}
```

### find_similar

```json
{
  "type": "function",
  "function": {
    "name": "toolset__builtin-exa-web-search__find_similar",
    "description": "Find web pages similar to a given URL. Useful for finding related content, competitor pages, or similar articles.",
    "parameters": {
      "type": "object",
      "properties": {
        "url": {
          "type": "string",
          "description": "The URL to find similar pages for"
        },
        "num_results": {
          "type": "integer",
          "description": "Number of similar results to return (default: 5, max: 10)",
          "default": 5
        },
        "exclude_source_domain": {
          "type": "boolean",
          "description": "Exclude results from the same domain as the input URL",
          "default": true
        }
      },
      "required": ["url"]
    }
  }
}
```

### get_contents

```json
{
  "type": "function",
  "function": {
    "name": "toolset__builtin-exa-web-search__get_contents",
    "description": "Get the full contents of web pages by their URLs. Returns cleaned text content from the pages.",
    "parameters": {
      "type": "object",
      "properties": {
        "urls": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Array of URLs to get contents from (max: 10)",
          "maxItems": 10
        },
        "max_characters": {
          "type": "integer",
          "description": "Maximum characters to return per page (default: 3000)",
          "default": 3000
        }
      },
      "required": ["urls"]
    }
  }
}
```

### answer

```json
{
  "type": "function",
  "function": {
    "name": "toolset__builtin-exa-web-search__answer",
    "description": "Get an AI-generated answer to a question based on web search results. Combines search with answer synthesis.",
    "parameters": {
      "type": "object",
      "properties": {
        "query": {
          "type": "string",
          "description": "The question to answer using web search"
        },
        "num_results": {
          "type": "integer",
          "description": "Number of search results to use for generating the answer (default: 5)",
          "default": 5
        }
      },
      "required": ["query"]
    }
  }
}
```

## ExaService Implementation

```rust
// crates/services/src/exa_service.rs

const EXA_BASE_URL: &str = "https://api.exa.ai";
const EXA_TIMEOUT_SECS: u64 = 30;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait ExaService: std::fmt::Debug + Send + Sync {
    async fn search(
        &self,
        api_key: &str,
        query: &str,
        num_results: Option<u32>,
    ) -> Result<ExaSearchResponse, ExaError>;

    async fn find_similar(
        &self,
        api_key: &str,
        url: &str,
        num_results: Option<u32>,
        exclude_source_domain: Option<bool>,
    ) -> Result<ExaSearchResponse, ExaError>;

    async fn get_contents(
        &self,
        api_key: &str,
        urls: Vec<String>,
        max_characters: Option<u32>,
    ) -> Result<ExaContentsResponse, ExaError>;

    async fn answer(
        &self,
        api_key: &str,
        query: &str,
        num_results: Option<u32>,
    ) -> Result<ExaAnswerResponse, ExaError>;
}

#[derive(Debug)]
pub struct DefaultExaService {
    client: reqwest::Client,
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
```

## Response Types

```rust
#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExaSearchResponse {
    pub results: Vec<ExaSearchResult>,
    pub autoprompt_string: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExaContentsResponse {
    pub results: Vec<ExaContentResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExaContentResult {
    pub url: String,
    pub title: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExaAnswerResponse {
    pub answer: String,
    pub sources: Vec<ExaSearchResult>,
}
```

## Tool Execution Response Format

Tool execution returns result as JSON for LLM:

**Success:**
```json
{
  "tool_call_id": "call_abc123",
  "result": {
    "results": [
      { "title": "Page Title", "url": "https://example.com", "snippet": "..." }
    ],
    "query_used": "optimized query"
  }
}
```

**Error:**
```json
{
  "tool_call_id": "call_abc123",
  "result": null,
  "error": "exa_rate_limited: Rate limit exceeded. Please try again later."
}
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ExaError {
    #[error("exa_request_failed: {0}")]
    #[error_meta(error_type = ErrorType::BadGateway)]
    RequestFailed(String),

    #[error("exa_rate_limited")]
    #[error_meta(error_type = ErrorType::TooManyRequests)]
    RateLimited,

    #[error("exa_invalid_api_key")]
    #[error_meta(error_type = ErrorType::Unauthorized)]
    InvalidApiKey,

    #[error("exa_timeout")]
    #[error_meta(error_type = ErrorType::GatewayTimeout)]
    Timeout,
}
```

HTTP status mapping:
- 401 → `InvalidApiKey`
- 429 → `RateLimited`
- Timeout → `Timeout`
- Other errors → `RequestFailed`

## Testing

### Unit Tests (mock Exa)
```rust
#[tokio::test]
async fn test_search_success() {
    let mut mock = MockExaService::new();
    mock.expect_search()
        .with(eq("test-key"), eq("rust programming"), eq(Some(5)))
        .returning(|_, _, _| Ok(ExaSearchResponse {
            results: vec![...],
            autoprompt_string: Some("rust programming language".to_string()),
        }));
}
```

### E2E Tests (real API)
- Provide `EXA_API_KEY` environment variable
- MSW mock for frontend tests
