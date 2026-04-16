use axum::{
  extract::{rejection::JsonRejection, FromRequest, Request},
  response::{IntoResponse, Response},
  Json,
};
use std::collections::HashMap;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

#[derive(Debug)]
pub enum ValidationRejection {
  JsonRejection(JsonRejection),
  Validation(validator::ValidationErrors),
}

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
  T: serde::de::DeserializeOwned + Validate,
  S: Send + Sync,
{
  type Rejection = ValidationRejection;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let Json(value) = Json::<T>::from_request(req, state)
      .await
      .map_err(ValidationRejection::JsonRejection)?;
    value.validate().map_err(ValidationRejection::Validation)?;
    Ok(ValidatedJson(value))
  }
}

impl IntoResponse for ValidationRejection {
  fn into_response(self) -> Response {
    crate::BodhiErrorResponse::from(self).into_response()
  }
}

impl From<ValidationRejection> for crate::BodhiErrorResponse {
  fn from(value: ValidationRejection) -> Self {
    match value {
      ValidationRejection::JsonRejection(rejection) => {
        use crate::JsonRejectionError;
        let err = JsonRejectionError::from(rejection);
        crate::BodhiErrorResponse::from(err)
      }
      ValidationRejection::Validation(errors) => {
        let args: HashMap<String, String> = errors
          .field_errors()
          .into_iter()
          .map(|(field, errs)| {
            let msg = errs
              .first()
              .and_then(|e| e.message.as_ref().map(|m| m.to_string()))
              .unwrap_or_else(|| errs.first().map(|e| format!("{}", e)).unwrap_or_default());
            (field.to_string(), msg)
          })
          .collect();
        let param = if args.is_empty() { None } else { Some(args) };
        crate::BodhiErrorResponse {
          error: crate::BodhiError {
            message: "Validation failed".to_string(),
            r#type: "invalid_request_error".to_string(),
            code: Some("validation_error".to_string()),
            param,
          },
          status: 400,
        }
      }
    }
  }
}

#[cfg(test)]
#[path = "test_validated_json.rs"]
mod test_validated_json;
