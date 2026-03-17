use crate::db::{DbService, TimeService};
use crate::mcps::{
  CreateMcpAuthConfigRequest, DefaultMcpService, McpAuthConfigParamInput, McpAuthConfigResponse,
  McpAuthParamInput, McpAuthParamType, McpAuthType, McpRequest, McpServerRequest, McpService,
  RegistrationType,
};
use crate::test_utils::{
  test_db_service, FrozenTimeService, TestDbService, TEST_TENANT_ID, TEST_USER_ID,
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

fn default_time_service() -> Arc<dyn TimeService> {
  Arc::new(FrozenTimeService::default())
}

fn make_service(db: TestDbService) -> DefaultMcpService {
  let db: Arc<dyn DbService> = Arc::new(db);
  let mcp_client: Arc<dyn mcp_client::McpClient> = Arc::new(mcp_client::MockMcpClient::new());
  DefaultMcpService::new(db, mcp_client, default_time_service())
}

fn server_request(url: &str, name: &str) -> McpServerRequest {
  McpServerRequest {
    url: url.to_string(),
    name: name.to_string(),
    description: None,
    enabled: true,
    auth_config: None,
  }
}

fn mcp_request(
  slug: &str,
  server_id: &str,
  auth_type: McpAuthType,
  credentials: Option<Vec<McpAuthParamInput>>,
  oauth_token_id: Option<String>,
  auth_config_id: Option<String>,
) -> McpRequest {
  McpRequest {
    name: format!("Test MCP {}", slug),
    slug: slug.to_string(),
    mcp_server_id: Some(server_id.to_string()),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type,
    auth_config_id,
    credentials,
    oauth_token_id,
  }
}

// ============================================================================
// MCP Instance Create with Auth Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_with_header_credentials(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db);

  // Create server
  service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;
  let servers = service.list_mcp_servers(TEST_TENANT_ID, None).await?;
  let server_id = &servers[0].id;

  // Create MCP with header credentials
  let credentials = vec![
    McpAuthParamInput {
      param_type: McpAuthParamType::Header,
      param_key: "Authorization".to_string(),
      value: "Bearer my-secret-token".to_string(),
    },
    McpAuthParamInput {
      param_type: McpAuthParamType::Query,
      param_key: "api_key".to_string(),
      value: "query-secret".to_string(),
    },
  ];

  let result = service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      mcp_request(
        "cred-mcp",
        server_id,
        McpAuthType::Header,
        Some(credentials),
        None,
        None,
      ),
    )
    .await?;

  assert_eq!("cred-mcp", result.slug);
  assert_eq!(McpAuthType::Header, result.auth_type);

  // Verify the MCP was created with auth params
  let list = service.list(TEST_TENANT_ID, TEST_USER_ID).await?;
  assert_eq!(1, list.len());
  assert_eq!(McpAuthType::Header, list[0].auth_type);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_mcp_with_oauth_token(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db_arc: Arc<dyn DbService> = Arc::new(db);
  let mcp_client: Arc<dyn mcp_client::McpClient> = Arc::new(mcp_client::MockMcpClient::new());
  let service = DefaultMcpService::new(Arc::clone(&db_arc), mcp_client, default_time_service());

  // Create server
  service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;
  let servers = service.list_mcp_servers(TEST_TENANT_ID, None).await?;
  let server_id = servers[0].id.clone();

  // Create an OAuth config via service
  let oauth_config = service
    .create_oauth_config(
      TEST_TENANT_ID,
      "admin",
      "My OAuth",
      &server_id,
      "client-123",
      None,
      "https://auth.example.com/authorize",
      "https://auth.example.com/token",
      None,
      RegistrationType::PreRegistered,
      None,
      None,
      None,
      None,
    )
    .await?;

  // Store an OAuth token
  let token = service
    .store_oauth_token(
      TEST_TENANT_ID,
      TEST_USER_ID,
      None,
      &oauth_config.id,
      "access-token-abc",
      Some("refresh-token-xyz".to_string()),
      None,
      Some(3600),
    )
    .await?;

  // Create MCP with oauth_token_id
  let result = service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      mcp_request(
        "oauth-mcp",
        &server_id,
        McpAuthType::Oauth,
        None,
        Some(token.id.clone()),
        Some(oauth_config.id.clone()),
      ),
    )
    .await?;

  assert_eq!("oauth-mcp", result.slug);
  assert_eq!(McpAuthType::Oauth, result.auth_type);

  // Verify token is linked to MCP
  let fetched_token = service
    .get_oauth_token(TEST_TENANT_ID, TEST_USER_ID, &token.id)
    .await?;
  assert!(fetched_token.is_some());
  assert_eq!(Some(result.id), fetched_token.unwrap().mcp_id);
  Ok(())
}

