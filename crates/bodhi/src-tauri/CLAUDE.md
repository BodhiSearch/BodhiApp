# CLAUDE.md - bodhi/src-tauri

This file provides guidance to Claude Code when working with the Tauri desktop application for BodhiApp.

## Purpose

The `bodhi/src-tauri` crate implements the Tauri desktop application:

- **Desktop Application**: Cross-platform desktop app using Tauri framework
- **Native Integration**: System integration with file system, notifications, and platform APIs
- **Embedded Server**: Integrated BodhiServer for local LLM inference
- **Web UI Integration**: Hosts the Next.js frontend in a native webview
- **System Tray**: Background operation with system tray integration
- **Auto-Updates**: Automatic application updates and distribution

## Key Components

### Tauri Application

- Main application entry point with window management
- Tauri commands for frontend-to-backend communication
- Native menu system and keyboard shortcuts
- Window state management and persistence

### Embedded Server Integration

- BodhiServer integration using `lib_bodhiserver`
- Local database and file management
- Authentication service for desktop environment
- Model management and chat functionality

### System Integration

- File system access for model storage and configuration
- System notifications for background operations
- Platform-specific features (Windows, macOS, Linux)
- Deep linking and protocol handlers

### IPC Commands

- Tauri commands for chat completions
- Model management operations
- Settings and configuration management
- File operations and system integration

## Dependencies

### Tauri Framework

- `tauri` - Main Tauri framework for desktop applications
- `tauri-build` - Build-time dependencies and configuration

### Server Integration

- `lib_bodhiserver` - Embedded BodhiApp server
- `services` - Business logic services
- `objs` - Domain objects and validation

### System Integration

- `dirs` - Platform-specific directory access
- `keyring` - Secure credential storage
- `notify` - File system change monitoring

## Architecture Position

The Tauri application sits at the desktop platform layer:

- **Integrates**: Web UI with native desktop functionality
- **Manages**: System resources and native platform features
- **Embeds**: Complete BodhiApp server functionality
- **Provides**: Desktop-specific features and system integration

## Usage Patterns

### Application Initialization

