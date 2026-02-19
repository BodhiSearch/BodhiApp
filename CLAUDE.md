# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Testing
- `make test` - Run all tests (backend, UI, NAPI)
- `make test.backend` - Run Rust backend tests (`cargo test` and `cargo test -p bodhi --features native`)
- `make test.ui` - Run frontend tests (`cd crates/bodhi && npm install && npm test`)
- `make test.napi` - Run NAPI bindings tests (`cd crates/lib_bodhiserver_napi && npm install && npm run test`)

### Building & Packaging
- `make ci.build` - Build Tauri desktop application
- `make build.ts-client` - Build TypeScript client package with tests
- `cd crates/bodhi && npm run build` - Build Next.js frontend
- `cd crates/lib_bodhiserver_napi && npm run build:release` - Build NAPI bindings

### Code Quality
- `make format` - Format all code (Rust, Node.js, Python)
- `make format.all` - Format and run Clippy fixes
- `cargo fmt --all` - Format Rust code only
- `cd crates/bodhi && npm run format` - Format frontend code
- `cd crates/bodhi && npm run lint` - Lint frontend code
- `cd crates/lib_bodhiserver_napi && npm run format` - Format NAPI package

### Coverage & Analysis
- `make test.coverage` - Generate code coverage report (outputs to `lcov.info`)

### OpenAPI & Client Generation
- `cargo run --package xtask openapi` - Generate OpenAPI specification
- `cd ts-client && npm run generate` - Generate TypeScript client types

### Running the Application
- `cd crates/bodhi && npm run dev` - Start Next.js development server
- `cd crates/bodhi/src-tauri && cargo tauri dev` - Run Tauri desktop app in dev mode
- `make run.app` - Run standalone HTTP server with development configuration
- `cargo run --bin bodhi -- serve --port 1135` - Run server directly with custom configuration

### Docker Development & Deployment
- `make docker.dev.cpu` - Build CPU-optimized Docker image (multi-platform AMD64/ARM64)
- `make docker.dev.cuda` - Build NVIDIA CUDA GPU-accelerated image
- `make docker.dev.cpu.amd64` - Build AMD64-specific CPU image for local testing
- `make docker.dev.cpu.arm64` - Build ARM64-specific CPU image for local testing
- `make docker.run.amd64` - Run locally built AMD64 Docker image
- `make docker.run.arm64` - Run locally built ARM64 Docker image
- `make docker.list` - List all locally built BodhiApp Docker images
- `make docker.clean` - Remove all locally built BodhiApp Docker images

### Release Management
- `make release.ts-client` - Create and push tag for TypeScript client package release (@bodhiapp/ts-client)
- `make release.app-bindings` - Create and push tag for NAPI bindings release (@bodhiapp/app-bindings)
- `make release.docker` - Create and push tag for production Docker image release
- `make release.docker-dev` - Create and push tag for development Docker image release
- `make docker.version-check` - Check latest versions of Docker images from GitHub Container Registry
- `make ci.ts-client-check` - Verify TypeScript client is synchronized with OpenAPI specification
- `make docs.context-update` - Update AI documentation context symlinks (CLAUDE.md/PACKAGE.md)

## Layered Development Methodology

BodhiApp uses strict upstream-to-downstream layered development. When implementing a feature or fix that spans multiple crates, always start from the most upstream crate and work downstream.

### Crate Dependency Chain

```
objs -> services -> (server_core, auth_middleware) -> routes_app -> server_app -> (lib_bodhiserver, bodhi/src-tauri)
```

### Development Flow

