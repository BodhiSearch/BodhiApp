use crate::AuthContext;
use axum::{
  body::Body,
  extract::{Request, State},
  middleware::Next,
  response::Response,
};
use objs::{ApiError, AppError, ErrorType};
use server_core::RouterState;
use services::db::DbService;
use std::sync::Arc;

pub trait AccessRequestValidator: Send + Sync + 'static {
  fn extract_entity_id(&self, path: &str) -> Result<String, AccessRequestAuthError>;
  fn validate_approved(
    &self,
    approved_json: &Option<String>,
    entity_id: &str,
  ) -> Result<(), AccessRequestAuthError>;
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AccessRequestAuthError {
  #[error("Authentication required.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingAuth,

  #[error("Entity not found in request path.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  EntityNotFound,

  #[error("Access request {access_request_id} not found.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AccessRequestNotFound { access_request_id: String },

  #[error("Access request {access_request_id} has status '{status}'. Only approved requests can access resources.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AccessRequestNotApproved {
    access_request_id: String,
    status: String,
  },

  #[error("Access request {access_request_id} is invalid: {reason}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AccessRequestInvalid {
    access_request_id: String,
    reason: String,
  },

  #[error("Entity {entity_id} is not included in your approved resources for this app.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  EntityNotApproved { entity_id: String },

  #[error("Access request app client ID mismatch: expected {expected}, found {found}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AppClientMismatch { expected: String, found: String },

  #[error("Access request user ID mismatch: expected {expected}, found {found}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  UserMismatch { expected: String, found: String },

  #[error("Invalid approved JSON in access request: {error}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidApprovedJson { error: String },

  #[error(transparent)]
  DbError(#[from] services::db::DbError),
}

enum AccessRequestAuthFlow {
  Session,
  OAuth {
    user_id: String,
    app_client_id: String,
    access_request_id: String,
  },
}

fn extract_uuid_from_path(path: &str) -> Result<String, AccessRequestAuthError> {
  path
    .split('/')
    .find(|seg| seg.len() == 36 && seg.contains('-'))
    .map(|s| s.to_string())
    .ok_or(AccessRequestAuthError::EntityNotFound)
}

async fn validate_access_request(
  db_service: &Arc<dyn DbService>,
  access_request_id: &str,
  app_client_id: &str,
  user_id: &str,
) -> Result<Option<String>, AccessRequestAuthError> {
  let access_request = db_service.get(access_request_id).await?.ok_or(
    AccessRequestAuthError::AccessRequestNotFound {
      access_request_id: access_request_id.to_string(),
    },
  )?;

  if access_request.status != "approved" {
    return Err(AccessRequestAuthError::AccessRequestNotApproved {
      access_request_id: access_request_id.to_string(),
      status: access_request.status,
    });
  }

  if access_request.app_client_id != app_client_id {
    return Err(AccessRequestAuthError::AppClientMismatch {
      expected: access_request.app_client_id,
      found: app_client_id.to_string(),
    });
  }

  let ar_user_id =
    access_request
      .user_id
      .as_ref()
      .ok_or(AccessRequestAuthError::AccessRequestInvalid {
        access_request_id: access_request_id.to_string(),
        reason: "Missing user_id in approved access request".to_string(),
      })?;

  if ar_user_id != user_id {
    return Err(AccessRequestAuthError::UserMismatch {
      expected: ar_user_id.clone(),
      found: user_id.to_string(),
    });
  }

  Ok(access_request.approved)
}

