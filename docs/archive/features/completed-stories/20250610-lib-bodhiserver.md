# App Setup Refactoring: `lib_bodhiserver` Technical Specification

## 1. Overview

This document specifies the technical plan for refactoring the application's initialization logic. The current approach suffers from significant code duplication between the main Tauri application (`crates/bodhi/src-tauri/src/lib_main.rs` and `app.rs`) and integration tests (`crates/integration-tests/tests/utils/live_server_utils.rs`), making maintenance difficult and hindering the creation of new deployment variants like CLI or Docker.

The solution is to create a new, platform-agnostic library crate, `lib_bodhiserver`, which will centralize all core server initialization logic while preserving the existing multi-phase initialization pattern.

### Primary Goals
1.  **Internal Health & Code Reduction:** Eliminate duplicated initialization code by creating a single source of truth for the complex service wiring logic currently duplicated between `app.rs:54-122` and `live_server_utils.rs:111-170`.
2.  **Developer Velocity & Simplified Testing:** Provide a clean, reusable interface that drastically simplifies the setup for integration tests, reducing the 160+ lines of setup code to a simple builder pattern.
3.  **Future Extensibility:** Establish a foundation that makes it easy to create new application frontends (e.g., a dedicated CLI binary, a Docker-based server) in the future.

### Out of Scope for this Initiative
*   Publishing the crate to `crates.io`.
*   Creating a C-style FFI interface.

### References
- @ai-docs/README.md - For AI coding assistant context about app architecture
- @ai-docs/01-architecture/README.md - For comprehensive architectural documentation

## 1.1. Codebase Analysis Findings

### Current Initialization Architecture

**File Analysis:**
- `crates/bodhi/src-tauri/src/lib_main.rs:60-169` - Main initialization entry point (109 lines)
- `crates/bodhi/src-tauri/src/app.rs:54-122` - Service composition and wiring (69 lines)
- `crates/integration-tests/tests/utils/live_server_utils.rs:42-171` - Test setup duplication (130 lines)
- `crates/services/src/init_service.rs:38-106` - Directory and environment setup utilities

**Multi-Phase Pattern Discovery:**
The codebase already implements a sophisticated 2-phase initialization pattern that should be preserved:

1. **Phase 1: Foundation Setup** (`lib_main.rs:60-153`)
   - Environment wrapper and `InitService` creation
   - Bodhi home directory discovery/creation
   - System settings construction with feature-specific overrides
   - Settings service initialization with defaults and environment loading
   - Directory structure creation (aliases, databases, logs, HF cache)

2. **Phase 2: Service Composition** (`app.rs:54-122`)
   - Database connections and migrations (app.db, session.db)
   - Service dependency resolution in correct order
   - Localization resource loading (8 different l10n resources)
   - Final `DefaultAppService` composition with 10+ dependencies

**Duplication Analysis:**
- **Total duplicated logic:** ~200 lines across production and test paths
- **Service wiring complexity:** 10+ service dependencies with specific initialization order
- **Test-specific variations:** OAuth configuration, temporary directories, test data management
- **Feature flag handling:** Production vs development vs native compilation differences

**Existing Patterns to Leverage:**
- `derive_builder` pattern established in `services/src/test_utils/app.rs:41-86`
- `test-utils` feature flag pattern documented in `ai-docs/01-architecture/backend-testing-utils.md`
- Error handling with `thiserror` and `ApiError` conversion patterns
- Service trait abstractions with `Arc<dyn Service>` composition

## 2. Architectural Design

The core principle of this refactoring is a strict separation of concerns between the platform-specific "caller" and the platform-agnostic "library", while preserving the existing multi-phase initialization pattern observed in `lib_main.rs`.

### 2.1. Multi-Phase Initialization Pattern

Based on analysis of `crates/bodhi/src-tauri/src/lib_main.rs:60-169`, the current initialization follows a clear 2-phase pattern:

**Phase 1: Environment & Home Setup** (`lib_main.rs:60-153`)
- Environment wrapper creation
- Bodhi home directory discovery/creation via `InitService::setup_bodhi_home_dir()`
- Settings service initialization with system defaults
- Feature-specific settings configuration
- Directory structure creation (aliases, databases, logs, HF cache)

**Phase 2: Service Initialization & Wiring** (`app.rs:54-122`)
- Database connection and migration
- Service instantiation (auth, data, hub, secret, cache, session)
- Localization resource loading
- Final `AppService` composition

### 2.2. Architectural Layers

```
┌─────────────────────────────────────────────────────────────┐
│                      Platform Layer                         │
│     (e.g., `bodhi` Tauri app, CLI binary, integration test)   │
│                                                             │
│  - Resolves paths (e.g., home directory)                    │
│  - Reads environment variables & command-line arguments     │
│  - Handles platform-specific logging and setup              │
│  - CONSTRUCTS the platform-agnostic `ServerConfig`          │
│  - Manages feature flags (production/development/native)    │
└─────────────────────────────────────────────────────────────┘
                              │
                  Passes `ServerConfig` object
                              │
┌─────────────────────────────────────────────────────────────┐
│               `lib_bodhiserver` Library Layer               │
│                                                             │
│  - Is completely platform-oblivious                         │
│  - Receives a `ServerConfig` with concrete values           │
│  - Executes Phase 1: Directory setup, settings loading     │
│  - Executes Phase 2: Service creation and wiring           │
│  - Returns a fully initialized `AppService` instance        │
└─────────────────────────────────────────────────────────────┘
```

