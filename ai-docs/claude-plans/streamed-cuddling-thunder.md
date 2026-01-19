# Multi-Instance Toolset Support Implementation Plan

## Summary

Enable users to create multiple toolset instances per toolset type (e.g., multiple Exa Web Search configurations with different API keys). Currently single instance per type per user.

## Layer-Specific Context Files

For test-driven, layer-by-layer implementation:

| Layer | File | Key Focus |
|-------|------|-----------|
| Domain Objects | [toolset-instances-objs.md](toolset-instances-objs.md) | New types, validation, error variants |
| Services | [toolset-instances-services.md](toolset-instances-services.md) | Schema, DbService, ToolService |
| Middleware/Routes | [toolset-instances-routes.md](toolset-instances-routes.md) | Auth middleware, REST API, DTOs |
| Frontend | [toolset-instances-ui.md](toolset-instances-ui.md) | Hooks, pages, chat integration |

## Retrospective: Key Design Decisions

### Instance Identification
- **UUID** for API paths (`/toolsets/{uuid}`)
- **Instance name** for LLM tool encoding (`toolset_{name}__{method}`)
- Frontend caches name→UUID mapping at chat start

### No Special Default Instance
- First instance pre-populates name with toolset_type (UX convenience only)
- All instances treated equally - no deletion restrictions

### Authorization Model
- **Type-level OAuth scope**: `scope_toolset-builtin-exa-web-search` grants access to ALL instances of that type
- **Type-level app enable/disable**: Admin controls entire type, not individual instances
- **Instance-level user enable**: User controls per-instance availability

### State Changes
- `enabledTools: Record<instanceId, toolNames[]>` (was `toolsetId`)
- Tool encoding: `toolset_{instance_name}__{method}` (was `toolset__{toolset_id}__{method}`)

### UI Structure
- `/ui/toolsets` - User instance list
- `/ui/toolsets/new` - Create instance
- `/ui/toolsets/edit?id={uuid}` - Edit instance
- `/ui/toolsets/admin` - Type-level admin config (separate route)

## Requirements Summary

| Aspect | Decision |
|--------|----------|
| Instance naming | Unique per user, alphanumeric + hyphens only |
| Default instance | No special handling, first instance name pre-populated with toolset_type |
| App-level enable/disable | Per toolset TYPE (not per instance) |
| OAuth scope | Type-level (scope_toolset-builtin-exa-web-search grants access to all instances of type) |
| API identifier | UUID in URLs, instance name for LLM tool encoding |
| Chat selection | Multiple instances of same type allowed, cache name→UUID at chat start |
| DB migration | Modify 0007 in place (no backwards compat needed) |

---

## Phase schema-migration: Database Schema Changes

**File:** `crates/services/migrations/0007_toolsets_config.up.sql`

Replace `user_toolset_configs` table:

```sql
CREATE TABLE IF NOT EXISTS user_toolset_configs (
    id TEXT PRIMARY KEY,                   -- UUID
    user_id TEXT NOT NULL,
    toolset_type TEXT NOT NULL,            -- e.g., "builtin-exa-web-search"
    name TEXT NOT NULL,                    -- alphanumeric + hyphens, unique per user
    description TEXT,                      -- max 255 chars
    enabled INTEGER NOT NULL DEFAULT 0,
    encrypted_api_key TEXT,
    salt TEXT,
    nonce TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, name)
);

CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_user_id ON user_toolset_configs(user_id);
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_toolset_type ON user_toolset_configs(toolset_type);
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_enabled_created
    ON user_toolset_configs(user_id, toolset_type, enabled, created_at);
```

Keep `app_toolset_configs` unchanged (type-level enable/disable).

---

## Phase db-objs: Database Object Updates

**File:** `crates/services/src/db/objs.rs`

Update `UserToolsetConfigRow`:
- `id: String` (UUID, primary key)
- `user_id: String`
- `toolset_type: String` (renamed from toolset_id)
- `name: String` (NEW)
- `description: Option<String>` (NEW)
- `enabled: bool`
- `encrypted_api_key`, `salt`, `nonce`, `created_at`, `updated_at`

---

## Phase db-service: Database Service Updates

**File:** `crates/services/src/db/service.rs`

Replace existing methods:

