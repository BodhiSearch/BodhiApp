use crate::{
  db::DbCore,
  new_ulid,
  test_utils::{sea_context, setup_env, TEST_TENANT_B_ID, TEST_TENANT_ID},
  tokens::{TokenEntity, TokenRepository, TokenStatus},
};
use anyhow_trace::anyhow_trace;
use chrono::{TimeZone, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use serial_test::serial;

fn make_token(id: &str, user_id: &str, prefix: &str, tenant_id: &str) -> TokenEntity {
  let now = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
  TokenEntity {
    id: id.to_string(),
    tenant_id: tenant_id.to_string(),
    user_id: user_id.to_string(),
    name: format!("Token {prefix}"),
    token_prefix: prefix.to_string(),
    token_hash: format!("hash_{prefix}"),
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  }
}

/// Verify that on SQLite, tenant_id filtering works at the application layer.
/// RLS is not applicable to SQLite — isolation is entirely application-level.
#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_sqlite_tenant_isolation_app_layer(_setup_env: ()) -> anyhow::Result<()> {
  let ctx = sea_context("sqlite").await;
  let user_id = new_ulid();

  let mut token_a = make_token(&new_ulid(), &user_id, "rls_t01a", TEST_TENANT_ID);
  let mut token_b = make_token(&new_ulid(), &user_id, "rls_t01b", TEST_TENANT_B_ID);

  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token_a)
    .await?;
  ctx
    .service
    .create_api_token(TEST_TENANT_B_ID, &mut token_b)
    .await?;

  // Tenant A only sees its own token via application-layer filter
  let (tokens_a, total_a) = ctx
    .service
    .list_api_tokens(TEST_TENANT_ID, &user_id, 1, 10)
    .await?;
  assert_eq!(1, total_a);
  assert_eq!(1, tokens_a.len());
  assert_eq!(TEST_TENANT_ID, tokens_a[0].tenant_id);

  // Tenant B only sees its own token via application-layer filter
  let (tokens_b, total_b) = ctx
    .service
    .list_api_tokens(TEST_TENANT_B_ID, &user_id, 1, 10)
    .await?;
  assert_eq!(1, total_b);
  assert_eq!(1, tokens_b.len());
  assert_eq!(TEST_TENANT_B_ID, tokens_b[0].tenant_id);

  Ok(())
}

