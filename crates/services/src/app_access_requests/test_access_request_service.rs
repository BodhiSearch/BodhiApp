use crate::{
  app_access_requests::{
    test_access_request_builders::{approved_request, make_request},
    AccessRequestRepository, AppAccessRequest, AppAccessRequestStatus, ApprovedResources,
    ApprovedResourcesV1, RequestedResources, RequestedResourcesV1,
  },
  db::DbService,
  test_utils::{
    test_db_service, FrozenTimeService, SettingServiceStub, StubNetworkService, TestDbService,
    TEST_TENANT_ID,
  },
  AccessRequestService, AuthServiceError, DefaultAccessRequestService, MockAuthService,
  RegisterAccessRequestConsentResponse, UserScope,
};
use anyhow_trace::anyhow_trace;
use chrono::Duration;
use errmeta::AppError;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use std::sync::Arc;

/// The service under test plus the shared db handle (for seeding rows). `mock_auth`
/// stubs the Keycloak consent call for the approve path; override it per-test with
/// `#[with(stub_consent_ok())]`. The default empty mock suits tests that never reach
/// Keycloak (validation/expiry/already-processed paths).
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
    Arc::new(SettingServiceStub::default()),
    Arc::new(StubNetworkService { ip: None }),
  );
  (db, service)
}

/// Keycloak consent succeeds, echoing back the access-request scope.
fn stub_consent_ok() -> MockAuthService {
  let mut mock = MockAuthService::new();
  mock
    .expect_register_access_request_consent()
    .returning(|_token, _client_id, id, _desc| {
      Ok(RegisterAccessRequestConsentResponse {
        access_request_id: id.to_string(),
        access_request_scope: format!("scope_access_request:{}", id),
      })
    });
  mock
}

