# Fixtures & Service Setup

## build_test_router()

The primary fixture for auth tier and integration tests. Returns a fully-composed router with all middleware layers (auth, session, CORS). Use for both auth tier tests and integration-style handler tests that need real middleware.

```rust
use routes_app::test_utils::build_test_router;

let (router, app_service, _temp) = build_test_router().await?;
```

Returns:
- `Router` -- fully composed with `build_routes()`, session layer, auth middleware
- `Arc<dyn AppService>` -- for accessing services to seed data or create sessions
- `Arc<TempDir>` -- keeps temp directory alive for test duration

### Services Wired

| Service | Implementation | Notes |
|---------|---------------|-------|
| DbService | TestDbService (real SQLite) | Safe to call |
| SessionService | SqliteSessionService | Safe to call |
| SecretService | SecretServiceStub | In-memory, AppRegInfo set |
| HubService | OfflineHubService | Downloads fail |
| DataService | LocalDataService | Real, file-based |
| AuthService | MockAuthService | **Panics** if called |
| ToolService | MockToolService | **Panics** if called |
| SharedContext | MockSharedContext | **Panics** if called |

### Creating Authenticated Sessions

```rust
use routes_app::test_utils::create_authenticated_session;

let cookie = create_authenticated_session(
  app_service.session_service().as_ref(),
  &["resource_user"],  // or &["resource_admin"], &["resource_manager"], etc.
).await?;
```

Creates a JWT with specified roles, stores it in the session store, returns a cookie string.

### Data Seeding via Service Handles

```rust
// Seed aliases via DataService
let data_service = app_service.data_service();
// data_service.create_alias(...).await?;

// Seed tokens via DbService
let db_service = app_service.db_service();
// db_service.create_api_token(&mut token).await?;

// Access session store
let session_service = app_service.session_service();
```

## AppServiceStubBuilder

For isolated handler tests with specific mock expectations. Use when you need to control individual service behavior with mocks.

### Minimal (all mocks)

```rust
let app_service = AppServiceStubBuilder::default().build()?;
```

### With real DataService

```rust
let app_service = AppServiceStubBuilder::default()
  .with_data_service().await
  .build()?;
```

### With real DbService

```rust
let app_service = AppServiceStubBuilder::default()
  .db_service(Arc::new(db_service))
  .build()?;
```

### With injected mock service

```rust
let app_service = AppServiceStubBuilder::default()
  .with_tool_service(Arc::new(mock_tool_service))
  .build()?;
```

### Composite (multiple real services)

```rust
let app_service = AppServiceStubBuilder::default()
  .with_hub_service()
  .with_data_service().await
  .db_service(Arc::new(test_db_service))
  .with_session_service().await
  .with_secret_service()
  .to_owned()
  .build()?;
```

## Router Construction for Isolated Handler Tests

Always wrap state in `Arc<DefaultRouterState>` and register only the handler(s) under test.

```rust
let state = Arc::new(DefaultRouterState::new(
  Arc::new(MockSharedContext::default()),
  Arc::new(app_service),
));
let router = Router::new()
  .route("/v1/chat/completions", post(chat_completions_handler))
  .with_state(state);
```

## DB Fixtures

### rstest async fixture

```rust
async fn test_handler(
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
) -> anyhow::Result<()> {
  let db_service = Arc::new(db_service);
  db_service.create_api_token(&mut token).await?;
  let app_service = AppServiceStubBuilder::default()
    .db_service(db_service.clone())
    .build()?;
  // ... test ...
}
```

Requires `#[awt]` on the test because of `#[future]`.

## Mock Service Injection

### MockToolService

```rust
let mut mock = MockToolService::new();
mock.expect_list()
  .withf(|user_id| user_id == "user123")
  .times(1)
  .returning(move |_| Ok(vec![instance.clone()]));
```

### MockSharedContext (LLM forwarding)

```rust
let mut ctx = MockSharedContext::default();
ctx.expect_forward_request()
  .with(eq(LlmEndpoint::ChatCompletions), eq(request_value), eq(alias))
  .times(1)
  .return_once(move |_, _, _| Ok(non_streamed_response()));
```

### MockAuthService

```rust
let mut mock_auth = MockAuthService::default();
mock_auth.expect_exchange_auth_code()
  .times(1)
  .return_once(move |_, _, _, _, _| {
    Ok((AccessToken::new(token), RefreshToken::new("refresh".to_string())))
  });
```