| Method | Description |
|--------|-------------|
| `get_user_toolset_config_by_id(id)` | Get by UUID |
| `get_user_toolset_config_by_name(user_id, name)` | Get by user + name |
| `create_user_toolset_config(config)` | INSERT new instance |
| `update_user_toolset_config(id, config)` | UPDATE by UUID |
| `list_user_toolset_configs(user_id)` | List all user's instances |
| `list_enabled_toolset_configs_by_type(user_id, type)` | For chat default selection |
| `delete_user_toolset_config(id)` | DELETE by UUID |

---

## Phase domain-objs: Domain Objects

**File:** `crates/objs/src/toolsets.rs`

New/updated types:

```rust
pub struct UserToolsetInstance {
  pub id: String,           // UUID
  pub name: String,
  pub toolset_type: String,
  pub description: Option<String>,
  pub enabled: bool,
  pub has_api_key: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

pub struct ToolsetInstanceWithTools {
  #[serde(flatten)]
  pub instance: UserToolsetInstance,
  pub app_enabled: bool,
  pub tools: Vec<ToolDefinition>,
}
```

---

## Phase tool-service: Tool Service Updates

**File:** `crates/services/src/tool_service.rs`

### New ToolService Methods

| Method | Description |
|--------|-------------|
| `list_user_instances(user_id)` | List instances with tools |
| `create_instance(user_id, type, name, desc, enabled, api_key)` | Create |
| `get_instance(user_id, id)` | Get by UUID (validates ownership) |
| `update_instance(user_id, id, ...)` | Partial update |
| `delete_instance(user_id, id)` | Delete |
| `execute_instance_tool(user_id, instance_id, method, request)` | Execute |

### New Error Variants

- `InstanceNotFound(String)` - 404
- `InstanceNameExists(String)` - 409 Conflict
- `InvalidInstanceName(String)` - 400 Bad Request
- `InstanceNotOwned` - 403 Forbidden

### Name Validation

Pattern: `^[a-zA-Z0-9-]+$`, max 64 chars

---

## Phase routes-api: API Routes

