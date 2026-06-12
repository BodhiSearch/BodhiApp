# Access Request - Quick Reference

## Key Handler Signatures

```rust
// POST /apps/request-access - Create draft
pub async fn create_app_access_request_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<CreateAppAccessRequest>,
) -> Result<(StatusCode, Json<AppAccessResponseDTO>), ApiError>

// GET /apps/request-access/:id - Poll status
pub async fn get_app_access_request_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<AppAccessRequestDetail>, ApiError>
```

## Request/Response DTOs

```rust
// Request DTO (routes_app)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CreateAppAccessRequest {
  pub app_client_id: String,
  pub flow_type: String,        // "redirect" | "popup"
  #[serde(skip_serializing_if = "Option::is_none")]
  pub redirect_uri: Option<String>,
  pub tools: Vec<ToolTypeRequest>,
}

// Response DTO (routes_app)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AppAccessResponseDTO {
  pub access_request_id: String,
  pub review_url: String,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub scopes: Vec<String>,
}

// Detail DTO (services crate)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AppAccessRequestDetail {
  pub id: String,
  pub app_client_id: String,
  pub flow_type: String,
  pub redirect_uri: Option<String>,
  pub status: String,  // "draft" | "approved" | "denied" | "failed"
  pub tools_requested: Vec<ToolTypeRequest>,
  pub tools_approved: Option<Vec<ToolApproval>>,
  pub user_id: Option<String>,
  pub scopes: Vec<String>,
  pub error_message: Option<String>,
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}
```

## Domain Objects (objs crate)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolTypeRequest {
  pub tool_type: String,  // e.g., "builtin-exa-search"
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolApproval {
  pub tool_type: String,
  pub status: String,  // "approved" | "denied"
  pub toolset_id: Option<String>,
}
```

## Service Layer Interface

```rust
#[async_trait]
pub trait AccessRequestService: Send + Sync + std::fmt::Debug {
  async fn create_draft(
    &self,
    app_client_id: String,
    flow_type: String,
    redirect_uri: Option<String>,
    tools_requested: Vec<ToolTypeRequest>,
  ) -> Result<AppAccessRequestRow>;

  async fn get_request(&self, id: &str) -> Result<Option<AppAccessRequestRow>>;

  async fn approve_request(
    &self,
    id: &str,
    user_id: &str,
    user_token: &str,
    tool_approvals: Vec<ToolApproval>,
  ) -> Result<AppAccessRequestRow>;

  async fn deny_request(&self, id: &str, user_id: &str) -> Result<AppAccessRequestRow>;

  fn build_review_url(&self, access_request_id: &str) -> String;
}
```

## Test Template

```rust
use super::*;
use axum::{extract::State, http::StatusCode, Json};
use pretty_assertions::assert_eq;
use rstest::rstest;
use server_core::{DefaultRouterState, RouterState};
use services::{AppService, MockAccessRequestService};
use std::sync::Arc;

mod tests {
  use super::*;

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_create_draft_success() -> anyhow::Result<()> {
    // Arrange
    let mut mock_access_request_service = MockAccessRequestService::new();
    mock_access_request_service
      .expect_create_draft()
      .returning(|app_id, flow, uri, tools| {
        Ok(AppAccessRequestRow {
          id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
          app_client_id: app_id,
          flow_type: flow,
          redirect_uri: uri,
          status: "draft".to_string(),
          tools_requested: serde_json::to_string(&tools).unwrap(),
          tools_approved: None,
          user_id: None,
          resource_scope: None,
          access_request_scope: None,
          error_message: None,
          expires_at: 1234567890,
          created_at: 1234567890,
          updated_at: 1234567890,
        })
      });

    let mut mock_app_service = AppServiceStub::default();
    mock_app_service.access_request_service = Some(Arc::new(mock_access_request_service));

    let state = Arc::new(DefaultRouterState::new(
      Arc::new(SharedContextStub::default()),
      Arc::new(mock_app_service),
    ));

    let request = CreateAppAccessRequest {
      app_client_id: "app-abc123".to_string(),
      flow_type: "redirect".to_string(),
      redirect_uri: Some("https://example.com/callback".to_string()),
      tools: vec![ToolTypeRequest {
        tool_type: "builtin-exa-search".to_string(),
      }],
    };

    // Act
    let result = create_app_access_request_handler(
      State(state),
      Json(request),
    ).await;

    // Assert
    assert!(result.is_ok());
    let (status, Json(response)) = result.unwrap();
    assert_eq!(StatusCode::CREATED, status);
    assert_eq!("550e8400-e29b-41d4-a716-446655440000", response.access_request_id);
    assert!(!response.review_url.is_empty());

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_create_draft_invalid_flow_type() -> anyhow::Result<()> {
    // Arrange
    let state = /* ... setup ... */;
    let request = CreateAppAccessRequest {
      flow_type: "invalid".to_string(),
      // ... other fields
    };

    // Act
    let result = create_app_access_request_handler(State(state), Json(request)).await;

    // Assert
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!("app_access_request_error-invalid_flow_type", err.code());

    Ok(())
  }
}
```

## Error Codes

```
app_access_request_error-invalid_flow_type
app_access_request_error-missing_redirect_uri
app_access_request_error-invalid_tools_format
access_request_error-not_found
access_request_error-expired
access_request_error-already_processed
```

## Useful Commands

```bash
# Watch mode for development
cargo watch -x 'test -p routes_app routes_apps::tests::access_request_test'

# Run specific test
cargo test -p routes_app test_create_draft_success -- --nocapture

# Check compilation
cargo check -p routes_app

# Format code
cargo fmt --all
```

## File Locations

```
Handler:        crates/routes_app/src/routes_apps/access_request.rs
Error enum:     crates/routes_app/src/routes_apps/error.rs
Tests (NEW):    crates/routes_app/src/routes_apps/tests/access_request_test.rs
Service:        crates/services/src/access_request_service/service.rs
Repository:     crates/services/src/db/access_request_repository.rs
Domain objects: crates/objs/src/access_request.rs
Router:         crates/routes_app/src/routes.rs (lines 76-82)
OpenAPI:        crates/routes_app/src/shared/openapi.rs
```
