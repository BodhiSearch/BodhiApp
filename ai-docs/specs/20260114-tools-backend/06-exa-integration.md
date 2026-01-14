# Exa API Integration

> Layer: `services` crate | Status: ✅ Complete (6 tests passing)

**File**: `crates/services/src/exa_service.rs` (331 lines)

## Exa Search API

**Endpoint**: `POST https://api.exa.ai/search`

**Headers**:
```
x-api-key: <API_KEY>
Content-Type: application/json
```

**Request Body**:
```json
{
  "query": "search query",
  "type": "neural",
  "useAutoprompt": true,
  "numResults": 5,
  "contents": {
    "text": true,
    "highlights": true
  }
}
```

**Response**:
```json
{
  "results": [
    {
      "title": "Page Title",
      "url": "https://example.com",
      "publishedDate": "2024-01-15",
      "author": "Author Name",
      "score": 0.95,
      "text": "Full text content...",
      "highlights": ["relevant snippet..."]
    }
  ],
  "autopromptString": "optimized query"
}
```

## Tool Definition (OpenAI format)

```json
{
  "type": "function",
  "function": {
    "name": "builtin-exa-web-search",
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
          "description": "Number of results to return (default: 5, max: 10)"
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

const EXA_API_URL: &str = "https://api.exa.ai/search";
const EXA_TIMEOUT_SECS: u64 = 30;

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
                status,
                error_text
            )));
        }

        response
            .json::<ExaSearchResponse>()
            .await
            .map_err(|e| ExaError::RequestFailed(format!("Parse error: {}", e)))
    }
}
```

## Tool Execution Response Format

Tool execution returns result as JSON that LLM can interpret:

```json
{
  "tool_call_id": "call_abc123",
  "result": {
    "results": [
      {
        "title": "Page Title",
        "url": "https://example.com",
        "snippet": "relevant text..."
      }
    ],
    "query_used": "optimized query"
  }
}
```

On error:
```json
{
  "tool_call_id": "call_abc123",
  "result": null,
  "error": "exa_rate_limited: Rate limit exceeded. Please try again later."
}
```

## Testing

### Unit Tests (mock Exa)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_search_success() {
        let mut mock = MockExaService::new();
        mock.expect_search()
            .with(eq("test-key"), eq("rust programming"), eq(Some(5)))
            .returning(|_, _, _| Ok(ExaSearchResponse {
                results: vec![ExaSearchResult {
                    title: "Rust Programming".to_string(),
                    url: "https://rust-lang.org".to_string(),
                    // ...
                }],
                autoprompt_string: Some("rust programming language".to_string()),
            }));
        // ...
    }
}
```

### E2E Tests (real API)
- Provide `EXA_API_KEY` environment variable
- MSW mock for frontend tests in `crates/bodhi`
