use crate::{API_TAG_SYSTEM, ENDPOINT_HEALTH, ENDPOINT_PING};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response to the ping endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "message": "pong"
}))]
pub struct PingResponse {
  /// Simple ping response message
  #[schema(example = "pong")]
  pub message: String,
}

/// Simple connectivity check endpoint
#[utoipa::path(
    get,
    path = ENDPOINT_PING,
    tag = API_TAG_SYSTEM,
    operation_id = "pingServer",
    summary = "Ping Server",
    description = "Simple connectivity check to verify the server is responding",
    responses(
        (status = 200, description = "Server is responding normally",
         body = PingResponse,
         content_type = "application/json",
         example = json!({"message": "pong"})
        )
    )
)]
#[tracing::instrument]
pub async fn ping_handler() -> Json<PingResponse> {
  tracing::info!("ping request received");
  Json(PingResponse {
    message: "pong".to_string(),
  })
}

/// Application health check endpoint
#[utoipa::path(
  get,
  path = ENDPOINT_HEALTH,
  tag = API_TAG_SYSTEM,
  operation_id = "healthCheck",
  summary = "Health Check",
  description = "Comprehensive health check to verify all application components are operational",
  responses(
      (status = 200, description = "Application is healthy and fully operational",
       body = PingResponse,
       content_type = "application/json",
       example = json!({"message": "pong"})
      )
  )
)]
#[tracing::instrument]
pub async fn health_handler() -> Json<PingResponse> {
  tracing::info!("health check request received");
  Json(PingResponse {
    message: "pong".to_string(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_utils::build_test_router;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
  };
  use rstest::rstest;
  use tower::ServiceExt;

  #[rstest]
  #[case::ping(ENDPOINT_PING)]
  #[case::health(ENDPOINT_HEALTH)]
  #[tokio::test]
  async fn test_ping_and_health_handlers(#[case] path: &str) -> anyhow::Result<()> {
    let (router, _app_service, _temp_dir) = build_test_router().await?;
    let req = Request::builder().uri(path).body(Body::empty()).unwrap();
    let response = router.oneshot(req).await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let ping_response: PingResponse = serde_json::from_slice(&body)?;
    assert_eq!(ping_response.message, "pong");
    Ok(())
  }
}
