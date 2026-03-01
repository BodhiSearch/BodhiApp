use crate::mcps::{
  McpAuthRepository, McpAuthType, McpInstanceRepository, McpRow, McpServerRepository,
};
use crate::test_utils::{sea_context, setup_env};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;

use crate::mcps::test_helpers::{make_auth_header_row, make_mcp, make_server};

// ============================================================================
// MCP Instance Tests
// ============================================================================

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_create_and_get_mcp(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;

  let mcp = make_mcp("m1", "s1", "my-mcp", "user-1", ctx.now);
  let created = ctx.service.create_mcp(&mcp).await?;
  assert_eq!("m1", created.id);
  assert_eq!("my-mcp", created.slug);

  let fetched = ctx.service.get_mcp("user-1", "m1").await?;
  assert!(fetched.is_some());
  let fetched = fetched.unwrap();
  assert_eq!("m1", fetched.id);
  assert_eq!("user-1", fetched.created_by);
  assert_eq!("s1", fetched.mcp_server_id);
  assert_eq!(ctx.now, fetched.created_at);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_wrong_user(
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
    .create_mcp(&make_mcp("m1", "s1", "my-mcp", "user-1", ctx.now))
    .await?;

  let result = ctx.service.get_mcp("user-2", "m1").await?;
  assert_eq!(None, result);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_get_mcp_by_slug(
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
    .create_mcp(&make_mcp("m1", "s1", "my-mcp-slug", "user-1", ctx.now))
    .await?;

  let found = ctx.service.get_mcp_by_slug("user-1", "my-mcp-slug").await?;
  assert!(found.is_some());
  assert_eq!("m1", found.unwrap().id);

  let wrong_user = ctx.service.get_mcp_by_slug("user-2", "my-mcp-slug").await?;
  assert_eq!(None, wrong_user);

  let wrong_slug = ctx
    .service
    .get_mcp_by_slug("user-1", "no-such-slug")
    .await?;
  assert_eq!(None, wrong_slug);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_update_mcp(
  _setup_env: (),
  #[values("sqlite", "postgres")] db_type: &str,
) -> anyhow::Result<()> {
  let ctx = sea_context(db_type).await;
  ctx
    .service
    .create_mcp_server(&make_server("s1", "https://mcp.example.com", ctx.now))
    .await?;
  let mcp = make_mcp("m1", "s1", "my-mcp", "user-1", ctx.now);
  ctx.service.create_mcp(&mcp).await?;

  let updated_at = ctx.now + chrono::Duration::seconds(30);
  let updated = McpRow {
    name: "Updated MCP".to_string(),
    slug: "updated-mcp".to_string(),
    description: Some("Now with description".to_string()),
    enabled: false,
    tools_cache: Some("{\"tools\":[]}".to_string()),
    tools_filter: Some("[\"tool1\"]".to_string()),
    auth_type: McpAuthType::Header,
    auth_uuid: Some("auth-1".to_string()),
    updated_at,
    ..mcp
  };
  let result = ctx.service.update_mcp(&updated).await?;
  assert_eq!("Updated MCP", result.name);
  assert_eq!("updated-mcp", result.slug);
  assert_eq!(false, result.enabled);
  assert_eq!(Some("{\"tools\":[]}".to_string()), result.tools_cache);
  assert_eq!(updated_at, result.updated_at);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_mcp(
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
    .create_mcp(&make_mcp("m1", "s1", "my-mcp", "user-1", ctx.now))
    .await?;

  ctx.service.delete_mcp("user-1", "m1").await?;
  let gone = ctx.service.get_mcp("user-1", "m1").await?;
  assert_eq!(None, gone);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_delete_mcp_wrong_user_noop(
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
    .create_mcp(&make_mcp("m1", "s1", "my-mcp", "user-1", ctx.now))
    .await?;

  // delete by wrong user should be a no-op
  ctx.service.delete_mcp("user-2", "m1").await?;
  let still_exists = ctx.service.get_mcp("user-1", "m1").await?;
  assert!(still_exists.is_some());
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_mcps_with_server(
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
    .create_mcp(&make_mcp("m1", "s1", "mcp-one", "user-1", ctx.now))
    .await?;
  ctx
    .service
    .create_mcp(&make_mcp("m2", "s1", "mcp-two", "user-1", ctx.now))
    .await?;
  // Different user — should not appear
  ctx
    .service
    .create_mcp(&make_mcp("m3", "s1", "mcp-three", "user-2", ctx.now))
    .await?;

  let results = ctx.service.list_mcps_with_server("user-1").await?;
  assert_eq!(2, results.len());
  assert_eq!("https://mcp.example.com", results[0].server_url);
  assert_eq!("Server s1", results[0].server_name);
  assert_eq!(true, results[0].server_enabled);
  Ok(())
}

#[rstest]
#[tokio::test]
#[serial(pg_app)]
#[anyhow_trace]
async fn test_list_mcps_with_server_with_auth(
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
    .create_mcp_auth_header(&make_auth_header_row("ah-1", "s1", ctx.now))
    .await?;

  let mcp = McpRow {
    auth_type: McpAuthType::Header,
    auth_uuid: Some("ah-1".to_string()),
    ..make_mcp("m1", "s1", "mcp-with-auth", "user-1", ctx.now)
  };
  ctx.service.create_mcp(&mcp).await?;

  let results = ctx.service.list_mcps_with_server("user-1").await?;
  assert_eq!(1, results.len());
  assert_eq!(McpAuthType::Header, results[0].auth_type);
  assert_eq!(Some("ah-1".to_string()), results[0].auth_uuid);
  Ok(())
}
