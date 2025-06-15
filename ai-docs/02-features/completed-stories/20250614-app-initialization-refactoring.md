# App Initialization Refactoring: CLI-First Architecture Technical Specification

**Status**: ✅ Completed
**Priority**: High
**Estimated Effort**: 4-6 hours
**Actual Effort**: ~4 hours
**Target Completion**: 2025-06-14
**Actual Completion**: 2025-06-14

## Implementation Summary

**✅ Phase 1: CLI-First Entry Point** - Completed
- Updated `main.rs` to call `app::main()` instead of `lib_main::_main()`
- Moved CLI parsing from `app::start()` to `app::main()`
- Created `initialize_and_execute()` function signature for delegation
- Updated CLI structure to be feature-flag conditional

**✅ Phase 2: Conditional Module Implementation** - Completed
- Created `native_init.rs` with native-specific initialization logic
- Created `server_init.rs` with server-specific initialization logic
- Extracted initialization logic from `lib_main.rs` into respective modules
- Implemented feature-specific constants and logging
- Updated `lib.rs` with conditional module compilation

**✅ Phase 3: Complete `lib_main.rs` Removal** - Completed
- Removed `lib_main.rs` file entirely
- Updated all references to use new module structure
- Verified compilation with both feature flag configurations
- Cleaned up unused imports and code

**Verification Results**:
- ✅ `cargo check --features native` - Compiles successfully
- ✅ `cargo check` (server mode) - Compiles successfully
- ✅ `cargo fmt` - Code formatting applied
- ✅ CLI tests updated for feature-conditional behavior

## 1. Context

This specification defines a complete replacement of the current app initialization architecture with a CLI-first approach using compile-time feature flag resolution. The current implementation initializes the entire app service before CLI parsing, creating inefficiency and architectural complexity.

**Current Problem**: `main.rs` → `lib_main.rs` → Full App Init → `app.rs` → CLI Parse → Execute

**Target Solution**: `main.rs` → `app.rs` → CLI Parse → Feature-Specific Init → Execute

### Current Initialization Flow
The current flow follows this sequence:
1. `main.rs` → `lib_main::_main()`
2. `lib_main::execute()` → Full app initialization (directories, settings, services)
3. `lib_main::aexecute()` → Build complete app service
4. `app::start()` → Parse CLI arguments and execute command

### References
- `ai-docs/README.md` - Documentation index and navigation
- `ai-docs/01-architecture/README.md` - Architecture documentation overview
- `ai-docs/02-features/completed-stories/20250610-lib-bodhiserver.md` - Previous initialization refactoring
- `crates/bodhi/src-tauri/src/native.rs` - Existing conditional compilation pattern

## 2. Architectural Changes

### 2.1. Feature Flag Resolution Strategy

**Single Feature Flag**: Use `native` feature exclusively for compile-time resolution:
- **Enabled** (`--features native`): Tauri desktop application
- **Disabled** (default): Server deployment mode
- other feature flag --features production exists, that embeds the auth server url depending on production is active or inactive. that should not affect our native vs server distinction

**Replace Runtime Checks**: Eliminate `setting_service.is_native()` runtime checks with compile-time `cfg!(feature = "native")` resolution.


### 2.2. CLI-First Entry Point with Conditional Commands

**CLI Behavior by Feature Flag**:
- **Native mode**: App executed by double-clicking, no CLI arguments expected - only default behavior
- **Server mode**: CLI accepts server host and port parameters for deployment

**Conditional CLI Structure**:
```rust
// app.rs
#[derive(Parser, Debug)]
#[command(name = "bodhi")]
#[command(about = "Bodhi App - Your personal, private, open-source AI Stack")]
#[command(version)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
  #[cfg(not(feature = "native"))]
  /// Start the server in deployment mode
  Serve {
    /// Host address to bind to
    #[arg(short = 'H', long, default_value = services::DEFAULT_HOST)]
    host: String,
    /// Port number to bind to
    #[arg(short, long, default_value_t = services::DEFAULT_PORT)]
    port: u16,
  },
}

pub fn main() {
    let cli = Cli::parse(); // Parse CLI first, before any initialization
    let command = match cli.command {
        #[cfg(not(feature = "native"))]
        Some(Commands::Serve { host, port }) => AppCommand::Server(host, port),
        None => AppCommand::Default,
    };

    if let Err(err) = initialize_and_execute(command) {
        tracing::error!("fatal error: {err}\nexiting application");
        std::process::exit(1);
    }
}
```

