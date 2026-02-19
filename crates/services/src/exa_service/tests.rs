use super::*;
use anyhow_trace::anyhow_trace;
use mockito::{Matcher, Server};
use objs::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_search_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
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

  let service = DefaultExaService::with_base_url(server.url());
  let response = service.search("test-key", "rust programming", None).await?;

  assert_eq!(1, response.results.len());
  assert_eq!("Rust Programming Language", response.results[0].title);
  assert_eq!("https://rust-lang.org", response.results[0].url);
  assert_eq!(
    Some("rust programming language".to_string()),
    response.autoprompt_string
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_search_unauthorized() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/search")
    .with_status(401)
    .with_body("Unauthorized")
    .create_async()
    .await;

  let service = DefaultExaService::with_base_url(server.url());
  let result = service.search("invalid-key", "test", None).await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("exa_error-invalid_api_key", err.code());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_search_rate_limited() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/search")
    .with_status(429)
    .with_body("Rate limit exceeded")
    .create_async()
    .await;

  let service = DefaultExaService::with_base_url(server.url());
  let result = service.search("test-key", "test", None).await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("exa_error-rate_limited", err.code());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_search_server_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/search")
    .with_status(500)
    .with_body("Internal server error")
    .create_async()
    .await;

  let service = DefaultExaService::with_base_url(server.url());
  let result = service.search("test-key", "test", None).await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("exa_error-request_failed", err.code());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_search_num_results_default() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/search")
    .match_body(Matcher::JsonString(
      json!({
        "query": "test",
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
        "results": [],
        "autopromptString": null
      })
      .to_string(),
    )
    .create_async()
    .await;

  let service = DefaultExaService::with_base_url(server.url());
  let _response = service.search("test-key", "test", None).await?;

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_search_num_results_clamped() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/search")
    .match_body(Matcher::JsonString(
      json!({
        "query": "test",
        "type": "neural",
        "useAutoprompt": true,
        "numResults": 10,
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
        "results": [],
        "autopromptString": null
      })
      .to_string(),
    )
    .create_async()
    .await;

  let service = DefaultExaService::with_base_url(server.url());
  let _response = service.search("test-key", "test", Some(20)).await?;

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_find_similar_success() -> anyhow::Result<()> {
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

  let service = DefaultExaService::with_base_url(server.url());
  let result = service
    .find_similar("test-key", "https://example.com", None)
    .await?;

  assert_eq!(1, result.results.len());
  assert_eq!("Similar Page", result.results[0].title);

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_find_similar_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/findSimilar")
    .with_status(500)
    .with_body("Internal server error")
    .create_async()
    .await;

  let service = DefaultExaService::with_base_url(server.url());
  let result = service
    .find_similar("test-key", "https://example.com", None)
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("exa_error-request_failed", err.code());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_contents_success() -> anyhow::Result<()> {
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

  let service = DefaultExaService::with_base_url(server.url());
  let result = service
    .get_contents("test-key", vec!["https://example.com".to_string()], true)
    .await?;

  assert_eq!(1, result.results.len());
  assert_eq!("https://example.com", result.results[0].url);
  assert_eq!(Some("Example Page".to_string()), result.results[0].title);

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_get_contents_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/contents")
    .with_status(500)
    .with_body("Internal server error")
    .create_async()
    .await;

  let service = DefaultExaService::with_base_url(server.url());
  let result = service
    .get_contents("test-key", vec!["https://example.com".to_string()], true)
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("exa_error-request_failed", err.code());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_answer_success() -> anyhow::Result<()> {
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

  let service = DefaultExaService::with_base_url(server.url());
  let result = service
    .answer("test-key", "what is the answer", true)
    .await?;

  assert_eq!("The answer is 42", result.answer);
  assert_eq!(1, result.results.len());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_answer_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/answer")
    .with_status(500)
    .with_body("Internal server error")
    .create_async()
    .await;

  let service = DefaultExaService::with_base_url(server.url());
  let result = service.answer("test-key", "what is the answer", true).await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("exa_error-request_failed", err.code());

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_search_parse_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/search")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body("invalid json {")
    .create_async()
    .await;

  let service = DefaultExaService::with_base_url(server.url());
  let result = service.search("test-key", "test", None).await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("exa_error-request_failed", err.code());

  Ok(())
}
