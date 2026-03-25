use crate::middleware::access_requests::AccessRequestAuthError;
use services::{ApprovalStatus, ApprovedResources};

pub trait AccessRequestValidator: Send + Sync + 'static {
  fn extract_entity_id(&self, path: &str) -> Result<String, AccessRequestAuthError>;
  fn validate_approved(
    &self,
    approved_json: &Option<String>,
    entity_id: &str,
  ) -> Result<(), AccessRequestAuthError>;
}

pub(crate) fn extract_id_from_path(
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

    let instance_approved = match &approvals {
      ApprovedResources::V1(v1) => v1.toolsets.iter().any(|a| {
        a.status == ApprovalStatus::Approved
          && a.instance.as_ref().is_some_and(|i| i.id == entity_id)
      }),
    };

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

    let instance_approved = match &approvals {
      ApprovedResources::V1(v1) => v1.mcps.iter().any(|a| {
        a.status == ApprovalStatus::Approved
          && a.instance.as_ref().is_some_and(|i| i.id == entity_id)
      }),
    };

    if !instance_approved {
      return Err(AccessRequestAuthError::EntityNotApproved {
        entity_id: entity_id.to_string(),
      });
    }

    Ok(())
  }
}