**Execution Paths by Feature Flag**:

**Native Binary Path** (`--features native`):
- **Usage**: `bodhi` (no subcommand, no CLI arguments)
- **Requirements**: Full app service, native features, UI router, system tray
- **Initialization**: Complete service initialization with native-specific settings

**Server Binary Path** (default build):
- **Usage**: `bodhi serve --host <host> --port <port>` or `bodhi serve` or `bodhi`
- **Requirements**: App service without native features, server-specific settings, UI router for frontend serving
- **Initialization**: Service initialization optimized for server deployment

**Help/Version Path** (both feature flags):
- **Usage**: `bodhi --help`, `bodhi --version`
- **Requirements**: No app initialization
- **Behavior**: Immediate CLI response, automatically handled by clap framework


### 2.3. Conditional Module Compilation

**File Structure**:
```
crates/bodhi/src-tauri/src/
├── main.rs                    # Entry point
├── app.rs                     # CLI parsing and delegation
├── native_init.rs             # #[cfg(feature = "native")]
├── server_init.rs             # #[cfg(not(feature = "native"))]
├── error.rs                   # Error types
└── ui.rs                      # UI router
```

**Module Declaration Pattern**:
**Implementation Pattern** (following `crates/bodhi/src-tauri/src/native.rs`):
```rust
// lib.rs
#[cfg(feature = "native")]
mod native_init;
#[cfg(not(feature = "native"))]
mod server_init;

// Re-export appropriate implementation
#[cfg(feature = "native")]
pub use native_init::initialize_and_execute;
#[cfg(not(feature = "native"))]
pub use server_init::initialize_and_execute;
```

### 2.4. Feature-Specific Logging

**Native Logging** (`native_init.rs`):
```rust
// Tauri-compatible logging initialization 
```

**Server Logging** (`server_init.rs`):
```rust
fn setup_logs(setting_service: &DefaultSettingService) -> WorkerGuard {
    // Extract setup_logs() function from lib_main.rs
    // File-based logging with stdout configuration for server deployment
    // Include tracing_appender and WorkerGuard management
}
```

## 3. Implementation Phases

### Phase 1: CLI-First Entry Point

**Replace `lib_main.rs` Flow**:
1. Update `main.rs` to call `app::main()` instead of `lib_main::_main()`
2. Move CLI parsing from `app::start()` to `app::main()`
3. Create `initialize_and_execute()` function signature for delegation
4. Remove `lib_main.rs` entirely

**CLI Parsing Independence**:
- CLI parsing must not depend on `SettingService` initialization
- Use `clap::Cli::parse()` directly in `app::main()`
- Handle help/version commands without any app initialization

### Phase 2: Conditional Module Implementation

**Create Feature-Specific Modules**:

`native_init.rs`:
```rust
use crate::native;

pub async fn initialize_and_execute(command: AppCommand) -> Result<(), ApiError> {
    // Extract initialization logic from lib_main.rs execute() and aexecute()
    // Include native-specific feature settings
    // Handle native commands (default)
}
```

`server_init.rs`:
```rust
pub async fn initialize_and_execute(command: AppCommand) -> Result<(), ApiError> {
    // Extract initialization logic from lib_main.rs execute() and aexecute()
    // Include server-specific logging setup
    // Handle server commands (serve, default)
}
```

**Feature Flag Resolution**:
- Replace `ENV_TYPE`, `AUTH_URL`, `AUTH_REALM` constants with direct values in each module
- Replace `APP_TYPE` runtime resolution with compile-time feature detection
- Move `set_feature_settings()` logic into respective initialization modules

### Phase 3: Complete `lib_main.rs` Removal

**Final Architecture**:
1. Remove `lib_main.rs` file entirely
2. Update `lib.rs` module declarations to use conditional compilation
3. Verify compilation with both `cargo build` and `cargo build --features native`
4. Update any remaining references to `lib_main` module

## 4. Testing Strategy

