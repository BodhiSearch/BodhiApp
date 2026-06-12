# Fix: test_update_alias_handler returning 500 instead of 200

## Context

The test `test_update_alias_handler` in `crates/routes_app/src/routes_models/test_aliases_crud.rs:307` fails with a 500 (Internal Server Error) instead of the expected 200. This is NOT a ULID issue -- the test uses hardcoded string IDs like `"test-tinyllama-instruct"` which work fine regardless of ULID migration.

## Root Cause

The `app_service_stub_builder` fixture in `crates/services/src/test_utils/app.rs:32-43` creates **two separate DB instances**:

```rust
AppServiceStubBuilder::default()
    .with_hub_service()
    .with_data_service()              // creates DB-A, seeds aliases into DB-A, creates LocalDataService with DB-A
    .await
    .db_service(Arc::new(test_db_service))  // REPLACES db_service with DB-B (empty, no aliases!)
    .with_session_service()
    .await
    .to_owned()
```

The `update_alias_handler` uses both services:
1. `data_service().get_user_alias_by_id()` -- reads from DB-A (has seeded aliases) -- succeeds
2. `db_service().update_user_alias()` -- writes to DB-B (empty) -- **fails because row doesn't exist**

SeaORM's `ActiveModel::update()` errors when the target row doesn't exist, which becomes a 500 InternalServer error.

## Fix

**File**: `crates/services/src/test_utils/app.rs:32-43`

Reorder the builder calls so `db_service` is set BEFORE `with_data_service()`. This way `with_data_service()` calls `get_db_service()`, finds the existing instance, seeds it, and creates `LocalDataService` using the same DB:

```rust
pub async fn app_service_stub_builder(
  #[future] test_db_service: TestDbService,
) -> AppServiceStubBuilder {
  AppServiceStubBuilder::default()
    .with_hub_service()
    .db_service(Arc::new(test_db_service))   // Set DB FIRST
    .with_data_service()                      // Seeds into the same DB, creates LocalDataService with it
    .await
    .with_session_service()
    .await
    .to_owned()
}
```

Now both `data_service()` and `db_service()` use the same DB instance, with seeded aliases present in both.

## Impact

12 files reference `app_service_stub` or `app_service_stub_builder`. This fix is safe because:
- No test should depend on `data_service` and `db_service` having **different** databases
- Tests that only use `data_service` are unaffected (same DB, same seed data)
- Tests that only use `db_service` now also have seeded aliases (no test should break from having extra rows)

## Verification

```bash
cargo test -p routes_app -- test_update_alias_handler
cargo test -p routes_app -- test_aliases_crud
```
