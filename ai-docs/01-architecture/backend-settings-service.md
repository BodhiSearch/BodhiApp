# Backend Settings Service Architecture

## Overview

The BodhiApp settings service provides a sophisticated, cascaded configuration management system that supports multiple setting sources, environment-specific configurations, and runtime updates. The architecture enables flexible configuration management while maintaining security and type safety.

## Core Architecture

### Setting Categories

The settings system organizes configuration into four distinct categories:

**System Settings** (Immutable, baked into system):
- `BODHI_ENV_TYPE`, `BODHI_APP_TYPE`, `BODHI_VERSION`
- `BODHI_AUTH_URL`, `BODHI_AUTH_REALM`, `BODHI_HOME`
- Set during application initialization, cannot be overridden

**App Settings** (Configurable via settings.yaml and API):
- `BODHI_LOGS`, `BODHI_LOG_LEVEL`, `BODHI_LOG_STDOUT`
- `BODHI_SCHEME`, `BODHI_HOST`, `BODHI_PORT`
- `BODHI_EXEC_VARIANT`, `BODHI_EXEC_LOOKUP_PATH`, `BODHI_KEEP_ALIVE_SECS`
- `HF_HOME` (Hugging Face cache directory)

**Environment Settings** (Environment variables only):
- Standard environment variables that can override defaults
- Loaded from system environment and `.env` files

**Sensitive Settings** (Environment variables only, not persisted):
- `BODHI_ENCRYPTION_KEY` - Encryption key for secrets service
- `BODHI_DEV_PROXY_UI` - Development mode UI proxy settings
- Only accessible via environment variables, never stored in files

### Setting Sources & Precedence

The cascading priority system resolves settings in this order (highest to lowest):

1. **System** - Immutable system settings
2. **CommandLine** - Runtime command-line overrides
3. **Environment** - Environment variables and `.env` files
4. **SettingsFile** - `settings.yaml` configuration
5. **Default** - Built-in default values

<augment_code_snippet path="crates/services/src/setting_service.rs" mode="EXCERPT">
````rust
fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource) {
  if let Some(setting) = self.system_settings.iter().find(|s| s.key == key) {
    return (Some(setting.value.clone()), SettingSource::System);
  }

  let result = self.with_cmd_lines_read_lock(|cmd_lines| cmd_lines.get(key).cloned());
  if let Some(value) = result {
    return (Some(value), SettingSource::CommandLine);
  }
  if let Ok(value) = self.env_wrapper.var(key) {
    let value = metadata.parse(Value::String(value));
    return (Some(value), SettingSource::Environment);
  }
  // ... continues with SettingsFile and Default sources
}
````
</augment_code_snippet>

## Core Components

### SettingService Trait

The `SettingService` trait defines the interface for configuration management:

<augment_code_snippet path="crates/services/src/setting_service.rs" mode="EXCERPT">
````rust
pub trait SettingService: std::fmt::Debug + Send + Sync {
  fn load(&self, path: &Path);
  fn load_default_env(&self);
  fn get_setting(&self, key: &str) -> Option<String>;
  fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource);
  fn set_setting_with_source(&self, key: &str, value: &Value, source: SettingSource);
  fn delete_setting(&self, key: &str) -> Result<()>;
  fn add_listener(&self, listener: Arc<dyn SettingsChangeListener>);
  // ... convenience methods for typed access
}
````
</augment_code_snippet>

### DefaultSettingService Implementation

The
`DefaultSettingService` provides thread-safe configuration management with file-based persistence:

<augment_code_snippet path="crates/services/src/setting_service.rs" mode="EXCERPT">
````rust
pub struct DefaultSettingService {
  env_wrapper: Arc<dyn EnvWrapper>,
  path: PathBuf,                    // settings.yaml path
  system_settings: Vec<Setting>,    // Immutable system settings
  settings_lock: RwLock<()>,        // File access synchronization
  defaults: RwLock<HashMap<String, Value>>,
  listeners: RwLock<Vec<Arc<dyn SettingsChangeListener>>>,
  cmd_lines: RwLock<HashMap<String, Value>>,
}
````
</augment_code_snippet>

## Configuration Files

### settings.yaml Structure

The `settings.yaml` file in `$BODHI_HOME` stores user-configurable settings:

```yaml
BODHI_HOST: localhost
BODHI_PORT: 1135
BODHI_LOG_LEVEL: info
BODHI_LOG_STDOUT: true
BODHI_EXEC_VARIANT: metal
BODHI_KEEP_ALIVE_SECS: 600
```

### .env File Support

The service automatically loads `.env` files from `$BODHI_HOME`:

<augment_code_snippet path="crates/services/src/setting_service.rs" mode="EXCERPT">
````rust
fn load_default_env(&self) {
  let bodhi_home = self.get_setting(BODHI_HOME).expect("BODHI_HOME should be set");
  let env_file = PathBuf::from(bodhi_home).join(".env");
  if env_file.exists() {
    self.load(&env_file);
  }
}
````
</augment_code_snippet>

## Environment-Specific Features

### Developer Mode Settings

Settings that only function in non-production environments:

<augment_code_snippet path="crates/services/src/setting_service.rs" mode="EXCERPT">
````rust
#[cfg(debug_assertions)]
fn get_dev_env(&self, key: &str) -> Option<String> {
  SettingService::get_env(self, key)
}

