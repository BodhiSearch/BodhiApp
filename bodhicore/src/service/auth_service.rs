#![allow(unused_variables)] // TODO: remove this
use async_trait::async_trait;
use derive_new::new;
use oauth2::{AccessToken, RefreshToken};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthServiceError {}

type Result<T> = std::result::Result<T, AuthServiceError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterClientResponse {
  pub client_id: String,
  pub client_secret: String,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthService: Send + Sync + std::fmt::Debug {
  async fn register_client(&self) -> Result<RegisterClientResponse>;

  async fn get_auth_url(&self, code_verifier: &str) -> Result<String>;

  async fn exchange_auth_code(
    &self,
    code: &str,
    code_verifier: &str,
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

#[derive(Debug, Default, new)]
pub struct KeycloakAuthService {}

impl KeycloakAuthService {}

#[async_trait]
impl AuthService for KeycloakAuthService {
  async fn register_client(&self) -> Result<RegisterClientResponse> {
    todo!()
  }

  async fn get_auth_url(&self, code_verifier: &str) -> Result<String> {
    todo!()
  }

  async fn exchange_auth_code(
    &self,
    code: &str,
    code_verifier: &str,
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