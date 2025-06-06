# Tauri Desktop Architecture

## Overview

Bodhi App uses Tauri to create a native desktop application that embeds a full HTTP server with a React frontend. Unlike typical Tauri apps that use IPC commands, Bodhi runs a complete web server internally and opens the browser to interact with it, providing maximum compatibility with existing web tooling.

## Architecture Components

### Frontend (React+Vite)
- **Location**: `crates/bodhi/src/`
- **Technology**: React 18 + TypeScript + Vite
- **Purpose**: Web-based user interface served by embedded server
- **Build Output**: Static web assets embedded in the binary via `include_dir!`

### Tauri Application
- **Location**: `crates/bodhi/src-tauri/`
- **Technology**: Rust with Tauri framework (optional feature)
- **Purpose**: Native desktop wrapper with system tray integration
- **Key Feature**: Embeds complete HTTP server instead of using Tauri IPC

### Embedded HTTP Server
- **Technology**: Axum web framework via `server_app` crate
- **Purpose**: Full REST API server running inside the desktop app
- **Integration**: Spawned as async task during Tauri setup
- **Accessibility**: Opens system browser to interact with local server

## Project Structure

```
crates/bodhi/
├── src/                    # React frontend
│   ├── components/         # Feature-organized React components
│   ├── pages/             # Route page components
│   ├── hooks/             # Custom React hooks
│   ├── lib/               # Utilities and API client
│   └── ...
├── src-tauri/             # Tauri desktop application
│   ├── src/
│   │   ├── main.rs        # Entry point (calls lib_main::_main)
│   │   ├── lib.rs         # Module exports
│   │   ├── lib_main.rs    # Main application logic
│   │   ├── app.rs         # CLI command handling
│   │   ├── native.rs      # Tauri-specific native features
│   │   ├── ui.rs          # Static asset serving
│   │   ├── convert.rs     # Command conversion utilities
│   │   └── error.rs       # Tauri-specific errors
│   ├── tauri.conf.json    # Tauri configuration
│   ├── Cargo.toml         # Dependencies (all workspace crates)
│   ├── build.rs           # Build script (frontend + llama bins)
│   └── bin/               # Bundled llama.cpp binaries
├── dist/                  # Built frontend assets (embedded)
└── package.json           # Frontend dependencies
```

## Build Process

### Development Mode
1. **Frontend**: Vite dev server runs on `http://localhost:3000`
2. **Backend**: Tauri app starts embedded server and opens browser
3. **Command**: `pnpm run dev` triggers both frontend and Tauri dev mode
4. **Hot Reloading**: Frontend changes reflected via Vite HMR

### Production Build
1. **Frontend**: Vite builds static assets to `dist/`
2. **Asset Embedding**: `build.rs` copies frontend assets and llama binaries
3. **Binary Creation**: Tauri creates native executable with embedded assets
4. **Command**: `pnpm run build` followed by `cargo tauri build`

### Build Configuration

```json
// tauri.conf.json (actual configuration)
{
  "$schema": "https://schema.tauri.app/config/2.0.0-rc",
  "productName": "Bodhi App",
  "version": "0.1.0",
  "identifier": "app.getbodhi.native",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:3000",
    "beforeDevCommand": "pnpm run dev",
    "beforeBuildCommand": "pnpm run build"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "resources": ["bin/**/*"]  // Includes llama.cpp binaries
  }
}
```

## Native Integration

### System Features
- **System Tray**: Menu with "Open Homepage" and "Quit" options
- **Browser Integration**: Opens system browser to local server URL
- **Window Management**: Hide to tray instead of closing (macOS)
- **Resource Bundling**: Embeds llama.cpp binaries in app bundle
- **Logging**: File-based logging with configurable targets

### Actual Implementation

```rust
// native.rs - Key implementation details
pub async fn aexecute(&self, static_router: Option<Router>) -> Result<()> {
  // Configure logging
  let log_plugin = tauri_plugin_log::Builder::default()
    .level(log_level)
    .max_file_size(50_000)
    .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll);

  tauri::Builder::default()
    .plugin(log_plugin)
    .setup(move |app| {
      // Set macOS activation policy
      #[cfg(target_os = "macos")]
      app.set_activation_policy(tauri::ActivationPolicy::Accessory);

      // Setup binary path for llama.cpp
      let bodhi_exec_lookup_path = app.path().resolve("bin", BaseDirectory::Resource)?;

      // Start embedded HTTP server
      let cmd = ServeCommand::ByParams { host, port };
      tokio::spawn(async move {
        match cmd.get_server_handle(app_service, static_router).await {
          Ok(server_handle) => { /* Server started */ },
          Err(err) => tracing::error!(?err, "failed to start backend server"),
        }
      });

      // Create system tray
      let menu = Menu::with_items(app, &[&homepage, &quit])?;
      TrayIconBuilder::new()
        .menu(&menu)
        .menu_on_left_click(true)
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;

      // Open browser to server URL
      if ui {
        webbrowser::open(setting_service.server_url().as_str())?;
      }
    })
    .on_window_event(on_window_event)  // Hide instead of close
    .run(tauri::generate_context!())?;
}
```

