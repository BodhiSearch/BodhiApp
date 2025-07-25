use anyhow::Result;
use axum::http::StatusCode;
use base64::{engine::general_purpose, Engine as _};
use derive_builder::Builder;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Configuration for the auth server test client
#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct AuthServerConfig {
  pub auth_server_url: String,
  pub realm: String,
  pub dev_console_client_id: String,
  pub dev_console_client_secret: String,
}

/// OAuth token response structure
#[derive(Debug, Deserialize)]
struct TokenResponse {
  access_token: String,
}

/// Client creation response
#[derive(Debug, Deserialize)]
struct ClientCreateResponse {
  client_id: String,
  #[serde(default)]
  client_secret: Option<String>,
  #[serde(default)]
  scope: Option<String>,
}

/// App client creation request
#[derive(Debug, Serialize)]
struct AppClientRequest {
  name: String,
  description: String,
  redirect_uris: Vec<String>,
}

/// Resource client creation request
#[derive(Debug, Serialize)]
struct ResourceClientRequest {
  name: String,
  description: String,
  redirect_uris: Vec<String>,
}

/// Request access request
#[derive(Debug, Serialize)]
struct RequestAccessRequest {
  app_client_id: String,
}

/// Make admin request
#[derive(Debug, Serialize)]
struct MakeAdminRequest {
  username: String,
}

/// Add user to group request
#[derive(Debug, Serialize)]
struct AddToGroupRequest {
  username: String,
  group: String,
  add: bool,
}

/// Created client information
#[derive(Debug, Clone)]
pub struct ClientInfo {
  pub client_id: String,
  pub client_secret: Option<String>,
  pub scope: Option<String>,
}

/// Dynamic clients setup result
#[derive(Debug, Clone)]
pub struct DynamicClients {
  pub app_client: ClientInfo,
  pub resource_client: ClientInfo,
  pub resource_scope_name: String,
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

