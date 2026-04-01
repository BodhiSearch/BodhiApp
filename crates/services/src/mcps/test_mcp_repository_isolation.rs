use crate::mcps::{McpAuthType, McpEntity, McpRepository, McpServerEntity, McpServerRepository};
use crate::test_utils::{
  sea_context, setup_env, TEST_TENANT_A_USER_B_ID, TEST_TENANT_B_ID, TEST_TENANT_ID, TEST_USER_ID,
};
use anyhow_trace::anyhow_trace;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

fn make_server_for(tenant_id: &str, id: &str, url: &str, now: DateTime<Utc>) -> McpServerEntity {
  McpServerEntity {
    id: id.to_string(),
    tenant_id: tenant_id.to_string(),
    url: url.to_string(),
    name: format!("Server {}", id),
    description: Some("A test server".to_string()),
    enabled: true,
    created_by: "admin".to_string(),
    updated_by: "admin".to_string(),
    created_at: now,
    updated_at: now,
  }
}

fn make_mcp_for(
  tenant_id: &str,
  id: &str,
  server_id: &str,
  slug: &str,
  user_id: &str,
  now: DateTime<Utc>,
) -> McpEntity {
  McpEntity {
    id: id.to_string(),
    tenant_id: tenant_id.to_string(),
    user_id: user_id.to_string(),
    mcp_server_id: server_id.to_string(),
    name: format!("MCP {}", id),
    slug: slug.to_string(),
    description: None,
    enabled: true,
    auth_type: McpAuthType::Public,
    auth_config_id: None,
    created_at: now,
    updated_at: now,
  }
}

// ============================================================================
// Cross-Tenant MCP Server Isolation
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_cross_tenant_mcp_server_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  // Create servers in tenant A
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server_for(TEST_TENANT_ID, "s-a1", "https://a1.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server_for(TEST_TENANT_ID, "s-a2", "https://a2.example.com", ctx.now),
    )
    .await?;

  // Create server in tenant B
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_B_ID,
      &make_server_for(TEST_TENANT_B_ID, "s-b1", "https://b1.example.com", ctx.now),
    )
    .await?;

  // Tenant A sees only its own servers
  let servers_a = ctx.service.list_mcp_servers(TEST_TENANT_ID, None).await?;
  assert_eq!(2, servers_a.len());
  let ids_a: Vec<&str> = servers_a.iter().map(|s| s.id.as_str()).collect();
  assert!(ids_a.contains(&"s-a1"));
  assert!(ids_a.contains(&"s-a2"));

  // Tenant B sees only its own server
  let servers_b = ctx.service.list_mcp_servers(TEST_TENANT_B_ID, None).await?;
  assert_eq!(1, servers_b.len());
  assert_eq!("s-b1", servers_b[0].id);

  // Cross-tenant get returns None
  let cross = ctx.service.get_mcp_server(TEST_TENANT_B_ID, "s-a1").await?;
  assert_eq!(None, cross);

  let cross = ctx.service.get_mcp_server(TEST_TENANT_ID, "s-b1").await?;
  assert_eq!(None, cross);

  Ok(())
}

// ============================================================================
// Cross-Tenant MCP Instance Isolation
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_cross_tenant_mcp_instance_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  // Create server + instance in tenant A
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server_for(TEST_TENANT_ID, "s-a1", "https://a1.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp(
      TEST_TENANT_ID,
      &make_mcp_for(
        TEST_TENANT_ID,
        "m-a1",
        "s-a1",
        "mcp-a1",
        TEST_USER_ID,
        ctx.now,
      ),
    )
    .await?;

  // Create server + instance in tenant B (same user_id)
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_B_ID,
      &make_server_for(TEST_TENANT_B_ID, "s-b1", "https://b1.example.com", ctx.now),
    )
    .await?;
  ctx
    .service
    .create_mcp(
      TEST_TENANT_B_ID,
      &make_mcp_for(
        TEST_TENANT_B_ID,
        "m-b1",
        "s-b1",
        "mcp-b1",
        TEST_USER_ID,
        ctx.now,
      ),
    )
    .await?;

  // Tenant A list returns only its instance
  let mcps_a = ctx
    .service
    .list_mcps_with_server(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, mcps_a.len());
  assert_eq!("m-a1", mcps_a[0].id);

  // Tenant B list returns only its instance
  let mcps_b = ctx
    .service
    .list_mcps_with_server(TEST_TENANT_B_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, mcps_b.len());
  assert_eq!("m-b1", mcps_b[0].id);

  // Cross-tenant get returns None
  let cross = ctx
    .service
    .get_mcp(TEST_TENANT_B_ID, TEST_USER_ID, "m-a1")
    .await?;
  assert_eq!(None, cross);

  let cross = ctx
    .service
    .get_mcp(TEST_TENANT_ID, TEST_USER_ID, "m-b1")
    .await?;
  assert_eq!(None, cross);

  Ok(())
}

// ============================================================================
// Intra-Tenant User MCP Instance Isolation
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_intra_tenant_user_mcp_instance_isolation(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;

  // Create one shared server in tenant A
  ctx
    .service
    .create_mcp_server(
      TEST_TENANT_ID,
      &make_server_for(
        TEST_TENANT_ID,
        "s-shared",
        "https://shared.example.com",
        ctx.now,
      ),
    )
    .await?;

  // Create instance for user A
  ctx
    .service
    .create_mcp(
      TEST_TENANT_ID,
      &make_mcp_for(
        TEST_TENANT_ID,
        "m-u1",
        "s-shared",
        "mcp-user-a",
        TEST_USER_ID,
        ctx.now,
      ),
    )
    .await?;

  // Create instance for user B (same tenant)
  ctx
    .service
    .create_mcp(
      TEST_TENANT_ID,
      &make_mcp_for(
        TEST_TENANT_ID,
        "m-u2",
        "s-shared",
        "mcp-user-b",
        TEST_TENANT_A_USER_B_ID,
        ctx.now,
      ),
    )
    .await?;

  // User A sees only their instance
  let mcps_u1 = ctx
    .service
    .list_mcps_with_server(TEST_TENANT_ID, TEST_USER_ID)
    .await?;
  assert_eq!(1, mcps_u1.len());
  assert_eq!("m-u1", mcps_u1[0].id);

  // User B sees only their instance
  let mcps_u2 = ctx
    .service
    .list_mcps_with_server(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID)
    .await?;
  assert_eq!(1, mcps_u2.len());
  assert_eq!("m-u2", mcps_u2[0].id);

  // Cross-user get returns None
  let cross = ctx
    .service
    .get_mcp(TEST_TENANT_ID, TEST_TENANT_A_USER_B_ID, "m-u1")
    .await?;
  assert_eq!(None, cross);

  let cross = ctx
    .service
    .get_mcp(TEST_TENANT_ID, TEST_USER_ID, "m-u2")
    .await?;
  assert_eq!(None, cross);

  Ok(())
}
