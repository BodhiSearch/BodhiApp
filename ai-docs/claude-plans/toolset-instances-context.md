# Toolset Multi-Instance: Implementation Context & Constraints

This file captures all design decisions, constraints, and preferences from the planning conversation. AI coding assistants should follow these guidelines strictly during implementation.

---

## Migration & Compatibility

| Constraint | Details |
|------------|---------|
| **No backwards compatibility** | No fallback code, no delta migrations, no deprecated APIs |
| **Modify migrations in-place** | Edit `0007_toolsets_config.up.sql` directly, do not create new migration files |
| **Rename objects as appropriate** | `toolset_id` → `toolset_type`, update all references |
| **Clean break** | Remove old patterns entirely, no parallel endpoints |

---

## Database Schema

| Decision | Details |
|----------|---------|
| **Primary key** | `id TEXT` (UUID), not `INTEGER AUTOINCREMENT` |
| **Unique constraint** | `UNIQUE(user_id, name)` - instance name unique per user |
| **Keep `app_toolset_configs`** | No changes to admin table, controls type-level enable/disable |
| **New columns** | `name TEXT NOT NULL`, `description TEXT` (nullable, max 255 chars) |
| **Rename column** | `toolset_id` → `toolset_type` |

---

## Instance Naming

| Rule | Details |
|------|---------|
| **Character set** | Alphanumeric + hyphens only: `^[a-zA-Z0-9-]+$` |
| **Max length** | 64 characters |
| **Uniqueness scope** | Per-user (different users can have same instance name) |
| **Name recycling** | Allowed - after deleting, name can be reused |
| **First instance prefill** | Pre-populate with `toolset_type` only if user has no instances of that type |
| **No reserved names** | No special "default" instance concept |

---

## Instance Lifecycle

| Behavior | Details |
|----------|---------|
| **Creation** | Created when user first configures, not auto-created |
| **All instances equal** | No special handling for first/default instance |
| **Delete allowed** | Any instance can be deleted, including first one |
| **Edit allowed** | Any instance can be edited at any time |
| **API key required on create** | Cannot save instance without API key |

---

## API Design

| Aspect | Decision |
|--------|----------|
| **Identifier in URLs** | UUID (`/toolsets/{uuid}`) |
| **REST style** | `GET/POST /toolsets`, `GET/PUT/DELETE /toolsets/{uuid}` |
| **Execute path** | `POST /toolsets/{uuid}/execute/{method}` |
| **Type admin paths** | `/toolsets/types`, `/toolsets/types/{type_id}/app-config` |
| **Response format** | Flat with `toolset_type` field, client groups on frontend |
| **Tools in response** | Include `tools[]` array in instance responses |
| **Partial updates** | PUT accepts partial body (like aliases pattern) |
| **Error messages** | Specific errors: "Instance name 'X' already exists" |

---

## Authorization Model

| Level | Scope |
|-------|-------|
| **OAuth scope** | Type-level: `scope_toolset-builtin-exa-web-search` |
| **Scope grants** | Access to ALL instances of that toolset type |
| **App-level control** | Per toolset TYPE, not per instance |
| **Instance-level** | User enable/disable per instance |

### Auth Flow for Execute (in order)
1. Resolve UUID to instance (validates existence)
2. Verify user owns instance
3. Get `toolset_type` from instance
4. Check app-level type enabled
5. Check instance enabled by user
6. Check API key configured

---

## List Endpoint Filtering

| Auth Type | Behavior |
|-----------|----------|
| **Session auth** | Return ALL user's instances |
| **OAuth token** | Filter by `toolset_type` matching scopes in token |
| **API token** | No toolset access |

---

## Tool Encoding for LLM

| Aspect | Format |
|--------|--------|
| **Tool name format** | `toolset_{instance_name}__{method}` |
| **Example** | `toolset_my-exa__search` |
| **Uses instance name** | Human-readable in LLM context |
| **Frontend resolves** | Cache name→UUID mapping at chat start |

---

## Chat Integration

