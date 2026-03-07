use crate::middleware::{access_token_key, SESSION_KEY_ACTIVE_CLIENT_ID};
use crate::shared::AuthScope;
use crate::tenants::dashboard_helpers::ensure_valid_dashboard_token;
use crate::{
  ApiError, CreateTenantRequest, CreateTenantResponse, DashboardAuthRouteError, TenantListItem,
  TenantListResponse, ValidatedJson, API_TAG_TENANTS, ENDPOINT_TENANTS,
};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use services::{extract_claims, AppStatus, Claims, LOGIN_CALLBACK_PATH};
use tower_sessions::Session;

/// List all tenants visible to the authenticated dashboard user
#[utoipa::path(
    get,
    path = ENDPOINT_TENANTS,
    tag = API_TAG_TENANTS,
    operation_id = "tenantsList",
    summary = "List Tenants",
    description = "Lists all tenants visible to the authenticated dashboard user. Each tenant includes whether it is currently active and whether the user has a valid session for it.",
    responses(
        (status = 200, description = "List of tenants", body = TenantListResponse),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn tenants_index(
  auth_scope: AuthScope,
  session: Session,
) -> Result<Json<TenantListResponse>, ApiError> {
  let settings = auth_scope.settings();

  if !settings.is_multi_tenant().await {
    return Err(DashboardAuthRouteError::NotMultiTenant)?;
  }

  let auth_service = auth_scope.auth_service();
  let dashboard_token = ensure_valid_dashboard_token(
    &session,
    auth_service.as_ref(),
    settings.as_ref(),
    auth_scope.time().as_ref(),
  )
  .await?;

  let spi_response = auth_service.list_tenants(&dashboard_token).await?;

  let active_client_id = session
    .get::<String>(SESSION_KEY_ACTIVE_CLIENT_ID)
    .await
    .unwrap_or(None);

  let mut tenants = Vec::with_capacity(spi_response.tenants.len());
  for spi_tenant in spi_response.tenants {
    let is_active = active_client_id
      .as_ref()
      .map(|active| active == &spi_tenant.client_id)
      .unwrap_or(false);

    let logged_in = session
      .get::<String>(&access_token_key(&spi_tenant.client_id))
      .await
      .unwrap_or(None)
      .is_some();

    tenants.push(TenantListItem {
      client_id: spi_tenant.client_id,
      name: spi_tenant.name,
      description: spi_tenant.description,
      is_active,
      logged_in,
    });
  }

  Ok(Json(TenantListResponse { tenants }))
}

/// Create a new tenant via the Bodhi SPI
#[utoipa::path(
    post,
    path = ENDPOINT_TENANTS,
    tag = API_TAG_TENANTS,
    operation_id = "tenantsCreate",
    summary = "Create Tenant",
    description = "Creates a new tenant (Keycloak client) via the Bodhi SPI and registers it locally.",
    request_body(
        content = CreateTenantRequest,
        description = "Tenant creation parameters"
    ),
    responses(
        (status = 201, description = "Tenant created successfully", body = CreateTenantResponse),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn tenants_create(
  auth_scope: AuthScope,
  session: Session,
  ValidatedJson(form): ValidatedJson<CreateTenantRequest>,
) -> Result<(StatusCode, Json<CreateTenantResponse>), ApiError> {
  let settings = auth_scope.settings();

  if !settings.is_multi_tenant().await {
    return Err(DashboardAuthRouteError::NotMultiTenant)?;
  }

  let auth_service = auth_scope.auth_service();
  let dashboard_token = ensure_valid_dashboard_token(
    &session,
    auth_service.as_ref(),
    settings.as_ref(),
    auth_scope.time().as_ref(),
  )
  .await?;

  let public_url = settings.public_server_url().await;
  let redirect_uris = vec![format!("{}{}", public_url, LOGIN_CALLBACK_PATH)];

  let spi_response = auth_service
    .create_tenant(
      &dashboard_token,
      &form.name,
      &form.description.unwrap_or_default(),
      redirect_uris,
    )
    .await?;

  // Extract user_id from dashboard JWT sub claim
  let user_id = extract_claims::<Claims>(&dashboard_token)
    .ok()
    .map(|claims| claims.sub);

  // Create local tenant row
  let tenant_service = auth_scope.tenant();
  let local_result = tenant_service
    .create_tenant(
      &spi_response.client_id,
      &spi_response.client_secret,
      AppStatus::Ready,
      user_id,
    )
    .await;

  if let Err(e) = local_result {
    tracing::error!(
      client_id = %spi_response.client_id,
      error = %e,
      "Failed to create local tenant row after SPI creation — orphan accepted (D52)"
    );
  }

  Ok((
    StatusCode::CREATED,
    Json(CreateTenantResponse {
      client_id: spi_response.client_id,
    }),
  ))
}

/// Activate a tenant for the current session
#[utoipa::path(
    post,
    path = "/bodhi/v1/tenants/{client_id}/activate",
    tag = API_TAG_TENANTS,
    operation_id = "tenantsActivate",
    summary = "Activate Tenant",
    description = "Sets the given tenant as the active tenant for the current session. The user must already be logged into this tenant.",
    params(
        ("client_id" = String, Path, description = "The client_id of the tenant to activate")
    ),
    responses(
        (status = 200, description = "Tenant activated successfully"),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn tenants_activate(
  auth_scope: AuthScope,
  session: Session,
  Path(client_id): Path<String>,
) -> Result<StatusCode, ApiError> {
  let settings = auth_scope.settings();

  if !settings.is_multi_tenant().await {
    return Err(DashboardAuthRouteError::NotMultiTenant)?;
  }

  // Verify the user has a token for this tenant
  let has_token = session
    .get::<String>(&access_token_key(&client_id))
    .await
    .unwrap_or(None)
    .is_some();

  if !has_token {
    return Err(DashboardAuthRouteError::TenantNotLoggedIn)?;
  }

  session
    .insert(SESSION_KEY_ACTIVE_CLIENT_ID, &client_id)
    .await
    .map_err(DashboardAuthRouteError::from)?;

  Ok(StatusCode::OK)
}

#[cfg(test)]
#[path = "test_tenants.rs"]
mod test_tenants;
