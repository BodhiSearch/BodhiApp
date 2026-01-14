# Domain Objects - Tools Feature

> Layer: `objs` crate | Status: ✅ Complete (16 tests passing)

## Implementation Note

All tool-related domain objects were consolidated into a single file `crates/objs/src/tools.rs` (344 lines) for better cohesion and maintainability.

## New Types

### ToolScope Enum

```rust
// crates/objs/src/tools.rs
#[derive(
  Debug, Clone, Copy, PartialEq, Eq, Hash,
  EnumString, strum::Display, EnumIter,
  Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ToolScope {
    #[strum(serialize = "scope_tool-builtin-exa-web-search")]
    #[serde(rename = "scope_tool-builtin-exa-web-search")]
    BuiltinExaWebSearch,
}

impl ToolScope {
    /// Extract tool scopes from space-separated scope string
    pub fn from_scope_string(scope: &str) -> Vec<Self>;
    
    /// Get corresponding tool_id for this scope
    pub fn tool_id(&self) -> &'static str;
    
    /// Get scope for a given tool_id
    pub fn scope_for_tool_id(tool_id: &str) -> Option<Self>;
    
    /// Get the scope string for OAuth authorization
    pub fn scope_string(&self) -> String;
}
```

### ToolDefinition (OpenAI format)

```rust
// crates/objs/src/tools.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,  // always "function"
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}
```

### UserToolConfig (Public API model)

```rust
// crates/objs/src/tools.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UserToolConfig {
    pub tool_id: String,           // e.g., "builtin-exa-web-search"
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Note: API key is NEVER exposed in this public model
}
```

### UserToolConfigRow (Database model)

```rust
// crates/services/src/db/objs.rs
pub struct UserToolConfigRow {
    pub id: i64,
    pub user_id: String,
    pub tool_id: String,
    pub enabled: bool,
    pub encrypted_api_key: Option<String>,
    pub salt: Option<String>,
    pub nonce: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### ToolExecutionRequest/Response

```rust
// crates/objs/src/tools.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolExecutionRequest {
    pub tool_call_id: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolExecutionResponse {
    pub tool_call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

## Test Coverage

**16 tests passing** covering:
- ToolScope parsing from space-separated strings
- ToolScope serialization/deserialization (kebab-case)
- ToolScope tool_id mapping
- ToolDefinition JSON schema validation
- UserToolConfig timestamp conversions
- ToolExecution request/response round-trips
