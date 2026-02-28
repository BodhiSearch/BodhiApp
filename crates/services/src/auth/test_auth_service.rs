use crate::{test_utils::test_auth_service, AuthService, ClientRegistrationResponse};
use anyhow_trace::anyhow_trace;
use mockito::{Matcher, Server};
use oauth2::{AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl};
use objs::AppError;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_service_register_client_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let mock_server = server
    .mock("POST", "/realms/test-realm/bodhi/resources")
    .with_status(201)
    .with_header("content-type", "application/json")
    .with_body(
      json!({
          "client_id": "test-client",
          "client_secret": "test-secret"
      })
      .to_string(),
    )
    .create();

  let service = test_auth_service(&url);
  let client_reg = service
    .register_client(
      "Test Resource Server Name".to_string(),
      "Test resource client description".to_string(),
      vec!["http://0.0.0.0:1135/bodhi/v1/auth/callback".to_string()],
    )
    .await?;
  assert_eq!(
    ClientRegistrationResponse {
      client_id: "test-client".to_string(),
      client_secret: "test-secret".to_string(),
    },
    client_reg
  );
  mock_server.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_service_register_client_server_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let mock_server = server
    .mock("POST", "/realms/test-realm/bodhi/resources")
    .with_status(500)
    .with_header("Content-Type", "application/json")
    .with_body(r#"{"error": "cannot complete request"}"#)
    .create();

  let service = test_auth_service(&url);
  let result = service
    .register_client(
      "Test Resource Server Name".to_string(),
      "Test resource client description".to_string(),
      vec!["http://0.0.0.0:1135/bodhi/v1/auth/callback".to_string()],
    )
    .await;
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("auth_service_error-auth_service_api_error", err.code());
  mock_server.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_refresh_token() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let client_id = "test_client_id";
  let client_secret = "test_client_secret";
  let old_refresh_token = "old_refresh_token";
  let new_access_token = "new_access_token";
  let new_refresh_token = "new_refresh_token";

  let mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_header("content-type", "application/x-www-form-urlencoded")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "refresh_token".into()),
      Matcher::UrlEncoded("refresh_token".into(), old_refresh_token.into()),
      Matcher::UrlEncoded("client_id".into(), client_id.into()),
      Matcher::UrlEncoded("client_secret".into(), client_secret.into()),
    ]))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      json!({
        "access_token": new_access_token,
        "refresh_token": new_refresh_token,
        "id_token": "test_id_token",
        "token_type": "Bearer",
        "expires_in": 3600,
        "refresh_expires_in": 172800,
        "scope": "openid profile email roles",
      })
      .to_string(),
    )
    .create();

  let service = test_auth_service(&url);
  let (access_token, refresh_token) = service
    .refresh_token(client_id, client_secret, old_refresh_token)
    .await?;

  assert_eq!(new_access_token, access_token);
  assert_eq!(Some(new_refresh_token.to_string()), refresh_token);

  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_refresh_token_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let client_id = "test_client_id";
  let client_secret = "test_client_secret";
  let invalid_refresh_token = "invalid_refresh_token";

  let mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_header("content-type", "application/x-www-form-urlencoded")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "refresh_token".into()),
      Matcher::UrlEncoded("refresh_token".into(), invalid_refresh_token.into()),
      Matcher::UrlEncoded("client_id".into(), client_id.into()),
      Matcher::UrlEncoded("client_secret".into(), client_secret.into()),
    ]))
    .with_status(400)
    .with_header("content-type", "application/json")
    .with_body(
      json!({
        "error": "invalid_grant",
        "error_description": "Invalid refresh token"
      })
      .to_string(),
    )
    .create();

  let service = test_auth_service(&url);
  let result = service
    .refresh_token(client_id, client_secret, invalid_refresh_token)
    .await;

  assert!(result.is_err());
  let error = result.unwrap_err();
  assert_eq!("auth_service_error-auth_service_api_error", error.code());
  mock.assert();
  Ok(())
}

