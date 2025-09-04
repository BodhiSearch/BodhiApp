use async_trait::async_trait;
use objs::{UserAlias, AliasSource, ApiModelAlias, AppError, ErrorType};
use services::{
  db::{DbError, DbService},
  AliasNotFoundError, DataService,
};
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ModelRouterError {
  #[error(transparent)]
  AliasNotFound(#[from] AliasNotFoundError),

  #[error("ai_api_model_not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ApiModelNotFound(String),

  #[error(transparent)]
  DatabaseError(#[from] DbError),
}

type Result<T> = std::result::Result<T, ModelRouterError>;

/// Represents the destination for a model request
#[derive(Debug, Clone)]
pub enum RouteDestination {
  /// Route to local model via existing SharedContext
  Local(UserAlias),
  /// Route to remote API via AiApiService
  Remote(ApiModelAlias),
}

/// Service for routing model requests to appropriate destinations
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait ModelRouter: Send + Sync + std::fmt::Debug {
  /// Route a model request to the appropriate destination
  ///
  /// Resolution order (corrected per user feedback):
  /// 1. User alias (first priority)
  /// 2. Model alias (second priority)
  /// 3. API models (last priority)
  async fn route_request(&self, model: &str) -> Result<RouteDestination>;
}

/// Default implementation of ModelRouter
#[derive(Debug, Clone)]
pub struct DefaultModelRouter {
  data_service: Arc<dyn DataService>,
  db_service: Arc<dyn DbService>,
}

impl DefaultModelRouter {
  pub fn new(data_service: Arc<dyn DataService>, db_service: Arc<dyn DbService>) -> Self {
    Self {
      data_service,
      db_service,
    }
  }
}

#[async_trait]
impl ModelRouter for DefaultModelRouter {
  async fn route_request(&self, model: &str) -> Result<RouteDestination> {
    // Step 1: Check user aliases first (highest priority)
    if let Some(alias) = self
      .data_service
      .find_alias(model)
      .filter(|a| matches!(a.source, AliasSource::User))
    {
      return Ok(RouteDestination::Local(alias));
    }

    // Step 2: Check model aliases second (middle priority)
    if let Some(alias) = self
      .data_service
      .find_alias(model)
      .filter(|a| matches!(a.source, AliasSource::Model))
    {
      return Ok(RouteDestination::Local(alias));
    }

    // Step 3: Check API models last (lowest priority)
    if let Some(api_alias) = self.db_service.get_api_model_alias(model).await? {
      return Ok(RouteDestination::Remote(api_alias));
    }

    // If nothing found, return not found error
    Err(ModelRouterError::ApiModelNotFound(model.to_string()))
  }
}

#[cfg(test)]
mod tests {
  use crate::model_router::{DefaultModelRouter, ModelRouter, ModelRouterError, RouteDestination};
  use objs::{AliasSource, ApiModelAlias};
  use rstest::rstest;
  use services::db::MockDbService;
  use services::MockDataService;
  use std::sync::Arc;

  #[rstest]
  #[tokio::test]
  async fn test_route_to_user_alias_first_priority() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    let mock_db = MockDbService::new();

    let user_alias = objs::UserAlias {
      alias: "test-model".to_string(),
      source: AliasSource::User,
      ..Default::default()
    };

    mock_data
      .expect_find_alias()
      .with(mockall::predicate::eq("test-model"))
      .times(1)
      .returning(move |_| Some(user_alias.clone()));

    let router = DefaultModelRouter::new(Arc::new(mock_data), Arc::new(mock_db));
    let result = router.route_request("test-model").await?;

    match result {
      RouteDestination::Local(alias) => {
        assert_eq!(alias.alias, "test-model");
        assert!(matches!(alias.source, AliasSource::User));
      }
      _ => panic!("Expected Local destination with user alias"),
    }

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_route_to_model_alias_second_priority() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    let mock_db = MockDbService::new();

    let model_alias = objs::UserAlias {
      alias: "test-model".to_string(),
      source: AliasSource::Model,
      ..Default::default()
    };

    mock_data
      .expect_find_alias()
      .with(mockall::predicate::eq("test-model"))
      .times(2)
      .returning({
        let alias = model_alias.clone();
        move |_| {
          static mut CALL_COUNT: usize = 0;
          unsafe {
            CALL_COUNT += 1;
            match CALL_COUNT {
              1 => None,
              2 => Some(alias.clone()),
              _ => None,
            }
          }
        }
      });

    let router = DefaultModelRouter::new(Arc::new(mock_data), Arc::new(mock_db));
    let result = router.route_request("test-model").await?;

    match result {
      RouteDestination::Local(alias) => {
        assert_eq!(alias.alias, "test-model");
        assert!(matches!(alias.source, AliasSource::Model));
      }
      _ => panic!("Expected Local destination with model alias"),
    }

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_route_to_api_model_third_priority() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    let mut mock_db = MockDbService::new();

    let api_alias = ApiModelAlias::new(
      "test-api",
      AliasSource::RemoteApi,
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-3.5-turbo".to_string()],
      chrono::Utc::now(),
    );

    mock_data
      .expect_find_alias()
      .with(mockall::predicate::eq("test-api"))
      .times(2)
      .returning(|_| None);

    mock_db
      .expect_get_api_model_alias()
      .with(mockall::predicate::eq("test-api"))
      .times(1)
      .returning(move |_| Ok(Some(api_alias.clone())));

    let router = DefaultModelRouter::new(Arc::new(mock_data), Arc::new(mock_db));
    let result = router.route_request("test-api").await?;

    match result {
      RouteDestination::Remote(alias) => {
        assert_eq!(alias.id, "test-api");
        assert_eq!(alias.provider, "openai");
      }
      _ => panic!("Expected Remote destination with API alias"),
    }

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_alias_overrides_model_alias() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    let mock_db = MockDbService::new();

    let user_alias = objs::UserAlias {
      alias: "test-model".to_string(),
      source: AliasSource::User,
      repo: "user/repo".parse().unwrap(),
      ..Default::default()
    };

    mock_data
      .expect_find_alias()
      .with(mockall::predicate::eq("test-model"))
      .times(1)
      .returning(move |_| Some(user_alias.clone()));

    let router = DefaultModelRouter::new(Arc::new(mock_data), Arc::new(mock_db));
    let result = router.route_request("test-model").await?;

    match result {
      RouteDestination::Local(alias) => {
        assert_eq!(alias.repo.to_string(), "user/repo");
        assert!(matches!(alias.source, AliasSource::User));
      }
      _ => panic!("Expected Local destination with user alias"),
    }

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_model_not_found() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    let mut mock_db = MockDbService::new();

    // No aliases found anywhere
    mock_data
      .expect_find_alias()
      .with(mockall::predicate::eq("unknown-model"))
      .times(2)
      .returning(|_| None);

    mock_db
      .expect_get_api_model_alias()
      .with(mockall::predicate::eq("unknown-model"))
      .times(1)
      .returning(|_| Ok(None));

    let router = DefaultModelRouter::new(Arc::new(mock_data), Arc::new(mock_db));
    let result = router.route_request("unknown-model").await;
    assert!(matches!(result, Err(ModelRouterError::ApiModelNotFound(_))));
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_database_error_propagation() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    let mut mock_db = MockDbService::new();

    mock_data
      .expect_find_alias()
      .with(mockall::predicate::eq("error-model"))
      .times(2)
      .returning(|_| None);

    mock_db
      .expect_get_api_model_alias()
      .with(mockall::predicate::eq("error-model"))
      .times(1)
      .returning(|_| {
        Err(services::db::DbError::TokenValidation(
          "Connection failed".to_string(),
        ))
      });

    let router = DefaultModelRouter::new(Arc::new(mock_data), Arc::new(mock_db));
    let result = router.route_request("error-model").await;

    assert!(matches!(result, Err(ModelRouterError::DatabaseError(_))));
    Ok(())
  }
}
