use objs::{AppError, ErrorType};
use tokio::task::JoinError;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Background task failed: {source}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct TaskJoinError {
  #[from]
  source: JoinError,
}

#[cfg(test)]
mod tests {
  use crate::TaskJoinError;
  use objs::AppError;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  async fn build_join_error() -> tokio::task::JoinError {
    let handle = tokio::spawn(async move {
      async fn null() {}
      null().await;
      if true {
        panic!("fail");
      }
    })
    .await;
    assert!(handle.is_err());
    handle.unwrap_err()
  }

  #[rstest]
  #[tokio::test]
  async fn test_task_join_error_display() {
    let join_error = build_join_error().await;
    let error = TaskJoinError::from(join_error);
    let message = error.to_string();
    assert!(message.starts_with("Background task failed: "));
    assert_eq!("task_join_error", error.code());
  }
}