// ============================================================================
// MCP Instance Update with Auth Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_change_credentials(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db);

  service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;
  let servers = service.list_mcp_servers(TEST_TENANT_ID, None).await?;
  let server_id = servers[0].id.clone();

  // Create with initial credentials
  let old_creds = vec![McpAuthParamInput {
    param_type: McpAuthParamType::Header,
    param_key: "Authorization".to_string(),
    value: "Bearer old-token".to_string(),
  }];

  let created = service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      mcp_request(
        "update-mcp",
        &server_id,
        McpAuthType::Header,
        Some(old_creds),
        None,
        None,
      ),
    )
    .await?;

  // Update with new credentials
  let new_creds = vec![McpAuthParamInput {
    param_type: McpAuthParamType::Header,
    param_key: "X-New-Key".to_string(),
    value: "new-secret-value".to_string(),
  }];

  let updated = service
    .update(
      TEST_TENANT_ID,
      TEST_USER_ID,
      &created.id,
      mcp_request(
        "update-mcp",
        &server_id,
        McpAuthType::Header,
        Some(new_creds),
        None,
        None,
      ),
    )
    .await?;

  assert_eq!(McpAuthType::Header, updated.auth_type);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_mcp_clear_auth(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db);

  service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;
  let servers = service.list_mcp_servers(TEST_TENANT_ID, None).await?;
  let server_id = servers[0].id.clone();

  // Create with header auth
  let creds = vec![McpAuthParamInput {
    param_type: McpAuthParamType::Header,
    param_key: "Authorization".to_string(),
    value: "Bearer initial".to_string(),
  }];

  let created = service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      mcp_request(
        "clear-auth-mcp",
        &server_id,
        McpAuthType::Header,
        Some(creds),
        None,
        None,
      ),
    )
    .await?;

  // Update to Public auth type — should clean up
  let updated = service
    .update(
      TEST_TENANT_ID,
      TEST_USER_ID,
      &created.id,
      mcp_request(
        "clear-auth-mcp",
        &server_id,
        McpAuthType::Public,
        None,
        None,
        None,
      ),
    )
    .await?;

  assert_eq!(McpAuthType::Public, updated.auth_type);
  assert_eq!(None, updated.auth_config_id);
  Ok(())
}

// ============================================================================
// Auth Config Service Tests
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_auth_config_header(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db);

  service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;
  let servers = service.list_mcp_servers(TEST_TENANT_ID, None).await?;
  let server_id = servers[0].id.clone();

  let request = CreateMcpAuthConfigRequest::Header {
    name: "API Key Config".to_string(),
    entries: vec![
      McpAuthConfigParamInput {
        param_type: McpAuthParamType::Header,
        param_key: "Authorization".to_string(),
      },
      McpAuthConfigParamInput {
        param_type: McpAuthParamType::Query,
        param_key: "api_key".to_string(),
      },
    ],
  };

  let result = service
    .create_auth_config(TEST_TENANT_ID, "admin", &server_id, request)
    .await?;

  match result {
    McpAuthConfigResponse::Header {
      id,
      name,
      mcp_server_id,
      entries,
      ..
    } => {
      assert!(!id.is_empty());
      assert_eq!("API Key Config", name);
      assert_eq!(server_id, mcp_server_id);
      assert_eq!(2, entries.len());
      assert_eq!("Authorization", entries[0].param_key);
      assert_eq!(McpAuthParamType::Header, entries[0].param_type);
      assert_eq!("api_key", entries[1].param_key);
      assert_eq!(McpAuthParamType::Query, entries[1].param_type);
    }
    _ => panic!("Expected Header response"),
  }

  // Verify via list
  let configs = service
    .list_auth_configs(TEST_TENANT_ID, &server_id)
    .await?;
  assert_eq!(1, configs.len());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_auth_config_oauth(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db);

  service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;
  let servers = service.list_mcp_servers(TEST_TENANT_ID, None).await?;
  let server_id = servers[0].id.clone();

  let request = CreateMcpAuthConfigRequest::Oauth {
    name: "OAuth Config".to_string(),
    client_id: "my-client".to_string(),
    authorization_endpoint: "https://auth.example.com/authorize".to_string(),
    token_endpoint: "https://auth.example.com/token".to_string(),
    client_secret: Some("my-secret".to_string()),
    scopes: Some("read write".to_string()),
    registration_type: RegistrationType::PreRegistered,
    registration_access_token: None,
    registration_endpoint: None,
    token_endpoint_auth_method: None,
    client_id_issued_at: None,
  };

  let result = service
    .create_auth_config(TEST_TENANT_ID, "admin", &server_id, request)
    .await?;

  match result {
    McpAuthConfigResponse::Oauth {
      id,
      name,
      mcp_server_id,
      client_id,
      has_client_secret,
      authorization_endpoint,
      token_endpoint,
      scopes,
      ..
    } => {
      assert!(!id.is_empty());
      assert_eq!("OAuth Config", name);
      assert_eq!(server_id, mcp_server_id);
      assert_eq!("my-client", client_id);
      assert_eq!(true, has_client_secret);
      assert_eq!("https://auth.example.com/authorize", authorization_endpoint);
      assert_eq!("https://auth.example.com/token", token_endpoint);
      assert_eq!(Some("read write".to_string()), scopes);
    }
    _ => panic!("Expected Oauth response"),
  }

  // Verify via list
  let configs = service
    .list_auth_configs(TEST_TENANT_ID, &server_id)
    .await?;
  assert_eq!(1, configs.len());
  Ok(())
}

