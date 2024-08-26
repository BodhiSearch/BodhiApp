use super::RouterStateFn;
use crate::service::{HttpError, KEY_APP_AUTHZ, KEY_APP_STATUS};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct AppInfo {
  version: String,
  status: String,
  authz: bool,
}

pub(crate) async fn app_info_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
) -> Result<Json<AppInfo>, HttpError> {
  let secret_service = &state.app_service().secret_service();
  let authz = secret_service
    .get_secret_string(KEY_APP_AUTHZ)
    .unwrap_or(Some("true".to_string()))
    .unwrap_or("true".to_string());
  let status = secret_service
    .get_secret_string(KEY_APP_STATUS)
    .unwrap_or(Some("setup".to_string()))
    .unwrap_or("setup".to_string());
  Ok(Json(AppInfo {
    version: env!("CARGO_PKG_VERSION").to_string(),
    status,
    authz: authz == "true",
  }))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    server::RouterState,
    test_utils::{AppServiceStubBuilder, MockSharedContext, ResponseTestExt, SecretServiceStub},
  };
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
  };
  use rstest::rstest;
  use std::sync::Arc;
  use tower::ServiceExt;

  #[rstest]
  #[case(
    SecretServiceStub::new(),
    AppInfo {
      version: env!("CARGO_PKG_VERSION").to_string(),
      status: "setup".to_string(),
      authz: true,
    }
  )]
  #[case(
    SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_STATUS.to_string() => "setup".to_string(),
      KEY_APP_AUTHZ.to_string() => "true".to_string(),
    }),
    AppInfo {
      version: env!("CARGO_PKG_VERSION").to_string(),
      status: "setup".to_string(),
      authz: true,
    }
  )]
  #[case(
    SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_STATUS.to_string() => "setup".to_string(),
      KEY_APP_AUTHZ.to_string() => "false".to_string(),
    }),
    AppInfo {
      version: env!("CARGO_PKG_VERSION").to_string(),
      status: "setup".to_string(),
      authz: false,
    }
  )]
  #[tokio::test]
  async fn test_app_info_handler(
    #[case] secret_service: SecretServiceStub,
    #[case] expected: AppInfo,
  ) -> anyhow::Result<()> {
    let app_service = AppServiceStubBuilder::default()
      .secret_service(Arc::new(secret_service))
      .build()?;
    let state = Arc::new(RouterState::new(
      Arc::new(MockSharedContext::default()),
      Arc::new(app_service),
    ));
    let router = Router::new()
      .route("/app/info", get(app_info_handler))
      .with_state(state);
    let resp = router
      .oneshot(Request::get("/app/info").body(Body::empty())?)
      .await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let value = resp.json::<AppInfo>().await?;
    assert_eq!(expected, value);
    Ok(())
  }
}