/// Keycloak consent fails with the given error (approve calls it exactly once).
fn stub_consent_err(err: AuthServiceError) -> MockAuthService {
  let mut mock = MockAuthService::new();
  mock
    .expect_register_access_request_consent()
    .return_once(move |_token, _client_id, _id, _desc| Err(err));
  mock
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_draft_valid(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (_db, service) = access_request_service;

  let result = service
    .create_draft(
      "app-client-1".to_string(),
      RequestedResources::V1(RequestedResourcesV1 {
        mcp_servers: vec![],
        ..Default::default()
      }),
      UserScope::User,
      None,
    )
    .await?;

  assert_eq!(AppAccessRequestStatus::Draft, result.status);
  assert_eq!("app-client-1", result.app_client_id);
  assert_eq!("scope_user_user", result.requested_role);
  assert_eq!(None, result.approved_role);
  assert_eq!(None, result.user_id);
  assert_eq!(None, result.tenant_id);
  assert_eq!(None, result.source_access_request_id);
  assert!(
    result.requested.contains(r#""version":"1""#),
    "Expected serialized requested to contain version tag, got: {}",
    result.requested
  );

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_build_review_url_reflects_request_host(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;

  // Loopback request host is reflected (fixture network service detects no IP).
  assert_eq!(
    "http://127.0.0.1:1135/ui/apps/access-requests/review?id=ar-1",
    service.build_review_url(Some("127.0.0.1"), "ar-1").await
  );

  // No request host → falls back to the configured public server URL (localhost default).
  assert_eq!(
    "http://localhost:1135/ui/apps/access-requests/review?id=ar-2",
    service.build_review_url(None, "ar-2").await
  );

  // A LAN IP that matches the detected server IP is reflected; a mismatch falls back.
  let with_ip = DefaultAccessRequestService::new(
    db.clone() as Arc<dyn DbService>,
    Arc::new(MockAuthService::new()),
    Arc::new(FrozenTimeService::default()),
    Arc::new(SettingServiceStub::default()),
    Arc::new(StubNetworkService {
      ip: Some("192.168.1.42".to_string()),
    }),
  );
  assert_eq!(
    "http://192.168.1.42:1135/ui/apps/access-requests/review?id=ar-3",
    with_ip.build_review_url(Some("192.168.1.42"), "ar-3").await
  );
  assert_eq!(
    "http://localhost:1135/ui/apps/access-requests/review?id=ar-4",
    with_ip.build_review_url(Some("10.0.0.9"), "ar-4").await
  );

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_already_processed(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;
  let now = db.now();
  db.create(&approved_request(
    "ar-approved-001",
    TEST_TENANT_ID,
    "user-1",
    now,
  ))
  .await?;

  let result = service
    .approve_request(
      "ar-approved-001",
      "user-2",
      TEST_TENANT_ID,
      "fake-token",
      ApprovedResources::default(),
      UserScope::User,
    )
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("access_request_error-already_processed", err.code());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_threads_approved_role(
  #[future]
  #[with(stub_consent_ok())]
  access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;
  let now = db.now();
  db.create(&make_request("ar-draft-approve", TEST_TENANT_ID, now))
    .await?;

  let result = service
    .approve_request(
      "ar-draft-approve",
      "user-1",
      TEST_TENANT_ID,
      "fake-token",
      ApprovedResources::V1(ApprovedResourcesV1 {
        mcps: vec![],
        ..Default::default()
      }),
      UserScope::PowerUser,
    )
    .await?;

  assert_eq!(AppAccessRequestStatus::Approved, result.status);
  assert_eq!(Some("user-1".to_string()), result.user_id);
  assert_eq!(
    Some("scope_user_power_user".to_string()),
    result.approved_role
  );
  assert_eq!(
    Some("scope_access_request:ar-draft-approve".to_string()),
    result.access_request_scope
  );

  Ok(())
}

/// KC returns a 409 / UUID-collision error: the service records the failure and
/// returns the Failed row (NOT an Err).
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_kc_409_collision_marks_failed(
  #[future]
  #[with(stub_consent_err(AuthServiceError::AuthServiceApiError {
    status: 409,
    body: "UUID collision".to_string(),
  }))]
  access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;
  let now = db.now();
  db.create(&make_request("ar-kc-409", TEST_TENANT_ID, now))
    .await?;

  let result = service
    .approve_request(
      "ar-kc-409",
      "user-1",
      TEST_TENANT_ID,
      "fake-token",
      ApprovedResources::default(),
      UserScope::User,
    )
    .await?;

  assert_eq!(AppAccessRequestStatus::Failed, result.status);
  assert!(
    result.error_message.is_some(),
    "Expected a recorded error_message on the Failed row"
  );

  Ok(())
}

/// KC returns any non-409 error: the service surfaces KcRegistrationFailed.
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_kc_error_returns_err(
  #[future]
  #[with(stub_consent_err(AuthServiceError::AuthServiceApiError {
    status: 500,
    body: "boom".to_string(),
  }))]
  access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;
  let now = db.now();
  db.create(&make_request("ar-kc-err", TEST_TENANT_ID, now))
    .await?;

  let result = service
    .approve_request(
      "ar-kc-err",
      "user-1",
      TEST_TENANT_ID,
      "fake-token",
      ApprovedResources::default(),
      UserScope::User,
    )
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("access_request_error-kc_registration_failed", err.code());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_request_expired_draft_returns_expired_record(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;
  let now = db.now();
  let row = AppAccessRequest {
    expires_at: now - Duration::minutes(5),
    ..make_request("ar-expired-001", TEST_TENANT_ID, now)
  };
  db.create(&row).await?;

  let result = service.get_request("ar-expired-001").await?;
  assert!(result.is_some());
  let record = result.unwrap();
  assert_eq!(AppAccessRequestStatus::Expired, record.status);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_rejects_expired_draft(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;
  let now = db.now();
  let row = AppAccessRequest {
    expires_at: now - Duration::minutes(5),
    ..make_request("ar-expired-approve", TEST_TENANT_ID, now)
  };
  db.create(&row).await?;

  let result = service
    .approve_request(
      "ar-expired-approve",
      "user-1",
      TEST_TENANT_ID,
      "fake-token",
      ApprovedResources::default(),
      UserScope::User,
    )
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("access_request_error-expired", err.code());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_deny_request_rejects_expired_draft(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;
  let now = db.now();
  let row = AppAccessRequest {
    expires_at: now - Duration::minutes(5),
    ..make_request("ar-expired-deny", TEST_TENANT_ID, now)
  };
  db.create(&row).await?;

  let result = service.deny_request("ar-expired-deny", "user-1").await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("access_request_error-expired", err.code());

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_approve_request_unsupported_requested_version(
  #[future] access_request_service: (Arc<TestDbService>, DefaultAccessRequestService),
) -> anyhow::Result<()> {
  let (db, service) = access_request_service;
  let now = db.now();
  // Draft row carrying a hypothetical V2 (unsupported) requested JSON.
  let row = AppAccessRequest {
    requested: r#"{"version":"2"}"#.to_string(),
    ..make_request("ar-bad-version", TEST_TENANT_ID, now)
  };
  db.create(&row).await?;

  let result = service
    .approve_request(
      "ar-bad-version",
      "user-1",
      TEST_TENANT_ID,
      "fake-token",
      ApprovedResources::default(),
      UserScope::User,
    )
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("access_request_error-invalid_status", err.code());
  assert!(
    err.to_string().contains("Unsupported resources version"),
    "Expected error about unsupported version, got: {}",
    err
  );

  Ok(())
}