```rust
use tauri::{App, Manager, State};
use lib_bodhiserver::BodhiServer;

#[derive(Clone)]
struct AppState {
    server: Arc<Mutex<Option<BodhiServer>>>,
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            server: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            initialize_server,
            chat_completion,
            list_models,
            create_model,
            get_settings,
            save_settings,
        ])
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Server Integration

```rust
#[tauri::command]
async fn initialize_server(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app_data_dir = app.path_resolver()
        .app_data_dir()
        .ok_or("Failed to get app data directory")?;

    let server = BodhiServer::builder()
        .database_url(&format!("sqlite:///{}/bodhi.db", app_data_dir.display()))
        .data_dir(&app_data_dir.join("data"))
        .enable_http(false)
        .build()
        .await
        .map_err(|e| e.to_string())?;

    server.start().await.map_err(|e| e.to_string())?;

    *state.server.lock().unwrap() = Some(server);
    Ok(())
}
```

### IPC Commands

```rust
#[tauri::command]
async fn chat_completion(
    request: ChatCompletionRequest,
    state: State<'_, AppState>,
) -> Result<ChatCompletionResponse, String> {
    let server = state.server.lock().unwrap();
    let server = server.as_ref().ok_or("Server not initialized")?;

    server.chat_completion(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_models(
    state: State<'_, AppState>,
) -> Result<Vec<Model>, String> {
    let server = state.server.lock().unwrap();
    let server = server.as_ref().ok_or("Server not initialized")?;

    server.list_models()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_model(
    request: CreateModelRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let server = state.server.lock().unwrap();
    let server = server.as_ref().ok_or("Server not initialized")?;

    server.create_model(request)
        .await
        .map_err(|e| e.to_string())
}
```

### Frontend Integration

```typescript
// In the Next.js frontend
import { invoke } from '@tauri-apps/api/tauri';

interface ChatCompletionRequest {
  model: string;
  messages: Message[];
  temperature?: number;
  maxTokens?: number;
}

export async function chatCompletion(request: ChatCompletionRequest) {
  return await invoke<ChatCompletionResponse>('chat_completion', { request });
}

export async function listModels() {
  return await invoke<Model[]>('list_models');
}

export async function createModel(request: CreateModelRequest) {
  return await invoke('create_model', { request });
}

// Usage in React components
function ChatInterface() {
  const [models, setModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    listModels().then(setModels).catch(console.error);
  }, []);

  const handleSendMessage = async (message: string) => {
    setLoading(true);
    try {
      const response = await chatCompletion({
        model: selectedModel,
        messages: [...messages, { role: 'user', content: message }],
        temperature: 0.7,
      });
      setMessages(prev => [...prev, response.choices[0].message]);
    } catch (error) {
      console.error('Chat completion failed:', error);
    }
    setLoading(false);
  };

  return (
    // ... React component JSX
  );
}
```

## System Integration

### File System Access

```rust
#[tauri::command]
async fn select_model_file() -> Result<String, String> {
    use tauri::api::dialog::FileDialogBuilder;

    let file_path = FileDialogBuilder::new()
        .add_filter("GGUF Models", &["gguf"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .await
        .ok_or("No file selected")?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn open_data_directory(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::api::shell;

    let data_dir = app.path_resolver()
        .app_data_dir()
        .ok_or("Failed to get data directory")?;

    shell::open(&app.shell_scope(), data_dir.to_string_lossy(), None)
        .map_err(|e| e.to_string())
}
```

### System Notifications

```rust
use tauri::api::notification::Notification;

#[tauri::command]
async fn notify_model_download_complete(
    app: tauri::AppHandle,
    model_name: String,
) -> Result<(), String> {
    Notification::new(&app.config().tauri.bundle.identifier)
        .title("Model Download Complete")
        .body(&format!("Model '{}' has been downloaded successfully", model_name))
        .show()
        .map_err(|e| e.to_string())
}
```

### System Tray Integration

```rust
use tauri::{CustomMenuItem, Menu, MenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent};

fn create_system_tray() -> SystemTray {
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(MenuItem::Separator)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

fn handle_system_tray_event(app: &tauri::AppHandle, event: SystemTrayEvent) {
    match event {
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
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
        _ => {}
    }
}
```

## Configuration and Settings

### Tauri Configuration

```json
// tauri.conf.json
{
  "build": {
    "beforeDevCommand": "cd ../.. && npm run dev",
    "beforeBuildCommand": "cd ../.. && npm run build",
    "devPath": "http://localhost:3000",
    "distDir": "../../out"
  },
  "package": {
    "productName": "Bodhi",
    "version": "1.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "all": true,
        "scope": ["$APPDATA/bodhi/**", "$HOME/.cache/huggingface/**"]
      },
      "dialog": {
        "all": true
      },
      "notification": {
        "all": true
      },
      "shell": {
        "all": false,
        "open": true
      }
    },
    "bundle": {
      "active": true,
      "identifier": "com.bodhi.app",
      "icon": ["icons/icon.png"],
      "targets": ["dmg", "msi", "appimage"]
    },
    "security": {
      "csp": "default-src 'self'; connect-src ipc: http://ipc.localhost"
    },
    "systemTray": {
      "iconPath": "icons/tray-icon.png"
    },
    "windows": [
      {
        "title": "Bodhi",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true
      }
    ]
  }
}
```

### Application Settings

```rust
#[derive(Serialize, Deserialize)]
struct AppSettings {
    theme: String,
    default_model: String,
    auto_update: bool,
    start_minimized: bool,
    cache_size: usize,
}

#[tauri::command]
async fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let settings_path = app.path_resolver()
        .app_config_dir()
        .ok_or("Failed to get config directory")?
        .join("settings.json");

    if settings_path.exists() {
        let content = fs::read_to_string(settings_path)
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&content)
            .map_err(|e| e.to_string())
    } else {
        Ok(AppSettings::default())
    }
}

#[tauri::command]
async fn save_settings(
    app: tauri::AppHandle,
    settings: AppSettings,
) -> Result<(), String> {
    let config_dir = app.path_resolver()
        .app_config_dir()
        .ok_or("Failed to get config directory")?;

    fs::create_dir_all(&config_dir)
        .map_err(|e| e.to_string())?;

    let settings_path = config_dir.join("settings.json");
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| e.to_string())?;

    fs::write(settings_path, content)
        .map_err(|e| e.to_string())
}
```

## Auto-Updates

### Update Configuration

```rust
use tauri::updater;

async fn check_for_updates(app: tauri::AppHandle) -> Result<(), String> {
    let update = app.updater().check().await.map_err(|e| e.to_string())?;

    if update.is_update_available() {
        update.download_and_install().await.map_err(|e| e.to_string())?;
        app.restart();
    }

    Ok(())
}

#[tauri::command]
async fn manual_update_check(app: tauri::AppHandle) -> Result<bool, String> {
    let update = app.updater().check().await.map_err(|e| e.to_string())?;
    Ok(update.is_update_available())
}
```

## Platform-Specific Features

### macOS Integration

```rust
#[cfg(target_os = "macos")]
use tauri::api::process::Command;

#[tauri::command]
#[cfg(target_os = "macos")]
async fn set_as_login_item(enable: bool) -> Result<(), String> {
    if enable {
        Command::new("osascript")
            .args(["-e", "tell application \"System Events\" to make login item at end with properties {path:\"/Applications/Bodhi.app\", hidden:false}"])
            .output()
            .await
            .map_err(|e| e.to_string())?;
    } else {
        Command::new("osascript")
            .args(["-e", "tell application \"System Events\" to delete login item \"Bodhi\""])
            .output()
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

### Windows Integration

```rust
#[cfg(target_os = "windows")]
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

#[tauri::command]
#[cfg(target_os = "windows")]
async fn set_startup(enable: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = Path::new("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run");
    let key = hkcu.open_subkey_with_flags(&path, KEY_WRITE)
        .map_err(|e| e.to_string())?;

    if enable {
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        key.set_value("Bodhi", &exe_path.to_string_lossy().as_ref())
            .map_err(|e| e.to_string())?;
    } else {
        key.delete_value("Bodhi").map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

## Development Guidelines

### Adding New Commands

1. Define Rust function with `#[tauri::command]` attribute
2. Add proper error handling and type conversion
3. Register command in `invoke_handler`
4. Add TypeScript bindings in frontend
5. Include comprehensive error handling

### State Management

- Use thread-safe state containers (Arc, Mutex)
- Initialize state in setup function
- Handle state access errors gracefully
- Clean up resources on application exit

### Security Considerations

- Validate all inputs from frontend
- Use Tauri's allowlist for restricting capabilities
- Implement proper CSP (Content Security Policy)
- Handle file system access securely

## Testing Strategy

### Unit Testing

- Test individual Tauri commands
- Mock server interactions
- Validate error handling
- Test state management

### Integration Testing

- End-to-end application testing
- Frontend-backend communication
- File system operations
- System integration features

### Platform Testing

- Test on Windows, macOS, and Linux
- Validate platform-specific features
- Test installer and update mechanisms
- Performance testing on different hardware

## Distribution

### Building for Release

```bash
# Build for current platform
npm run tauri build

# Build for specific platform
npm run tauri build -- --target x86_64-pc-windows-msvc
npm run tauri build -- --target x86_64-apple-darwin
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

### Distribution Channels

- Direct download from GitHub releases
- Platform-specific stores (Microsoft Store, Mac App Store)
- Package managers (Homebrew, Chocolatey, Snap)
- Auto-update mechanism for seamless updates

## Future Extensions

The Tauri application can be extended with:

- Plugin system for community extensions
- Advanced system integration features
- Enhanced offline capabilities
- Multi-window support for complex workflows
- Custom protocol handlers for deep linking
- Advanced security features and sandboxing
