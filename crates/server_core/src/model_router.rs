use async_trait::async_trait;
use objs::{Alias, ApiAlias, AppError, ErrorType};
use services::{DataServiceError, DataService};
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ModelRouterError {
  #[error(transparent)]
  DataService(#[from] DataServiceError),

  #[error("Model '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ApiModelNotFound(String),
}

type Result<T> = std::result::Result<T, ModelRouterError>;

/// Represents the destination for a model request
#[derive(Debug, Clone)]
pub enum RouteDestination {
  /// Route to local model via existing SharedContext
  Local(Alias),
  /// Route to remote API via AiApiService
  Remote(ApiAlias),
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
}

impl DefaultModelRouter {
  pub fn new(data_service: Arc<dyn DataService>) -> Self {
    Self { data_service }
  }
}

#[async_trait]
impl ModelRouter for DefaultModelRouter {
  async fn route_request(&self, model: &str) -> Result<RouteDestination> {
    // Use unified DataService to find alias across all types
    if let Some(alias) = self.data_service.find_alias(model).await {
      match alias {
        // Local models (User and Model aliases) route to SharedContext
        Alias::User(_) | Alias::Model(_) => Ok(RouteDestination::Local(alias)),
        // Remote API models route to AiApiService
        Alias::Api(api_alias) => Ok(RouteDestination::Remote(api_alias)),
      }
    } else {
      // If nothing found, return not found error
      Err(ModelRouterError::ApiModelNotFound(model.to_string()))
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::model_router::{DefaultModelRouter, ModelRouter, ModelRouterError, RouteDestination};
  use mockall::predicate::eq;
  use objs::{Alias, ApiFormat};
  use rstest::rstest;
  use services::MockDataService;
  use std::sync::Arc;

  #[rstest]
  #[tokio::test]
  async fn test_route_to_user_alias() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    let user_alias = Alias::User(objs::UserAlias {
      alias: "test-model".to_string(),
      ..Default::default()
    });
    mock_data
      .expect_find_alias()
      .with(eq("test-model"))
      .times(1)
      .returning(move |_| Some(user_alias.clone()));

    let router = DefaultModelRouter::new(Arc::new(mock_data));
    let result = router.route_request("test-model").await?;

    match result {
      RouteDestination::Local(Alias::User(alias)) => {
        assert_eq!(alias.alias, "test-model");
      }
      _ => panic!("Expected Local destination with user alias"),
    }

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_route_to_model_alias() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    let model_alias = Alias::Model(objs::ModelAlias {
      alias: "test-model".to_string(),
      ..Default::default()
    });
    mock_data
      .expect_find_alias()
      .with(eq("test-model"))
      .times(1)
      .returning(move |_| Some(model_alias.clone()));

    let router = DefaultModelRouter::new(Arc::new(mock_data));
    let result = router.route_request("test-model").await?;

    match result {
      RouteDestination::Local(Alias::Model(alias)) => {
        assert_eq!(alias.alias, "test-model");
      }
      _ => panic!("Expected Local destination with model alias"),
    }

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_route_to_api_model() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    let api_alias = Alias::Api(
      objs::ApiAliasBuilder::test_default()
        .id("test-api")
        .base_url("https://api.openai.com/v1")
        .models(vec!["gpt-3.5-turbo".to_string()])
        .build_with_time(chrono::Utc::now())
        .unwrap(),
    );
    mock_data
      .expect_find_alias()
      .with(eq("test-api"))
      .returning(move |_| Some(api_alias.clone()));
    let router = DefaultModelRouter::new(Arc::new(mock_data));
    let result = router.route_request("test-api").await?;
    match result {
      RouteDestination::Remote(alias) => {
        assert_eq!(alias.id, "test-api");
        assert_eq!(alias.api_format, ApiFormat::OpenAI);
      }
      _ => panic!("Expected Remote destination with API alias"),
    }
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_user_alias_overrides_model_alias() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    let user_alias = Alias::User(objs::UserAlias {
      alias: "test-model".to_string(),
      repo: "user/repo".parse().unwrap(),
      ..Default::default()
    });

    mock_data
      .expect_find_alias()
      .with(eq("test-model"))
      .times(1)
      .returning(move |_| Some(user_alias.clone()));

    let router = DefaultModelRouter::new(Arc::new(mock_data));
    let result = router.route_request("test-model").await?;
    match result {
      RouteDestination::Local(Alias::User(alias)) => {
        assert_eq!(alias.repo.to_string(), "user/repo");
      }
      _ => panic!("Expected Local destination with user alias"),
    }
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_model_not_found() -> anyhow::Result<()> {
    let mut mock_data = MockDataService::new();
    // No aliases found anywhere
    mock_data
      .expect_find_alias()
      .with(eq("unknown-model"))
      .times(1)
      .returning(|_| None);

    let router = DefaultModelRouter::new(Arc::new(mock_data));
    let result = router.route_request("unknown-model").await;
    assert!(matches!(result, Err(ModelRouterError::ApiModelNotFound(_))));
    Ok(())
  }
}
