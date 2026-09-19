use crate::{
  shared::AuthScope, BodhiErrorResponse, API_TAG_SETTINGS, ENDPOINT_TUNNEL, ENDPOINT_TUNNEL_SETUP,
  ENDPOINT_TUNNEL_SYNC,
};
use axum::Json;
use services::{
  EnableTunnelRequest, TunnelSetupRequest, TunnelStatus, UpdateTunnelPreferencesRequest,
};

#[utoipa::path(
    get,
    path = ENDPOINT_TUNNEL,
    tag = API_TAG_SETTINGS,
    operation_id = "getTunnelStatus",
    summary = "Get Cloudflare Tunnel status",
    responses((status = 200, body = TunnelStatus)),
    security(("session_auth" = ["resource_admin"]))
)]
pub async fn tunnel_status(
  auth_scope: AuthScope,
) -> Result<Json<TunnelStatus>, BodhiErrorResponse> {
  Ok(Json(auth_scope.tunnels().status().await?))
}

#[utoipa::path(
    put,
    path = ENDPOINT_TUNNEL,
    tag = API_TAG_SETTINGS,
    operation_id = "enableTunnel",
    summary = "Enable a Cloudflare named tunnel",
    request_body(content = EnableTunnelRequest),
    responses((status = 200, body = TunnelStatus), (status = 400, body = BodhiErrorResponse)),
    security(("session_auth" = ["resource_admin"]))
)]
pub async fn tunnel_enable(
  auth_scope: AuthScope,
  Json(request): Json<EnableTunnelRequest>,
) -> Result<Json<TunnelStatus>, BodhiErrorResponse> {
  Ok(Json(auth_scope.tunnels().enable(request).await?))
}

#[utoipa::path(
    put,
    path = ENDPOINT_TUNNEL_SETUP,
    tag = API_TAG_SETTINGS,
    operation_id = "setupTunnel",
    summary = "Validate Cloudflare Tunnel prerequisites",
    request_body(content = TunnelSetupRequest),
    responses((status = 200, body = TunnelStatus), (status = 400, body = BodhiErrorResponse)),
    security(("session_auth" = ["resource_admin"]))
)]
pub async fn tunnel_setup(
  auth_scope: AuthScope,
  Json(request): Json<TunnelSetupRequest>,
) -> Result<Json<TunnelStatus>, BodhiErrorResponse> {
  Ok(Json(auth_scope.tunnels().setup(request).await?))
}

#[utoipa::path(
    patch,
    path = ENDPOINT_TUNNEL,
    tag = API_TAG_SETTINGS,
    operation_id = "updateTunnelPreferences",
    summary = "Update Cloudflare Tunnel preferences",
    request_body(content = UpdateTunnelPreferencesRequest),
    responses((status = 200, body = TunnelStatus)),
    security(("session_auth" = ["resource_admin"]))
)]
pub async fn tunnel_preferences(
  auth_scope: AuthScope,
  Json(request): Json<UpdateTunnelPreferencesRequest>,
) -> Result<Json<TunnelStatus>, BodhiErrorResponse> {
  Ok(Json(
    auth_scope.tunnels().update_preferences(request).await?,
  ))
}

#[utoipa::path(
    post,
    path = ENDPOINT_TUNNEL_SYNC,
    tag = API_TAG_SETTINGS,
    operation_id = "syncTunnelAuthorization",
    summary = "Retry tunnel authorization-server synchronization",
    responses((status = 200, body = TunnelStatus)),
    security(("session_auth" = ["resource_admin"]))
)]
pub async fn tunnel_sync(auth_scope: AuthScope) -> Result<Json<TunnelStatus>, BodhiErrorResponse> {
  Ok(Json(auth_scope.tunnels().sync_authorization().await?))
}

#[utoipa::path(
    delete,
    path = ENDPOINT_TUNNEL,
    tag = API_TAG_SETTINGS,
    operation_id = "disableTunnel",
    summary = "Stop the Cloudflare Tunnel connector",
    responses((status = 200, body = TunnelStatus)),
    security(("session_auth" = ["resource_admin"]))
)]
pub async fn tunnel_disable(
  auth_scope: AuthScope,
) -> Result<Json<TunnelStatus>, BodhiErrorResponse> {
  Ok(Json(auth_scope.tunnels().disable().await?))
}
