#[derive(
  Debug, Clone, PartialEq, Eq, strum::Display, strum::AsRefStr, strum::EnumString, Default,
)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorType {
  #[strum(serialize = "invalid_request_error")]
  BadRequest,
  #[strum(serialize = "invalid_app_state")]
  InvalidAppState,
  #[strum(serialize = "internal_server_error")]
  InternalServer,
  #[strum(serialize = "authentication_error")]
  Authentication,
  #[strum(serialize = "forbidden_error")]
  Forbidden,
  #[strum(serialize = "not_found_error")]
  NotFound,
  #[strum(serialize = "conflict_error")]
  Conflict,
  #[strum(serialize = "unprocessable_entity_error")]
  UnprocessableEntity,
  #[default]
  #[strum(serialize = "unknown_error")]
  Unknown,
  #[strum(serialize = "service_unavailable")]
  ServiceUnavailable,
}

impl ErrorType {
  pub fn status(&self) -> u16 {
    match self {
      ErrorType::InternalServer => 500,
      ErrorType::BadRequest => 400,
      ErrorType::InvalidAppState => 500,
      ErrorType::Authentication => 401,
      ErrorType::NotFound => 404,
      ErrorType::Conflict => 409,
      ErrorType::UnprocessableEntity => 422,
      ErrorType::Unknown => 500,
      ErrorType::Forbidden => 403,
      ErrorType::ServiceUnavailable => 503,
    }
  }
}

#[cfg(test)]
#[path = "test_error_type.rs"]
mod test_error_type;