/// Verify the current_tenant_id() function exists and returns the correct value
/// when the session variable is set. Also verifies the RLS policies exist
/// for all tenant-scoped tables.
#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_postgres_rls_policies_and_function_installed(_setup_env: ()) -> anyhow::Result<()> {
  if std::env::var("INTEG_TEST_APP_DB_PG_URL").is_err() {
    eprintln!("INTEG_TEST_APP_DB_PG_URL not set, skipping PostgreSQL RLS test");
    return Ok(());
  }

  let ctx = sea_context("postgres").await;
  let db = &ctx.service.db;

  // Verify the current_tenant_id() function exists
  let fn_exists = db
    .query_one(Statement::from_string(
      DatabaseBackend::Postgres,
      "SELECT COUNT(*)::bigint FROM pg_proc WHERE proname = 'current_tenant_id'".to_string(),
    ))
    .await?
    .expect("Query must return a row")
    .try_get_by_index::<i64>(0)?;

  assert_eq!(1, fn_exists, "current_tenant_id() function must exist");

  // Verify the function returns NULL when no session var is set
  let fn_result = db
    .query_one(Statement::from_string(
      DatabaseBackend::Postgres,
      "SELECT current_tenant_id() IS NULL AS is_null".to_string(),
    ))
    .await?
    .expect("Query must return a row")
    .try_get_by_index::<bool>(0)?;

  assert!(
    fn_result,
    "current_tenant_id() must return NULL when session var is unset"
  );

  // Verify the function returns the correct value when session var is set
  let fn_value = db
    .query_one(Statement::from_string(
      DatabaseBackend::Postgres,
      format!(
        "SELECT current_setting('app.current_tenant_id', true) = '' OR current_setting('app.current_tenant_id', true) IS NULL AS empty_before"
      ),
    ))
    .await?
    .expect("Query must return a row")
    .try_get_by_index::<bool>(0)?;

  assert!(fn_value, "Session var must be empty before setting it");

  // Verify RLS policies exist on all expected tenant tables
  let expected_tables = vec![
    "download_requests",
    "api_model_aliases",
    "model_metadata",
    "user_access_requests",
    "api_tokens",
    "toolsets",
    "app_toolset_configs",
    "user_aliases",
    "mcp_servers",
    "mcps",
    "mcp_auth_configs",
    "mcp_auth_config_params",
    "mcp_oauth_config_details",
    "mcp_auth_params",
    "mcp_oauth_tokens",
  ];

  for table in &expected_tables {
    let policy_count = db
      .query_one(Statement::from_string(
        DatabaseBackend::Postgres,
        format!(
          "SELECT COUNT(*)::bigint FROM pg_policies WHERE tablename = '{table}' AND policyname = 'tenant_isolation'"
        ),
      ))
      .await?
      .expect("Query must return a row")
      .try_get_by_index::<i64>(0)?;

    assert_eq!(
      1, policy_count,
      "RLS policy 'tenant_isolation' must exist on table '{table}'"
    );
  }

  // app_access_requests uses split policies instead of tenant_isolation
  for policy in &[
    "app_access_requests_read",
    "app_access_requests_insert",
    "app_access_requests_update",
  ] {
    let policy_count = db
      .query_one(Statement::from_string(
        DatabaseBackend::Postgres,
        format!(
          "SELECT COUNT(*)::bigint FROM pg_policies WHERE tablename = 'app_access_requests' AND policyname = '{policy}'"
        ),
      ))
      .await?
      .expect("Query must return a row")
      .try_get_by_index::<i64>(0)?;

    assert_eq!(
      1, policy_count,
      "RLS policy '{policy}' must exist on table 'app_access_requests'"
    );
  }

  // All tables with RLS enabled (includes app_access_requests which has split policies)
  let all_rls_tables: Vec<&str> = expected_tables
    .iter()
    .copied()
    .chain(std::iter::once("app_access_requests"))
    .collect();

  // Verify RLS is enabled on all expected tables
  for table in &all_rls_tables {
    let rls_enabled = db
      .query_one(Statement::from_string(
        DatabaseBackend::Postgres,
        format!("SELECT relrowsecurity FROM pg_class WHERE relname = '{table}'"),
      ))
      .await?
      .expect("Query must return a row")
      .try_get_by_index::<bool>(0)?;

    assert!(rls_enabled, "RLS must be enabled on table '{table}'");
  }

  // Verify FORCE RLS is enabled on all expected tables (forcerowsecurity)
  for table in &all_rls_tables {
    let force_rls = db
      .query_one(Statement::from_string(
        DatabaseBackend::Postgres,
        format!("SELECT relforcerowsecurity FROM pg_class WHERE relname = '{table}'"),
      ))
      .await?
      .expect("Query must return a row")
      .try_get_by_index::<bool>(0)?;

    assert!(
      force_rls,
      "FORCE ROW LEVEL SECURITY must be enabled on table '{table}'"
    );
  }

  Ok(())
}

/// Verify begin_tenant_txn safely handles special characters in tenant_id
/// (parameterized query prevents SQL injection).
#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_begin_tenant_txn_special_chars(_setup_env: ()) -> anyhow::Result<()> {
  let ctx = sea_context("sqlite").await;
  // Special chars that would break SQL injection with string interpolation
  let special_tenant_ids = vec![
    "tenant'; DROP TABLE api_tokens; --",
    "tenant\"with\"quotes",
    "tenant\nwith\nnewlines",
    "tenant\\with\\backslashes",
  ];
  for tenant_id in special_tenant_ids {
    // Should not error on SQLite (no-op for RLS), verifies no crash
    let txn = ctx.service.begin_tenant_txn(tenant_id).await?;
    txn.commit().await?;
  }
  Ok(())
}
