use crate::mcps::{McpInstanceRepository, McpRow, McpServerRepository, McpServerRow};
use crate::test_utils::{sea_context, setup_env};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

use crate::mcps::test_helpers::{make_mcp, make_server};

// ============================================================================
// MCP Server Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_mcp_server(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let row = McpServerRow {
    id: "s1".to_string(),
    url: "https://mcp.example.com".to_string(),
    name: "Server s1".to_string(),
    description: Some("A test server".to_string()),
    enabled: true,
    created_by: "admin".to_string(),
    updated_by: "admin".to_string(),
    created_at: ctx.now,
    updated_at: ctx.now,
  };
  let created = ctx.service.create_mcp_server(&row).await?;
  assert_eq!(row.id, created.id);
  assert_eq!(row.url, created.url);
  assert_eq!(row.name, created.name);
  assert_eq!(row.description, created.description);
  assert_eq!(row.enabled, created.enabled);

  let fetched = ctx.service.get_mcp_server("s1").await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("s1", fetched.id);
  assert_eq!("https://mcp.example.com", fetched.url);
  assert_eq!(ctx.now, fetched.created_at);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_server_not_found(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let result = ctx.service.get_mcp_server("nonexistent").await?;
  assert_eq!(None, result);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_mcp_server(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let row = McpServerRow {
    id: "s1".to_string(),
    url: "https://mcp.example.com".to_string(),
    name: "Server s1".to_string(),
    description: Some("A test server".to_string()),
    enabled: true,
    created_by: "admin".to_string(),
    updated_by: "admin".to_string(),
    created_at: ctx.now,
    updated_at: ctx.now,
  };
  ctx.service.create_mcp_server(&row).await?;

  let updated_at = ctx.now + chrono::Duration::seconds(60);
  let updated_row = McpServerRow {
    url: "https://mcp-updated.example.com".to_string(),
    name: "Updated Server".to_string(),
    description: Some("Updated description".to_string()),
    enabled: false,
    updated_by: "admin2".to_string(),
    updated_at,
    ..row
  };
  let updated = ctx.service.update_mcp_server(&updated_row).await?;
  assert_eq!("https://mcp-updated.example.com", updated.url);
  assert_eq!("Updated Server", updated.name);
  assert_eq!(false, updated.enabled);
  assert_eq!(updated_at, updated.updated_at);

  let fetched = ctx.service.get_mcp_server("s1").await?.unwrap();
  assert_eq!("https://mcp-updated.example.com", fetched.url);
  assert_eq!(false, fetched.enabled);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_server_by_url(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  let row = McpServerRow {
    id: "s1".to_string(),
    url: "https://mcp.example.com/api".to_string(),
    name: "Server s1".to_string(),
    description: Some("A test server".to_string()),
    enabled: true,
    created_by: "admin".to_string(),
    updated_by: "admin".to_string(),
    created_at: ctx.now,
    updated_at: ctx.now,
  };
  ctx.service.create_mcp_server(&row).await?;

  let found = ctx
    .service
    .get_mcp_server_by_url("https://mcp.example.com/api")
    .await?;
  assert!(found.is_some());
  assert_eq!("s1", found.unwrap().id);

  let not_found = ctx
    .service
    .get_mcp_server_by_url("https://other.example.com")
    .await?;
  assert_eq!(None, not_found);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_mcp_servers(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&McpServerRow {
      id: "s1".to_string(),
      url: "https://one.example.com".to_string(),
      name: "Server s1".to_string(),
      description: Some("A test server".to_string()),
      enabled: true,
      created_by: "admin".to_string(),
      updated_by: "admin".to_string(),
      created_at: ctx.now,
      updated_at: ctx.now,
    })
    .await?;
  ctx
    .service
    .create_mcp_server(&McpServerRow {
      id: "s2".to_string(),
      url: "https://two.example.com".to_string(),
      name: "Server s2".to_string(),
      description: Some("A test server".to_string()),
      enabled: false,
      created_by: "admin".to_string(),
      updated_by: "admin".to_string(),
      created_at: ctx.now,
      updated_at: ctx.now,
    })
    .await?;

  let all = ctx.service.list_mcp_servers(None).await?;
  assert_eq!(2, all.len());

  let enabled = ctx.service.list_mcp_servers(Some(true)).await?;
  assert_eq!(1, enabled.len());
  assert_eq!("s1", enabled[0].id);

  let disabled = ctx.service.list_mcp_servers(Some(false)).await?;
  assert_eq!(1, disabled.len());
  assert_eq!("s2", disabled[0].id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_count_mcps_by_server_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  use crate::mcps::McpAuthType;
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;

  // Create 2 enabled + 1 disabled
  ctx
    .service
    .create_mcp(&make_mcp("m1", "s1", "mcp-one", "user-1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp(&make_mcp("m2", "s1", "mcp-two", "user-1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp(&McpRow {
      enabled: false,
      auth_type: McpAuthType::Public,
      ..make_mcp("m3", "s1", "mcp-three", "user-1", ctx.now)
    })
    .await?;

  let (enabled, disabled) = ctx.service.count_mcps_by_server_id("s1").await?;
  assert_eq!(2, enabled);
  assert_eq!(1, disabled);

  // Non-existent server returns (0, 0)
  let (e, d) = ctx.service.count_mcps_by_server_id("nonexistent").await?;
  assert_eq!(0, e);
  assert_eq!(0, d);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_clear_mcp_tools_by_server_id(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp_server(&make_server("s2", "https://other.example.com", ctx.now))
    .await?;

  // Create MCPs with tools on both servers
  ctx
    .service
    .create_mcp(&McpRow {
      tools_cache: Some("{\"tools\":[]}".to_string()),
      tools_filter: Some("[\"tool1\"]".to_string()),
      ..make_mcp("m1", "s1", "mcp-one", "user-1", ctx.now)
    })
    .await?;
  ctx
    .service
    .create_mcp(&McpRow {
      tools_cache: Some("{\"tools\":[\"t2\"]}".to_string()),
      ..make_mcp("m2", "s1", "mcp-two", "user-1", ctx.now)
    })
    .await?;
  ctx
    .service
    .create_mcp(&McpRow {
      tools_cache: Some("{\"other\":[]}".to_string()),
      ..make_mcp("m3", "s2", "mcp-three", "user-1", ctx.now)
    })
    .await?;

  let affected = ctx.service.clear_mcp_tools_by_server_id("s1").await?;
  assert_eq!(2, affected);

  // Verify s1 MCPs cleared
  let m1 = ctx.service.get_mcp("user-1", "m1").await?.unwrap();
  assert_eq!(None, m1.tools_cache);
  assert_eq!(None, m1.tools_filter);

  let m2 = ctx.service.get_mcp("user-1", "m2").await?.unwrap();
  assert_eq!(None, m2.tools_cache);

  // Verify s2 MCP untouched
  let m3 = ctx.service.get_mcp("user-1", "m3").await?.unwrap();
  assert_eq!(Some("{\"other\":[]}".to_string()), m3.tools_cache);
  Ok(())
}
