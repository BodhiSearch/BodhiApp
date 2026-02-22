# CLAUDE.md - services Crate

See [crates/services/PACKAGE.md](crates/services/PACKAGE.md) for implementation details.

## Purpose

The `services` crate implements BodhiApp's **business logic orchestration layer**, providing 16 interconnected services that coordinate OAuth2 authentication, AI API integrations, model management, toolset execution, MCP server management, user access control, data persistence, concurrency control, and multi-layer security. This crate bridges domain objects from `objs` with external systems while maintaining deployment flexibility across standalone servers, desktop applications, and embedded contexts.

## Architecture Position

**Upstream dependencies** (crates this depends on):
- [`objs`](../objs/CLAUDE.md) -- domain objects, error types, `IoError`, `impl_error_from!` macro
- [`server_core`](../server_core/CLAUDE.md) -- `SharedContext`, `RouterState` HTTP infrastructure
- [`llama_server_proc`](../llama_server_proc/CLAUDE.md) -- LLM process management
- [`mcp_client`](../mcp_client/) -- MCP protocol client for tool discovery and execution
- [`errmeta_derive`](../errmeta_derive/CLAUDE.md) -- `#[derive(ErrorMeta)]` proc macro

**Downstream consumers** (crates that depend on this):
- [`routes_app`](../routes_app/CLAUDE.md) -- HTTP route handlers consume service traits
- [`server_app`](../server_app/CLAUDE.md) -- standalone server bootstraps `DefaultAppService`
- [`lib_bodhiserver`](../lib_bodhiserver/CLAUDE.md) -- embeddable library bootstraps `DefaultAppService`
- [`bodhi/src-tauri`](../bodhi/src-tauri/CLAUDE.md) -- Tauri desktop app bootstraps services

## Architectural Design Rationale

### Why Service Registry Pattern

BodhiApp chose a trait-based service registry pattern over traditional dependency injection frameworks for several critical reasons:

1. **Compile-Time Safety**: The `AppService` trait ensures all service dependencies are satisfied at compile time, preventing runtime surprises in production deployments
2. **Testing Isolation**: Each service trait has `#[mockall::automock]` annotations enabling comprehensive mock generation for unit testing without external dependencies
3. **Deployment Flexibility**: The registry pattern allows different service implementations across deployment contexts (server vs desktop vs embedded) without architectural changes
4. **Thread-Safe Concurrency**: All services implement `Send + Sync + Debug`, enabling safe sharing across async tasks and worker threads
5. **Explicit Dependencies**: Service constructors explicitly declare dependencies through the `derive-new` pattern, making the dependency graph clear and maintainable

### Why Multi-Layer Authentication

The authentication system spans multiple services rather than a monolithic auth module because:

1. **Separation of Concerns**: Each service handles a specific aspect - OAuth2 flows (AuthService), credential encryption (SecretService), session management (SessionService), and persistent storage (KeyringService)
2. **Platform Abstraction**: KeyringService abstracts OS-specific credential storage (Keychain, Secret Service, Windows Credential Manager) behind a unified interface
3. **Security Defense in Depth**: Multiple layers ensure credentials are never exposed - encrypted in database, protected by platform keyring, and transmitted via secure sessions
4. **Token Lifecycle Management**: Complex JWT refresh logic, token exchange protocols, and session coordination require specialized handling across service boundaries
5. **Keycloak Integration**: Custom Bodhi API endpoints for resource management and dynamic admin assignment require coordinated service interactions

### Why Separated Model Management Services

Model management is split between HubService and DataService rather than a single service because:

1. **External vs Local Concerns**: HubService handles Hugging Face API interactions while DataService manages local file system operations
2. **Offline Capability**: DataService can operate without network access, enabling offline model usage and testing
3. **Error Recovery Boundaries**: Network failures in HubService don't affect local model operations in DataService
4. **Cache Coherency**: Separation allows independent caching strategies - API responses in HubService, file metadata in DataService
5. **Testing Isolation**: OfflineHubService enables testing without external API dependencies while DataService tests focus on file operations

## Error Handling Architecture

### Consolidated IO Error Pattern

