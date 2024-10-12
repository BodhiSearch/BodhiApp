use objs::{AppError, ErrorType};
use tokio::task::JoinError;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("task_join_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
pub struct TaskJoinError {
  #[from]
  source: JoinError,
}

#[cfg(test)]
mod tests {
  use crate::{test_utils::setup_l10n_server_app, TaskJoinError};
  use objs::{test_utils::assert_error_message, AppError, FluentLocalizationService};
  use rstest::rstest;
  use std::sync::Arc;
  use tokio::task::JoinError;

  async fn build_join_error() -> JoinError {
    let handle = tokio::spawn(async move {
      async fn null() {}
      null().await;
      if true {
        panic!("fail");
      }
      ()
    })
    .await;
    assert!(handle.is_err());
    handle.unwrap_err()
  }

  #[rstest]
  #[serial_test::serial(localization)]
  #[tokio::test]
  async fn test_task_join_error_messages(
    #[from(setup_l10n_server_app)] localization_service: Arc<FluentLocalizationService>,
  ) {
    let join_error = build_join_error().await;
    let expected = format!("failed to join task: {}", join_error);
    let error = TaskJoinError::from(join_error);
    assert_error_message(localization_service, &error.code(), error.args(), &expected);
  }
}