## Communication Patterns

### Frontend to Backend
1. **HTTP API**: Standard REST calls to embedded server (primary method)
2. **Browser-based**: Uses fetch/axios like a normal web application
3. **No Tauri IPC**: Deliberately avoids Tauri commands for maximum compatibility

### Backend to Frontend
1. **HTTP Responses**: Standard JSON API responses
2. **Server-Sent Events**: Real-time streaming for chat completions
3. **WebSocket**: Bidirectional communication for real-time features

### Key Architectural Decision

Unlike typical Tauri applications that use IPC commands, Bodhi runs a complete HTTP server and opens the system browser. This provides:

- **Maximum Compatibility**: Works with any web development tools
- **Standard Debugging**: Use browser dev tools normally
- **API Consistency**: Same API for desktop and server deployments
- **Development Simplicity**: No need to learn Tauri-specific patterns

## Deployment Architecture

### Application Bundle Structure
- **Native Binary**: Contains embedded HTTP server and all dependencies
- **Embedded Assets**: Frontend built into binary via `include_dir!` macro
- **Bundled Binaries**: llama.cpp executables for target platform
- **Resource Directory**: `bin/` folder with platform-specific llama binaries

### Platform Support
- **Current**: macOS (primary development platform)
- **Architecture**: Cross-platform Rust + Tauri foundation
- **Binary Variants**: Different llama.cpp builds per platform

### Actual Bundle Configuration

```toml
# Cargo.toml features
[features]
default = ["native"]
native = ["tauri/native"]
production = []
development = []
```

```rust
// build.rs - Asset embedding
fn copy_frontend(bodhiapp_dir: &Path) -> anyhow::Result<()> {
  // Copies built frontend to be embedded
}

fn copy_llama_bins(project_dir: &Path) -> anyhow::Result<()> {
  // Copies llama.cpp binaries to bundle
}
```

## Performance Considerations

### Startup Optimization
- **Async Server Start**: HTTP server starts in background task
- **Browser Launch**: Opens system browser while server initializes
- **Resource Bundling**: All assets embedded for fast access

### Runtime Performance
- **Native HTTP Server**: Full Axum server performance
- **Memory Efficiency**: Rust's zero-cost abstractions
- **Concurrent Processing**: Tokio async runtime for all I/O

## Development Workflow

### Local Development
1. **Frontend**: `pnpm run dev` starts Vite dev server on port 3000
2. **Tauri**: `cargo tauri dev` starts native app pointing to dev server
3. **Backend**: Full HTTP server runs inside Tauri with hot reloading
4. **Browser**: System browser opens to `http://localhost:PORT`

### Command Line Interface

The Tauri app also functions as a CLI tool:

```rust
// app.rs - CLI command handling
match cli.command {
  Command::App { ui } => {
    // Start native Tauri app
    native::NativeCommand::new(service, ui).aexecute(router).await?;
  }
  Command::Serve { host, port } => {
    // Start as HTTP server only
    serve_command.aexecute(service, router).await?;
  }
  Command::Pull { alias, repo, filename, snapshot } => {
    // Download models via CLI
  }
  // ... other commands
}
```

### Testing Strategy
- **Frontend Tests**: Vitest for React components (standard web testing)
- **Backend Tests**: Standard Rust unit and integration tests
- **No Tauri-specific Tests**: Since it's just an HTTP server wrapper

### Debugging
- **Frontend**: Standard browser dev tools (Chrome/Firefox/Safari)
- **Backend**: Standard Rust debugging with `tracing` logs
- **Network**: Browser network tab shows all API calls
- **No Special Tools**: Works like debugging any web application

## Related Documentation

- **[Frontend Architecture](frontend-architecture.md)** - React frontend details
- **[Backend Integration](backend-integration.md)** - API integration patterns
- **[Authentication](authentication.md)** - Security implementation
- **[App Overview](app-overview.md)** - High-level system architecture

---

*This architecture enables Bodhi to deliver a native desktop experience while leveraging modern web technologies for rapid development and rich user interfaces.*
