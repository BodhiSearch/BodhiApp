use axum::response::Response;
use services::ai_apis::ai_api_client::MockAiApiClient;
use services::{AiApiClientFactoryError, MockAiApiClientFactory};
use std::sync::Arc;

/// Test helper: builds a `MockAiApiClientFactory` whose `for_alias` returns a
/// `MockAiApiClient` that delegates `forward_request_with_method` to the given
/// `response_fn`. Lets tests focus on response flow without re-mocking the
/// factory plumbing each time.
pub fn mock_ai_factory_returning<F>(response_fn: F) -> Arc<MockAiApiClientFactory>
where
  F: Fn() -> Result<Response, AiApiClientFactoryError> + Send + Sync + Clone + 'static,
{
  let mut mock_factory = MockAiApiClientFactory::new();
  mock_factory.expect_safe_http_client().returning(|| {
    services::SafeReqwest::builder()
      .allow_private_ips()
      .build()
      .unwrap()
  });
  mock_factory
    .expect_for_alias()
    .returning(move |_alias, _key| {
      let inner = response_fn.clone();
      let mut mock_client = MockAiApiClient::new();
      mock_client
        .expect_forward_request_with_method()
        .returning(move |_, _, _, _, _| inner());
      Ok(Box::new(mock_client))
    });
  Arc::new(mock_factory)
}
