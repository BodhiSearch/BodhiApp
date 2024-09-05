#![allow(unused_variables)] // TODO: remove this
use super::{AppRegInfo, HttpError, HttpErrorBuilder};
use async_trait::async_trait;
use oauth2::{
  AccessToken, AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl,
  RefreshToken,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthServiceError {
  #[error("reqwest: {0}")]
  Reqwest(String),
  #[error("api_error: {0}")]
  AuthServiceApiError(String),
}

impl From<reqwest::Error> for AuthServiceError {
  fn from(value: reqwest::Error) -> Self {
    Self::Reqwest(value.to_string())
  }
}

impl From<AuthServiceError> for HttpError {
  fn from(value: AuthServiceError) -> Self {
    let msg = match value {
      AuthServiceError::Reqwest(msg) => msg,
      AuthServiceError::AuthServiceApiError(msg) => msg,
    };
    HttpErrorBuilder::default()
      .internal_server(Some(&msg))
      .build()
      .unwrap()
  }
}

type Result<T> = std::result::Result<T, AuthServiceError>;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthService: Send + Sync + std::fmt::Debug {
  async fn register_client(&self, redirect_uris: Vec<String>) -> Result<AppRegInfo>;

  async fn check_access_token(&self, access_token: &AccessToken) -> Result<bool>;

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
    refresh_token: &RefreshToken,
  ) -> Result<(AccessToken, RefreshToken)>;

  async fn exchange_for_resource_token(
    &self,
    client_token: &str,
  ) -> Result<(AccessToken, RefreshToken)>;
}

#[derive(Debug)]
pub struct KeycloakAuthService {
  auth_url: String,
  realm: String,
}

impl KeycloakAuthService {
  pub fn new(auth_url: String, realm: String) -> Self {
    Self { auth_url, realm }
  }

  fn auth_api_url(&self) -> String {
    format!("{}/realms/{}/bodhi", self.auth_url, self.realm)
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
    let res = reqwest::Client::new()
      .post(client_endpoint)
      .json(&RegisterClientRequest { redirect_uris })
      .send()
      .await?;
    if res.status().is_success() {
      Ok(res.json::<AppRegInfo>().await?)
    } else {
      let error = res.json::<Value>().await?;
      let error_msg = error["error"]
        .as_str()
        .unwrap_or("error at id-server registering as resource");
      Err(AuthServiceError::AuthServiceApiError(error_msg.to_string()))
    }
  }

  async fn check_access_token(&self, access_token: &AccessToken) -> Result<bool> {
    // TODO: returning true to complete the flow, implement this
    Ok(true)
  }

  async fn exchange_auth_code(
    &self,
    code: AuthorizationCode,
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_uri: RedirectUrl,
    code_verifier: PkceCodeVerifier,
  ) -> Result<(AccessToken, RefreshToken)> {
    todo!()
  }

  async fn refresh_token(
    &self,
    refresh_token: &RefreshToken,
  ) -> Result<(AccessToken, RefreshToken)> {
    todo!()
  }

  async fn exchange_for_resource_token(
    &self,
    client_token: &str,
  ) -> Result<(AccessToken, RefreshToken)> {
    todo!()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use jsonwebtoken::Algorithm;
  use mockito::Server;
  use rstest::rstest;
  use serde_json::json;

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

    let service = KeycloakAuthService::new(url, "test-realm".to_string());
    let result = service
      .register_client(vec!["http://0.0.0.0:1135/app/login/callback".to_string()])
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

    let service = KeycloakAuthService::new(url, "test-realm".to_string());
    let result = service
      .register_client(vec!["http://0.0.0.0:1135/app/login/callback".to_string()])
      .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AuthServiceError::AuthServiceApiError(_)));
    assert_eq!("api_error: cannot complete request", err.to_string());
    mock_server.assert();

    Ok(())
  }
}
