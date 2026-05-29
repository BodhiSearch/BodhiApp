use crate::shared::AuthScope;
use crate::{BodhiErrorResponse, ValidatedJson};
use crate::{API_TAG_MODELS_ROUTER, ENDPOINT_MODELS_ROUTER};
use axum::{extract::Path, http::StatusCode, Json};
use services::{ModelRouterRequest, ModelRouterResponse};

/// Get a specific model-router configuration
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS_ROUTER.to_owned() + "/{id}",
    tag = API_TAG_MODELS_ROUTER,
    operation_id = "getModelRouter",
    summary = "Get Model Router Configuration",
    description = "Retrieves the configuration for a specific model-router (composite) alias by ID.",
    params(
        ("id" = String, Path, description = "Unique identifier for the model-router alias")
    ),
    responses(
        (status = 200, description = "Model-router configuration retrieved", body = ModelRouterResponse),
        (status = 404, description = "Model-router with specified ID not found", body = BodhiErrorResponse),
    ),
    security(
        ("session_auth" = ["resource_user"])
    )
)]
pub async fn model_router_show(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<ModelRouterResponse>, BodhiErrorResponse> {
  let result = auth_scope.model_routers().get(&id).await?;
  Ok(Json(result))
}

/// Create a new model-router configuration
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS_ROUTER,
    tag = API_TAG_MODELS_ROUTER,
    operation_id = "createModelRouter",
    request_body = ModelRouterRequest,
    responses(
        (status = 201, description = "Model-router created", body = ModelRouterResponse),
        (status = 400, description = "Validation error", body = BodhiErrorResponse),
    ),
    security(
        ("session_auth" = ["resource_user"])
    )
)]
pub async fn model_router_create(
  auth_scope: AuthScope,
  ValidatedJson(form): ValidatedJson<ModelRouterRequest>,
) -> Result<(StatusCode, Json<ModelRouterResponse>), BodhiErrorResponse> {
  let result = auth_scope.model_routers().create(form).await?;
  Ok((StatusCode::CREATED, Json(result)))
}

/// Update an existing model-router configuration
#[utoipa::path(
    put,
    path = ENDPOINT_MODELS_ROUTER.to_owned() + "/{id}",
    tag = API_TAG_MODELS_ROUTER,
    operation_id = "updateModelRouter",
    params(
        ("id" = String, Path, description = "Model-router ID")
    ),
    request_body = ModelRouterRequest,
    responses(
        (status = 200, description = "Model-router updated", body = ModelRouterResponse),
        (status = 404, description = "Model-router not found", body = BodhiErrorResponse),
    ),
    security(
        ("session_auth" = ["resource_user"])
    )
)]
pub async fn model_router_update(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  ValidatedJson(form): ValidatedJson<ModelRouterRequest>,
) -> Result<Json<ModelRouterResponse>, BodhiErrorResponse> {
  let result = auth_scope.model_routers().update(&id, form).await?;
  Ok(Json(result))
}

/// Delete a model-router configuration
#[utoipa::path(
    delete,
    path = ENDPOINT_MODELS_ROUTER.to_owned() + "/{id}",
    tag = API_TAG_MODELS_ROUTER,
    operation_id = "deleteModelRouter",
    params(
        ("id" = String, Path, description = "Model-router ID")
    ),
    responses(
        (status = 204, description = "Model-router deleted"),
        (status = 404, description = "Model-router not found", body = BodhiErrorResponse),
    ),
    security(
        ("session_auth" = ["resource_user"])
    )
)]
pub async fn model_router_destroy(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<StatusCode, BodhiErrorResponse> {
  auth_scope.model_routers().delete(&id).await?;
  Ok(StatusCode::NO_CONTENT)
}