The services crate uses the unified `IoError` enum from `objs` for all IO-related error handling. This represents a deliberate consolidation from the previous approach of 6 separate IO error structs (`IoError` struct, `IoWithPathError`, `IoDirCreateError`, `IoFileReadError`, `IoFileWriteError`, `IoFileDeleteError`) into a single enum with 6 variants:

**Why a Single IoError Enum**:
- **Simplified Error Propagation**: Service error enums (e.g., `DataServiceError`, `HubServiceError`, `SecretServiceError`, `SettingServiceError`) previously needed multiple IO-related variants for each struct. Now each has a single `Io(#[from] IoError)` or `IoError(#[from] IoError)` variant
- **Ergonomic Construction**: Convenience constructors like `IoError::file_read(err, path)`, `IoError::file_write(err, path)`, `IoError::dir_create(err, path)`, and `IoError::file_delete(err, path)` provide clear, context-specific error creation without separate struct constructors
- **Consistent Pattern Matching**: Consumers can match on a single enum rather than checking multiple error types
- **Preserved Context**: Each variant captures the `std::io::Error` source and relevant path, maintaining full diagnostic information

**Error Flow Through Services**:
- Raw `std::io::Error` values are converted to `IoError` variants with path context using convenience constructors
- The `impl_error_from!` macro generates `From<std::io::Error>` implementations that wrap into the service's error enum via the `IoError` intermediate type (e.g., `impl_error_from!(::std::io::Error, DataServiceError::Io, ::objs::IoError)`)
- Service error enums propagate transparently through `#[from]` derives on the `IoError` variant

**Service Error Enums Using IoError**:
- `DataServiceError::Io(IoError)` - file read/write/delete operations for alias and model management
- `HubServiceError::IoError(IoError)` - filesystem operations during model cache operations
- `SecretServiceError::IoError(IoError)` - file operations for encrypted secret storage
- `SettingServiceError::Io(IoError)` - configuration file read/write operations

### Error Translation Chain

- Service method returns domain-specific error (e.g., `HubServiceError::HubApiError(GatedAccess)`)
- Error implements `AppError` trait via `errmeta_derive`
- `RouterStateError` wraps service error with HTTP context
- Error converts to `ApiError` with OpenAI-compatible format
- Response includes user-friendly error message from thiserror template

### impl_error_from! Macro Pattern

The `impl_error_from!` macro from `objs` generates two-step `From` conversions for external library errors. This avoids Rust's orphan rule by going through an intermediate wrapper type:

```
std::io::Error -> IoError (via IoError::from) -> DataServiceError::Io (via #[from])
```

The macro signature is: `impl_error_from!(source_type, target_enum::variant, intermediate_type)`

This pattern is used consistently across all service error enums for external errors like `std::io::Error`, `reqwest::Error`, `serde_yaml::Error`, `serde_json::Error`, and `sqlx::Error`.

## Cross-Crate Coordination Patterns

### Service to HTTP Infrastructure Flow

Services integrate with HTTP infrastructure through specific coordination points:

**Model Resolution Pipeline**:
- HTTP request arrives at routes_oai chat completion endpoint
- Route handler queries DataService.find_alias() for model resolution
- DataService returns Alias object (User, Model, or Api variant) with request parameter overlays
- SharedContext uses HubService.find_local_file() to locate GGUF file
- LLM server process launched with resolved model path

**Alias Resolution Priority**:
1. User aliases (from YAML configuration files)
2. Model aliases (auto-discovered local GGUF files)
3. API aliases (from database, with prefix-aware routing via `supports_model()`)

### Service to Authentication Middleware Coordination

Authentication flows span services and middleware with precise coordination:

**Token Exchange Flow**:
1. External client presents access token to auth_middleware
2. Middleware queries AuthService for token validation
3. AuthService checks DbService for cached validation result
4. If expired, AuthService initiates RFC 8693 token exchange with Keycloak
5. New token stored in DbService with expiration tracking
6. SessionService creates/updates HTTP session with user context
7. Middleware attaches authenticated user to request extensions

