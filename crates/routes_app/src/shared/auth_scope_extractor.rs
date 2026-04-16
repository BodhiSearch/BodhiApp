use crate::BodhiErrorResponse;
use axum::{
  extract::{FromRef, FromRequestParts},
  http::request::Parts,
};
use services::AuthContext;
use services::{AppService, AuthScopedAppService};
use std::{ops::Deref, sync::Arc};

/// Newtype wrapper around `AuthScopedAppService` that implements `FromRequestParts`
/// for use as an Axum extractor in route handlers.
///
/// Extracts `AuthContext` from request extensions (populated by auth middleware)
/// and the `AppService` from the router state, then combines them into an
/// `AuthScopedAppService` for user-scoped service access.
///
/// Falls back to `AuthContext::Anonymous` if no auth middleware has populated the
/// extension (e.g., handlers behind `optional_auth_middleware` or public endpoints).
pub struct AuthScope(pub AuthScopedAppService);

impl Deref for AuthScope {
  type Target = AuthScopedAppService;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<S> FromRequestParts<S> for AuthScope
where
  S: Send + Sync,
  Arc<dyn AppService>: FromRef<S>,
{
  type Rejection = BodhiErrorResponse;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    // Extract AuthContext from extensions (set by auth middleware before handler runs).
    // Falls back to Anonymous if no middleware has set the extension (e.g., public endpoints
    // or optional auth endpoints where middleware may not have populated it).
    let auth_context =
      parts
        .extensions
        .get::<AuthContext>()
        .cloned()
        .unwrap_or(AuthContext::Anonymous {
          deployment: services::DeploymentMode::Standalone,
        });

    // Extract the app service using FromRef (same mechanism as State<T> extractor)
    let app_service = Arc::<dyn AppService>::from_ref(state);

    Ok(AuthScope(AuthScopedAppService::new(
      app_service,
      auth_context,
    )))
  }
}