**File:** `crates/routes_app/src/routes_toolsets.rs`

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/toolsets` | List user instances (session: all, OAuth: filtered by scope types) |
| POST | `/toolsets` | Create instance |
| GET | `/toolsets/:id` | Get instance by UUID |
| PUT | `/toolsets/:id` | Partial update |
| DELETE | `/toolsets/:id` | Delete instance |
| POST | `/toolsets/:id/execute/:method` | Execute tool |
| GET | `/toolsets/types` | List toolset types (admin) |
| PUT | `/toolsets/types/:type_id/app-config` | Enable type (admin) |
| DELETE | `/toolsets/types/:type_id/app-config` | Disable type (admin) |

### Request DTOs

**CreateInstanceRequest:**
```json
{
  "toolset_type": "builtin-exa-web-search",
  "name": "my-exa",
  "description": "Work API key",
  "enabled": true,
  "api_key": "sk-..."
}
```

**UpdateInstanceRequest:** (partial, all optional)
```json
{
  "name": "renamed-exa",
  "description": null,  // null = clear
  "enabled": false,
  "api_key": "sk-new..."  // or null to clear
}
```

### Response DTO

```json
{
  "id": "uuid",
  "name": "my-exa",
  "toolset_type": "builtin-exa-web-search",
  "description": "Work API key",
  "enabled": true,
  "has_api_key": true,
  "app_enabled": true,
  "created_at": "2026-01-19T...",
  "updated_at": "2026-01-19T...",
  "tools": [...]
}
```

---

## Phase auth-middleware: Authorization Updates

**File:** `crates/auth_middleware/src/toolset_auth_middleware.rs`

### Execute Authorization Flow

1. Resolve UUID to instance (validates user owns it)
2. Get toolset_type from instance
3. Check app-level type enabled
4. Check instance enabled
5. Check has_api_key
6. For OAuth: check scope_toolset-{type} in token

---

## Phase frontend-hooks: Frontend Hooks

**File:** `crates/bodhi/src/hooks/useToolsets.ts`

- `useToolsetInstances()` - list user's instances
- `useCreateInstance(options)` - create
- `useUpdateInstance(options)` - update
- `useDeleteInstance(options)` - delete
- `useToolsetTypes()` - admin: list types
- `useSetAppToolsetEnabled(options)` - admin: enable type
- `useSetAppToolsetDisabled(options)` - admin: disable type

---

## Phase frontend-pages: Frontend Pages

### `/ui/toolsets` (User Instances List)

- Table: Name, Type, API Key indicator, Status, Actions
- "New Instance" button → `/ui/toolsets/new`
- Edit/Delete actions per row
- Non-admin: no type-level controls
- **Empty state**: "No toolsets configured. Click 'New Instance' to get started."

### `/ui/toolsets/new` (Create Instance)

- Dropdown: toolset type (always shown, even with single option)
- Name input (prefill with type if first instance of that type)
- Description input
- API key input (required)
- Enabled toggle (default: true)
- Save/Cancel buttons

### `/ui/toolsets/edit?id={uuid}` (Edit Instance)

- Type: read-only display
- Name, Description, API key, Enabled: editable
- Delete button with confirmation modal
- If type disabled by admin: redirect to `/ui/toolsets`

### `/ui/toolsets/admin` (Admin Type Config)

- Separate route for admins only
- Table: Type Name, Description, Status
- Enable/Disable button per type with confirmation modal
- Non-admin: no access (follow AppInitializer pattern)

---

## Phase frontend-chat: Chat Integration

### Tool Name Encoding

Format: `toolset_{instance_name}__{method}`

Example: `toolset_my-exa__search`

### State Structure

```typescript
enabledTools: Record<instanceId, toolNames[]>
// Example:
{
  "uuid-1": ["search", "contents"],
  "uuid-2": ["search"]
}
```

### Name → UUID Mapping

Cache at chat start:
```typescript
const instanceNameToId = new Map<string, string>();
instances.forEach(i => instanceNameToId.set(i.name, i.id));
```

### Tool Execution

Parse instance name from tool call → lookup UUID → call API with UUID.

### Multiple Instances of Same Type

- Allowed (checkbox behavior)
- Default selection: first by created_at ASC for each type

### Chat ToolsetsPopover UI

- **Group by toolset_type** with collapsible sections
- Section header: "Exa Web Search" (type display name)
- Instances listed under each section with checkboxes
- Collapse section if only single instance of that type

---

## Phase openapi: OpenAPI & TypeScript Client

1. Update OpenAPI schema registration
2. Regenerate: `cargo run --package xtask openapi`
3. Regenerate TS types: `cd ts-client && npm run generate`

---

## Critical Files

| File | Changes |
|------|---------|
| `crates/services/migrations/0007_toolsets_config.up.sql` | Schema: add name, description, UUID id |
| `crates/services/migrations/0007_toolsets_config.down.sql` | Drop indexes |
| `crates/services/src/db/objs.rs` | Update UserToolsetConfigRow |
| `crates/services/src/db/service.rs` | CRUD methods |
| `crates/objs/src/toolsets.rs` | New domain types |
| `crates/services/src/tool_service.rs` | Instance management |
| `crates/routes_app/src/routes_toolsets.rs` | REST endpoints |
| `crates/routes_app/src/toolsets_dto.rs` | Request/Response DTOs |
| `crates/auth_middleware/src/toolset_auth_middleware.rs` | UUID resolution |
| `crates/bodhi/src/hooks/useToolsets.ts` | API hooks |
| `crates/bodhi/src/hooks/use-toolset-selection.ts` | Instance-based state |
| `crates/bodhi/src/hooks/use-chat.tsx` | Tool name encoding |
| `crates/bodhi/src/app/ui/toolsets/page.tsx` | Instances list |
| `crates/bodhi/src/app/ui/toolsets/new/page.tsx` | Create page (NEW) |
| `crates/bodhi/src/app/ui/toolsets/edit/page.tsx` | Edit page |
| `crates/bodhi/src/app/ui/toolsets/admin/page.tsx` | Admin page (NEW) |
| `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx` | Chat tool selection |

---

## Verification

### Backend Tests
```bash
cargo test -p services -- toolset
cargo test -p routes_app -- toolset
cargo test -p auth_middleware -- toolset
```

### Frontend Tests
```bash
cd crates/bodhi && npm test -- toolset
```

### E2E Tests
```bash
make build.ui-rebuild
cargo test -p integration-tests -- toolset
```

### Manual Testing
1. Create multiple instances of same type
2. Select instances in chat
3. Execute tool calls
4. Toggle instance enable/disable
5. Admin enable/disable type affects all instances
6. Delete instance mid-chat (verify graceful error)