pub async fn access_request_auth_middleware(
  validator: Arc<dyn AccessRequestValidator>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request<Body>,
  next: Next,
) -> Result<Response, ApiError> {
  let auth_context = req
    .extensions()
    .get::<AuthContext>()
    .ok_or(AccessRequestAuthError::MissingAuth)?
    .clone();

  let auth_flow = match &auth_context {
    AuthContext::Session { .. } => AccessRequestAuthFlow::Session,
    AuthContext::ExternalApp {
      user_id,
      app_client_id,
      access_request_id: Some(ar_id),
      ..
    } => AccessRequestAuthFlow::OAuth {
      user_id: user_id.clone(),
      app_client_id: app_client_id.clone(),
      access_request_id: ar_id.clone(),
    },
    _ => return Err(AccessRequestAuthError::MissingAuth.into()),
  };

  if let AccessRequestAuthFlow::OAuth {
    user_id,
    app_client_id,
    access_request_id,
  } = &auth_flow
  {
    let entity_id = validator.extract_entity_id(req.uri().path())?;
    let db_service = state.app_service().db_service();
    let approved =
      validate_access_request(&db_service, access_request_id, app_client_id, user_id).await?;
    validator.validate_approved(&approved, &entity_id)?;
  }

  Ok(next.run(req).await)
}

pub struct ToolsetAccessRequestValidator;

impl AccessRequestValidator for ToolsetAccessRequestValidator {
  fn extract_entity_id(&self, path: &str) -> Result<String, AccessRequestAuthError> {
    extract_uuid_from_path(path)
  }

  fn validate_approved(
    &self,
    approved_json: &Option<String>,
    entity_id: &str,
  ) -> Result<(), AccessRequestAuthError> {
    let Some(approved_json) = approved_json else {
      return Err(AccessRequestAuthError::EntityNotApproved {
        entity_id: entity_id.to_string(),
      });
    };

    let approvals: serde_json::Value = serde_json::from_str(approved_json).map_err(|e| {
      AccessRequestAuthError::InvalidApprovedJson {
        error: e.to_string(),
      }
    })?;

    let toolsets = approvals.get("toolsets").and_then(|v| v.as_array()).ok_or(
      AccessRequestAuthError::InvalidApprovedJson {
        error: "Missing toolsets array".to_string(),
      },
    )?;

    let instance_approved = toolsets.iter().any(|approval| {
      approval.get("status").and_then(|s| s.as_str()) == Some("approved")
        && approval
          .get("instance")
          .and_then(|i| i.get("id"))
          .and_then(|id| id.as_str())
          == Some(entity_id)
    });

    if !instance_approved {
      return Err(AccessRequestAuthError::EntityNotApproved {
        entity_id: entity_id.to_string(),
      });
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::AuthContext;
  use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::from_fn_with_state,
    routing::post,
    Router,
  };
  use objs::{ResourceRole, UserScope};
  use rstest::{fixture, rstest};
  use server_core::{DefaultRouterState, MockSharedContext};
  use services::{
    db::AccessRequestRepository,
    test_utils::{AppServiceStubBuilder, MockDbService, TestDbService},
  };
  use std::sync::Arc;
  use tower::ServiceExt;

  async fn test_handler() -> Response<Body> {
    Response::builder()
      .status(StatusCode::OK)
      .body(Body::empty())
      .unwrap()
  }

  async fn inject_auth_context(
    auth_context: AuthContext,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
  ) -> axum::response::Response {
    req.extensions_mut().insert(auth_context);
    next.run(req).await
  }

  #[fixture]
  fn toolset_validator() -> Arc<dyn AccessRequestValidator> {
    Arc::new(ToolsetAccessRequestValidator)
  }

  async fn test_router(
    validator: Arc<dyn AccessRequestValidator>,
    auth_context: AuthContext,
  ) -> Router {
    let mock_db_service = MockDbService::new();

    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(mock_db_service))
      .build()
      .await
      .unwrap();

    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    let ctx = auth_context.clone();
    let v = validator.clone();
    Router::new()
      .route(
        "/toolsets/{id}/execute/{method}",
        post(test_handler).route_layer(from_fn_with_state(
          state.clone(),
          move |state, req, next| {
            let v = v.clone();
            access_request_auth_middleware(v, state, req, next)
          },
        )),
      )
      .layer(axum::middleware::from_fn(move |req, next| {
        let ctx = ctx.clone();
        inject_auth_context(ctx, req, next)
      }))
      .with_state(state)
  }

