use crate::shared::AuthScope;
use crate::{ApiError, ValidatedJson};
use crate::{API_TAG_MODELS_ALIAS, ENDPOINT_MODELS_ALIAS};
use axum::{extract::Path, http::StatusCode, Json};
use services::{CopyAliasRequest, UserAliasResponse};

/// Create Alias
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS_ALIAS,
    tag = API_TAG_MODELS_ALIAS,
    operation_id = "models_alias_create",
    request_body = services::UserAliasRequest,
    responses(
      (status = 201, description = "Alias created succesfully", body = UserAliasResponse),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn models_create(
  auth_scope: AuthScope,
  ValidatedJson(form): ValidatedJson<services::UserAliasRequest>,
) -> Result<(StatusCode, Json<UserAliasResponse>), ApiError> {
  let alias = auth_scope.data().create_alias_from_form(form).await?;
  Ok((StatusCode::CREATED, Json(UserAliasResponse::from(alias))))
}

/// Update Alias
#[utoipa::path(
    put,
    path = ENDPOINT_MODELS_ALIAS.to_owned() + "/{id}",
    tag = API_TAG_MODELS_ALIAS,
    params(
        ("id" = String, Path, description = "UUID of the alias to update")
    ),
    operation_id = "models_alias_update",
    request_body = services::UserAliasRequest,
    responses(
      (status = 200, description = "Alias updated succesfully", body = UserAliasResponse),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn models_update(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  ValidatedJson(form): ValidatedJson<services::UserAliasRequest>,
) -> Result<(StatusCode, Json<UserAliasResponse>), ApiError> {
  let updated_alias = auth_scope.data().update_alias_from_form(&id, form).await?;
  Ok((StatusCode::OK, Json(UserAliasResponse::from(updated_alias))))
}

/// Delete a model alias by UUID
#[utoipa::path(
    delete,
    path = ENDPOINT_MODELS_ALIAS.to_owned() + "/{id}",
    tag = API_TAG_MODELS_ALIAS,
    operation_id = "models_alias_destroy",
    params(("id" = String, Path, description = "UUID of the alias to delete")),
    responses(
      (status = 200, description = "Alias deleted successfully"),
      (status = 404, description = "Alias not found"),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn models_destroy(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
  auth_scope.data().delete_alias(&id).await?;
  Ok(StatusCode::OK)
}

/// Copy a model alias
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS_ALIAS.to_owned() + "/{id}/copy",
    tag = API_TAG_MODELS_ALIAS,
    operation_id = "models_alias_copy",
    params(("id" = String, Path, description = "UUID of the alias to copy")),
    request_body = CopyAliasRequest,
    responses(
      (status = 201, description = "Alias copied successfully", body = UserAliasResponse),
      (status = 404, description = "Source alias not found"),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn models_copy(
  auth_scope: AuthScope,
  Path(id): Path<String>,
  ValidatedJson(payload): ValidatedJson<CopyAliasRequest>,
) -> Result<(StatusCode, Json<UserAliasResponse>), ApiError> {
  let new_alias = auth_scope.data().copy_alias(&id, &payload.alias).await?;
  Ok((
    StatusCode::CREATED,
    Json(UserAliasResponse::from(new_alias)),
  ))
}

#[cfg(test)]
#[path = "test_aliases_crud.rs"]
mod test_aliases_crud;

#[cfg(test)]
#[path = "test_aliases_index.rs"]
mod test_aliases_index;

#[cfg(test)]
#[path = "test_aliases_api_formats.rs"]
mod test_aliases_api_formats;