## 3. Technical Implementation

### 3.1. Crate Structure

A new library crate will be created in the workspace.

```
crates/
├── lib_bodhiserver/          # New platform-agnostic library crate
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs         # Platform-agnostic configuration struct
│   │   ├── initializer.rs    # Core initialization logic
│   │   ├── error.rs          # Crate-specific error types
│   │   └── test_utils/       # Test helpers (feature-gated)
│   └── Cargo.toml
├── bodhi/                    # Existing Tauri binary crate
│   └── src-tauri/
│       └── src/
│           └── main.rs       # Will use `lib_bodhiserver` to initialize
├── integration-tests/        # Existing integration test crate
│   └── tests/
│       └── utils/
│           └── ...           # Will be refactored to use `lib_bodhiserver`
...
```

### 3.2. Configuration Design Analysis

**Current State Analysis:**
- `lib_main.rs:74-105` creates system settings with hardcoded values for environment, app type, version, auth URL/realm
- `app.rs:54-122` requires 10+ service dependencies with complex initialization order
- `live_server_utils.rs:70-101` duplicates environment setup with test-specific overrides
- Existing `derive_builder` pattern in `services/src/test_utils/app.rs:41-86` provides proven approach

**Proposed Configuration Approach:**
- **Platform-agnostic `ServerConfig`** struct using established `derive_builder` pattern
- **Captures resolved values only** - no platform-specific logic (paths, environment variables)
- **Supports all deployment modes** - production (Tauri), development (CLI), testing (integration tests)
- **Leverages existing patterns** - follows `AppServiceStubBuilder` design from test utilities

**Key Configuration Categories Identified:**
1. **Core Environment Settings** - `EnvType`, `AppType`, version, bodhi_home path
2. **Authentication Configuration** - auth URL, realm, optional encryption keys
3. **Service Dependencies** - HF cache paths, exec lookup paths, logging preferences
4. **Test-specific Fields** - temporary directory management, feature toggles

**Design Principles:**
- Use `derive_builder` with `strip_option` for optional fields (established pattern)
- Provide derived path methods matching `SettingService` interface for compatibility
- Support both production and test scenarios through optional fields
- Maintain type safety with existing domain objects (`EnvType`, `AppType`)

### 3.3. Initialization Strategy Analysis

**Current Duplication Analysis:**
- `lib_main.rs:60-153` (94 lines) handles Phase 1: environment setup, settings service creation
- `app.rs:54-122` (69 lines) handles Phase 2: service instantiation and wiring
- `live_server_utils.rs:111-170` (60 lines) duplicates both phases with test-specific variations
- **Total duplication:** ~160 lines of complex initialization logic

**Multi-Phase Pattern Validation:**
The existing codebase already implements a proven 2-phase initialization pattern:

**Phase 1: Environment & Settings Foundation** (`lib_main.rs:60-153`)
- Environment wrapper creation and `InitService` setup
- Bodhi home directory discovery/creation via `InitService::setup_bodhi_home_dir()`
- System settings construction (env type, app type, version, auth config)
- `DefaultSettingService` initialization with defaults and environment loading
- Directory structure creation (aliases, databases, logs, HF cache)

**Phase 2: Service Instantiation & Composition** (`app.rs:54-122`)
- Database pool connections and migrations (app.db, session.db)
- Service dependency resolution in correct order:
  - Core services: `HfHubService`, `LocalDataService`, `KeycloakAuthService`
  - Storage services: `SqliteDbService`, `SqliteSessionService`, `DefaultSecretService`
  - Supporting services: `MokaCacheService`, `FluentLocalizationService`, `DefaultTimeService`
- Final `DefaultAppService` composition with all dependencies

**Proposed Centralization Approach:**
- **Preserve the proven 2-phase pattern** - don't reinvent working architecture
- **Extract common logic** into library while maintaining phase separation
- **Platform-specific concerns remain in callers** - path resolution, environment variables, feature flags
- **Library handles platform-agnostic logic** - service creation, dependency wiring, error handling

**Service Dependency Graph Analysis:**
From `app.rs:54-122`, identified critical dependency order:
1. `SettingService` (foundation for all other services)
2. Database services (`DbService`, `SessionService`) - require settings for paths
3. Security services (`SecretService`) - requires encryption key and settings
4. Business services (`HubService`, `DataService`, `AuthService`) - require settings and storage
5. `AppService` composition - requires all dependencies

**Integration Test Simplification Opportunity:**
Current `live_server_utils.rs` complexity can be reduced from 160+ lines to simple builder pattern:
```rust
// Instead of complex setup, enable simple test initialization
let app_service = ServerConfigBuilder::test_default()
    .with_oauth_config(client_id, client_secret, issuer)
    .build_and_initialize().await?;
```