#[rstest]
#[case(
  "test_client_id",
  "test_client_secret",
  "550e8400-e29b-41d4-a716-446655440000"
)]
#[tokio::test]
#[anyhow_trace]
async fn test_make_resource_admin_success(
  #[case] client_id: &str,
  #[case] client_secret: &str,
  #[case] user_id: &str,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  // Mock token endpoint
  let token_mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
      Matcher::UrlEncoded("client_id".into(), client_id.into()),
      Matcher::UrlEncoded("client_secret".into(), client_secret.into()),
    ]))
    .with_status(200)
    .with_body(
      json!({
          "access_token": "test_access_token",
          "token_type": "Bearer",
          "expires_in": 300,
      })
      .to_string(),
    )
    .create();

  // Mock make-resource-admin endpoint
  let admin_mock = server
    .mock(
      "POST",
      "/realms/test-realm/bodhi/resources/make-resource-admin",
    )
    .match_header("Authorization", "Bearer test_access_token")
    .match_body(Matcher::Json(json!({"user_id": user_id})))
    .with_status(200)
    .with_body("{}")
    .create();

  let service = test_auth_service(&url);
  service
    .make_resource_admin(client_id, client_secret, user_id)
    .await?;

  token_mock.assert();
  admin_mock.assert();

  Ok(())
}

#[rstest]
#[case(
  "test_client_id",
  "test_client_secret",
  "550e8400-e29b-41d4-a716-446655440001"
)]
#[tokio::test]
#[anyhow_trace]
async fn test_make_resource_admin_token_failure(
  #[case] client_id: &str,
  #[case] client_secret: &str,
  #[case] user_id: &str,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  // Mock token endpoint with failure
  let token_mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .with_status(400)
    .with_body(json!({"error": "invalid_client"}).to_string())
    .create();

  let service = test_auth_service(&url);
  let result = service
    .make_resource_admin(client_id, client_secret, user_id)
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("auth_service_error-auth_service_api_error", err.code());
  token_mock.assert();

  Ok(())
}

