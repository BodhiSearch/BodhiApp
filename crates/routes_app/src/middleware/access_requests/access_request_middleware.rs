use crate::middleware::access_requests::AccessRequestAuthError;
use crate::BodhiErrorResponse;
use axum::{
  body::Body,
  extract::{Request, State},
  middleware::Next,
  response::Response,
};
use services::AppAccessRequestStatus;
use services::AppService;
use services::AuthContext;
use services::DbService;
use std::sync::Arc;

enum AccessRequestAuthFlow {
  Session,
  OAuth {
    tenant_id: String,
    user_id: String,
    app_client_id: String,
    access_request_id: String,
  },
}

/// Validate the access-request **lifecycle** for an external app: the request
/// exists, is Approved, and its app-client / user match the bearer. Per-resource
/// authorization (which MCP instance / model) is enforced downstream by
/// `AccessPolicy` in the handler, using the approved grants carried on the
/// AuthContext — so it is not re-checked here.
async fn validate_access_request(
  db_service: &Arc<dyn DbService>,
  tenant_id: &str,
  access_request_id: &str,
  app_client_id: &str,
  user_id: &str,
) -> Result<(), AccessRequestAuthError> {
  let access_request = db_service.get(tenant_id, access_request_id).await?.ok_or(
    AccessRequestAuthError::AccessRequestNotFound {
      access_request_id: access_request_id.to_string(),
    },
  )?;

  if access_request.status != AppAccessRequestStatus::Approved {
    return Err(AccessRequestAuthError::AccessRequestNotApproved {
      access_request_id: access_request_id.to_string(),
      status: access_request.status.to_string(),
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

  Ok(())
}

pub async fn access_request_auth_middleware(
  State(app_service): State<Arc<dyn AppService>>,
  req: Request<Body>,
  next: Next,
) -> Result<Response, BodhiErrorResponse> {
  let auth_context = req
    .extensions()
    .get::<AuthContext>()
    .ok_or(AccessRequestAuthError::MissingAuth)?
    .clone();

  let auth_flow = match &auth_context {
    // Session and API tokens skip access-request validation. API tokens are instead
    // grant-enforced in the handler (AccessPolicy); they have no access request.
    AuthContext::Session { .. }
    | AuthContext::MultiTenantSession { .. }
    | AuthContext::ApiToken { .. } => AccessRequestAuthFlow::Session,
    AuthContext::ExternalApp {
      tenant_id,
      user_id,
      app_client_id,
      access_request_id: Some(ar_id),
      ..
    } => AccessRequestAuthFlow::OAuth {
      tenant_id: tenant_id.clone(),
      user_id: user_id.clone(),
      app_client_id: app_client_id.clone(),
      access_request_id: ar_id.clone(),
    },
    _ => return Err(AccessRequestAuthError::MissingAuth.into()),
  };

  if let AccessRequestAuthFlow::OAuth {
    tenant_id,
    user_id,
    app_client_id,
    access_request_id,
  } = &auth_flow
  {
    let db_service = app_service.db_service();
    validate_access_request(
      &db_service,
      tenant_id,
      access_request_id,
      app_client_id,
      user_id,
    )
    .await?;
  }

  Ok(next.run(req).await)
}

#[cfg(test)]
#[path = "test_access_request_middleware.rs"]
mod test_access_request_middleware;
