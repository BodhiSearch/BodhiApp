use objs::SettingSource;
use server_core::SharedContext;
use services::{SettingsChangeListener, BODHI_EXEC_VARIANT};
use std::sync::Arc;
use tokio::task;

#[derive(Debug, derive_new::new)]
pub struct VariantChangeListener {
  ctx: Arc<dyn SharedContext>,
}

impl SettingsChangeListener for VariantChangeListener {
  fn on_change(
    &self,
    key: &str,
    prev_value: &Option<serde_yaml::Value>,
    _prev_source: &SettingSource,
    new_value: &Option<serde_yaml::Value>,
    _new_source: &SettingSource,
  ) {
    if key != BODHI_EXEC_VARIANT {
      return;
    }
    if prev_value.is_some() && new_value.is_some() && prev_value.as_ref() == new_value.as_ref() {
      return;
    }
    let ctx_clone = self.ctx.clone();
    let new_value = if let Some(serde_yaml::Value::String(new_value)) = new_value {
      new_value.to_string()
    } else {
      tracing::error!(
        "BODHI_EXEC_VARIANT is not set, or not a string type, skipping updating the server config"
      );
      return;
    };
    task::spawn(async move {
      if let Err(err) = ctx_clone.set_exec_variant(&new_value).await {
        tracing::error!(?err, "failed to set exec variant");
      }
    });
  }
}

#[cfg(test)]
mod tests {
  use super::VariantChangeListener;
  use mockall::predicate::eq;
  use objs::SettingSource;
  use rstest::rstest;
  use server_core::{ContextError, MockSharedContext};
  use services::{SettingsChangeListener, BODHI_EXEC_VARIANT};
  use std::sync::Arc;

  #[rstest]
  #[tokio::test]
  async fn test_variant_change_listener_triggers_if_bodhi_exec_variant_changes() {
    let mut mock_shared_ctx = MockSharedContext::default();
    mock_shared_ctx
      .expect_set_exec_variant()
      .with(eq("cpu".to_string()))
      .times(1)
      .returning(|_| Ok(()));
    let listener = VariantChangeListener::new(Arc::new(mock_shared_ctx));

    listener.on_change(
      BODHI_EXEC_VARIANT,
      &None,
      &SettingSource::Default,
      &Some(serde_yaml::Value::String("cpu".to_string())),
      &SettingSource::Default,
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_variant_change_listener_handles_error() {
    let mut mock_shared_ctx = MockSharedContext::default();
    mock_shared_ctx
      .expect_set_exec_variant()
      .with(eq("cpu".to_string()))
      .times(1)
      .returning(|_| Err(ContextError::ExecNotExists("cpu".to_string())));
    let listener = VariantChangeListener::new(Arc::new(mock_shared_ctx));
    listener.on_change(
      BODHI_EXEC_VARIANT,
      &None,
      &SettingSource::Default,
      &Some(serde_yaml::Value::String("cpu".to_string())),
      &SettingSource::Default,
    );
  }

  #[rstest]
  #[case::key_does_not_match("some_other_key", None, SettingSource::Default, Some(serde_yaml::Value::String("new_variant".to_string())), SettingSource::Default)]
  #[case::no_change_in_value(BODHI_EXEC_VARIANT, Some(serde_yaml::Value::String("cpu".to_string())), SettingSource::Default, Some(serde_yaml::Value::String("cpu".to_string())), SettingSource::Database)]
  #[case::invalid_value(
    BODHI_EXEC_VARIANT,
    None,
    SettingSource::Default,
    Some(serde_yaml::Value::Null),
    SettingSource::Database
  )]
  #[tokio::test]
  async fn test_variant_change_listener_noop(
    #[case] key: &str,
    #[case] prev_value: Option<serde_yaml::Value>,
    #[case] prev_source: SettingSource,
    #[case] new_value: Option<serde_yaml::Value>,
    #[case] new_source: SettingSource,
  ) {
    let mut mock_shared_ctx = MockSharedContext::default();
    mock_shared_ctx.expect_set_exec_variant().never();
    let listener = VariantChangeListener::new(Arc::new(mock_shared_ctx));
    listener.on_change(key, &prev_value, &prev_source, &new_value, &new_source); // so that the async task completes
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  }
}
