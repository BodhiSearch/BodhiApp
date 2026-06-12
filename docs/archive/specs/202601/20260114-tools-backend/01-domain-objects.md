# Domain Objects - Toolsets Feature

> Layer: `objs` crate | Status: ✅ Complete

## Implementation Note

All toolset-related domain objects are consolidated into a single file `crates/objs/src/toolsets.rs` for better cohesion and maintainability.

## Domain Model

```
Toolset (Connector)              Tool (Function)
builtin-exa-web-search      ->   toolset__builtin-exa-web-search__search
                                 toolset__builtin-exa-web-search__find_similar
                                 toolset__builtin-exa-web-search__get_contents
                                 toolset__builtin-exa-web-search__answer
```

## New Types

### ToolsetScope Enum

OAuth scope for toolset authorization. Grants access to all tools within the toolset.

```rust
// crates/objs/src/toolsets.rs
#[derive(
  Debug, Clone, Copy, PartialEq, Eq, Hash,
  EnumString, strum::Display, EnumIter,
  Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ToolsetScope {
    #[strum(serialize = "scope_toolset-builtin-exa-web-search")]
    #[serde(rename = "scope_toolset-builtin-exa-web-search")]
    BuiltinExaWebSearch,
}

impl ToolsetScope {
    /// Extract toolset scopes from space-separated scope string
    pub fn from_scope_string(scope: &str) -> Vec<Self>;
    
    /// Get corresponding toolset_id for this scope
    pub fn toolset_id(&self) -> &'static str;
    
    /// Get scope for a given toolset_id
    pub fn scope_for_toolset_id(toolset_id: &str) -> Option<Self>;
    
    /// Get the scope string for OAuth authorization
    pub fn scope_string(&self) -> String;
}
```

### ToolsetDefinition

Represents a toolset containing multiple tools.

```rust
// crates/objs/src/toolsets.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetDefinition {
    /// Unique toolset identifier (e.g., "builtin-exa-web-search")
    pub toolset_id: String,
    /// Human-readable name (e.g., "Exa Web Search")
    pub name: String,
    /// Description of the toolset
    pub description: String,
    /// Tools provided by this toolset (in OpenAI format)
    pub tools: Vec<ToolDefinition>,
}
```

### ToolDefinition (OpenAI format)

Individual tool definition in OpenAI function calling format. Tool name follows Claude MCP convention: `toolset__{toolset_id}__{tool_name}`.

```rust
// crates/objs/src/toolsets.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,  // always "function"
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct FunctionDefinition {
    /// Fully qualified tool name: toolset__{toolset_id}__{tool_name}
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}
```

### UserToolsetConfig (Public API model)

Per-user toolset configuration. API key is stored at toolset level (one key for all tools).

```rust
// crates/objs/src/toolsets.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UserToolsetConfig {
    pub toolset_id: String,           // e.g., "builtin-exa-web-search"
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Note: API key is NEVER exposed in this public model
}
```

### AppToolsetConfig (Public API model)

App-level toolset configuration (admin-controlled).

```rust
// crates/objs/src/toolsets.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AppToolsetConfig {
    pub toolset_id: String,
    pub enabled: bool,
    pub updated_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### UserToolsetConfigRow (Database model)

```rust
// crates/services/src/db/objs.rs
pub struct UserToolsetConfigRow {
    pub id: i64,
    pub user_id: String,
    pub toolset_id: String,
    pub enabled: bool,
    pub encrypted_api_key: Option<String>,
    pub salt: Option<String>,
    pub nonce: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### ToolsetExecutionRequest/Response

Request includes `tool_name` to specify which tool within the toolset to execute.

```rust
// crates/objs/src/toolsets.rs
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetExecutionRequest {
    pub tool_call_id: String,
    /// Fully qualified tool name: toolset__{toolset_id}__{tool_name}
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetExecutionResponse {
    pub tool_call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

## Exa Toolset Definition

The builtin Exa web search toolset with 4 tools:

```json
{
  "toolset_id": "builtin-exa-web-search",
  "name": "Exa Web Search",
  "description": "Search the web using Exa AI semantic search with multiple capabilities",
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "toolset__builtin-exa-web-search__search",
        "description": "Search the web for current information using Exa AI semantic search",
        "parameters": {
          "type": "object",
          "properties": {
            "query": { "type": "string", "description": "Search query" },
            "num_results": { "type": "integer", "default": 5, "maximum": 10 }
          },
          "required": ["query"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "toolset__builtin-exa-web-search__find_similar",
        "description": "Find web pages similar to a given URL",
        "parameters": {
          "type": "object",
          "properties": {
            "url": { "type": "string", "description": "URL to find similar pages for" },
            "num_results": { "type": "integer", "default": 5 },
            "exclude_source_domain": { "type": "boolean", "default": true }
          },
          "required": ["url"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "toolset__builtin-exa-web-search__get_contents",
        "description": "Get full contents of web pages by URLs",
        "parameters": {
          "type": "object",
          "properties": {
            "urls": { "type": "array", "items": { "type": "string" }, "maxItems": 10 },
            "max_characters": { "type": "integer", "default": 3000 }
          },
          "required": ["urls"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "toolset__builtin-exa-web-search__answer",
        "description": "Get AI-generated answer based on web search",
        "parameters": {
          "type": "object",
          "properties": {
            "query": { "type": "string", "description": "Question to answer" },
            "num_results": { "type": "integer", "default": 5 }
          },
          "required": ["query"]
        }
      }
    }
  ]
}
```

## Test Coverage

Tests cover:
- ToolsetScope parsing from space-separated strings
- ToolsetScope serialization/deserialization (kebab-case)
- ToolsetScope toolset_id mapping
- ToolsetDefinition with multiple tools
- ToolDefinition JSON schema validation
- UserToolsetConfig timestamp conversions
- ToolsetExecution request/response round-trips
