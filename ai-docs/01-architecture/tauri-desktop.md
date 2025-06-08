# Tauri Desktop Application

This document provides guidance for Tauri desktop application development, native OS integration, and desktop-specific patterns in the Bodhi App.

## Required Documentation References

**MUST READ for desktop development:**
- `ai-docs/01-architecture/system-overview.md` - System architecture and crate organization
- `ai-docs/01-architecture/rust-backend.md` - Backend service patterns and integration

**FOR FRONTEND INTEGRATION:**
- `ai-docs/01-architecture/frontend-react.md` - React component patterns for desktop UI

## Architecture Overview

The Bodhi App desktop application is built with Tauri, combining a Rust backend with a React frontend in a native desktop shell. This provides the performance and security of native applications while leveraging modern web technologies for the UI.

### Key Architectural Decision

Unlike typical Tauri applications that use IPC commands for frontend-backend communication, Bodhi runs a complete HTTP server and opens the system browser. This unique architectural choice provides several critical advantages:

- **Maximum Compatibility**: Works with any web development tools and libraries without Tauri-specific modifications
- **Standard Debugging**: Use browser developer tools normally without special Tauri debugging setup
- **API Consistency**: Same HTTP API endpoints work for both desktop and server deployments
- **Development Simplicity**: No need to learn Tauri-specific IPC patterns or command structures
- **Testing Benefits**: Standard web testing tools work without modification
- **Deployment Flexibility**: Easy to switch between desktop and server deployment modes

This approach means that **no Tauri-specific testing or debugging tools are needed** because the application is essentially a standard web application running in a native shell.

### Desktop Application Stack
```
┌─────────────────────────────────────────────────────────┐
│                 Native Desktop Shell                    │
│                    (Tauri)                             │
├─────────────────────────────────────────────────────────┤
│              WebView (React Frontend)                   │
│           React + TypeScript + Vite                    │
├─────────────────────────────────────────────────────────┤
│               Tauri Core (Rust)                        │
│          Native APIs + Backend Services                │
├─────────────────────────────────────────────────────────┤
│                Operating System                         │
│         Windows / macOS / Linux                        │
└─────────────────────────────────────────────────────────┘
```

## Project Structure

### Tauri Application Structure
```
crates/bodhi/
├── src/                    # React frontend source
│   ├── components/         # React components
│   ├── pages/             # Page components
│   └── lib/               # Frontend utilities
├── src-tauri/             # Tauri backend source
│   ├── src/               # Rust source code
│   │   ├── main.rs        # Application entry point
│   │   ├── commands.rs    # Tauri commands
│   │   ├── menu.rs        # Application menu
│   │   └── lib.rs         # Library code
│   ├── Cargo.toml         # Rust dependencies
│   ├── tauri.conf.json    # Tauri configuration
│   └── icons/             # Application icons
├── public/                # Static assets
└── package.json           # Frontend dependencies
```

### Tauri Configuration
```json
// src-tauri/tauri.conf.json
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
      "dialog": {
        "all": false,
        "open": true,
        "save": true
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "readDir": true,
        "createDir": true
      }
    }
  }
}
```

## Native Integration Patterns

### Tauri Commands
Tauri commands provide a bridge between the frontend and native Rust backend:

```rust
// src-tauri/src/commands.rs
use tauri::command;

#[command]
pub async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[command]
pub async fn open_file_dialog() -> Result<Option<String>, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;
    
    let file_path = FileDialogBuilder::new()
        .add_filter("GGUF files", &["gguf"])
        .pick_file();
    
    Ok(file_path.map(|p| p.to_string_lossy().to_string()))
}

#[command]
pub async fn save_file(path: String, contents: String) -> Result<(), String> {
    use std::fs;
    
    fs::write(path, contents)
        .map_err(|e| e.to_string())
}
```

### Frontend Command Usage
```typescript
// Frontend usage of Tauri commands
import { invoke } from '@tauri-apps/api/tauri';

// Get app version
const getAppVersion = async (): Promise<string> => {
  return await invoke('get_app_version');
};

// Open file dialog
const openFileDialog = async (): Promise<string | null> => {
  return await invoke('open_file_dialog');
};

// Save file
const saveFile = async (path: string, contents: string): Promise<void> => {
  await invoke('save_file', { path, contents });
};
```

## File System Integration

### File Operations
```rust
// src-tauri/src/commands.rs
use tauri::command;
use std::path::PathBuf;

#[command]
pub async fn read_config_file(app_handle: tauri::AppHandle) -> Result<String, String> {
    let app_dir = app_handle.path_resolver()
        .app_config_dir()
        .ok_or("Failed to get app config directory")?;
    
    let config_path = app_dir.join("config.json");
    
    match std::fs::read_to_string(config_path) {
        Ok(contents) => Ok(contents),
        Err(_) => Ok("{}".to_string()), // Return empty config if file doesn't exist
    }
}

#[command]
pub async fn write_config_file(
    app_handle: tauri::AppHandle,
    config: String
) -> Result<(), String> {
    let app_dir = app_handle.path_resolver()
        .app_config_dir()
        .ok_or("Failed to get app config directory")?;
    
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| e.to_string())?;
    
    let config_path = app_dir.join("config.json");
    
    std::fs::write(config_path, config)
        .map_err(|e| e.to_string())
}
```