### 3.4. Test Utilities Integration Strategy

**Existing Test-Utils Pattern Analysis:**
The codebase has a sophisticated `test-utils` feature flag pattern documented in `ai-docs/01-architecture/backend-testing-utils.md`:
- **Conditional compilation** with `#[cfg(feature = "test-utils")]` and `#[cfg(all(not(feature = "test-utils"), test))]`
- **Layered service architecture testing** via `AppServiceStubBuilder` in `services/src/test_utils/app.rs`
- **Cross-crate type safety** with shared test utilities across workspace

**Integration Test Complexity Analysis:**
Current `live_server_utils.rs:42-171` implements complex setup:
- Environment variable loading from `.env.test` files
- Temporary directory management with test data copying
- OAuth configuration with real credentials for integration testing
- Database setup and migration
- Session management for authenticated requests

**Proposed Test Utilities Approach:**
- **Leverage existing patterns** - follow established `test-utils` feature flag conventions
- **Provide builder-based setup** - similar to `AppServiceStubBuilder` but for real services
- **Support both unit and integration testing** - mock services for unit tests, real services for integration
- **Maintain test data isolation** - proper temporary directory and database management

**Key Test Utility Functions Needed:**
1. **Simple test setup** - minimal configuration for unit tests
2. **Integration test setup** - full OAuth configuration with real credentials
3. **Authenticated session helpers** - session creation and cookie management
4. **Test data management** - copying test fixtures and managing temporary directories

**Compatibility with Existing Integration Tests:**
The new library should seamlessly replace the current `live_server_utils.rs` setup while:
- Preserving OAuth integration test capabilities
- Maintaining session-based authentication patterns
- Supporting the existing test data structure in `tests/data/live/bodhi`
- Keeping compatibility with `rstest` fixtures and `#[awt]` async test patterns

## 4. Implementation Roadmap

### 4.1. Phased Approach Rationale

**Risk Mitigation Strategy:**
- Each phase maintains working application state with `cargo test --workspace` validation
- Incremental refactoring reduces integration complexity
- Production path stabilized before test path changes
- Foundation established before extensibility features

**Dependency Analysis:**
- **Phase 1** addresses the core duplication problem (94 lines in `lib_main.rs` + 69 lines in `app.rs`)
- **Phase 2** leverages Phase 1 foundation to simplify integration tests (160+ lines in `live_server_utils.rs`)
- **Phase 3** demonstrates extensibility value for future CLI/Docker deployment modes

### Phase 1: Production Path Consolidation

**Scope:** Extract and centralize `lib_main.rs` and `app.rs` initialization logic
**Target Files:**
- `crates/bodhi/src-tauri/src/lib_main.rs:60-153` (Phase 1 logic)
- `crates/bodhi/src-tauri/src/app.rs:54-122` (Phase 2 logic)

**Key Deliverables:**
1. **New `lib_bodhiserver` crate** with workspace integration
2. **Platform-agnostic configuration** using `derive_builder` pattern
3. **2-phase initializer** preserving existing architecture
4. **Tauri app refactoring** to use library instead of inline logic

**Success Criteria:**
- Tauri application starts and functions identically
- All existing functionality preserved (native app, serve command)
- No behavioral changes in production deployment
- `cargo test --workspace` passes without regressions

### Phase 2: Integration Test Simplification

**Scope:** Replace `live_server_utils.rs` complexity with library-based approach
**Target Files:**
- `crates/integration-tests/tests/utils/live_server_utils.rs:42-171`
- `crates/integration-tests/tests/test_live_chat_completions_streamed.rs`

**Key Deliverables:**
1. **Test utilities module** following established `test-utils` feature pattern
2. **OAuth integration helpers** for real credential testing
3. **Session management utilities** for authenticated request testing
4. **Simplified test setup** reducing boilerplate from 160+ lines to builder pattern

**Success Criteria:**
- All integration tests pass with identical behavior
- Test setup code significantly reduced and more maintainable
- OAuth authentication flow preserved
- Session-based testing capabilities maintained

### Phase 3: Extensibility Foundation

**Scope:** Demonstrate library value for new deployment modes
**Target:** Enhance existing `crates/server_app` or create new CLI binary

**Key Deliverables:**
1. **CLI deployment mode** using library initialization
2. **Minimal configuration examples** for different use cases
3. **Documentation and examples** for future Docker/headless deployments

**Success Criteria:**
- CLI binary successfully uses library for initialization
- Different configuration profiles work (minimal vs full)
- Foundation established for future deployment modes
- Clear path for Docker containerization

### 4.2. Technical Considerations

**Workspace Integration:**
- Add `lib_bodhiserver` to `Cargo.toml` workspace members
- Follow existing dependency patterns and feature flag conventions
- Maintain compatibility with existing `derive_builder` and `test-utils` patterns

**Error Handling Strategy:**
- Leverage existing error handling patterns from `services` crate
- Provide clear error messages for initialization failures
- Maintain compatibility with existing error propagation

**Logging and Observability:**
- Preserve existing logging setup from `lib_main.rs:155-224`
- Support both file and stdout logging based on configuration
- Maintain tracing integration for debugging initialization issues