#[cfg(not(debug_assertions))]
fn get_dev_env(&self, _key: &str) -> Option<String> {
  None
}
````
</augment_code_snippet>

### Production vs Development Configuration

Environment-specific constants are defined per build target:

<augment_code_snippet path="crates/bodhi/src-tauri/src/env.rs" mode="EXCERPT">
````rust
#[cfg(feature = "production")]
mod env_config {
  pub static ENV_TYPE: EnvType = EnvType::Production;
  pub static AUTH_URL: &str = "https://id.getbodhi.app";
}

#[cfg(not(feature = "production"))]
mod env_config {
  pub static ENV_TYPE: EnvType = EnvType::Development;
  pub static AUTH_URL: &str = "https://dev-id.getbodhi.app";
}
````
</augment_code_snippet>

## API Integration

### REST Endpoints

Settings are exposed via authenticated REST API endpoints:

<augment_code_snippet path="crates/routes_app/src/routes_settings.rs" mode="EXCERPT">
````rust
// GET /v1/bodhi/settings - List all settings
pub async fn list_settings_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<Vec<SettingInfo>>, ApiError>

// PUT /v1/bodhi/settings/{key} - Update setting
pub async fn update_setting_handler(
  Path(key): Path<String>,
  Json(payload): Json<UpdateSettingRequest>,
) -> Result<Json<SettingInfo>, ApiError>

// DELETE /v1/bodhi/settings/{key} - Reset to default
pub async fn delete_setting_handler(
  Path(key): Path<String>,
) -> Result<Json<SettingInfo>, ApiError>
````
</augment_code_snippet>

### API Security & Restrictions

- All settings endpoints require admin role authentication
- Only specific settings are editable via API: `BODHI_EXEC_VARIANT`, `BODHI_KEEP_ALIVE_SECS`
- `BODHI_HOME` cannot be modified via API for security
- Settings validation occurs against metadata constraints

## Testing Infrastructure

### SettingServiceStub for Testing

<augment_code_snippet path="crates/services/src/test_utils/envs.rs" mode="EXCERPT">
````rust
pub struct SettingServiceStub {
  settings: Arc<RwLock<HashMap<String, serde_yaml::Value>>>,
  envs: HashMap<String, String>,
}

impl SettingServiceStub {
  pub fn new_with_env(envs: HashMap<String, String>, settings: HashMap<String, String>) -> Self
  pub fn with_settings(self, settings: HashMap<String, String>) -> Self
}
````
</augment_code_snippet>

### EnvWrapperStub for Environment Testing

<augment_code_snippet path="crates/services/src/test_utils/envs.rs" mode="EXCERPT">
````rust
pub struct EnvWrapperStub {
  envs: Arc<RwLock<HashMap<String, String>>>,
  temp_dir: TempDir,
}

impl EnvWrapper for EnvWrapperStub {
  fn var(&self, key: &str) -> Result<String, VarError>
  fn home_dir(&self) -> Option<PathBuf>
  fn load(&self, _path: &Path)
}
````
</augment_code_snippet>

## Integration Points

### Service Builder Integration

Settings service is initialized early in the application lifecycle:

<augment_code_snippet path="crates/lib_bodhiserver/src/app_dirs_builder.rs" mode="EXCERPT">
````rust
fn setup_settings(
  options: &AppOptions,
  bodhi_home: PathBuf,
  source: SettingSource,
) -> Result<DefaultSettingService, AppDirsBuilderError> {
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let app_settings = build_system_settings(options);
  let setting_service = DefaultSettingService::new_with_defaults(
    options.env_wrapper.clone(),
    bodhi_home_setting,
    app_settings,
    settings_file,
  );
  setting_service.load_default_env();
  Ok(setting_service)
}
````
</augment_code_snippet>

### Change Notification System

Settings support change listeners for reactive updates:

<augment_code_snippet path="crates/services/src/setting_service.rs" mode="EXCERPT">
````rust
pub trait SettingsChangeListener: std::fmt::Debug + Send + Sync {
  fn on_change(
    &self,
    key: &str,
    prev_value: &Option<Value>,
    prev_source: &SettingSource,
    new_value: &Option<Value>,
    new_source: &SettingSource,
  );
}
````
</augment_code_snippet>

## Best Practices

### Usage Patterns

1. **Initialization**: Set system settings during app startup
2. **Runtime Access**: Use typed convenience methods (`port()`, `log_level()`)
3. **Configuration**: Store user preferences in `settings.yaml`
4. **Secrets**: Use environment variables for sensitive data
5. **Testing**: Use stubs for isolated unit testing

### Security Considerations

- Sensitive settings (encryption keys) only via environment variables
- API access restricted to admin users
- Settings validation prevents invalid configurations
- File-based settings use atomic writes with file locking

### Performance Optimization

- Thread-safe concurrent access with RwLock
- Cached default values to avoid repeated computation
- Lazy loading of configuration files
- Efficient cascading lookup with early returns

---

*Reference: `crates/services/src/setting_service.rs:1-717`,
`crates/routes_app/src/routes_settings.rs:1-239`,
`crates/lib_bodhiserver/src/app_dirs_builder.rs:59-81`*
