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
  #[error("Authentication service API error (status {status}): {body}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  AuthServiceApiError { status: u16, body: String },
  #[error("Network error during authentication: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ReqwestMiddlewareError(String),
  #[error("Token exchange failed: {0}.")]
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

  async fn assign_user_role(&self, reveiwer_token: &str, user_id: &str, role: &str) -> Result<()>;

  async fn remove_user(&self, reviewer_token: &str, user_id: &str) -> Result<()>;

  async fn list_users(
    &self,
    reviewer_token: &str,
    page: Option<u32>,
    page_size: Option<u32>,
  ) -> Result<UserListResponse>;

  async fn register_access_request_consent(
    &self,
    user_token: &str,
    app_client_id: &str,
    access_request_id: &str,
    description: &str,
  ) -> Result<RegisterAccessRequestConsentResponse>;

  /// Register resource access for auto-approve flow (no user token needed)
  /// KC endpoint: POST /resources/apps/request-access
  /// Uses resource service account token, NOT user token
  async fn register_resource_access(
    &self,
    client_id: &str,
    client_secret: &str,
    app_client_id: &str,
  ) -> Result<RegisterResourceAccessResponse>;

  /// Get app client info (name, description) from Keycloak
  /// KC endpoint: GET /users/apps/{app_client_id}/info
  async fn get_app_client_info(
    &self,
    app_client_id: &str,
    user_token: &str,
  ) -> Result<AppClientInfo>;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAccessRequestConsentResponse {
  pub scope: String,
  pub access_request_id: String,
  pub access_request_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResourceAccessResponse {
  pub scope: String, // Only resource_scope returned by KC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppClientInfo {
  pub name: String,
  pub description: String,
}

impl From<KeycloakError> for AuthServiceError {
  fn from(value: KeycloakError) -> Self {
    let body = if let Some(desc) = value.error_description {
      format!("{}: {}", value.error, desc)
    } else {
      value.error
    };
    AuthServiceError::AuthServiceApiError { status: 0, body }
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

/// Toolset configuration from app-client registration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AppClientToolset {
  pub id: String,
  pub scope: String,
  /// client scope UUID for cache validation
  #[serde(default)]
  pub scope_id: String,
  /// True if scope has been added to resource-client as optional scope
  #[serde(skip_serializing_if = "Option::is_none")]
  pub added_to_resource_client: Option<bool>,
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

    let status = response.status();
    let token_response: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> =
      if status.is_success() {
        response.json().await?
      } else {
        let status_code = status.as_u16();
        let error = response.json::<KeycloakError>().await?;
        log::log_http_error("POST", &url, "auth_service", &error.error);
        return Err(AuthServiceError::AuthServiceApiError {
          status: status_code,
          body: error.error,
        });
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
      last_error.unwrap_or_else(|| AuthServiceError::AuthServiceApiError {
        status: 0,
        body: "Max retries exceeded".to_string(),
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

  async fn register_access_request_consent(
    &self,
    user_token: &str,
    app_client_id: &str,
    access_request_id: &str,
    description: &str,
  ) -> Result<RegisterAccessRequestConsentResponse> {
    let endpoint = format!("{}/users/request-access", self.auth_api_url());

    log::log_http_request("POST", &endpoint, "auth_service", None);

    let request_body = serde_json::json!({
      "app_client_id": app_client_id,
      "access_request_id": access_request_id,
      "description": description,
    });

    let response = self
      .client
      .post(&endpoint)
      .json(&request_body)
      .header("Authorization", format!("Bearer {}", user_token))
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    let status = response.status();

    if status.is_success() {
      // 201 Created or 200 OK (idempotent retry)
      Ok(
        response
          .json::<RegisterAccessRequestConsentResponse>()
          .await?,
      )
    } else if status == 409 {
      // UUID collision - different context
      let error_text = response.text().await?;
      log::log_http_error("POST", &endpoint, "auth_service", &error_text);
      Err(AuthServiceError::TokenExchangeError(format!(
        "UUID collision (409): {}",
        error_text
      )))
    } else {
      // 400, 401, or other errors
      let error_text = response.text().await?;
      log::log_http_error("POST", &endpoint, "auth_service", &error_text);
      Err(AuthServiceError::AuthServiceApiError {
        status: status.as_u16(),
        body: error_text,
      })
    }
  }

  async fn register_resource_access(
    &self,
    client_id: &str,
    client_secret: &str,
    app_client_id: &str,
  ) -> Result<RegisterResourceAccessResponse> {
    let access_token = self
      .get_client_access_token(client_id, client_secret)
      .await?;
    let endpoint = format!("{}/resources/apps/request-access", self.auth_api_url());

    log::log_http_request("POST", &endpoint, "auth_service", None);

    let request_body = serde_json::json!({
      "app_client_id": app_client_id,
    });

    let response = self
      .client
      .post(&endpoint)
      .json(&request_body)
      .bearer_auth(access_token.secret())
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    let status = response.status();

    if status.is_success() {
      Ok(response.json::<RegisterResourceAccessResponse>().await?)
    } else {
      let error_text = response.text().await?;
      log::log_http_error("POST", &endpoint, "auth_service", &error_text);
      Err(AuthServiceError::AuthServiceApiError {
        status: status.as_u16(),
        body: error_text,
      })
    }
  }

  async fn get_app_client_info(
    &self,
    app_client_id: &str,
    user_token: &str,
  ) -> Result<AppClientInfo> {
    // TODO: KC endpoint not yet implemented
    let endpoint = format!("{}/users/apps/{}/info", self.auth_api_url(), app_client_id);

    log::log_http_request("GET", &endpoint, "auth_service", None);

    let response = self
      .client
      .get(&endpoint)
      .header("Authorization", format!("Bearer {}", user_token))
      .header(HEADER_BODHI_APP_VERSION, &self.app_version)
      .send()
      .await?;

    let status = response.status();

    if status.is_success() {
      Ok(response.json::<AppClientInfo>().await?)
    } else {
      let error_text = response.text().await?;
      log::log_http_error("GET", &endpoint, "auth_service", &error_text);
      Err(AuthServiceError::AuthServiceApiError {
        status: status.as_u16(),
        body: error_text,
      })
    }
  }
}