**Concurrency Control for Token Refresh**:
- ConcurrencyService provides `with_lock_auth()` to prevent duplicate token refresh operations
- Per-key locking (e.g., by session ID) allows concurrent refreshes for different users
- LocalConcurrencyService uses `RwLock<HashMap<String, Arc<Mutex<()>>>>` for in-process coordination

### Service to CLI Integration Patterns

Services adapt to CLI context through specialized error handling:

- Service errors bubble up to commands crate
- CLI-specific formatting applied (no JSON envelopes)
- Progress bars integrate with HubService download tracking
- Interactive prompts use DataService for model selection
- Configuration updates through SettingService persist across sessions

## Domain-Specific Architecture Patterns

### OAuth2 with Dynamic Client Registration

BodhiApp implements a sophisticated OAuth2 flow with runtime client registration:

**Why Dynamic Registration**:
- Eliminates pre-shared client credentials reducing deployment complexity
- Enables per-installation client isolation for security
- Supports custom Bodhi API endpoints for resource administration
- Allows runtime realm configuration without rebuild

**Registration Sequence**:
1. AuthService detects missing client configuration
2. Registers new OAuth client with Keycloak using Bodhi API
3. Stores encrypted client credentials via SecretService
4. Persists registration metadata in platform keyring
5. Subsequent requests use cached credentials

### Toolset and External API Integration Architecture

The `ToolService` and `ExaService` represent the extensibility pattern for external service integrations:

**ToolService Design** (module: `tool_service/`):
- Manages toolset definitions (function calling schemas) for LLM integrations
- Dual-level configuration: app-level admin enable/disable (`AppToolsetConfig`) and per-user enable/disable
- Toolset type system with `ToolsetScope` for type validation and built-in toolset registration
- Built-in toolset: `builtin-exa-search` (Exa search, findSimilar, contents, answer)
- Toolset execution delegates to specialized services (e.g., ExaService for web search)
- Terminology: `slug` (unique identifier), `toolset_type` (type classification), `name` (display name)

**ExaService Design**:
- Isolated external API client for Exa AI semantic search
- Timeout-based request management (30 second default)
- Error classification separates auth failures, rate limits, and timeouts

### MCP Server Management Architecture

The `McpService` (module: `mcp_service/`) manages Model Context Protocol server integrations:

**McpService Design**:
- CRUD operations for MCP server instances with slug-based identification
- Server allowlist management: `is_url_enabled`, `set_mcp_server_enabled`, `list_mcp_servers`, `get_mcp_server_by_url`
- Auth config management: `list_auth_headers_by_server(mcp_server_id)` returns auth headers for a given server. Auth header creation requires `name` and `mcp_server_id`; OAuth config creation requires `name`. Auth configs are admin-managed per server; users select from existing configs when creating MCP instances.
- Auth header preservation: When an MCP instance switches auth type away from `Header`, the auth header is **not** deleted. Auth headers are admin-managed resources that can be reused by other instances. OAuth tokens **are** cleaned up on type switch since they are per-user.
- Tool discovery via `fetch_tools` and execution via `execute` delegating to `mcp_client` crate
- Admin enable flow: new MCP URLs require explicit admin approval before tools can be fetched
- Error types: `McpError` with variants for not-found, URL not allowed, disabled, tool-specific errors, connection/execution failures
- OAuth token refresh has per-key concurrency guard (Mutex-based, keyed by `oauth_refresh:{config_id}`)
- Ownership checks: `get_mcp_auth_header`, `delete_mcp_auth_header`, `get_mcp_oauth_token`, `delete_mcp_oauth_token` require `user_id`
- `DefaultMcpService` shares a single `reqwest::Client` instance

**Dependencies**: `mcp_client` crate for MCP protocol communication, `DbService` for persistence, `TimeService` for timestamps

**Migration 0012**: Indexes on `mcp_oauth_configs(mcp_server_id)` and `mcp_oauth_tokens(mcp_oauth_config_id)`

### Access Request Management Architecture

The `AccessRequestService` (module: `access_request_service/`) handles user access control workflows:

