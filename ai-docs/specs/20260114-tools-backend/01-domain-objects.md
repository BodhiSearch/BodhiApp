# Domain Objects - Tools Feature

> Layer: `objs` crate | Status: Planning

## New Types

### ToolScope Enum

```rust
// crates/objs/src/tool_scope.rs
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ToolScope {
    #[strum(serialize = "scope_tools-builtin-exa-web-search")]
    #[serde(rename = "scope_tools-builtin-exa-web-search")]
    BuiltinExaWebSearch,
    // Future: more tools
}
```

### ToolDefinition (OpenAI format)

```rust
// crates/objs/src/tool_definition.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,  // always "function"
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}
```

### ToolConfig (DB model)

```rust
// crates/objs/src/tool_config.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolConfig {
    pub tool_id: String,           // e.g., "builtin-exa-web-search"
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### ToolExecutionRequest/Response

```rust
// crates/objs/src/tool_execution.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolExecutionRequest {
    pub tool_call_id: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolExecutionResponse {
    pub tool_call_id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}
```

## Open Questions

- [ANSWERED] ToolScope naming: `scope_tools-builtin-exa-web-search` vs `scope_tool_exa_search`?
  - Decision: Use kebab-case with `scope_tools-` prefix for consistency
