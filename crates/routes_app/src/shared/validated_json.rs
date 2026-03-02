use axum::{
  body::Body,
  extract::{FromRequest, Request},
  Json,
};
use serde::de::DeserializeOwned;
use crate::{ApiError, JsonRejectionError};
use services::ObjValidationError;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = ApiError;

  async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
    let Json(value) = Json::<T>::from_request(req, state)
      .await
      .map_err(|e| ApiError::from(JsonRejectionError::from(e)))?;
    value
      .validate()
      .map_err(|e| ApiError::from(ObjValidationError::from(e)))?;
    Ok(ValidatedJson(value))
  }
}