- User access request creation, approval, and denial
- Status tracking (pending, approved, denied)
- Integration with `DbService` for persistence via `AccessRequestRepository`

### Queue-Based Metadata Extraction

The QueueService implements background processing for model metadata:

**Architecture**:
- Asynchronous queue with notify-based signaling
- Extracts GGUF metadata from local model files
- Stores metadata in database via DbService
- Graceful shutdown via AtomicBool flag
- Error handling allows partial failures without queue disruption

### Platform-Agnostic Credential Storage

The layered credential storage system ensures security across platforms:

**Storage Layers**:
1. **Database**: Encrypted credentials with AES-GCM
2. **Platform Keyring**: OS-specific secure storage (Keychain, Secret Service, Windows Credential Manager)
3. **Session Cookies**: Temporary authentication state
4. **Memory Cache**: Short-lived token cache

**Why This Layering**:
- Database encryption protects at-rest credentials
- Platform keyring leverages OS security features
- Session cookies enable stateless HTTP requests
- Memory cache reduces token validation overhead

## Critical Design Decisions

### Time Service Abstraction

All timestamp operations flow through TimeService rather than direct `Utc::now()` calls:

**Rationale**:
- Enables deterministic testing with FrozenTimeService
- Ensures consistent timestamp format across services
- Removes nanosecond precision for cross-platform compatibility
- Facilitates time-travel testing for token expiration

**Implementation Impact**:
- Service constructors accept TimeService parameter
- Database records use TimeService for created_at/updated_at
- Token validation checks expiration via TimeService
- Tests inject FrozenTimeService for reproducibility

### Settings Change Notification

SettingService implements a listener pattern for configuration changes:

**Design**:
- `SettingsChangeListener` trait notifies dependents of setting mutations
- Change events carry previous value/source and new value/source
- Settings sourced from multiple layers: System > CommandLine > Environment > User
- Persistent YAML-based settings file for user configuration

### Offline Testing with Service Stubs

Each external service has an offline stub implementation:

**OfflineHubService**: Returns predefined model metadata, simulates download progress, enables testing without Hugging Face API

**MockAuthService**: Provides deterministic token generation, simulates OAuth2 flows locally, enables auth testing without Keycloak

**Benefits**: Fast unit tests without network dependencies, deterministic test execution, reduced API rate limit consumption, simplified CI/CD pipeline

## Security Architecture Decisions

### Why PBKDF2 with 1000 Iterations

The SecretService uses PBKDF2 for key derivation with specific parameters:

- 1000 iterations balances security with performance for interactive operations
- Random salt per encryption prevents rainbow table attacks
- AES-GCM provides authenticated encryption detecting tampering
- Base64 encoding enables safe storage in text-based formats

### Session Security Configuration

HTTP sessions use specific security settings:

- SameSite::Strict prevents CSRF attacks
- SQLite backend enables horizontal scaling
- AppSessionStore wraps tower-sessions SqliteStore with user_id tracking
- Session clearing by user_id enables targeted session invalidation

## Testing Conventions

All tests in the services crate follow a standardized canonical pattern. For comprehensive reference with code examples, see the `.claude/skills/test-services/` skill directory.

### Test File Organization

For files where combined source + tests exceed ~500 lines, tests are extracted to `test_*.rs` sibling files using `#[cfg(test)] #[path = "test_<name>.rs"] mod test_<name>;`. Example: `queue_service.rs` declares `test_queue_service`. Inline `#[cfg(test)] mod tests {}` is used for smaller files.

### Canonical Test Pattern

- **Async tests**: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]`, return `-> anyhow::Result<()>`
- **Sync tests**: `#[rstest]` only (no `#[anyhow_trace]`), return `-> anyhow::Result<()>`
- **Database fixture tests**: Add `#[awt]` only when `#[future]` fixture params are used
- **Module naming**: Always `mod tests` (not `mod test`)

### Error Code Assertions

Assert error codes via `.code()` method, never error message text:

```rust
let err = result.unwrap_err();
assert_eq!("auth_service_error-auth_service_api_error", err.code());
```