**Compilation Verification**:
Since this codebase lacks comprehensive tests, focus on compilation verification across feature flag configurations:

```bash
# Test native feature compilation
cargo build --features native
cargo test -p bodhi --features native

# Test server compilation (default)
cargo build
cargo test -p bodhi

# Test lib_bodhiserver integration
cargo test -p lib_bodhiserver
```

**Integration Test Compatibility**:
- Integration tests in `crates/integration-tests/` use `lib_bodhiserver` crate
- No changes required to integration test setup
- Tests will verify app service initialization continues to work

**Error Verification**:
- CLI parsing errors should occur before any initialization
- Help/version commands should execute without app service creation
- Invalid commands should fail fast without resource allocation

## 5. Implementation Guidance

### 5.1. Code Extraction from `lib_main.rs`

**Extract to `native_init.rs`**:
- `execute()` function logic with native-specific paths
- `ENV_TYPE`, `AUTH_URL`, `AUTH_REALM` constants for development/production
- `set_feature_settings()` for native feature configuration
- Native-specific error handling

**Extract to `server_init.rs`**:
- `execute()` function logic with server-specific paths
- `setup_logs()` function for server deployment logging
- Server-specific environment configuration
- Container app type handling

**Common Patterns**:
- Both modules use `lib_bodhiserver::AppOptionsBuilder` and `setup_app_dirs()`
- Both modules use `lib_bodhiserver::build_app_service()` for service creation
- Error handling patterns remain consistent across modules

### 5.2. Feature Flag Constants

**Replace Conditional Constants**:
```rust
// Instead of runtime resolution in lib_main.rs
#[cfg(feature = "native")]
const APP_TYPE: AppType = AppType::Native;

// Use direct values in each module
// native_init.rs
const APP_TYPE: AppType = AppType::Native;

// server_init.rs
const APP_TYPE: AppType = AppType::Container;
```

### 5.3. Command Delegation Pattern

**Command Enum Definition**:
```rust
// AppCommand remains same, just that AppCommand::Serve is never composed for native build, might have to add clippy allow to have unused variant
pub enum AppCommand {
    Server(String, u16), // host, port
    Default,
}
```

**Delegation Logic** (matches CLI structure above):
```rust
pub fn main() {
    let cli = Cli::parse();
    let command = match cli.command {
        #[cfg(not(feature = "native"))]
        Some(Commands::Serve { host, port }) => AppCommand::Server(host, port),
        None => AppCommand::Default,
    };

    if let Err(err) = initialize_and_execute(command) {
        tracing::error!("fatal error: {err}\nexiting application");
        std::process::exit(1);
    }
}
```

### 5.4. Module Re-export Pattern

**Following `native.rs` Pattern**:
```rust
// lib.rs - Conditional module compilation and re-export
#[cfg(feature = "native")]
mod native_init;
#[cfg(not(feature = "native"))]
mod server_init;

// Single function signature exported regardless of feature
#[cfg(feature = "native")]
pub use native_init::initialize_and_execute;
#[cfg(not(feature = "native"))]
pub use server_init::initialize_and_execute;
```

**Testing After Each Phase**:
```bash
# After Phase 1 - CLI-first entry point
cargo build --features native
cargo build
cargo test -p bodhi --features native
cargo test -p bodhi

# After Phase 2 - Conditional modules
cargo build --features native
cargo build
cargo test -p bodhi --features native
cargo test -p bodhi
cargo test -p lib_bodhiserver

# After Phase 3 - lib_main.rs removal
cargo build --features native
cargo build
cargo test --workspace
```

## Related Documentation

- **[Documentation Index](../../README.md)** - Complete documentation navigation and overview
- **[Architecture Overview](../../01-architecture/README.md)** - Technical architecture documentation index
- **[System Overview](../../01-architecture/system-overview.md)** - High-level system architecture
- **[Architectural Decisions](../../01-architecture/architectural-decisions.md)** - Key design decisions and rationale
- **[Lib Bodhiserver](20250610-lib-bodhiserver.md)** - Previous initialization refactoring foundation

---

*This specification defines a complete replacement architecture using CLI-first initialization with compile-time feature flag resolution. Implementation should follow existing conditional compilation patterns from `native.rs` while eliminating runtime feature detection in favor of compile-time optimization.*