1. **Upstream Rust crate first**: Make changes to the most upstream crate affected (e.g., `objs`). Run and fix tests for that crate (`cargo test -p objs`). Then run tests for all upstream + current crates to verify no regressions. Mark the crate milestone done.
2. **Repeat downstream**: Move to the next downstream crate (e.g., `services`). Make changes, run its tests, run cumulative tests for all crates changed so far.
3. **Continue through the chain**: `routes_app` -> `server_app` -> any other affected crates. At each step, run cumulative tests.
4. **Full backend validation**: Once all Rust crate changes are done, run `make test.backend` to verify the complete backend.
5. **Regenerate TypeScript types**: Run `make build.ts-client` to regenerate the TypeScript types used by the frontend in `crates/bodhi/src/`. The frontend imports request/response types from `@bodhiapp/ts-client`.
6. **Frontend component tests**: Make UI changes in `crates/bodhi/src/`, always using `@bodhiapp/ts-client` for request/response types. Run `cd crates/bodhi && npm run test` to fix component tests. Mark the UI milestone done.
7. **E2E tests**: Run `make build.ui-rebuild` to rebuild the NAPI bindings with UI changes. Then add/update E2E tests in `crates/lib_bodhiserver_napi/tests-js/`. Run updated/relevant specs first, then `make test.napi` for full regression. Analyze any failures for relevance (some tests are flaky). Mark the E2E milestone done.
8. **Documentation**: Update crate-level `CLAUDE.md` / `PACKAGE.md` for each modified crate, and project root `CLAUDE.md` for architectural changes.

## Architecture Overview

BodhiApp is a Rust-based application providing local Large Language Model (LLM) inference with a modern React web interface and Tauri desktop app.

### Technology Stack
- **Backend**: Rust with Axum HTTP framework
- **Frontend**: React + TypeScript + Next.js v14 + TailwindCSS + Shadcn UI  
- **Desktop**: Tauri for native desktop application
- **LLM Integration**: llama.cpp for local inference
- **Database**: SQLite with SQLx
- **Authentication**: OAuth2 + JWT
- **API**: OpenAI-compatible endpoints

### Key Crates Structure
The project uses a Cargo workspace with these main crates:

**Foundation Crates:**
- `objs` - Domain objects, types, errors, validation
- `services` - Business logic, external integrations
- `server_core` - HTTP server infrastructure
- `auth_middleware` - Authentication and authorization

**API Crates:**
- `routes_app` - Unified route composition

**Application Crates:**
- `server_app` - Standalone HTTP server
- `crates/bodhi/src-tauri` - Tauri desktop application
- `commands` - CLI interface

**Utility Crates:**
- `llama_server_proc` - LLM process management
- `lib_bodhiserver_napi` - Node.js bindings for server functionality
- `xtask` - Build automation and code generation

### Crate-Level Documentation (Progressive Disclosure)

**When working on a task, load the CLAUDE.md and PACKAGE.md for the relevant crate(s).** Each crate's docs guide you to the right source files -- read those source files rather than guessing conventions. For cross-crate tasks, load docs for each involved crate.

