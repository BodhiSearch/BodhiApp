use crate::AppRegInfo;
use async_trait::async_trait;
use oauth2::{
  basic::BasicTokenType, AccessToken, AuthorizationCode, ClientId, ClientSecret,
  EmptyExtraTokenFields, PkceCodeVerifier, RedirectUrl, RefreshToken, StandardTokenResponse,
  TokenResponse,
};
use objs::{impl_error_from, AppError, ErrorType, ReqwestError};
use serde::{Deserialize, Serialize};
use tracing::debug;

pub const GRANT_REFRESH_TOKEN: &str = "refresh_token";
pub const TOKEN_TYPE_OFFLINE: &str = "Offline";
pub const TOKEN_TYPE_BEARER: &str = "Bearer";
pub const HEADER_BODHI_APP_VERSION: &str = "x-bodhi-app-version";

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AuthServiceError {
  #[error(transparent)]
  Reqwest(#[from] ReqwestError),
  #[error("auth_service_api_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  AuthServiceApiError(String),
  #[error("token_exchange_error")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  TokenExchangeError(String),
}

impl_error_from!(
  reqwest::Error,
  AuthServiceError::Reqwest,
  ::objs::ReqwestError
);

type Result<T> = std::result::Result<T, AuthServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AuthService: Send + Sync + std::fmt::Debug {
  async fn register_client(&self, redirect_uris: Vec<String>) -> Result<AppRegInfo>;

  async fn exchange_auth_code(
    &self,
    code: AuthorizationCode,
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_uri: RedirectUrl,
    code_verifier: PkceCodeVerifier,
  ) -> Result<(AccessToken, RefreshToken)>;

  async fn refresh_token(
    &self,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
  ) -> Result<(String, Option<String>)>;

  async fn exchange_token(
    &self,
    client_id: &str,
    client_secret: &str,
    subject_token: &str,
    token_type: &str,
    scopes: Vec<String>,
  ) -> Result<(String, Option<String>)>;

  async fn make_resource_admin(
    &self,
    client_id: &str,
    client_secret: &str,
    email: &str,
  ) -> Result<()>;
}

#[derive(Debug)]
pub struct KeycloakAuthService {
  app_version: String,
  auth_url: String,
  realm: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeycloakError {
  error: String,
}

impl From<KeycloakError> for AuthServiceError {
  fn from(value: KeycloakError) -> Self {
    AuthServiceError::AuthServiceApiError(value.error)
  }
}

impl KeycloakAuthService {
  pub fn new(app_version: &str, auth_url: String, realm: String) -> Self {
    Self {
      app_version: app_version.to_string(),
      auth_url,
      realm,
    }
  }

  fn auth_url(&self) -> String {
    format!("{}/realms/{}", self.auth_url, self.realm)
  }

  fn auth_api_url(&self) -> String {
    format!("{}/bodhi", self.auth_url())
  }

  fn auth_token_url(&self) -> String {
    format!("{}/protocol/openid-connect/token", self.auth_url())
  }

  async fn get_client_access_token(
    &self,
    client_id: &str,
    client_secret: &str,
  ) -> Result<AccessToken> {
    let params = [
      ("grant_type", "client_credentials"),
      ("client_id", client_id),
      ("client_secret", client_secret),
    ];

    let client = reqwest::Client::new();
    let response = client
      .post(self.auth_token_url())
      .form(&params)
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    if response.status().is_success() {
      let token_response: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> =
        response.json().await?;
      Ok(token_response.access_token().to_owned())
    } else {
      let error = response.json::<KeycloakError>().await?;
      Err(error.into())
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterClientRequest {
  pub redirect_uris: Vec<String>,
}

#[async_trait]
impl AuthService for KeycloakAuthService {
  async fn register_client(&self, redirect_uris: Vec<String>) -> Result<AppRegInfo> {
    let client_endpoint = format!("{}/clients", self.auth_api_url());
    let response = reqwest::Client::new()
      .post(client_endpoint)
      .json(&RegisterClientRequest { redirect_uris })
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;
    if response.status().is_success() {
      Ok(response.json::<AppRegInfo>().await?)
    } else {
      let response = response.text().await?;
      debug!(
        ?response,
        service = "auth_service",
        method = "register_client",
        "error registering client"
      );
      let error: KeycloakError =
        serde_json::from_str(&response).unwrap_or(KeycloakError { error: response });
      Err(error.into())
    }
  }

  async fn exchange_auth_code(
    &self,
    code: AuthorizationCode,
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_uri: RedirectUrl,
    code_verifier: PkceCodeVerifier,
  ) -> Result<(AccessToken, RefreshToken)> {
    let params = [
      ("grant_type", "authorization_code"),
      ("code", code.secret()),
      ("client_id", client_id.as_str()),
      ("client_secret", client_secret.secret()),
      ("redirect_uri", redirect_uri.as_str()),
      ("code_verifier", code_verifier.secret()),
    ];

    let client = reqwest::Client::new();
    let response = client
      .post(self.auth_token_url())
      .form(&params)
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;
    if response.status().is_success() {
      let token_response: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> =
        response.json().await?;
      Ok((
        token_response.access_token().to_owned(),
        token_response
          .refresh_token()
          .expect("refresh token is not present when exchaging for auth code")
          .to_owned(),
      ))
    } else {
      let error = response.json::<KeycloakError>().await?;
      Err(error.into())
    }
  }

  async fn refresh_token(
    &self,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
  ) -> Result<(String, Option<String>)> {
    let params = [
      ("grant_type", "refresh_token"),
      ("refresh_token", refresh_token),
      ("client_id", client_id),
      ("client_secret", client_secret),
    ];
    let client = reqwest::Client::new();
    let response = client
      .post(self.auth_token_url())
      .form(&params)
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    if response.status().is_success() {
      let token_response: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> =
        response.json().await?;
      Ok((
        token_response.access_token().secret().to_string(),
        token_response
          .refresh_token()
          .map(|s| s.secret().to_string()),
      ))
    } else {
      let error = response.json::<KeycloakError>().await?;
      Err(error.into())
    }
  }

  async fn exchange_token(
    &self,
    client_id: &str,
    client_secret: &str,
    subject_token: &str,
    token_type: &str,
    scopes: Vec<String>,
  ) -> Result<(String, Option<String>)> {
    let client = reqwest::Client::new();
    let scope = scopes.join(" ");
    let params = [
      (
        "grant_type",
        "urn:ietf:params:oauth:grant-type:token-exchange",
      ),
      ("client_id", client_id),
      ("client_secret", client_secret),
      ("subject_token", subject_token),
      ("requested_token_type", token_type),
      ("scope", &scope),
    ];

    let response = client
      .post(self.auth_token_url())
      .form(&params)
      .header("Authorization", format!("Berarer {}", subject_token))
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    if response.status().is_success() {
      let token_response: serde_json::Value = response.json().await?;
      let access_token = token_response["access_token"]
        .as_str()
        .ok_or_else(|| AuthServiceError::TokenExchangeError("access_token not found".to_string()))?
        .to_string();

      let refresh_token = token_response["refresh_token"]
        .as_str()
        .map(|s| s.to_string());

      Ok((access_token, refresh_token))
    } else {
      let error = response.json::<KeycloakError>().await?;
      Err(AuthServiceError::TokenExchangeError(error.error))
    }
  }

  async fn make_resource_admin(
    &self,
    client_id: &str,
    client_secret: &str,
    email: &str,
  ) -> Result<()> {
    // Get client access token
    let access_token = self
      .get_client_access_token(client_id, client_secret)
      .await?;

    // Make API call to make the user a resource admin
    let endpoint = format!("{}/clients/make-resource-admin", self.auth_api_url());
    let client = reqwest::Client::new();
    let response = client
      .post(endpoint)
      .bearer_auth(access_token.secret())
      .json(&serde_json::json!({ "username": email }))
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    if response.status().is_success() {
      Ok(())
    } else {
      let error = response.json::<KeycloakError>().await?;
      Err(error.into())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    test_utils::test_auth_service, AppRegInfo, AuthService, AuthServiceError, JsonWebTokenError,
  };
  use jsonwebtoken::{errors::ErrorKind, Algorithm};
  use mockito::{Matcher, Server};
  use objs::{
    test_utils::{assert_error_message, setup_l10n},
    AppError, FluentLocalizationService,
  };
  use rstest::rstest;
  use serde_json::json;
  use std::sync::Arc;

  #[rstest]
  #[case(&AuthServiceError::AuthServiceApiError("test".to_string()), "error from auth service: test")]
  #[case(&JsonWebTokenError::new(ErrorKind::InvalidToken.into()), "authentication token is invalid")]
  #[case(&JsonWebTokenError::new(ErrorKind::InvalidSignature.into()), "authentication token signature does not match")]
  #[case(&JsonWebTokenError::new(ErrorKind::InvalidIssuer.into()), "authentication token issuer is invalid")]
  #[case(&JsonWebTokenError::new(ErrorKind::InvalidAudience.into()), "authentication token audience is invalid")]
  #[case(&JsonWebTokenError::new(ErrorKind::InvalidSubject.into()), "authentication token is invalid, source: InvalidSubject")]
  fn test_error_messages_services(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected_message: &str,
  ) {
    assert_error_message(
      localization_service,
      &error.code(),
      error.args(),
      expected_message,
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_service_register_client_success() {
    let mut server = Server::new_async().await;
    let url = server.url();
    let mock_server = server
      .mock("POST", "/realms/test-realm/bodhi/clients")
      .with_status(201)
      .with_header("content-type", "application/json")
      .with_body(
        json!({
            "public_key": "test-public-key",
            "alg": "RS256",
            "kid": "test-kid",
            "issuer": "test-issuer",
            "client_id": "test-client",
            "client_secret": "test-secret"
        })
        .to_string(),
      )
      .create();

    let service = test_auth_service(&url);
    let result = service
      .register_client(vec![
        "http://0.0.0.0:1135/bodhi/v1/auth/callback".to_string()
      ])
      .await;
    assert!(result.is_ok());
    let app_reg_info = result.unwrap();
    assert_eq!(
      AppRegInfo {
        public_key: "test-public-key".to_string(),
        alg: Algorithm::RS256,
        kid: "test-kid".to_string(),
        issuer: "test-issuer".to_string(),
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
      },
      app_reg_info
    );
    mock_server.assert();
  }

  #[rstest]
  #[tokio::test]
  async fn test_auth_service_register_client_server_error() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;
    let url = server.url();
    let mock_server = server
      .mock("POST", "/realms/test-realm/bodhi/clients")
      .with_status(500)
      .with_header("Content-Type", "application/json")
      .with_body(r#"{"error": "cannot complete request"}"#)
      .create();

    let service = test_auth_service(&url);
    let result = service
      .register_client(vec![
        "http://0.0.0.0:1135/bodhi/v1/auth/callback".to_string()
      ])
      .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
      matches!(err, AuthServiceError::AuthServiceApiError(msg) if msg == "cannot complete request")
    );
    mock_server.assert();
    Ok(())
  }

  #[rstest]
  #[tokio::test]
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
          "scope": "openid scope_user profile email scope_power_user",
        })
        .to_string(),
      )
      .create();

    let service = test_auth_service(&url);
    let result = service
      .refresh_token(client_id, client_secret, old_refresh_token)
      .await;

    assert!(result.is_ok());
    let (access_token, refresh_token) = result.unwrap();
    assert_eq!(access_token, new_access_token);
    assert_eq!(refresh_token, Some(new_refresh_token.to_string()));

    mock.assert();
    Ok(())
  }

  #[rstest]
  #[tokio::test]
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
    assert!(matches!(
      error,
      AuthServiceError::AuthServiceApiError(msg) if msg == "invalid_grant"
    ));
    mock.assert();
    Ok(())
  }

  #[rstest]
  #[case("test_client_id", "test_client_secret", "test@example.com")]
  #[tokio::test]
  async fn test_make_resource_admin_success(
    #[case] client_id: &str,
    #[case] client_secret: &str,
    #[case] email: &str,
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
        "/realms/test-realm/bodhi/clients/make-resource-admin",
      )
      .match_header("Authorization", "Bearer test_access_token")
      .match_body(Matcher::Json(json!({"username": email})))
      .with_status(200)
      .with_body("{}")
      .create();

    let service = test_auth_service(&url);
    let result = service
      .make_resource_admin(client_id, client_secret, email)
      .await;

    assert!(result.is_ok());
    token_mock.assert();
    admin_mock.assert();

    Ok(())
  }

  #[rstest]
  #[case("test_client_id", "test_client_secret", "test@example.com")]
  #[tokio::test]
  async fn test_make_resource_admin_token_failure(
    #[case] client_id: &str,
    #[case] client_secret: &str,
    #[case] email: &str,
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
      .make_resource_admin(client_id, client_secret, email)
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      AuthServiceError::AuthServiceApiError(_)
    ));
    token_mock.assert();

    Ok(())
  }

  #[rstest]
  #[case("test_client_id", "test_client_secret", "test@example.com")]
  #[tokio::test]
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
        "/realms/test-realm/bodhi/clients/make-resource-admin",
      )
      .with_status(400)
      .with_body(json!({"error": "user_not_found"}).to_string())
      .create();

    let service = test_auth_service(&url);
    let result = service
      .make_resource_admin(client_id, client_secret, email)
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      AuthServiceError::AuthServiceApiError(_)
    ));
    token_mock.assert();
    admin_mock.assert();

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_exchange_for_resource_token_success() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;
    let url = server.url();

    let mock = server
      .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
      .match_header("content-type", "application/x-www-form-urlencoded")
      .match_body(Matcher::AllOf(vec![
        Matcher::UrlEncoded(
          "grant_type".into(),
          "urn:ietf:params:oauth:grant-type:token-exchange".into(),
        ),
        Matcher::UrlEncoded("subject_token".into(), "test_token".into()),
        Matcher::UrlEncoded(
          "requested_token_type".into(),
          "urn:ietf:params:oauth:grant-type:refresh_token".into(),
        ),
        Matcher::UrlEncoded(
          "scope".into(),
          "openid offline_access scope_token_user".into(),
        ),
        Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
        Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      ]))
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(
        json!({
          "access_token": "new_access_token",
          "refresh_token": "new_refresh_token",
          "token_type": "Bearer",
          "expires_in": 3600,
          "scope": "openid offline_access scope_token_user"
        })
        .to_string(),
      )
      .create();

    let service = test_auth_service(&url);
    let result = service
      .exchange_token(
        "test_client_id",
        "test_client_secret",
        "test_token",
        "urn:ietf:params:oauth:grant-type:refresh_token",
        vec![
          "openid".to_string(),
          "offline_access".to_string(),
          "scope_token_user".to_string(),
        ],
      )
      .await;

    assert!(result.is_ok());
    let (access_token, refresh_token) = result.unwrap();
    assert_eq!(access_token, "new_access_token");
    assert_eq!(refresh_token, Some("new_refresh_token".to_string()));

    mock.assert();
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_exchange_for_resource_token_access_token_only() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;
    let url = server.url();

    let mock = server
      .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
      .match_header("content-type", "application/x-www-form-urlencoded")
      .match_body(Matcher::AllOf(vec![
        Matcher::UrlEncoded(
          "grant_type".into(),
          "urn:ietf:params:oauth:grant-type:token-exchange".into(),
        ),
        Matcher::UrlEncoded("subject_token".into(), "test_token".into()),
        Matcher::UrlEncoded(
          "requested_token_type".into(),
          "urn:ietf:params:oauth:token-type:access_token".into(),
        ),
        Matcher::UrlEncoded("scope".into(), "openid scope_token_user".into()),
        Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
        Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      ]))
      .with_status(200)
      .with_header("content-type", "application/json")
      .with_body(
        json!({
          "access_token": "new_access_token",
          "token_type": "Bearer",
          "expires_in": 3600,
          "scope": "openid scope_token_user"
        })
        .to_string(),
      )
      .create();

    let service = test_auth_service(&url);
    let result = service
      .exchange_token(
        "test_client_id",
        "test_client_secret",
        "test_token",
        "urn:ietf:params:oauth:token-type:access_token",
        vec!["openid".to_string(), "scope_token_user".to_string()],
      )
      .await;

    assert!(result.is_ok());
    let (access_token, refresh_token) = result.unwrap();
    assert_eq!(access_token, "new_access_token");
    assert_eq!(refresh_token, None);

    mock.assert();
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_exchange_for_resource_token_error() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;
    let url = server.url();

    let mock = server
      .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
      .match_header("content-type", "application/x-www-form-urlencoded")
      .match_body(Matcher::AllOf(vec![
        Matcher::UrlEncoded(
          "grant_type".into(),
          "urn:ietf:params:oauth:grant-type:token-exchange".into(),
        ),
        Matcher::UrlEncoded("subject_token".into(), "invalid_token".into()),
        Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
        Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      ]))
      .with_status(400)
      .with_header("content-type", "application/json")
      .with_body(
        json!({
          "error": "invalid_token",
          "error_description": "Token validation failed"
        })
        .to_string(),
      )
      .create();

    let service = test_auth_service(&url);
    let result = service
      .exchange_token(
        "test_client_id",
        "test_client_secret",
        "invalid_token",
        "urn:ietf:params:oauth:grant-type:refresh_token",
        vec!["openid".to_string()],
      )
      .await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(
      error,
      AuthServiceError::TokenExchangeError(msg) if msg == "invalid_token"
    ));

    mock.assert();
    Ok(())
  }
}
