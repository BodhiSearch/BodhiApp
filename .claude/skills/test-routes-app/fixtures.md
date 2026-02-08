# Fixtures & Service Setup

## AppServiceStubBuilder

The primary fixture builder. Provides mock defaults for all services; override only what the test needs.

### Minimal (all mocks)

```rust
let app_service = AppServiceStubBuilder::default().build()?;
```

### With real DataService (copies test model data into temp dir)

```rust
let app_service = AppServiceStubBuilder::default()
  .with_data_service().await
  .build()?;
```

### With real DbService (SQLite)

```rust
let app_service = AppServiceStubBuilder::default()
  .db_service(Arc::new(db_service))
  .build()?;
```

### With real SessionService

```rust
let app_service = AppServiceStubBuilder::default()
  .build_session_service(dbfile).await
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

## Router Construction

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

For multi-route modules that share state, extract a helper:

```rust
fn test_router(mock_service: MockToolService) -> anyhow::Result<Router> {
  let app_service = AppServiceStubBuilder::default()
    .with_tool_service(Arc::new(mock_service))
    .build()?;
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));
  Ok(routes_toolsets(state))
}
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
  // Seed data
  db_service.create_api_token(&mut token).await?;
  let app_service = AppServiceStubBuilder::default()
    .db_service(db_service.clone())
    .build()?;
  // ... test ...
}
```

Requires `#[awt]` on the test because of `#[future]`.

### Manual temp dir fixture

```rust
async fn test_handler(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let db_service = test_db_service_with_temp_dir(Arc::new(temp_bodhi_home)).await;
  // Seed and test ...
}
```

No `#[awt]` needed (sync fixture).

## Mock Service Injection

### MockToolService example

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