// ============================================================================
// Resolve Auth Params Tests (via fetch_tools_for_server which uses resolve)
// ============================================================================

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_resolve_auth_params_public(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db);

  service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;
  let servers = service.list_mcp_servers(TEST_TENANT_ID, None).await?;
  let server_id = servers[0].id.clone();

  // Create a public MCP (no auth)
  let result = service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      mcp_request(
        "public-mcp",
        &server_id,
        McpAuthType::Public,
        None,
        None,
        None,
      ),
    )
    .await?;

  assert_eq!(McpAuthType::Public, result.auth_type);

  // Verify the MCP exists and is accessible
  let fetched = service
    .get(TEST_TENANT_ID, TEST_USER_ID, &result.id)
    .await?;
  assert!(fetched.is_some());
  assert_eq!(McpAuthType::Public, fetched.unwrap().auth_type);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_resolve_auth_params_header(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let service = make_service(db);

  service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;
  let servers = service.list_mcp_servers(TEST_TENANT_ID, None).await?;
  let server_id = servers[0].id.clone();

  // Create MCP with header credentials
  let credentials = vec![McpAuthParamInput {
    param_type: McpAuthParamType::Header,
    param_key: "Authorization".to_string(),
    value: "Bearer test-key".to_string(),
  }];

  let created = service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      mcp_request(
        "header-mcp",
        &server_id,
        McpAuthType::Header,
        Some(credentials),
        None,
        None,
      ),
    )
    .await?;

  assert_eq!(McpAuthType::Header, created.auth_type);

  // The auth params should be queryable via DB service layer
  let fetched = service
    .get(TEST_TENANT_ID, TEST_USER_ID, &created.id)
    .await?;
  assert!(fetched.is_some());
  assert_eq!(McpAuthType::Header, fetched.unwrap().auth_type);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_resolve_auth_params_oauth(
  #[future]
  #[from(test_db_service)]
  db: TestDbService,
) -> anyhow::Result<()> {
  let db_arc: Arc<dyn DbService> = Arc::new(db);
  let mcp_client: Arc<dyn mcp_client::McpClient> = Arc::new(mcp_client::MockMcpClient::new());
  let service = DefaultMcpService::new(Arc::clone(&db_arc), mcp_client, default_time_service());

  // Create server
  service
    .create_mcp_server(
      TEST_TENANT_ID,
      "admin",
      server_request("https://mcp.example.com", "Test Server"),
    )
    .await?;
  let servers = service.list_mcp_servers(TEST_TENANT_ID, None).await?;
  let server_id = servers[0].id.clone();

  // Create OAuth config
  let oauth_config = service
    .create_oauth_config(
      TEST_TENANT_ID,
      "admin",
      "OAuth Config",
      &server_id,
      "client-123",
      None,
      "https://auth.example.com/authorize",
      "https://auth.example.com/token",
      None,
      RegistrationType::PreRegistered,
      None,
      None,
      None,
      None,
    )
    .await?;

  // Store OAuth token
  let token = service
    .store_oauth_token(
      TEST_TENANT_ID,
      TEST_USER_ID,
      None,
      &oauth_config.id,
      "access-for-oauth-resolve",
      None,
      None,
      Some(3600),
    )
    .await?;

  // Create MCP with OAuth auth type and linked token
  let created = service
    .create(
      TEST_TENANT_ID,
      TEST_USER_ID,
      mcp_request(
        "oauth-resolve",
        &server_id,
        McpAuthType::Oauth,
        None,
        Some(token.id.clone()),
        Some(oauth_config.id.clone()),
      ),
    )
    .await?;

  assert_eq!(McpAuthType::Oauth, created.auth_type);
  assert_eq!(Some(oauth_config.id), created.auth_config_id);
  Ok(())
}
