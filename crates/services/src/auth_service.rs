use crate::AppRegInfo;
use async_trait::async_trait;
use oauth2::{
  basic::BasicTokenType, AccessToken, AuthorizationCode, ClientId, ClientSecret,
  EmptyExtraTokenFields, PkceCodeVerifier, RedirectUrl, RefreshToken, StandardTokenResponse,
  TokenResponse,
};
use objs::{
  impl_error_from, log, AppError, AppRole, ErrorType, ReqwestError, ResourceRole, UserInfo,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
  async fn register_client(
    &self,
    name: String,
    description: String,
    redirect_uris: Vec<String>,
  ) -> Result<AppRegInfo>;

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

  async fn exchange_app_token(
    &self,
    client_id: &str,
    client_secret: &str,
    subject_token: &str,
    scopes: Vec<String>,
  ) -> Result<(String, Option<String>)>;

  async fn make_resource_admin(
    &self,
    client_id: &str,
    client_secret: &str,
    user_id: &str,
  ) -> Result<()>;

  async fn request_access(
    &self,
    client_id: &str,
    client_secret: &str,
    app_client_id: &str,
  ) -> Result<RequestAccessResponse>;

  async fn assign_user_role(&self, reveiwer_token: &str, user_id: &str, role: &str) -> Result<()>;

  async fn remove_user(&self, reviewer_token: &str, user_id: &str) -> Result<()>;

  async fn list_users(
    &self,
    reviewer_token: &str,
    page: Option<u32>,
    page_size: Option<u32>,
  ) -> Result<UserListResponse>;
}