| Crate | CLAUDE.md | PACKAGE.md | Keywords |
|---|---|---|---|
| `objs` | `crates/objs/CLAUDE.md` | `crates/objs/PACKAGE.md` | Universal foundation layer. ErrorType, AppError, ApiError envelope, error propagation. Validation constants: slug length limits (MAX_MCP_SLUG_LEN, MAX_TOOLSET_SLUG_LEN), field limits. Serde conventions (default, skip_serializing_if). Role/scope hierarchy (Admin>Manager>PowerUser>User). GGUF model format. OAI request params. Repo/HubFile/Alias domain types. Test utilities: builders, fixtures, temp_bodhi_home |
| `errmeta_derive` | `crates/errmeta_derive/CLAUDE.md` | `crates/errmeta_derive/PACKAGE.md` | ErrorMeta proc macro generating error_type(), code(), args(). Error code naming: `{enum_name_snake_case}-{variant_name_snake_case}`. Integrates with thiserror. Supports transparent delegation, per-variant ErrorType override. trybuild compile-time tests |
| `services` | `crates/services/CLAUDE.md` | `crates/services/PACKAGE.md` | Business logic service traits. DB operations (SqliteDbService), TimeService (never Utc::now()), impl_error_from! macro. HubService, DataService, McpService, ToolService, AuthService, AccessRequestService. Error chain: service error→AppError→ApiError. Test infra: TestDbService, AppServiceStub builder, FrozenTimeService, OfflineHubService, SecretServiceStub |
| `auth_middleware` | `crates/auth_middleware/CLAUDE.md` | `crates/auth_middleware/PACKAGE.md` | AuthContext enum (Anonymous, Session, ApiToken, ExternalApp). auth_middleware, optional_auth_middleware. api_auth_middleware role/scope checks. AccessRequestValidator trait, AccessRequestAuthError enum (EntityNotApproved, etc.). ToolsetAuthError. JWT DefaultTokenService. Token digest. Same-origin CSRF. Test factories: test_session, test_external_app, RequestAuthContextExt |
| `server_core` | `crates/server_core/CLAUDE.md` | `crates/server_core/PACKAGE.md` | RouterState trait, SharedContext for LLM. SSE: DirectSSE, ForwardedSSE. Test utilities: ResponseTestExt (json/text/sse parsing), RequestTestExt, router_state_stub fixture, ServerFactoryStub |
| `routes_app` | `crates/routes_app/CLAUDE.md` | `crates/routes_app/PACKAGE.md` | API orchestration layer. Domain-specific error enums per module (LoginError, AccessRequestError, McpValidationError, etc.) with errmeta_derive. OpenAPI utoipa registration checklist (7 steps). AuthContext handler patterns. List response shapes: non-paginated use resource-plural field names ({mcps:[...]}, {toolsets:[...]}), paginated use {data:[], total, page, page_size}. Toolset/MCP CRUD, app access request workflow |
| `server_app` | `crates/server_app/CLAUDE.md` | `crates/server_app/PACKAGE.md` | Standalone HTTP server. Live integration tests (multi-turn, full stack, no mocks). OAuth2 test infra: TestServerHandle, ExternalTokenSimulator, authenticated sessions. Serial test execution |
| `lib_bodhiserver` | `crates/lib_bodhiserver/CLAUDE.md` | `crates/lib_bodhiserver/PACKAGE.md` | Embeddable server library. Service composition, app bootstrap, AppOptionsBuilder |
| `bodhi/src` | `crates/bodhi/src/CLAUDE.md` | `crates/bodhi/src/PACKAGE.md` | Next.js 14 frontend. @bodhiapp/ts-client generated types from OpenAPI. react-hook-form + zod schema validation. API hooks (useQuery, useMutation). Component architecture: pages + co-located components. Absolute imports with @/ prefix |
| `bodhi/src-tauri` | `crates/bodhi/src-tauri/CLAUDE.md` | -- | Tauri desktop app, native features, system tray, NativeError, AppSetupError |
| `llama_server_proc` | `crates/llama_server_proc/CLAUDE.md` | `crates/llama_server_proc/PACKAGE.md` | LLM process lifecycle, Server trait, health checks, binary management |
| `lib_bodhiserver_napi` | `crates/lib_bodhiserver_napi/CLAUDE.md` | `crates/lib_bodhiserver_napi/PACKAGE.md` | NAPI bindings embedding Rust server into Node.js. createTestServer |
| tests-js | `crates/lib_bodhiserver_napi/tests-js/CLAUDE.md` | -- | Playwright E2E user journey tests. Page Object Model (BasePage). Fixtures with static factory methods. Shared vs dedicated server patterns. Known quirks (SPA nav, KC session, toast). Also load `tests-js/E2E.md` for test writing conventions, anti-patterns, canonical examples |
| `xtask` | `xtask/CLAUDE.md` | `xtask/PACKAGE.md` | OpenAPI spec generation from BodhiOpenAPIDoc, TypeScript client pipeline |
| `ci_optims` | `crates/ci_optims/CLAUDE.md` | `crates/ci_optims/PACKAGE.md` | Docker layer caching, CI dependency pre-compilation |

### Frontend Structure
Located in `crates/bodhi/`, this is a Next.js 14 application using:
- React with TypeScript
- TailwindCSS + Shadcn UI components
- React Query for API state management
- Vitest for testing

### Key Features
- **Local LLM Inference**: llama.cpp integration with model management and hardware acceleration (CPU, CUDA, ROCm, Vulkan)
- **OpenAI Compatibility**: Full OpenAI API compatibility for chat completions and models endpoints
- **Web Interface**: Modern React-based chat UI with streaming responses and real-time model management
- **Desktop Application**: Tauri-based native app with system tray integration and cross-platform support
- **Multi-Platform Deployment**: Native desktop apps, Docker containers, and development server modes
- **Comprehensive Authentication**: OAuth2 + JWT with role-based access control, user administration, and access request workflows
- **User Management System**: Complete user administration with role assignment, access request approval, and resource management
- **Model Ecosystem Integration**: HuggingFace model discovery, download, validation, and local management

