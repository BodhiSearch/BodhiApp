use anyhow::Result;
use axum::http::StatusCode;
use derive_builder::Builder;
use reqwest::Client;
use serde::Deserialize;

/// Configuration for the auth server test client
#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct AuthServerConfig {
  pub auth_server_url: String,
  pub realm: String,
  pub resource_client_id: String,
  pub resource_client_secret: String,
  pub app_client_id: String,
}

/// OAuth token response structure
#[derive(Debug, Deserialize)]
struct TokenResponse {
  access_token: String,
}

/// Test user credentials for integration tests
#[derive(Debug, Clone)]
pub struct TestUser {
  pub username: String,
  pub user_id: String,
  pub password: String,
}

/// Auth server test client for integration testing
pub struct AuthServerTestClient {
  client: Client,
  config: AuthServerConfig,
}

impl AuthServerTestClient {
  /// Create a new auth server test client
  pub fn new(config: AuthServerConfig) -> Self {
    Self {
      client: Client::new(),
      config,
    }
  }

  /// Get App User Token via Direct Access Grant (with Resource Scope)
  pub async fn get_app_user_token_with_scope(
    &self,
    app_client_id: &str,
    username: &str,
    password: &str,
    scopes: &[&str],
  ) -> Result<String> {
    let token_url = format!(
      "{}/realms/{}/protocol/openid-connect/token",
      self.config.auth_server_url, self.config.realm
    );

    let mut params = vec![
      ("grant_type", "password"),
      ("client_id", app_client_id),
      ("username", username),
      ("password", password),
    ];
    let scope_string = scopes.join(" ");
    if !scopes.is_empty() {
      params.push(("scope", &scope_string));
    };

    let response = self.client.post(&token_url).form(&params).send().await?;

    if response.status() != StatusCode::OK {
      anyhow::bail!(
        "App user token with scope request failed: {}",
        response
          .text()
          .await
          .unwrap_or_else(|_| "Unable to read response body".to_string())
      );
    }

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response.access_token)
  }

  /// Get User Token using password grant (resource owner)
  pub async fn get_user_token(
    &self,
    client_id: &str,
    client_secret: &str,
    username: &str,
    password: &str,
    scopes: &[&str],
  ) -> Result<String> {
    let token_url = format!(
      "{}/realms/{}/protocol/openid-connect/token",
      self.config.auth_server_url, self.config.realm
    );

    let scope_string = if scopes.is_empty() {
      String::new()
    } else {
      scopes.join(" ")
    };

    let mut params = vec![
      ("grant_type", "password"),
      ("client_id", client_id),
      ("username", username),
      ("password", password),
    ];

    if !client_secret.is_empty() {
      params.push(("client_secret", client_secret));
    }

    if !scope_string.is_empty() {
      params.push(("scope", &scope_string));
    }

    let response = self.client.post(&token_url).form(&params).send().await?;

    if response.status() != StatusCode::OK {
      anyhow::bail!(
        "Token request failed: {}",
        response
          .text()
          .await
          .unwrap_or_else(|_| "Unable to read response body".to_string())
      );
    }

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response.access_token)
  }
}