#[derive(Debug)]
pub struct KeycloakAuthService {
  app_version: String,
  auth_url: String,
  realm: String,
  client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct KeycloakError {
  error: String,
  #[serde(default)]
  error_description: Option<String>,
}

impl From<KeycloakError> for AuthServiceError {
  fn from(value: KeycloakError) -> Self {
    let msg = if let Some(desc) = value.error_description {
      format!("{}: {}", value.error, desc)
    } else {
      value.error
    };
    AuthServiceError::AuthServiceApiError(msg)
  }
}

impl KeycloakAuthService {
  pub fn new(app_version: &str, auth_url: String, realm: String) -> Self {
    Self {
      app_version: app_version.to_string(),
      auth_url,
      realm,
      client: reqwest::Client::new(),
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
      ("scope", "service_account"),
    ];

    let url = self.auth_token_url();
    // Log request with masked parameters
    log::log_http_request("POST", &url, "auth_service", Some(&params));

    // Use traced client - reqwest-tracing will handle HTTP request/response logging
    let response = self
      .client
      .post(&url)
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
      log::log_http_error("POST", &url, "auth_service", &error.error);
      Err(error.into())
    }
  }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterClientRequest {
  #[schema(min_length = 10, example = "My Resource Server")]
  pub name: String,
  #[schema(example = "A description of my resource server")]
  pub description: String,
  pub redirect_uris: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AppAccessRequest {
  pub app_client_id: String,
  /// Optional version for cache lookup - if matches cached config, skips auth server call
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
}

/// Tool configuration from app-client registration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AppClientTool {
  pub tool_id: String,
  pub tool_scope: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AppAccessResponse {
  pub scope: String,
  /// List of tools the app-client is configured to access
  #[serde(default)]
  pub tools: Vec<AppClientTool>,
  /// Version of app-client's tool configuration on auth server
  pub app_client_config_version: String,
}

/// Internal request to auth server (without version field)
#[derive(Debug, Serialize, Deserialize)]
struct RequestAccessRequest {
  pub app_client_id: String,
}

/// Response from auth server /resources/request-access
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestAccessResponse {
  pub scope: String,
  #[serde(default)]
  pub tools: Vec<AppClientTool>,
  pub app_client_config_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfoResponse {
  pub user_id: String,
  pub username: String,
  pub first_name: Option<String>,
  pub last_name: Option<String>,
  pub role: String,
}

impl From<UserInfoResponse> for UserInfo {
  fn from(response: UserInfoResponse) -> Self {
    let role = if let Ok(parsed_role) = response.role.parse::<ResourceRole>() {
      Some(AppRole::Session(parsed_role))
    } else {
      None
    };

    Self {
      user_id: response.user_id,
      username: response.username,
      first_name: response.first_name,
      last_name: response.last_name,
      role,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct UserListResponseRaw {
  pub client_id: String,
  pub users: Vec<UserInfoResponse>,
  pub page: u32,
  pub page_size: u32,
  pub total_pages: u32,
  pub total_users: u32,
  pub has_next: bool,
  pub has_previous: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserListResponse {
  #[schema(example = "resource-abc123def456")]
  pub client_id: String,
  pub users: Vec<UserInfo>,
  #[schema(example = 1)]
  pub page: u32,
  #[schema(example = 10)]
  pub page_size: u32,
  #[schema(example = 5)]
  pub total_pages: u32,
  #[schema(example = 45)]
  pub total_users: u32,
  #[schema(example = true)]
  pub has_next: bool,
  #[schema(example = false)]
  pub has_previous: bool,
}

#[async_trait]
impl AuthService for KeycloakAuthService {
  async fn register_client(
    &self,
    name: String,
    description: String,
    redirect_uris: Vec<String>,
  ) -> Result<AppRegInfo> {
    let client_endpoint = format!("{}/resources", self.auth_api_url());
    log::log_http_request("POST", &client_endpoint, "auth_service", None);

    let request_body = RegisterClientRequest {
      name,
      description,
      redirect_uris,
    };

    let response = self
      .client
      .post(&client_endpoint)
      .json(&request_body)
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    if response.status().is_success() {
      Ok(response.json::<AppRegInfo>().await?)
    } else {
      let response_text = response.text().await?;
      log::log_http_error("POST", &client_endpoint, "auth_service", &response_text);
      let error: KeycloakError = serde_json::from_str(&response_text).unwrap_or(KeycloakError {
        error: response_text.clone(),
        error_description: None,
      });
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

    let url = self.auth_token_url();
    log::log_http_request("POST", &url, "auth_service", Some(&params));

    let response = self
      .client
      .post(&url)
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
      log::log_http_error("POST", &url, "auth_service", &error.error);
      Err(error.into())
    }
  }

  async fn exchange_app_token(
    &self,
    client_id: &str,
    client_secret: &str,
    subject_token: &str,
    scopes: Vec<String>,
  ) -> Result<(String, Option<String>)> {
    let params = [
      (
        "grant_type",
        "urn:ietf:params:oauth:grant-type:token-exchange",
      ),
      ("subject_token", subject_token),
      ("client_id", client_id),
      ("client_secret", client_secret),
      ("audience", client_id),
      ("scope", &scopes.join(" ")),
    ];

    let url = self.auth_token_url();
    log::log_http_request("POST", &url, "auth_service", Some(&params));

    let response = self
      .client
      .post(&url)
      .form(&params)
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    let token_response: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> =
      if response.status().is_success() {
        response.json().await?
      } else {
        let error = response.json::<KeycloakError>().await?;
        log::log_http_error("POST", &url, "auth_service", &error.error);
        return Err(AuthServiceError::AuthServiceApiError(error.error));
      };

    let access_token = token_response.access_token().secret().to_string();
    let refresh_token = token_response
      .refresh_token()
      .map(|s| s.secret().to_string());
    Ok((access_token, refresh_token))
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

    let url = self.auth_token_url();
    log::log_http_request("POST", &url, "auth_service", Some(&params));

    // Log token prefix for debugging (first 10 chars)
    tracing::debug!(
      "Attempting token refresh with token prefix: {}...",
      &refresh_token.chars().take(10).collect::<String>()
    );

    // Retry logic with exponential backoff for network errors only
    // Attempts: 1st try immediate, retry 1 after 100ms, retry 2 after 500ms, retry 3 after 2000ms
    let max_retries = 3;
    let mut last_error = None;

    for attempt in 0..=max_retries {
      if attempt > 0 {
        let delay_ms = match attempt {
          1 => 100,
          2 => 500,
          3 => 2000,
          _ => 0,
        };
        tracing::info!(
          "Retrying token refresh (attempt {}/{}) after {}ms delay",
          attempt,
          max_retries,
          delay_ms
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
      }

      let response = match self
        .client
        .post(&url)
        .form(&params)
        .header(HEADER_BODHI_APP_VERSION, &self.app_version)
        .send()
        .await
      {
        Ok(resp) => resp,
        Err(e) => {
          // Check if this is a network/timeout error that should be retried
          if e.is_timeout() || e.is_connect() || e.is_request() {
            tracing::warn!(
              "Network error during token refresh (attempt {}/{}): {}",
              attempt + 1,
              max_retries + 1,
              e
            );
            last_error = Some(e.into());
            continue; // Retry
          } else {
            // Other errors (e.g., invalid request structure) should not be retried
            tracing::error!("Non-retryable error during token refresh: {}", e);
            return Err(e.into());
          }
        }
      };

      // Check response status
      if response.status().is_success() {
        let token_response: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> =
          response.json().await?;
        tracing::info!(
          "Token refresh successful{}",
          if attempt > 0 {
            format!(" after {} retries", attempt)
          } else {
            String::new()
          }
        );
        return Ok((
          token_response.access_token().secret().to_string(),
          token_response
            .refresh_token()
            .map(|s| s.secret().to_string()),
        ));
      } else {
        // 4xx or 5xx response - parse error
        let status = response.status();
        let error = response.json::<KeycloakError>().await?;

        // 4xx errors (client errors) should not be retried
        if status.is_client_error() {
          let error_msg = if let Some(ref desc) = error.error_description {
            format!("{}: {}", error.error, desc)
          } else {
            error.error.clone()
          };
          tracing::error!(
            "Token refresh failed with client error ({}): {}",
            status,
            error_msg
          );
          log::log_http_error("POST", &url, "auth_service", &error_msg);
          return Err(error.into());
        }

        // 5xx errors (server errors) can be retried
        if status.is_server_error() && attempt < max_retries {
          let error_msg = if let Some(ref desc) = error.error_description {
            format!("{}: {}", error.error, desc)
          } else {
            error.error.clone()
          };
          tracing::warn!(
            "Token refresh failed with server error (attempt {}/{}): {} - {}",
            attempt + 1,
            max_retries + 1,
            status,
            error_msg
          );
          last_error = Some(error.into());
          continue; // Retry
        } else {
          let error_msg = if let Some(ref desc) = error.error_description {
            format!("{}: {}", error.error, desc)
          } else {
            error.error.clone()
          };
          tracing::error!(
            "Token refresh failed with server error (final attempt): {} - {}",
            status,
            error_msg
          );
          log::log_http_error("POST", &url, "auth_service", &error_msg);
          return Err(error.into());
        }
      }
    }

    // All retries exhausted
    tracing::error!("Token refresh failed after {} attempts", max_retries + 1);
    Err(
      last_error.unwrap_or_else(|| {
        AuthServiceError::AuthServiceApiError("Max retries exceeded".to_string())
      }),
    )
  }

  async fn make_resource_admin(
    &self,
    client_id: &str,
    client_secret: &str,
    user_id: &str,
  ) -> Result<()> {
    // Get client access token
    let access_token = self
      .get_client_access_token(client_id, client_secret)
      .await?;

    // Make API call to make the user a resource admin
    let endpoint = format!("{}/resources/make-resource-admin", self.auth_api_url());
    log::log_http_request("POST", &endpoint, "auth_service", None);

    let response = self
      .client
      .post(&endpoint)
      .bearer_auth(access_token.secret())
      .json(&serde_json::json!({ "user_id": user_id }))
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    if response.status().is_success() {
      Ok(())
    } else {
      let error = response.json::<KeycloakError>().await?;
      log::log_http_error("POST", &endpoint, "auth_service", &error.error);
      Err(error.into())
    }
  }

  async fn request_access(
    &self,
    client_id: &str,
    client_secret: &str,
    app_client_id: &str,
  ) -> Result<RequestAccessResponse> {
    // Get client access token
    let access_token = self
      .get_client_access_token(client_id, client_secret)
      .await?;

    // Make API call to request access
    let endpoint = format!(
      "{}/realms/{}/bodhi/resources/request-access",
      self.auth_url, self.realm
    );
    log::log_http_request("POST", &endpoint, "auth_service", None);

    let response = self
      .client
      .post(&endpoint)
      .bearer_auth(access_token.secret())
      .json(&RequestAccessRequest {
        app_client_id: app_client_id.to_string(),
      })
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    if response.status().is_success() {
      Ok(response.json().await?)
    } else {
      let error = response.json::<KeycloakError>().await?;
      log::log_http_error("POST", &endpoint, "auth_service", &error.error);
      Err(error.into())
    }
  }

  async fn assign_user_role(&self, reviewer_token: &str, user_id: &str, role: &str) -> Result<()> {
    // Make API call to assign role to user
    let endpoint = format!(
      "{}/realms/{}/bodhi/resources/assign-role",
      self.auth_url, self.realm
    );
    log::log_http_request("POST", &endpoint, "auth_service", None);

    let response = self
      .client
      .post(&endpoint)
      .bearer_auth(reviewer_token)
      .json(&serde_json::json!({
        "user_id": user_id,
        "role": role
      }))
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    if response.status().is_success() {
      Ok(())
    } else {
      let error = response.json::<KeycloakError>().await?;
      log::log_http_error("POST", &endpoint, "auth_service", &error.error);
      Err(error.into())
    }
  }

  async fn remove_user(&self, reviewer_token: &str, user_id: &str) -> Result<()> {
    // Make API call to remove user from all roles
    let endpoint = format!(
      "{}/realms/{}/bodhi/resources/remove-user",
      self.auth_url, self.realm
    );
    log::log_http_request("POST", &endpoint, "auth_service", None);

    let response = self
      .client
      .post(&endpoint)
      .bearer_auth(reviewer_token)
      .json(&serde_json::json!({
        "user_id": user_id
      }))
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    if response.status().is_success() {
      Ok(())
    } else {
      let error = response.json::<KeycloakError>().await?;
      log::log_http_error("POST", &endpoint, "auth_service", &error.error);
      Err(error.into())
    }
  }

  async fn list_users(
    &self,
    reviewer_token: &str,
    page: Option<u32>,
    page_size: Option<u32>,
  ) -> Result<UserListResponse> {
    let mut url = url::Url::parse(&format!("{}/resources/users", self.auth_api_url())).unwrap();
    if let Some(page) = page {
      url.query_pairs_mut().append_pair("page", &page.to_string());
    }
    if let Some(page_size) = page_size {
      url
        .query_pairs_mut()
        .append_pair("page_size", &page_size.to_string());
    }

    let endpoint = url.to_string();
    log::log_http_request("GET", &endpoint, "auth_service", None);
    let response = self
      .client
      .get(&endpoint)
      .bearer_auth(reviewer_token)
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    if response.status().is_success() {
      let raw_response: UserListResponseRaw = response.json().await?;
      let converted_response = UserListResponse {
        client_id: raw_response.client_id,
        users: raw_response
          .users
          .into_iter()
          .map(|user| user.into())
          .collect(),
        page: raw_response.page,
        page_size: raw_response.page_size,
        total_pages: raw_response.total_pages,
        total_users: raw_response.total_users,
        has_next: raw_response.has_next,
        has_previous: raw_response.has_previous,
      };
      return Ok(converted_response);
    } else {
      let error = response.json::<KeycloakError>().await?;
      log::log_http_error("GET", &endpoint, "auth_service", &error.error);
      Err(error.into())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    test_utils::test_auth_service, AppRegInfo, AuthService, AuthServiceError, JsonWebTokenError,
  };
  use jsonwebtoken::errors::ErrorKind;
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
    let result = service
      .register_client(
        "Test Resource Server Name".to_string(),
        "Test resource client description".to_string(),
        vec!["http://0.0.0.0:1135/bodhi/v1/auth/callback".to_string()],
      )
      .await;
    assert!(result.is_ok());
    let app_reg_info = result.unwrap();
    assert_eq!(
      AppRegInfo {
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
          "scope": "openid profile email roles",
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
      AuthServiceError::AuthServiceApiError(msg) if msg == "invalid_grant: Invalid refresh token"
    ));
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
    let result = service
      .make_resource_admin(client_id, client_secret, user_id)
      .await;

    assert!(result.is_ok());
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
  async fn test_request_access_success() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;
    let url = server.url();

    let client_id = "test_client_id";
    let client_secret = "test_client_secret";
    let app_client_id = "test_app_client_id";

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

    // Mock request-access endpoint - corrected path
    let access_mock = server
      .mock("POST", "/realms/test-realm/bodhi/resources/request-access")
      .match_header("Authorization", "Bearer test_access_token")
      .match_body(Matcher::Json(json!({"app_client_id": app_client_id})))
      .with_status(200)
      .with_body(
        json!({
          "scope": "scope_resource_test-resource-server",
          "tools": [{"tool_id": "builtin-exa-web-search", "tool_scope": "scope_tool-builtin-exa-web-search"}],
          "app_client_config_version": "v1.0.0"
        })
        .to_string(),
      )
      .create();

    let service = test_auth_service(&url);
    let result = service
      .request_access(client_id, client_secret, app_client_id)
      .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.scope, "scope_resource_test-resource-server");
    assert_eq!(response.tools.len(), 1);
    assert_eq!(response.tools[0].tool_id, "builtin-exa-web-search");
    assert_eq!(response.app_client_config_version, "v1.0.0");
    token_mock.assert();
    access_mock.assert();

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_request_access_error() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;
    let url = server.url();

    let client_id = "test_client_id";
    let client_secret = "test_client_secret";
    let app_client_id = "invalid_app_client_id";

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

    // Mock request-access endpoint with error
    let access_mock = server
      .mock("POST", "/realms/test-realm/bodhi/resources/request-access")
      .with_status(400)
      .with_body(json!({"error": "app_client_not_found"}).to_string())
      .create();

    let service = test_auth_service(&url);
    let result = service
      .request_access(client_id, client_secret, app_client_id)
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      AuthServiceError::AuthServiceApiError(_)
    ));
    token_mock.assert();
    access_mock.assert();

    Ok(())
  }
}
