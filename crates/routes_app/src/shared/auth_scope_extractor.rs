use crate::BodhiErrorResponse;
use axum::{
  extract::{FromRef, FromRequestParts},
  http::request::Parts,
};
use services::AuthContext;
use services::{AppService, AuthScopedAppService};
use std::{ops::Deref, sync::Arc};

/// Axum extractor combining `AuthContext` (from request extensions) with `AppService`
/// (from router state) into `AuthScopedAppService`. Falls back to `AuthContext::Anonymous`
/// if no auth middleware populated the extension (e.g. public endpoints).
pub struct AuthScope(pub AuthScopedAppService);

impl Deref for AuthScope {
  type Target = AuthScopedAppService;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl AuthScope {
  /// Per-resource access policy for the current principal — the single entry point
  /// handlers use to filter listings and guard inference/connect.
  pub fn access_policy(&self) -> crate::AccessPolicy<'_> {
    crate::AccessPolicy::of(self.auth_context())
  }
}

impl<S> FromRequestParts<S> for AuthScope
where
  S: Send + Sync,
  Arc<dyn AppService>: FromRef<S>,
{
  type Rejection = BodhiErrorResponse;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let auth_context =
      parts
        .extensions
        .get::<AuthContext>()
        .cloned()
        .unwrap_or(AuthContext::Anonymous {
          deployment: services::DeploymentMode::Standalone,
        });

    // Same mechanism as the State<T> extractor
    let app_service = Arc::<dyn AppService>::from_ref(state);

    Ok(AuthScope(AuthScopedAppService::new(
      app_service,
      auth_context,
    )))
  }
}