### Development Patterns
- **Error Handling**: Centralized error types with localization support and OpenAI-compatible API responses
- **Testing**: Unit tests per crate, integration tests, UI tests with Playwright, and comprehensive coverage reporting
- **Code Generation**: OpenAPI specs auto-generated from Rust code, TypeScript client generation, NAPI bindings
- **Configuration**: Environment-based config with runtime updates and secure credential management
- **Service Architecture**: Trait-based dependency injection with comprehensive mocking support for testing
- **Cross-Crate Coordination**: Layered architecture with clear separation between domain objects, services, and routes
- **Security Model**: Multi-layer authentication with JWT tokens, OAuth2 flows, and role-based authorization

## Architectural Decision Patterns

### Cross-Crate Dependencies and Data Flow
Understanding BodhiApp's layered architecture is crucial for effective development:

**Foundation Layer** (`objs` → `services`):
- Domain objects, error types, and validation rules flow from `objs` into service implementations
- Services coordinate business logic using domain objects while maintaining clear boundaries
- Centralized error handling ensures consistent API responses across all deployment modes

**Service Layer** (`services` → `routes_*`):
- Business services provide trait-based interfaces consumed by route handlers
- Authentication flows span multiple services: AuthService, SessionService, SecretService coordination
- User access control integrates UserService with Role-based authorization in routes

**API Layer** (`routes_*` → `server_*` / `bodhi`):
- Route composition separates OpenAI-compatible endpoints from app-specific functionality
- Unified error handling converts service errors to proper HTTP responses with localization
- OpenAPI generation ensures TypeScript client and documentation stay synchronized

**Deployment Layer**:
- Server applications embed route combinations appropriate for their deployment context
- Desktop app integrates HTTP server with Tauri system integration
- Docker containers provide hardware-accelerated variants with optimized configurations

### User Management and Access Control Architecture
Recent architectural enhancements introduce sophisticated user administration:

**User Lifecycle Management**:
- User registration triggers access request workflow with admin approval gates
- Role assignment system supports standard users and resource managers with escalating privileges
- Session management coordinates HTTP sessions with JWT token lifecycle and database persistence

**Access Request Workflow**:
- Self-service access requests with status tracking (pending, approved, denied)
- Admin dashboards provide comprehensive user management with role modification capabilities
- Resource manager permissions enable delegated administration without full system access

**Security Coordination**:
- Multi-service authentication flow ensures consistent security across API and UI endpoints
- Token exchange patterns support service-to-service communication with proper scope validation
- Session invalidation coordinated across HTTP sessions, JWT tokens, and database state

### Model Management Coordination
Local AI model management integrates multiple services and external systems:

**Model Discovery and Acquisition**:
- HubService integrates with HuggingFace API for model metadata and download coordination
- GGUF format validation ensures model compatibility before local storage
- Progress tracking provides real-time feedback during long-running model downloads

**Local Model Management**:
- DataService manages local model storage with validation and metadata extraction
- Model aliasing system provides user-friendly names while maintaining canonical model references
- Integration with llama.cpp process management for inference coordination

## Important Notes

### Development Guidelines
- Run `make test` before making changes to ensure nothing is broken across backend, UI, and NAPI components
- Use `make format.all` to format code and fix linting issues across Rust, TypeScript, and other languages
- Always regenerate OpenAPI specs and TypeScript types after API changes: `cargo run --package xtask openapi && cd ts-client && npm run generate`
- Frontend uses strict TypeScript - ensure proper typing and avoid `any` types
- NAPI bindings require Node.js >=22 and proper native compilation setup
- Desktop app development requires Tauri CLI and platform-specific dependencies

