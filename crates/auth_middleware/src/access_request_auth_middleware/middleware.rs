use crate::AuthContext;
use axum::{
  body::Body,
  extract::{Request, State},
  middleware::Next,
  response::Response,
};
use server_core::RouterState;
use services::db::DbService;
use services::AppAccessRequestStatus;
use services::{ApiError, AppError, ErrorType};
use services::{ApprovalStatus, ApprovedResources};
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

fn extract_id_from_path(
  path: &str,
  resource_prefix: &str,
) -> Result<String, AccessRequestAuthError> {
  let mut segments = path.split('/');
  while let Some(seg) = segments.next() {
    if seg == resource_prefix {
      return segments
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or(AccessRequestAuthError::EntityNotFound);
    }
  }
  Err(AccessRequestAuthError::EntityNotFound)
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
    extract_id_from_path(path, "toolsets")
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

    let approvals: ApprovedResources = serde_json::from_str(approved_json).map_err(|e| {
      AccessRequestAuthError::InvalidApprovedJson {
        error: e.to_string(),
      }
    })?;

    let instance_approved = approvals.toolsets.iter().any(|a| {
      a.status == ApprovalStatus::Approved && a.instance.as_ref().is_some_and(|i| i.id == entity_id)
    });

    if !instance_approved {
      return Err(AccessRequestAuthError::EntityNotApproved {
        entity_id: entity_id.to_string(),
      });
    }

    Ok(())
  }
}

pub struct McpAccessRequestValidator;

impl AccessRequestValidator for McpAccessRequestValidator {
  fn extract_entity_id(&self, path: &str) -> Result<String, AccessRequestAuthError> {
    extract_id_from_path(path, "mcps")
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

    let approvals: ApprovedResources = serde_json::from_str(approved_json).map_err(|e| {
      AccessRequestAuthError::InvalidApprovedJson {
        error: e.to_string(),
      }
    })?;

    let instance_approved = approvals.mcps.iter().any(|a| {
      a.status == ApprovalStatus::Approved && a.instance.as_ref().is_some_and(|i| i.id == entity_id)
    });

    if !instance_approved {
      return Err(AccessRequestAuthError::EntityNotApproved {
        entity_id: entity_id.to_string(),
      });
    }

    Ok(())
  }
}