  async fn test_router_with_db(
    validator: Arc<dyn AccessRequestValidator>,
    db_service: Arc<TestDbService>,
    auth_context: AuthContext,
  ) -> Router {
    let app_service = AppServiceStubBuilder::default()
      .db_service(db_service)
      .build()
      .await
      .unwrap();

    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::new()),
      Arc::new(app_service),
    ));

    let ctx = auth_context.clone();
    let v = validator.clone();
    Router::new()
      .route(
        "/toolsets/{id}/execute/{method}",
        post(test_handler).route_layer(from_fn_with_state(
          state.clone(),
          move |state, req, next| {
            let v = v.clone();
            access_request_auth_middleware(v, state, req, next)
          },
        )),
      )
      .layer(axum::middleware::from_fn(move |req, next| {
        let ctx = ctx.clone();
        inject_auth_context(ctx, req, next)
      }))
      .with_state(state)
  }

  // Session auth: passthrough (no access request checks)
  #[rstest]
  #[tokio::test]
  async fn test_session_auth_passes_through(toolset_validator: Arc<dyn AccessRequestValidator>) {
    let ctx = AuthContext::test_session("user123", "user@test.com", ResourceRole::User);
    let app = test_router(toolset_validator, ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }

  #[rstest]
  #[tokio::test]
  async fn test_missing_auth(toolset_validator: Arc<dyn AccessRequestValidator>) {
    let ctx = AuthContext::Anonymous;
    let app = test_router(toolset_validator, ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }

  // OAuth access request validation tests
  #[rstest]
  #[case::oauth_approved_instance_in_list(
    "approved",
    Some(r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"550e8400-e29b-41d4-a716-446655440000"}}]}"#.to_string()),
    StatusCode::OK,
  )]
  #[case::oauth_denied("denied", None, StatusCode::FORBIDDEN)]
  #[case::oauth_draft("draft", None, StatusCode::FORBIDDEN)]
  #[case::oauth_not_in_approved_list(
    "approved",
    Some(r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"different-toolset-id"}}]}"#.to_string()),
    StatusCode::FORBIDDEN,
  )]
  #[tokio::test]
  async fn test_oauth_access_request_validation(
    toolset_validator: Arc<dyn AccessRequestValidator>,
    #[case] status: &str,
    #[case] approved: Option<String>,
    #[case] expected_status: StatusCode,
  ) {
    use objs::test_utils::temp_dir;
    use services::test_utils::test_db_service_with_temp_dir;

    let temp_dir = Arc::new(temp_dir());
    let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
    let now = test_db.now();

    let access_request_row = services::db::AppAccessRequestRow {
      id: "ar-uuid".to_string(),
      app_client_id: "app1".to_string(),
      app_name: None,
      app_description: None,
      flow_type: "redirect".to_string(),
      redirect_uri: Some("http://localhost:3000/callback".to_string()),
      status: status.to_string(),
      requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
      approved,
      user_id: Some("user123".to_string()),
      resource_scope: None,
      access_request_scope: None,
      error_message: None,
      expires_at: (now + chrono::Duration::hours(1)).timestamp(),
      created_at: now.timestamp(),
      updated_at: now.timestamp(),
    };

    test_db.create(&access_request_row).await.unwrap();

    let ctx = AuthContext::test_external_app("user123", UserScope::User, "app1", Some("ar-uuid"));
    let app = test_router_with_db(toolset_validator, Arc::new(test_db), ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), expected_status);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_app_client_mismatch(toolset_validator: Arc<dyn AccessRequestValidator>) {
    use objs::test_utils::temp_dir;
    use services::test_utils::test_db_service_with_temp_dir;

    let temp_dir = Arc::new(temp_dir());
    let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
    let now = test_db.now();

    let access_request_row = services::db::AppAccessRequestRow {
      id: "ar-uuid".to_string(),
      app_client_id: "app1".to_string(),
      app_name: None,
      app_description: None,
      flow_type: "redirect".to_string(),
      redirect_uri: Some("http://localhost:3000/callback".to_string()),
      status: "approved".to_string(),
      requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
      approved: Some(
        r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"550e8400-e29b-41d4-a716-446655440000"}}]}"#
          .to_string(),
      ),
      user_id: Some("user123".to_string()),
      resource_scope: None,
      access_request_scope: None,
      error_message: None,
      expires_at: (now + chrono::Duration::hours(1)).timestamp(),
      created_at: now.timestamp(),
      updated_at: now.timestamp(),
    };

    test_db.create(&access_request_row).await.unwrap();

    let ctx = AuthContext::test_external_app("user123", UserScope::User, "app2", Some("ar-uuid"));
    let app = test_router_with_db(toolset_validator, Arc::new(test_db), ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_user_mismatch(toolset_validator: Arc<dyn AccessRequestValidator>) {
    use objs::test_utils::temp_dir;
    use services::test_utils::test_db_service_with_temp_dir;

    let temp_dir = Arc::new(temp_dir());
    let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
    let now = test_db.now();

    let access_request_row = services::db::AppAccessRequestRow {
      id: "ar-uuid".to_string(),
      app_client_id: "app1".to_string(),
      app_name: None,
      app_description: None,
      flow_type: "redirect".to_string(),
      redirect_uri: Some("http://localhost:3000/callback".to_string()),
      status: "approved".to_string(),
      requested: r#"{"toolset_types":[{"toolset_type":"builtin-exa-search"}]}"#.to_string(),
      approved: Some(
        r#"{"toolsets":[{"toolset_type":"builtin-exa-search","status":"approved","instance":{"id":"550e8400-e29b-41d4-a716-446655440000"}}]}"#
          .to_string(),
      ),
      user_id: Some("user1".to_string()),
      resource_scope: None,
      access_request_scope: None,
      error_message: None,
      expires_at: (now + chrono::Duration::hours(1)).timestamp(),
      created_at: now.timestamp(),
      updated_at: now.timestamp(),
    };

    test_db.create(&access_request_row).await.unwrap();

    let ctx = AuthContext::test_external_app("user2", UserScope::User, "app1", Some("ar-uuid"));
    let app = test_router_with_db(toolset_validator, Arc::new(test_db), ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_auto_approved_no_toolsets(
    toolset_validator: Arc<dyn AccessRequestValidator>,
  ) {
    use objs::test_utils::temp_dir;
    use services::test_utils::test_db_service_with_temp_dir;

    let temp_dir = Arc::new(temp_dir());
    let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;
    let now = test_db.now();

    let access_request_row = services::db::AppAccessRequestRow {
      id: "ar-uuid".to_string(),
      app_client_id: "app1".to_string(),
      app_name: None,
      app_description: None,
      flow_type: "redirect".to_string(),
      redirect_uri: Some("http://localhost:3000/callback".to_string()),
      status: "approved".to_string(),
      requested: r#"{"toolset_types":[]}"#.to_string(),
      approved: None,
      user_id: Some("user123".to_string()),
      resource_scope: None,
      access_request_scope: None,
      error_message: None,
      expires_at: (now + chrono::Duration::hours(1)).timestamp(),
      created_at: now.timestamp(),
      updated_at: now.timestamp(),
    };

    test_db.create(&access_request_row).await.unwrap();

    let ctx = AuthContext::test_external_app("user123", UserScope::User, "app1", Some("ar-uuid"));
    let app = test_router_with_db(toolset_validator, Arc::new(test_db), ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }

  #[rstest]
  #[tokio::test]
  async fn test_oauth_access_request_not_found(toolset_validator: Arc<dyn AccessRequestValidator>) {
    use objs::test_utils::temp_dir;
    use services::test_utils::test_db_service_with_temp_dir;

    let temp_dir = Arc::new(temp_dir());
    let test_db = test_db_service_with_temp_dir(temp_dir.clone()).await;

    let ctx = AuthContext::test_external_app(
      "user123",
      UserScope::User,
      "app1",
      Some("ar-uuid-nonexistent"),
    );
    let app = test_router_with_db(toolset_validator, Arc::new(test_db), ctx).await;

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/toolsets/550e8400-e29b-41d4-a716-446655440000/execute/search")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }
}
