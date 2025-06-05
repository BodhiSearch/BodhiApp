# bodhi (src-tauri) - Tauri Desktop Application

## Overview

The `bodhi` crate (located in `crates/bodhi/src-tauri`) provides the Tauri-based desktop application for BodhiApp. It combines the Rust backend with a modern web frontend to create a native desktop experience while maintaining the full functionality of the HTTP server.

## Purpose

- **Desktop Application**: Native desktop app with system integration
- **Embedded Server**: Embedded HTTP server for local API access
- **Native Features**: OS-specific features and integrations
- **Offline Capability**: Local operation without internet dependency
- **System Tray**: System tray integration for background operation

## Key Components

### Application Core

#### Application Entry (`app.rs`)
- **Tauri App Setup**: Main Tauri application configuration
- **Window Management**: Application window lifecycle management
- **Event Handling**: Inter-process communication between frontend and backend
- **Plugin Integration**: Tauri plugin system integration

#### Main Entry Point (`main.rs`)
- **Application Bootstrap**: Application startup and initialization
- **Environment Setup**: Development vs production environment setup
- **Error Handling**: Top-level error handling and crash reporting
- **Logging Configuration**: Application logging setup

#### Library Main (`lib_main.rs`)
- **Library Interface**: Shared library interface for the application
- **Service Integration**: Integration with all backend services
- **Configuration Management**: Application configuration and settings
- **Feature Flags**: Conditional feature compilation

### Native Integration

#### Native Features (`native.rs`)
- **OS Integration**: Operating system specific features
- **File System Access**: Native file system operations
- **System Notifications**: Native notification system
- **Hardware Access**: Hardware-specific functionality

#### UI Integration (`ui.rs`)
- **Frontend Communication**: Communication with React frontend
- **State Synchronization**: Sync backend state with frontend
- **Event Broadcasting**: Real-time event broadcasting to UI
- **Theme Integration**: Native theme and appearance integration

### Data Conversion

#### Type Conversion (`convert.rs`)
- **Tauri Commands**: Convert between Rust and JavaScript types
- **Serialization**: Custom serialization for Tauri IPC
- **Error Conversion**: Convert backend errors to frontend-compatible format
- **Data Transformation**: Transform data for frontend consumption

## Directory Structure

```
src/
├── main.rs                   # Application entry point
├── lib.rs                    # Library exports
├── lib_main.rs               # Library main interface
├── app.rs                    # Tauri application setup
├── ui.rs                     # UI integration and communication
├── native.rs                 # Native OS features
├── convert.rs                # Type conversion utilities
├── error.rs                  # Tauri-specific error handling
├── resources/                # Localization resources
│   └── en-US/
└── test_utils/               # Testing utilities
    └── mod.rs
```

## Tauri Integration

### Application Configuration
```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(|app| {
            // Initialize backend services
            let services = initialize_services()?;
            
            // Start embedded server
            let server = start_embedded_server(services).await?;
            
            // Setup app state
            app.manage(AppState::new(server));
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Tauri command handlers
            get_app_info,
            start_chat,
            load_model,
            get_settings,
            update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Tauri Commands
```rust
#[tauri::command]
async fn get_app_info(
    state: tauri::State<'_, AppState>,
) -> Result<AppInfo, TauriError> {
    let app_service = state.get_app_service();
    let info = app_service.get_app_info().await?;
    Ok(info.into())
}