### Architectural Patterns
- For time handling, use `TimeService` from `crates/services/src/db/service.rs` instead of `Utc::now()` directly - pass timestamps via constructors to maintain testability
- Avoid `use super::*` in Rust `#[cfg(test)]` modules as it creates refactoring issues - use explicit imports
- For error handling, follow the centralized error pattern: service errors → `ApiError` → OpenAI-compatible responses
- When implementing user access control, coordinate between `AuthService`, `SessionService`, and role-based authorization middleware
- Model management operations should coordinate between `HubService` (remote) and `DataService` (local) for consistency

### Testing Practices
- Write tests that provide maintenance value - avoid testing trivial constructors, derive macros, or standard serialization
- Use `assert_eq!(expected, actual)` convention for consistency with JUnit patterns
- For React integration/UI tests, prefer `data-testid` attributes with `getByTestId` over CSS selectors that can change
- Tests should be deterministic - no if-else logic or try-catch blocks in test code
- Use `console.log` for error scenarios in tests only, and avoid unnecessary comments unless logic is complex
- Do not add timeouts for Playwright UI tests except on ChatPage which needs model warm-up time
- **Use rstest for all Rust tests** - leverage rstest features to reduce duplication and improve maintainability:
  - `#[case]` for parameterized tests: test the same logic with multiple inputs/expected outputs instead of writing separate test functions
  - `#[values]` for combinatorial testing: generate test cases from all combinations of input values
  - `#[fixture]` for shared test setup: extract common setup into reusable fixtures with dependency injection
  - Prefer parameterized tests over multiple assert statements in a single test or duplicated test functions with minor variations

## Critical UI Development Workflow

**IMPORTANT: After making changes to UI components, you MUST rebuild the embedded UI:**

1. `make build.ui-clean` - Clean the embedded UI build (removes crates/bodhi/out)
2. `make build.ui` - Build the embedded UI with changes (builds Next.js and NAPI bindings)

The application embeds the UI build, so changes to React components won't be visible until rebuilt. This is required for:
- Adding/modifying data-testid attributes
- Any component changes in crates/bodhi/src/
- UI styling or functionality updates
- Testing UI changes in integration tests

**Development Mode**: For active development, use `cd crates/bodhi && npm run dev` to run Next.js dev server with hot reload.

## Security and Deployment Considerations

### Authentication Architecture
- **Multi-Service Coordination**: Authentication flows span multiple services and require careful coordination between HTTP sessions, JWT tokens, and database state
- **Role-Based Access Control**: Support for standard users and resource managers with different privilege levels - ensure proper authorization checks in new endpoints
- **Token Lifecycle Management**: JWT access/refresh tokens have different lifespans and scopes - coordinate token refresh operations across services
- **Session Management**: HTTP sessions integrate with Tauri desktop app and web interfaces - maintain consistency across deployment modes

### Docker Deployment Patterns
- **Hardware Acceleration Variants**: Each Docker variant (CPU, CUDA, ROCm, Vulkan) requires specific runtime configurations and device access
- **Volume Management**: Model storage and application data require persistent volumes with proper permissions across container environments
- **Multi-Platform Support**: CPU variant supports both AMD64 and ARM64 architectures with automatic platform detection
- **Environment Configuration**: Container deployments require proper environment variable configuration for authentication, model paths, and hardware acceleration

### Release and Versioning
- **Automated Release Flows**: GitHub Actions coordinate package releases, Docker image builds, and version tagging across multiple artifacts
- **Package Synchronization**: TypeScript client, NAPI bindings, and Docker images maintain independent versioning but require synchronization for compatibility
- **OpenAPI Contract Maintenance**: API changes trigger automatic TypeScript client regeneration and documentation updates
- **Documentation Context**: AI documentation context symlinks provide unified access to crate-specific CLAUDE.md and PACKAGE.md files

### Backwards Compatibility
- Do not plan for backwards compatibility unless specifically mentioned - BodhiApp prioritizes architectural improvement over legacy support
- Do not add timeouts for Playwright UI tests except on ChatPage which requires model warm-up time
- if you make changes to @crates/bodhi/src/ you have to run `make build.ui-rebuild` in project root for playwright test to get the ui updates
- do not add inline timeout in component test in crates/bodhi/src, instead rely on the default timeout, or modify the source/test for it so we do not have to override the default timeout
- for ui test, do not add inline timeouts, this fix hides the actual issue