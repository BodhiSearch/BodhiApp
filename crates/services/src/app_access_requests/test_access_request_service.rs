use anyhow_trace::anyhow_trace;
use chrono::Duration;
use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use std::sync::Arc;

use crate::{
  app_access_requests::{
    AppAccessRequestStatus, ApprovedResources, ApprovedResourcesV1, CreateApprovedAccessRequest,
    RequestedResources, RequestedResourcesV1,
  },
  db::DbService,
  test_utils::{
    approved_request, test_db_service, FrozenTimeService, TestDbService, TEST_TENANT_ID,
  },
  AccessRequestRepository, AccessRequestService, DefaultAccessRequestService, MockAuthService,
  ModelGrant, UserScope,
};

#[fixture]
#[awt]
async fn access_request_service(
  #[default(MockAuthService::new())] mock_auth: MockAuthService,
  #[future] test_db_service: TestDbService,
) -> (Arc<TestDbService>, DefaultAccessRequestService) {
  let db = Arc::new(test_db_service);
  let service = DefaultAccessRequestService::new(
    db.clone() as Arc<dyn DbService>,
    Arc::new(mock_auth),
    Arc::new(FrozenTimeService::default()),
  );
  (db, service)
}

fn create_input() -> CreateApprovedAccessRequest {
  CreateApprovedAccessRequest {
    app_client_id: "app-client-1".to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "user-1".to_string(),
    resource_client_id: "resource-client".to_string(),
    requested: RequestedResources::V1(RequestedResourcesV1 {
      models_list: true,
      ..Default::default()
    }),
    requested_role: UserScope::User,
    approved: ApprovedResources::V1(ApprovedResourcesV1 {
      models_access: ModelGrant::Specific {
        ids: vec!["m1".to_string()],
      },
      ..Default::default()
    }),
    approved_role: UserScope::PowerUser,
    source_access_request_id: None,
  }
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_access_request_service_create_approved(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (_db, service) = access_request_service;
  let input = create_input();

  let result = service.create_approved(input.clone()).await?;

  assert_eq!(AppAccessRequestStatus::Approved, result.status);
  assert_eq!(Some(TEST_TENANT_ID.to_string()), result.tenant_id);
  assert_eq!(Some("user-1".to_string()), result.user_id);
  assert_eq!("app-client-1", result.app_client_id);
  assert_eq!("scope_user_user", result.requested_role);
  assert_eq!(
    Some("scope_user_power_user".to_string()),
    result.approved_role
  );
  assert_eq!(
    Some(format!(
      "scope_access_request:resource-client.{}",
      result.id
    )),
    result.access_request_scope
  );
  assert_eq!(None, result.source_access_request_id);
  assert_eq!(result.created_at, result.expires_at);

  let requested: RequestedResources = serde_json::from_str(&result.requested)?;
  assert_eq!(input.requested, requested);
  let approved_json = result.approved.expect("approved JSON must be stored");
  let approved: ApprovedResources = serde_json::from_str(&approved_json)?;
  assert_eq!(input.approved, approved);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_access_request_service_create_approved_invalid_resource_client(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (_db, service) = access_request_service;
  let input = CreateApprovedAccessRequest {
    resource_client_id: "".to_string(),
    ..create_input()
  };

  let err = service.create_approved(input).await.unwrap_err();
  assert_eq!("app_scope_error-invalid_scope_composition", err.code());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_access_request_service_get_request_tenant_scoped(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (_db, service) = access_request_service;
  let created = service.create_approved(create_input()).await?;

  let hit = service.get_request(TEST_TENANT_ID, &created.id).await?;
  assert_eq!(Some(created.clone()), hit);

  let miss = service.get_request("other-tenant", &created.id).await?;
  assert_eq!(None, miss);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_access_request_service_latest_approved_for_app_user(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;
  let now = db.now();

  db.create(&approved_request("ar-older", TEST_TENANT_ID, "owner", now))
    .await?;
  db.create(&approved_request(
    "ar-newer",
    TEST_TENANT_ID,
    "owner",
    now + Duration::minutes(5),
  ))
  .await?;
  // Distractors: other user, other app client, non-approved status.
  db.create(&approved_request(
    "ar-other-user",
    TEST_TENANT_ID,
    "other-user",
    now + Duration::minutes(10),
  ))
  .await?;
  let mut other_app = approved_request(
    "ar-other-app",
    TEST_TENANT_ID,
    "owner",
    now + Duration::minutes(10),
  );
  other_app.app_client_id = "other-client".to_string();
  db.create(&other_app).await?;
  let mut revoked = approved_request(
    "ar-revoked",
    TEST_TENANT_ID,
    "owner",
    now + Duration::minutes(10),
  );
  revoked.status = AppAccessRequestStatus::Revoked;
  db.create(&revoked).await?;

  let latest = service
    .latest_approved_for_app_user(TEST_TENANT_ID, "test-client", "owner")
    .await?;
  assert_eq!(Some("ar-newer".to_string()), latest.map(|row| row.id),);

  let none = service
    .latest_approved_for_app_user(TEST_TENANT_ID, "unknown-client", "owner")
    .await?;
  assert_eq!(None, none);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_access_request_service_revoke_request(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;
  let now = db.now();
  db.create(&approved_request("ar-revoke", TEST_TENANT_ID, "owner", now))
    .await?;

  let revoked = service
    .revoke_request(TEST_TENANT_ID, "ar-revoke", "owner")
    .await?;
  assert_eq!(AppAccessRequestStatus::Revoked, revoked.status);

  let listed = service
    .list_approved_for_user(TEST_TENANT_ID, "owner")
    .await?;
  assert_eq!(0, listed.len());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_access_request_service_build_authorize_endpoint(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let mut mock_auth = MockAuthService::new();
  mock_auth
    .expect_authorize_url()
    .return_const("https://auth.example.com/realms/r/protocol/openid-connect/auth".to_string());
  let service = DefaultAccessRequestService::new(
    Arc::new(test_db_service) as Arc<dyn DbService>,
    Arc::new(mock_auth),
    Arc::new(FrozenTimeService::default()),
  );

  assert_eq!(
    "https://auth.example.com/realms/r/protocol/openid-connect/auth",
    service.build_authorize_endpoint()
  );

  Ok(())
}