#[rstest]
#[case("test_client_id", "test_client_secret", "test@example.com")]
#[tokio::test]
#[anyhow_trace]
async fn test_make_resource_admin_api_failure(
  #[case] client_id: &str,
  #[case] client_secret: &str,
  #[case] email: &str,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  // Mock token endpoint
  let token_mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .with_status(200)
    .with_body(
      json!({
          "access_token": "test_access_token",
          "token_type": "Bearer",
          "expires_in": 300,
      })
      .to_string(),
    )
    .create();

  // Mock make-resource-admin endpoint with failure
  let admin_mock = server
    .mock(
      "POST",
      "/realms/test-realm/bodhi/resources/make-resource-admin",
    )
    .with_status(400)
    .with_body(json!({"error": "user_not_found"}).to_string())
    .create();

  let service = test_auth_service(&url);
  let result = service
    .make_resource_admin(client_id, client_secret, email)
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("auth_service_error-auth_service_api_error", err.code());
  token_mock.assert();
  admin_mock.assert();

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_exchange_auth_code_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let code = AuthorizationCode::new("test_auth_code".to_string());
  let client_id = ClientId::new("test_client_id".to_string());
  let client_secret = ClientSecret::new("test_client_secret".to_string());
  let redirect_uri = RedirectUrl::new("http://localhost:1135/callback".to_string())?;
  let code_verifier = PkceCodeVerifier::new("test_code_verifier".to_string());

  let mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_header("content-type", "application/x-www-form-urlencoded")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "authorization_code".into()),
      Matcher::UrlEncoded("code".into(), "test_auth_code".into()),
      Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
      Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      Matcher::UrlEncoded(
        "redirect_uri".into(),
        "http://localhost:1135/callback".into(),
      ),
      Matcher::UrlEncoded("code_verifier".into(), "test_code_verifier".into()),
    ]))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      json!({
        "access_token": "test_access_token",
        "refresh_token": "test_refresh_token",
        "id_token": "test_id_token",
        "token_type": "Bearer",
        "expires_in": 3600,
        "refresh_expires_in": 172800,
      })
      .to_string(),
    )
    .create();

  let service = test_auth_service(&url);
  let (access_token, refresh_token) = service
    .exchange_auth_code(code, client_id, client_secret, redirect_uri, code_verifier)
    .await?;

  assert_eq!("test_access_token", access_token.secret());
  assert_eq!("test_refresh_token", refresh_token.secret());
  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_exchange_auth_code_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let code = AuthorizationCode::new("invalid_code".to_string());
  let client_id = ClientId::new("test_client_id".to_string());
  let client_secret = ClientSecret::new("test_client_secret".to_string());
  let redirect_uri = RedirectUrl::new("http://localhost:1135/callback".to_string())?;
  let code_verifier = PkceCodeVerifier::new("test_code_verifier".to_string());

  let mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .with_status(400)
    .with_header("content-type", "application/json")
    .with_body(
      json!({
        "error": "invalid_grant",
        "error_description": "Code not valid"
      })
      .to_string(),
    )
    .create();

  let service = test_auth_service(&url);
  let result = service
    .exchange_auth_code(code, client_id, client_secret, redirect_uri, code_verifier)
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("auth_service_error-auth_service_api_error", err.code());
  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_exchange_app_token_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let client_id = "test_client_id";
  let client_secret = "test_client_secret";
  let subject_token = "test_subject_token";
  let scopes = vec!["scope1".to_string(), "scope2".to_string()];

  let mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_header("content-type", "application/x-www-form-urlencoded")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded(
        "grant_type".into(),
        "urn:ietf:params:oauth:grant-type:token-exchange".into(),
      ),
      Matcher::UrlEncoded("subject_token".into(), subject_token.into()),
      Matcher::UrlEncoded("client_id".into(), client_id.into()),
      Matcher::UrlEncoded("client_secret".into(), client_secret.into()),
      Matcher::UrlEncoded("audience".into(), client_id.into()),
      Matcher::UrlEncoded("scope".into(), "scope1 scope2".into()),
    ]))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      json!({
        "access_token": "exchanged_access_token",
        "refresh_token": "exchanged_refresh_token",
        "token_type": "Bearer",
        "expires_in": 3600,
      })
      .to_string(),
    )
    .create();

  let service = test_auth_service(&url);
  let (access_token, refresh_token) = service
    .exchange_app_token(client_id, client_secret, subject_token, scopes)
    .await?;

  assert_eq!("exchanged_access_token", access_token);
  assert_eq!(Some("exchanged_refresh_token".to_string()), refresh_token);
  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_exchange_app_token_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let client_id = "test_client_id";
  let client_secret = "test_client_secret";
  let subject_token = "invalid_token";
  let scopes = vec!["scope1".to_string()];

  let mock = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .with_status(400)
    .with_header("content-type", "application/json")
    .with_body(
      json!({
        "error": "invalid_token",
        "error_description": "Token exchange failed"
      })
      .to_string(),
    )
    .create();

  let service = test_auth_service(&url);
  let result = service
    .exchange_app_token(client_id, client_secret, subject_token, scopes)
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("auth_service_error-auth_service_api_error", err.code());
  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_assign_user_role_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let reviewer_token = "test_reviewer_token";
  let user_id = "550e8400-e29b-41d4-a716-446655440000";
  let role = "resource_manager";

  let mock = server
    .mock("POST", "/realms/test-realm/bodhi/resources/assign-role")
    .match_header("Authorization", "Bearer test_reviewer_token")
    .match_body(Matcher::Json(json!({
      "user_id": user_id,
      "role": role
    })))
    .with_status(200)
    .with_body("{}")
    .create();

  let service = test_auth_service(&url);
  service
    .assign_user_role(reviewer_token, user_id, role)
    .await?;

  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_assign_user_role_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let reviewer_token = "test_reviewer_token";
  let user_id = "invalid_user_id";
  let role = "resource_manager";

  let mock = server
    .mock("POST", "/realms/test-realm/bodhi/resources/assign-role")
    .with_status(404)
    .with_body(json!({"error": "user_not_found"}).to_string())
    .create();

  let service = test_auth_service(&url);
  let result = service
    .assign_user_role(reviewer_token, user_id, role)
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("auth_service_error-auth_service_api_error", err.code());
  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_remove_user_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let reviewer_token = "test_reviewer_token";
  let user_id = "550e8400-e29b-41d4-a716-446655440000";

  let mock = server
    .mock("POST", "/realms/test-realm/bodhi/resources/remove-user")
    .match_header("Authorization", "Bearer test_reviewer_token")
    .match_body(Matcher::Json(json!({"user_id": user_id})))
    .with_status(200)
    .with_body("{}")
    .create();

  let service = test_auth_service(&url);
  service.remove_user(reviewer_token, user_id).await?;

  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_remove_user_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let reviewer_token = "test_reviewer_token";
  let user_id = "invalid_user_id";

  let mock = server
    .mock("POST", "/realms/test-realm/bodhi/resources/remove-user")
    .with_status(404)
    .with_body(json!({"error": "user_not_found"}).to_string())
    .create();

  let service = test_auth_service(&url);
  let result = service.remove_user(reviewer_token, user_id).await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("auth_service_error-auth_service_api_error", err.code());
  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_users_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let reviewer_token = "test_reviewer_token";

  let mock = server
    .mock(
      "GET",
      "/realms/test-realm/bodhi/resources/users?page=1&page_size=10",
    )
    .match_header("Authorization", "Bearer test_reviewer_token")
    .with_status(200)
    .with_body(
      json!({
        "client_id": "resource-test",
        "users": [
          {
            "user_id": "user-1",
            "username": "testuser1",
            "first_name": "Test",
            "last_name": "User1",
            "role": "standard_user"
          },
          {
            "user_id": "user-2",
            "username": "testuser2",
            "first_name": "Test",
            "last_name": "User2",
            "role": "resource_manager"
          }
        ],
        "page": 1,
        "page_size": 10,
        "total_pages": 1,
        "total_users": 2,
        "has_next": false,
        "has_previous": false
      })
      .to_string(),
    )
    .create();

  let service = test_auth_service(&url);
  let response = service
    .list_users(reviewer_token, Some(1), Some(10))
    .await?;

  assert_eq!("resource-test", response.client_id);
  assert_eq!(2, response.users.len());
  assert_eq!("user-1", response.users[0].user_id);
  assert_eq!("testuser1", response.users[0].username);
  assert_eq!(1, response.page);
  assert_eq!(10, response.page_size);
  assert_eq!(1, response.total_pages);
  assert_eq!(2, response.total_users);
  assert_eq!(false, response.has_next);
  assert_eq!(false, response.has_previous);
  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_users_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let reviewer_token = "invalid_token";

  let mock = server
    .mock("GET", "/realms/test-realm/bodhi/resources/users")
    .with_status(401)
    .with_body(json!({"error": "unauthorized"}).to_string())
    .create();

  let service = test_auth_service(&url);
  let result = service.list_users(reviewer_token, None, None).await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!("auth_service_error-auth_service_api_error", err.code());
  mock.assert();
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_refresh_token_retry_on_5xx() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let client_id = "test_client_id";
  let client_secret = "test_client_secret";
  let refresh_token = "test_refresh_token";

  // First request returns 503 (should retry)
  let mock_503 = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_header("content-type", "application/x-www-form-urlencoded")
    .with_status(503)
    .with_header("content-type", "application/json")
    .with_body(
      json!({
        "error": "service_unavailable",
        "error_description": "Service temporarily unavailable"
      })
      .to_string(),
    )
    .expect(1)
    .create();

  // Second request succeeds
  let mock_success = server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_header("content-type", "application/x-www-form-urlencoded")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      json!({
        "access_token": "new_access_token",
        "refresh_token": "new_refresh_token",
        "token_type": "Bearer",
        "expires_in": 3600,
      })
      .to_string(),
    )
    .expect(1)
    .create();

  let service = test_auth_service(&url);
  let (access_token, new_refresh_token) = service
    .refresh_token(client_id, client_secret, refresh_token)
    .await?;

  assert_eq!("new_access_token", access_token);
  assert_eq!(Some("new_refresh_token".to_string()), new_refresh_token);
  mock_503.assert();
  mock_success.assert();
  Ok(())
}