#[tauri::command]
async fn start_chat(
    request: ChatRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ChatResponse, TauriError> {
    let chat_service = state.get_chat_service();
    let response = chat_service.start_chat(request.into()).await?;
    Ok(response.into())
}

#[tauri::command]
async fn load_model(
    model_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<ModelInfo, TauriError> {
    let model_service = state.get_model_service();
    let model = model_service.load_model(model_path).await?;
    Ok(model.into())
}
```

## Key Features

### Embedded Server
- **Local HTTP Server**: Full HTTP server embedded in the desktop app
- **API Access**: All HTTP APIs available locally
- **Port Management**: Dynamic port allocation and management
- **Security**: Local-only access with optional external access

### Native Desktop Features
- **System Tray**: Background operation with system tray icon
- **File Associations**: Associate model files with the application
- **Auto-Start**: Optional auto-start on system boot
- **Window Management**: Multi-window support and window state persistence

### Real-Time Communication
- **IPC Events**: Real-time communication between frontend and backend
- **Progress Updates**: Real-time progress updates for long operations
- **Status Notifications**: System status change notifications
- **Error Reporting**: Real-time error reporting and handling

### Offline Operation
- **Local Models**: Full offline operation with local models
- **No Internet Required**: Core functionality works without internet
- **Local Storage**: All data stored locally
- **Privacy**: No data sent to external servers

## Frontend Integration

### React Frontend Communication
```typescript
// Frontend TypeScript integration
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

// Call Tauri commands
const appInfo = await invoke<AppInfo>('get_app_info');

const chatResponse = await invoke<ChatResponse>('start_chat', {
  request: {
    model: 'my-model',
    messages: [{ role: 'user', content: 'Hello!' }]
  }
});

// Listen to backend events
await listen<ProgressUpdate>('model-download-progress', (event) => {
  console.log('Download progress:', event.payload.progress);
});
```

### Event System
```rust
// Backend event emission
#[tauri::command]
async fn download_model(
    model_id: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), TauriError> {
    let hub_service = state.get_hub_service();
    
    let mut progress_stream = hub_service.download_model(model_id).await?;
    
    while let Some(progress) = progress_stream.next().await {
        app.emit_all("model-download-progress", &progress)?;
    }
    
    Ok(())
}
```

## Configuration Management

### Application Settings
```rust
#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub default_model: Option<String>,
    pub server_port: Option<u16>,
    pub enable_external_access: bool,
}

impl AppSettings {
    pub fn load() -> Result<Self, SettingsError> {
        // Load from app data directory
    }
    
    pub fn save(&self) -> Result<(), SettingsError> {
        // Save to app data directory
    }
}
```

### Environment Configuration
```rust
#[cfg(feature = "production")]
mod env_config {
    pub static ENV_TYPE: EnvType = EnvType::Production;
    pub static AUTH_URL: &str = "https://id.getbodhi.app";
    pub static AUTH_REALM: &str = "bodhi";
}

#[cfg(not(feature = "production"))]
mod env_config {
    pub static ENV_TYPE: EnvType = EnvType::Development;
    pub static AUTH_URL: &str = "http://localhost:8081";
    pub static AUTH_REALM: &str = "bodhi-dev";
}
```

## Dependencies

### Tauri Dependencies
- **tauri**: Core Tauri framework
- **tauri-plugin-log**: Logging plugin
- **serde**: Serialization for IPC

### Backend Integration
- **routes_all**: Complete HTTP API
- **server_app**: Embedded server functionality
- **services**: All business logic services
- **objs**: Domain objects and types

### System Integration
- **dirs**: System directory access
- **keyring**: Secure credential storage
- **webbrowser**: Browser integration

## Build Configuration

### Tauri Configuration (`tauri.conf.json`)
```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  },
  "package": {
    "productName": "Bodhi",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "fs": {
        "all": true,
        "scope": ["$APPDATA/*", "$HOME/.bodhi/*"]
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "app.getbodhi.bodhi",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ]
    },
    "security": {
      "csp": null
    },
    "windows": [
      {
        "fullscreen": false,
        "resizable": true,
        "title": "Bodhi",
        "width": 1200,
        "height": 800
      }
    ]
  }
}
```

### Cargo Features
```toml
[features]
default = ["native"]
native = ["tauri/native"]
production = []
development = []
test-utils = ["objs/test-utils", "services/test-utils"]
```

## Platform Support

### Supported Platforms
- **Windows**: Full Windows support with native features
- **macOS**: Native macOS integration with Metal GPU support
- **Linux**: Linux support with system integration

### Platform-Specific Features
- **Windows**: Windows-specific file associations and registry integration
- **macOS**: macOS menu bar integration and native notifications
- **Linux**: Linux desktop environment integration

## Security Considerations

### Tauri Security
- **CSP**: Content Security Policy configuration
- **Allowlist**: Restricted API access for security
- **IPC Validation**: Input validation for all Tauri commands
- **File System Access**: Scoped file system access

### Local Security
- **Local-Only Server**: Server bound to localhost by default
- **Secure Storage**: Sensitive data stored in OS keyring
- **Process Isolation**: Proper process isolation and sandboxing

## Testing Support

### Tauri Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{mock_app, MockRuntime};
    
    #[tokio::test]
    async fn test_tauri_command() {
        let app = mock_app();
        let result = get_app_info(app.state()).await;
        assert!(result.is_ok());
    }
}
```

### Integration Testing
- **Frontend-Backend Integration**: Test complete IPC communication
- **Native Feature Testing**: Test OS-specific features
- **Performance Testing**: Test desktop app performance
- **UI Testing**: Automated UI testing with WebDriver

## Future Extensions

The bodhi Tauri crate is designed to support:
- **Plugin System**: Extensible plugin architecture
- **Multi-Window Support**: Advanced multi-window management
- **Advanced Native Integration**: Deeper OS integration
- **Auto-Update**: Automatic application updates
- **Enhanced Security**: Advanced security features and sandboxing