| Behavior | Details |
|----------|---------|
| **State structure** | `Record<instanceId, toolNames[]>` (keyed by UUID) |
| **Multiple instances** | User can select multiple instances of same type |
| **Default selection** | First by `created_at ASC` for each type |
| **Name→UUID cache** | Build mapping when chat initializes |
| **Persistence** | Global default + per-conversation override |

---

## Frontend Pages

| Route | Purpose | Access |
|-------|---------|--------|
| `/ui/toolsets` | User instance list | All users |
| `/ui/toolsets/new` | Create instance | All users |
| `/ui/toolsets/edit?id={uuid}` | Edit instance | All users |
| `/ui/toolsets/admin` | Type enable/disable | Admin only |

### Admin Access Control
- Follow `AppInitializer` pattern for role-based rendering
- Non-admin users don't see admin tab/route
- Direct access to `/ui/toolsets/admin` by non-admin: no access (follow existing pattern)

---

## UI Behaviors

| Scenario | Behavior |
|----------|----------|
| **Empty state** | "No toolsets configured. Click 'New Instance' to get started." |
| **Type dropdown** | Always show, even with single option |
| **Instance columns** | Name, Type, API Key indicator, Status, Actions |
| **Disabled by admin** | Instance visible but not editable, edit action removed |
| **Edit when type disabled** | Redirect to `/ui/toolsets` |
| **Delete confirmation** | Simple: "Delete instance 'X'?" |

### Chat ToolsetsPopover
- Group instances by `toolset_type`
- Collapsible sections per type
- Collapse section if only single instance of that type
- Checkbox per tool per instance

---

## Existing Code Patterns to Follow

| Pattern | Reference |
|---------|-----------|
| **Partial update** | `routes_api_models.rs` alias update handling |
| **API key handling** | Encrypt on save, decrypt on execute, never return |
| **Admin access** | `AppInitializer.tsx` role-based rendering |
| **Error types** | `errmeta_derive` for error variants |
| **Validation** | `validator` crate with regex |

---

## Testing Requirements

| Principle | Details |
|-----------|---------|
| **No `use super::*`** | Explicit imports in test modules |
| **Assert convention** | `assert_eq!(expected, actual)` |
| **No if-else in tests** | Deterministic tests only |
| **No try-catch** | Let errors propagate |
| **Error logging** | Use `console.log` in tests only |
| **UI tests** | Use `data-testid` with `getByTestId` |

---

## Code Changes Summary

### Files to Modify In-Place
- `crates/services/migrations/0007_toolsets_config.up.sql`
- `crates/services/migrations/0007_toolsets_config.down.sql`
- `crates/services/src/db/objs.rs`
- `crates/services/src/db/service.rs`
- `crates/services/src/tool_service.rs`
- `crates/auth_middleware/src/toolset_auth_middleware.rs`
- `crates/routes_app/src/routes_toolsets.rs`
- `crates/routes_app/src/toolsets_dto.rs`
- `crates/bodhi/src/hooks/useToolsets.ts`
- `crates/bodhi/src/hooks/use-toolset-selection.ts`
- `crates/bodhi/src/hooks/use-chat.tsx`
- `crates/bodhi/src/app/ui/toolsets/page.tsx`
- `crates/bodhi/src/app/ui/toolsets/edit/page.tsx`
- `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx`

### Files to Create
- `crates/bodhi/src/app/ui/toolsets/new/page.tsx`
- `crates/bodhi/src/app/ui/toolsets/admin/page.tsx`
- `crates/bodhi/src/lib/toolsets.ts` (type display names)

### Files to Keep Unchanged
- `app_toolset_configs` table (type-level admin control)
- `app_client_toolset_configs` table (OAuth app registration)

---

## Implementation Order (Layer by Layer)

1. **objs** - Domain types, validation, errors
2. **services** - Schema, DbService, ToolService
3. **middleware/routes** - Auth middleware, API endpoints, DTOs
4. **ui** - Hooks, pages, chat integration

Each layer should be testable independently before moving to next.