Error codes are auto-generated as `enum_name-variant_name` in snake_case. **Important**: transparent errors delegate to the inner error code (e.g., `DbError::SqlxError` produces `"sqlx_error"`, not `"db_error-sqlx_error"`).

### Assertion Style

Use `assert_eq!(expected, actual)` with `use pretty_assertions::assert_eq;` in every test module.

### Key Test Infrastructure

- **TestDbService**: Real SQLite in temp directory with `FrozenTimeService` for deterministic timestamps, event broadcasting for operation verification. See `src/test_utils/db.rs`.
- **MockDbService**: Composite `mockall::mock!` implementation covering all repository traits. See `src/test_utils/db.rs`.
- **mockito::Server**: HTTP mock server for testing AuthService, AiApiService, ExaService. See `src/auth_service.rs`, `src/ai_api_service.rs`, `src/exa_service.rs`.
- **EnvWrapperStub**: In-memory environment variable stub for SettingService tests. See `src/test_utils/envs.rs`.
- **MockSettingsChangeListener**: Verifies setting change notifications with expectation-driven assertions.

### Skill Reference

For detailed patterns with full code examples:
- `.claude/skills/test-services/SKILL.md` -- Quick reference and migration checklist
- `.claude/skills/test-services/db-testing.md` -- TestDbService fixture, FrozenTimeService, real SQLite
- `.claude/skills/test-services/api-testing.md` -- mockito HTTP mocking patterns
- `.claude/skills/test-services/mock-patterns.md` -- mockall setup and MockDbService
- `.claude/skills/test-services/advanced.md` -- Concurrency, progress tracking, notifications, parameterized tests

## Extension Guidelines

### Adding New Services

When creating new services for the ecosystem:

1. **Define Service Trait**: Create trait with async methods and `#[mockall::automock]` annotation
2. **Implement Service**: Provide concrete implementation with proper error handling
3. **Add to Registry**: Extend `AppService` trait and `DefaultAppService` with new accessor
4. **Create Error Types**: Define domain-specific error enum with `AppError` implementation via `errmeta_derive`
5. **Use IoError for IO**: Wrap IO operations with `IoError` convenience constructors for path context
6. **Create Test Utils**: Add mock builders in test_utils module
7. **Document Dependencies**: Update service interdependency documentation

### Error Handling for New Services

When creating error types for new services:

1. **Define Domain Enum**: Create enum with domain-meaningful variants using `#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]`
2. **Use IoError for IO**: Add single `Io(#[from] IoError)` variant for all filesystem operations
3. **Bridge External Errors**: Use `impl_error_from!` macro for external library errors that need intermediate wrapper types
4. **Preserve Context**: Use convenience constructors (`IoError::file_read()`, `IoError::file_write()`, etc.) to capture path context
5. **Set Error Types**: Annotate each variant with appropriate `ErrorType` (BadRequest, NotFound, InternalServer, etc.)

### Adding External Integrations

When integrating new external services:

1. **Create Service Abstraction**: Hide external API behind trait
2. **Implement Offline Stub**: Enable testing without external dependency
3. **Add Error Classification**: Categorize failures with domain-specific error enum
4. **Implement Retry Logic**: Handle transient failures gracefully
5. **Document Rate Limits**: Specify any API constraints

## Service Initialization Order

Services must initialize in dependency order:
1. TimeService (no dependencies)
2. DbService (depends on TimeService)
3. SecretService (depends on file-based storage)
4. SettingService (depends on SecretService, environment)
5. AuthService (depends on above)
6. SessionService (depends on SQLite pool)
7. HubService, DataService, CacheService (depend on configuration)
8. ConcurrencyService, NetworkService (standalone)
9. AiApiService, ToolService, ExaService (depend on DbService)
10. McpService (depends on DbService, mcp_client, TimeService)
11. AccessRequestService (depends on DbService)
12. QueueProducer (depends on DataService, HubService, DbService)

## Thread Safety Requirements

All services must be thread-safe:
- Implement `Send + Sync + Debug`
- Use `Arc` for shared ownership
- Avoid interior mutability without synchronization
- Prefer immutable operations
- ConcurrencyService provides explicit locking for operations requiring coordination
