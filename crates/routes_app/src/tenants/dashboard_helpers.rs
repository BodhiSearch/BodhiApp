use crate::{DashboardAuthRouteError, DASHBOARD_ACCESS_TOKEN_KEY, DASHBOARD_REFRESH_TOKEN_KEY};
use services::{extract_claims, AuthService, Claims, SettingService, TimeService};
use tower_sessions::Session;

/// Ensures the dashboard access token stored in the session is valid.
/// If the token is expired, attempts to refresh it using the refresh token.
/// Returns the valid access token string on success.
pub async fn ensure_valid_dashboard_token(
  session: &Session,
  auth_service: &dyn AuthService,
  setting_service: &dyn SettingService,
  time_service: &dyn TimeService,
) -> Result<String, DashboardAuthRouteError> {
  // Read dashboard access token from session
  let access_token = session
    .get::<String>(DASHBOARD_ACCESS_TOKEN_KEY)
    .await
    .map_err(DashboardAuthRouteError::from)?
    .ok_or(DashboardAuthRouteError::SessionInfoNotFound)?;

  // Check JWT expiry
  let claims = extract_claims::<Claims>(&access_token)?;
  let now = time_service.utc_now().timestamp() as u64;

  if claims.exp > now {
    // Token is still valid
    return Ok(access_token);
  }

  // Token is expired, attempt refresh
  let refresh_token = session
    .get::<String>(DASHBOARD_REFRESH_TOKEN_KEY)
    .await
    .map_err(DashboardAuthRouteError::from)?
    .ok_or(DashboardAuthRouteError::SessionInfoNotFound)?;

  let client_id = setting_service
    .multitenant_client_id()
    .await
    .ok_or(DashboardAuthRouteError::MissingClientConfig)?;
  let client_secret = setting_service
    .multitenant_client_secret()
    .await
    .ok_or(DashboardAuthRouteError::MissingClientConfig)?;

  let (new_access_token, new_refresh_token) = auth_service
    .refresh_token(&client_id, &client_secret, &refresh_token)
    .await?;

  // Update session with new tokens
  session
    .insert(DASHBOARD_ACCESS_TOKEN_KEY, &new_access_token)
    .await
    .map_err(DashboardAuthRouteError::from)?;

  if let Some(new_refresh) = new_refresh_token {
    session
      .insert(DASHBOARD_REFRESH_TOKEN_KEY, &new_refresh)
      .await
      .map_err(DashboardAuthRouteError::from)?;
  }

  Ok(new_access_token)
}