### Path Resolution
```typescript
// Frontend path utilities
import { appConfigDir, join } from '@tauri-apps/api/path';

const getConfigPath = async (): Promise<string> => {
  const configDir = await appConfigDir();
  return await join(configDir, 'config.json');
};

const getModelsPath = async (): Promise<string> => {
  const configDir = await appConfigDir();
  return await join(configDir, 'models');
};
```

## Application Menu

### Menu Configuration
```rust
// src-tauri/src/menu.rs
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};

pub fn create_menu() -> Menu {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let close = CustomMenuItem::new("close".to_string(), "Close");
    let about = CustomMenuItem::new("about".to_string(), "About Bodhi");
    let preferences = CustomMenuItem::new("preferences".to_string(), "Preferences");
    
    let submenu = Submenu::new(
        "Bodhi",
        Menu::new()
            .add_item(about)
            .add_native_item(MenuItem::Separator)
            .add_item(preferences)
            .add_native_item(MenuItem::Separator)
            .add_item(quit)
    );
    
    let file_menu = Submenu::new(
        "File",
        Menu::new()
            .add_item(close)
    );
    
    Menu::new()
        .add_submenu(submenu)
        .add_submenu(file_menu)
}
```

### Menu Event Handling
```rust
// src-tauri/src/main.rs
use tauri::{Manager, WindowEvent};

fn main() {
    tauri::Builder::default()
        .menu(menu::create_menu())
        .on_menu_event(|event| {
            match event.menu_item_id() {
                "quit" => {
                    std::process::exit(0);
                }
                "close" => {
                    event.window().close().unwrap();
                }
                "about" => {
                    // Show about dialog
                }
                "preferences" => {
                    // Open preferences window
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Window Management

### Window Configuration
```rust
// src-tauri/src/main.rs
use tauri::{WindowBuilder, WindowUrl};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let main_window = WindowBuilder::new(
                app,
                "main",
                WindowUrl::App("index.html".into())
            )
            .title("Bodhi")
            .inner_size(1200.0, 800.0)
            .min_inner_size(800.0, 600.0)
            .resizable(true)
            .build()?;
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Window Events
```typescript
// Frontend window event handling
import { appWindow } from '@tauri-apps/api/window';

// Listen for window events
appWindow.listen('tauri://close-requested', () => {
  // Handle window close
  console.log('Window close requested');
});

// Window controls
const minimizeWindow = () => appWindow.minimize();
const maximizeWindow = () => appWindow.maximize();
const closeWindow = () => appWindow.close();
```

## System Integration

### Notifications
```rust
// src-tauri/src/commands.rs
use tauri::api::notification::Notification;

#[command]
pub async fn show_notification(
    app_handle: tauri::AppHandle,
    title: String,
    body: String
) -> Result<(), String> {
    Notification::new(&app_handle.config().tauri.bundle.identifier)
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}
```

### System Tray (Optional)
```rust
// src-tauri/src/main.rs
use tauri::{SystemTray, SystemTrayMenu, SystemTrayMenuItem, SystemTrayEvent};

fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show".to_string(), "Show"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));
    
    let system_tray = SystemTray::new().with_menu(tray_menu);
    
    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "show" => {
                        let window = app.get_window("main").unwrap();
                        window.show().unwrap();
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Build and Distribution

### Development Commands
```bash
# Start development server
cd crates/bodhi
npm run tauri dev

# Build for production
npm run tauri build

# Build for specific platform
npm run tauri build -- --target x86_64-pc-windows-msvc
```

### Build Configuration
```json
// src-tauri/tauri.conf.json
{
  "tauri": {
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "app.getbodhi.desktop",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "resources": [],
      "externalBin": [],
      "copyright": "",
      "category": "DeveloperTool",
      "shortDescription": "",
      "longDescription": ""
    }
  }
}
```

## Security Considerations

### Allowlist Configuration
```json
// src-tauri/tauri.conf.json
{
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "all": false,
        "scope": ["$APPCONFIG/**", "$APPDATA/**"]
      },
      "shell": {
        "all": false,
        "open": true
      },
      "dialog": {
        "all": false,
        "open": true,
        "save": true
      }
    }
  }
}
```

### Content Security Policy
```json
// src-tauri/tauri.conf.json
{
  "tauri": {
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
    }
  }
}
```

## Performance Optimization

### Bundle Size Optimization
- Use tree shaking for frontend dependencies
- Optimize Rust binary size with release profile
- Minimize included resources and assets

### Memory Management
- Implement proper cleanup in Tauri commands
- Use efficient data structures for large datasets
- Monitor memory usage in development

## Related Documentation

- **[System Overview](system-overview.md)** - High-level system architecture
- **[Rust Backend](rust-backend.md)** - Backend service patterns and integration
- **[Frontend React](frontend-react.md)** - React component patterns for desktop UI
- **[Build & Configuration](build-config.md)** - Build systems and deployment

---

*For detailed Tauri configuration and advanced patterns, see the official Tauri documentation. For backend integration, see [Rust Backend](rust-backend.md).*