  /// Step 1: Get Dev Console User Token (Direct Access Grant)
  pub async fn get_dev_console_user_token(&self, username: &str, password: &str) -> Result<String> {
    let token_url = format!(
      "{}/realms/{}/protocol/openid-connect/token",
      self.config.auth_server_url, self.config.realm
    );

    let auth_header = format!(
      "{}:{}",
      self.config.dev_console_client_id, self.config.dev_console_client_secret
    );
    let encoded_auth = general_purpose::STANDARD.encode(&auth_header);

    let params = vec![
      ("grant_type", "password"),
      ("username", username),
      ("password", password),
    ];

    let response = self
      .client
      .post(&token_url)
      .header("Authorization", format!("Basic {}", encoded_auth))
      .form(&params)
      .send()
      .await?;

    if response.status() != StatusCode::OK {
      anyhow::bail!(
        "Dev console user token request failed: {}",
        response
          .text()
          .await
          .unwrap_or_else(|_| "Unable to read response body".to_string())
      );
    }

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response.access_token)
  }

  /// Step 2: Create App Client (Public, No Secrets)
  pub async fn create_app_client(&self, dev_console_token: &str, name: &str) -> Result<ClientInfo> {
    let apps_url = format!(
      "{}/realms/{}/bodhi/apps",
      self.config.auth_server_url, self.config.realm
    );

    let request_body = AppClientRequest {
      name: name.to_string(),
      description: format!("App client for {}", name),
      redirect_uris: vec!["http://localhost:3000/callback".to_string()],
    };

    let response = self
      .client
      .post(&apps_url)
      .header("Authorization", format!("Bearer {}", dev_console_token))
      .json(&request_body)
      .send()
      .await?;

    if response.status() != StatusCode::CREATED {
      anyhow::bail!(
        "App client creation failed: {}",
        response
          .text()
          .await
          .unwrap_or_else(|_| "Unable to read response body".to_string())
      );
    }

    let create_response: ClientCreateResponse = response.json().await?;
    Ok(ClientInfo {
      client_id: create_response.client_id,
      client_secret: create_response.client_secret,
      scope: create_response.scope,
    })
  }

  /// Step 3: Create Resource Server Client (Confidential, With Secrets)
  pub async fn create_resource_client(&self, name: &str) -> Result<ClientInfo> {
    let resources_url = format!(
      "{}/realms/{}/bodhi/resources",
      self.config.auth_server_url, self.config.realm
    );

    let request_body = ResourceClientRequest {
      name: name.to_string(),
      description: format!("Resource client for {}", name),
      redirect_uris: vec!["http://localhost:8080/callback".to_string()],
    };

    let response = self
      .client
      .post(&resources_url)
      .json(&request_body)
      .send()
      .await?;

    if response.status() != StatusCode::CREATED {
      anyhow::bail!(
        "Resource client creation failed: {}",
        response
          .text()
          .await
          .unwrap_or_else(|_| "Unable to read response body".to_string())
      );
    }

    let create_response: ClientCreateResponse = response.json().await?;
    Ok(ClientInfo {
      client_id: create_response.client_id,
      client_secret: create_response.client_secret,
      scope: create_response.scope,
    })
  }

  /// Step 4: Get Resource Client Service Account Token
  pub async fn get_resource_service_token(&self, resource_client: &ClientInfo) -> Result<String> {
    let token_url = format!(
      "{}/realms/{}/protocol/openid-connect/token",
      self.config.auth_server_url, self.config.realm
    );

    let client_secret = resource_client
      .client_secret
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Resource client missing client secret"))?;

    let auth_header = format!("{}:{}", resource_client.client_id, client_secret);
    let encoded_auth = general_purpose::STANDARD.encode(&auth_header);

    let params = vec![
      ("grant_type", "client_credentials"),
      ("scope", "service_account"),
    ];

    let response = self
      .client
      .post(&token_url)
      .header("Authorization", format!("Basic {}", encoded_auth))
      .form(&params)
      .send()
      .await?;

    if response.status() != StatusCode::OK {
      anyhow::bail!(
        "Resource service token request failed: {}",
        response
          .text()
          .await
          .unwrap_or_else(|_| "Unable to read response body".to_string())
      );
    }

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response.access_token)
  }

  /// Step 5: Make First Resource Admin
  pub async fn make_first_resource_admin(
    &self,
    resource_service_token: &str,
    username: &str,
  ) -> Result<()> {
    let make_admin_url = format!(
      "{}/realms/{}/bodhi/resources/make-resource-admin",
      self.config.auth_server_url, self.config.realm
    );

    let request_body = MakeAdminRequest {
      username: username.to_string(),
    };

    let response = self
      .client
      .post(&make_admin_url)
      .header(
        "Authorization",
        format!("Bearer {}", resource_service_token),
      )
      .json(&request_body)
      .send()
      .await?;

    if response.status() != StatusCode::CREATED {
      anyhow::bail!(
        "Make admin failed: {}",
        response
          .text()
          .await
          .unwrap_or_else(|_| "Unable to read response body".to_string())
      );
    }

    Ok(())
  }

  /// Step 6: Resource Client Requests Audience Access (Dynamic On-Demand)
  pub async fn request_audience_access(
    &self,
    resource_service_token: &str,
    app_client_id: &str,
  ) -> Result<String> {
    let request_access_url = format!(
      "{}/realms/{}/bodhi/resources/request-access",
      self.config.auth_server_url, self.config.realm
    );

    let request_body = RequestAccessRequest {
      app_client_id: app_client_id.to_string(),
    };

    let response = self
      .client
      .post(&request_access_url)
      .header(
        "Authorization",
        format!("Bearer {}", resource_service_token),
      )
      .json(&request_body)
      .send()
      .await?;

    let status = response.status();
    if status != StatusCode::CREATED && status != StatusCode::OK {
      anyhow::bail!(
        "Audience access request failed: {}",
        response
          .text()
          .await
          .unwrap_or_else(|_| "Unable to read response body".to_string())
      );
    }

    let response_body: serde_json::Value = response.json().await?;
    Ok(
      response_body["scope"]
        .as_str()
        .unwrap_or_default()
        .to_string(),
    )
  }

  /// Step 7: Add User to Group (optional for advanced role management)
  pub async fn add_user_to_group(
    &self,
    admin_token: &str,
    username: &str,
    group: &str,
    add: bool,
  ) -> Result<()> {
    let add_to_group_url = format!(
      "{}/realms/{}/bodhi/resources/add-user-to-group",
      self.config.auth_server_url, self.config.realm
    );

    let request_body = AddToGroupRequest {
      username: username.to_string(),
      group: group.to_string(),
      add,
    };

    let response = self
      .client
      .post(&add_to_group_url)
      .header("Authorization", format!("Bearer {}", admin_token))
      .json(&request_body)
      .send()
      .await?;

    let status = response.status();
    if status != StatusCode::CREATED && status != StatusCode::OK {
      anyhow::bail!(
        "Group operation failed: {}",
        response
          .text()
          .await
          .unwrap_or_else(|_| "Unable to read response body".to_string())
      );
    }

    Ok(())
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

  /// Complete dynamic client setup following httpyac script pattern
  pub async fn setup_dynamic_clients(&self, username: &str, password: &str) -> Result<DynamicClients> {
    // Step 1: Get Dev Console User Token
    let dev_console_token = self.get_dev_console_user_token(username, password).await?;

    // Step 2: Create App Client
    let app_client = self
      .create_app_client(&dev_console_token, "Test App Client")
      .await?;

    // Step 3: Create Resource Server Client
    let resource_client = self
      .create_resource_client("Test Resource Server")
      .await?;

    // Step 4: Get Resource Client Service Account Token
    let resource_service_token = self.get_resource_service_token(&resource_client).await?;

    // Step 5: Make First Resource Admin
    self
      .make_first_resource_admin(&resource_service_token, username)
      .await?;

    // Step 6: Request Audience Access
    let final_scope_name = self
      .request_audience_access(&resource_service_token, &app_client.client_id)
      .await?;

    let resource_scope_name = if final_scope_name.is_empty() {
      resource_client.scope.clone().unwrap_or_default()
    } else {
      final_scope_name
    };

    Ok(DynamicClients {
      app_client,
      resource_client,
      resource_scope_name,
    })
  }
}
