use crate::ResourceAccess;
use serde::{Deserialize, Serialize};
use services::{AppAccessRequest, AppAccessRequestStatus, ApprovedResources, UserScope};
use utoipa::ToSchema;

/// App identity shown on the consent screen; `redirect_uri` is the validated target the
/// flow will return to.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConsentAppInfo {
  pub client_id: String,
  pub name: String,
  pub description: String,
  pub redirect_uri: String,
}

/// Parsed app-facing scope driving the consent screen: which sections render and the
/// requested role ceiling. `passthrough` tokens are forwarded to the auth server verbatim.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConsentScopeInfo {
  pub role: UserScope,
  pub llms: bool,
  pub mcps: bool,
  pub passthrough: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConsentPriorGrantSource {
  /// Named by `source_access_request_id` — a reauthorization; prefill the diff.
  Explicit,
  /// Newest prior grant for this app+user — offer as an unselected restore affordance.
  Latest,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConsentPriorGrant {
  pub id: String,
  pub approved_role: UserScope,
  pub approved: ApprovedResources,
  pub source: ConsentPriorGrantSource,
}

/// Everything the consent page needs before render. `error` outcomes carry a
/// `redirect_url` only when the redirect target was validated; `null` means render
/// in-app and navigate nowhere (RFC 6749 §4.1.2.1).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum ConsentContextResponse {
  Ok {
    app: ConsentAppInfo,
    scope: ConsentScopeInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    prior_grant: Option<ConsentPriorGrant>,
    /// `false` for sessions below the User role — the page renders a blocked state
    /// with a decline-only action.
    can_approve: bool,
  },
  Error {
    error: String,
    error_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_url: Option<String>,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConsentDecision {
  Approve,
  Deny,
}

/// Consent decision for one authorize request. `query` is the page's query string exactly
/// as received — the backend re-validates it rather than trusting a client-side reading.
#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate, ToSchema)]
#[schema(example = json!({
    "query": "client_id=app-acme&redirect_uri=https%3A%2F%2Facme.dev%2Fcb&response_type=code&state=abc&code_challenge=xyz&code_challenge_method=S256&scope=scope_user_user",
    "decision": "approve",
    "approved_role": "scope_user_user",
    "approved": {"version": "1", "models_access": {"type": "specific", "ids": ["llama3:8b"]}}
}))]
pub struct SubmitConsentRequest {
  pub query: String,
  pub decision: ConsentDecision,
  /// Required when `decision` is `approve`.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub approved_role: Option<UserScope>,
  /// Required when `decision` is `approve`.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub approved: Option<ApprovedResources>,
}

/// The one mutating consent response: the page navigates to `redirect_url` unconditionally.
/// Approve → the auth server's authorize endpoint with the composed scope; deny or a
/// redirectable request error → the app's redirect_uri carrying the OAuth error params.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubmitConsentResponse {
  /// Created access request id; omitted on deny and on redirected errors.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub redirect_url: String,
}

/// One issued app token (approved access request) with its effective grant summary.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppAccessSummary {
  pub id: String,
  pub app_client_id: String,
  pub app_name: Option<String>,
  pub app_description: Option<String>,
  pub status: AppAccessRequestStatus,
  pub approved_role: Option<UserScope>,
  /// Effective model access granted to this app.
  pub models: ResourceAccess,
  /// Effective MCP access granted to this app.
  pub mcps: ResourceAccess,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: chrono::DateTime<chrono::Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl AppAccessSummary {
  /// Defaults to no access when the approved JSON is missing/unparsable.
  ///
  /// `caller_max_scope` clamps a (possibly DB-tampered) stored `approved_role` to the
  /// ceiling the session caller could actually have granted — mirroring the
  /// privilege-escalation guard enforced at token-exchange — so the list/revoke views
  /// never advertise a role the issued token cannot actually use. `None` ⇒ no clamp.
  pub fn from_row(row: AppAccessRequest, caller_max_scope: Option<UserScope>) -> Self {
    let approved = row
      .approved
      .as_deref()
      .and_then(|json| serde_json::from_str::<ApprovedResources>(json).ok());
    let (models, mcps) = match approved.as_ref().map(|a| a.v1()) {
      Some(v1) => (ResourceAccess::app_models(v1), ResourceAccess::app_mcps(v1)),
      None => (
        ResourceAccess::Specific {
          list: false,
          ids: vec![],
        },
        ResourceAccess::Specific {
          list: false,
          ids: vec![],
        },
      ),
    };
    Self {
      id: row.id,
      app_client_id: row.app_client_id,
      app_name: row.app_name,
      app_description: row.app_description,
      status: row.status,
      approved_role: row
        .approved_role
        .and_then(|r| r.parse::<UserScope>().ok())
        .map(|r| match caller_max_scope {
          Some(max) if r > max => max,
          _ => r,
        }),
      models,
      mcps,
      created_at: row.created_at,
      updated_at: row.updated_at,
    }
  }
}

/// Response for GET /access-requests/apps — the caller's issued app tokens.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListAppAccessResponse {
  pub data: Vec<AppAccessSummary>,
}
