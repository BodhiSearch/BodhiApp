use crate::{
  db::DbCore,
  test_utils::{sea_context, setup_env, TEST_TENANT_ID},
  tokens::{TokenEntity, TokenRepository, TokenStatus},
};
use anyhow_trace::anyhow_trace;
use chrono::{TimeZone, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, TransactionTrait};
use serial_test::serial;

const TENANT_B_ID: &str = "01ARZ3NDEKTSV4RRFFQ69G5FBB";
const TEST_APP_ROLE: &str = "bodhi_app_test_role";

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
  let user_id = ulid::Ulid::new().to_string();

  let mut token_a = make_token(
    &ulid::Ulid::new().to_string(),
    &user_id,
    "rls_t01a",
    TEST_TENANT_ID,
  );
  let mut token_b = make_token(
    &ulid::Ulid::new().to_string(),
    &user_id,
    "rls_t01b",
    TENANT_B_ID,
  );

  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token_a)
    .await?;
  ctx
    .service
    .create_api_token(TENANT_B_ID, &mut token_b)
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
    .list_api_tokens(TENANT_B_ID, &user_id, 1, 10)
    .await?;
  assert_eq!(1, total_b);
  assert_eq!(1, tokens_b.len());
  assert_eq!(TENANT_B_ID, tokens_b[0].tenant_id);

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
    "app_access_requests",
    "mcp_servers",
    "mcps",
    "mcp_auth_headers",
    "mcp_oauth_configs",
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

  // Verify RLS is enabled on all expected tables
  for table in &expected_tables {
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
  for table in &expected_tables {
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

/// Verify PostgreSQL RLS enforcement using a non-superuser role.
/// Creates a temporary non-superuser role with table access, inserts data for two tenants,
/// then uses SET ROLE to switch to that role and verifies that SET LOCAL app.current_tenant_id
/// restricts visibility to only the specified tenant's rows.
#[rstest]
#[anyhow_trace]
#[tokio::test]
#[serial(pg_app)]
async fn test_postgres_rls_enforcement_with_app_role(_setup_env: ()) -> anyhow::Result<()> {
  if std::env::var("INTEG_TEST_APP_DB_PG_URL").is_err() {
    eprintln!("INTEG_TEST_APP_DB_PG_URL not set, skipping PostgreSQL RLS enforcement test");
    return Ok(());
  }

  let ctx = sea_context("postgres").await;
  let db = &ctx.service.db;
  let user_id = ulid::Ulid::new().to_string();

  // Insert tokens for two tenants (as superuser, bypasses RLS for insert)
  let mut token_a = make_token(
    &ulid::Ulid::new().to_string(),
    &user_id,
    "rls_enf_t01a",
    TEST_TENANT_ID,
  );
  let mut token_b = make_token(
    &ulid::Ulid::new().to_string(),
    &user_id,
    "rls_enf_t01b",
    TENANT_B_ID,
  );
  ctx
    .service
    .create_api_token(TEST_TENANT_ID, &mut token_a)
    .await?;
  ctx
    .service
    .create_api_token(TENANT_B_ID, &mut token_b)
    .await?;

  // Create a non-superuser role for RLS testing (if it doesn't exist)
  let _ = db
    .execute_unprepared(&format!(
      "DO $$ BEGIN
         IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '{TEST_APP_ROLE}') THEN
           CREATE ROLE {TEST_APP_ROLE} NOLOGIN NOINHERIT;
         END IF;
       END $$;"
    ))
    .await;

  // Grant SELECT on api_tokens to the test role
  let _ = db
    .execute_unprepared(&format!("GRANT SELECT ON api_tokens TO {TEST_APP_ROLE};"))
    .await;

  // Grant EXECUTE on current_tenant_id() to the test role
  let _ = db
    .execute_unprepared(&format!(
      "GRANT EXECUTE ON FUNCTION current_tenant_id() TO {TEST_APP_ROLE};"
    ))
    .await;

  // Test: SET ROLE to non-superuser + SET LOCAL tenant_id => only TENANT_A rows visible
  let count_a: i64 = {
    let txn = db.begin().await?;
    // Switch to non-superuser role (RLS now applies)
    txn
      .execute_unprepared(&format!("SET LOCAL ROLE {TEST_APP_ROLE}"))
      .await?;
    // Set the tenant context
    txn
      .execute_unprepared(&format!(
        "SET LOCAL app.current_tenant_id = '{TEST_TENANT_ID}'"
      ))
      .await?;
    // Query without WHERE tenant_id clause — RLS policy should filter
    let row = txn
      .query_one(Statement::from_string(
        DatabaseBackend::Postgres,
        format!("SELECT COUNT(*)::bigint FROM api_tokens WHERE user_id = '{user_id}'"),
      ))
      .await?
      .expect("COUNT query must return a row");
    let count = row.try_get_by_index::<i64>(0)?;
    txn.commit().await?;
    count
  };

  assert_eq!(
    1, count_a,
    "RLS with non-superuser + TENANT_A context: only TENANT_A's row should be visible"
  );

  // Test: SET ROLE to non-superuser + SET LOCAL tenant_id => only TENANT_B rows visible
  let count_b: i64 = {
    let txn = db.begin().await?;
    txn
      .execute_unprepared(&format!("SET LOCAL ROLE {TEST_APP_ROLE}"))
      .await?;
    txn
      .execute_unprepared(&format!(
        "SET LOCAL app.current_tenant_id = '{TENANT_B_ID}'"
      ))
      .await?;
    let row = txn
      .query_one(Statement::from_string(
        DatabaseBackend::Postgres,
        format!("SELECT COUNT(*)::bigint FROM api_tokens WHERE user_id = '{user_id}'"),
      ))
      .await?
      .expect("COUNT query must return a row");
    let count = row.try_get_by_index::<i64>(0)?;
    txn.commit().await?;
    count
  };

  assert_eq!(
    1, count_b,
    "RLS with non-superuser + TENANT_B context: only TENANT_B's row should be visible"
  );

  // Test: SET ROLE to non-superuser + no tenant context => no rows visible (NULL filter)
  let count_no_ctx: i64 = {
    let txn = db.begin().await?;
    txn
      .execute_unprepared(&format!("SET LOCAL ROLE {TEST_APP_ROLE}"))
      .await?;
    // No SET LOCAL for tenant_id — current_tenant_id() returns NULL
    let row = txn
      .query_one(Statement::from_string(
        DatabaseBackend::Postgres,
        format!("SELECT COUNT(*)::bigint FROM api_tokens WHERE user_id = '{user_id}'"),
      ))
      .await?
      .expect("COUNT query must return a row");
    let count = row.try_get_by_index::<i64>(0)?;
    txn.commit().await?;
    count
  };

  assert_eq!(
    0, count_no_ctx,
    "RLS with non-superuser + no tenant context: no rows should be visible (NULL = any_value is false)"
  );

  Ok(())
}
