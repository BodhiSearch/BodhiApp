# Phase 1: Database & Domain Objects

## Purpose

Create database schema and domain objects for the new access request flow. This establishes the foundation for service and API layers.

## Dependencies

- **Phase 0**: Keycloak SPI contract verified and deployed

## Key Changes

### 1. Database Migration

**Create**: `crates/services/migrations/0010_app_access_requests.up.sql`

**Schema**:
```sql
CREATE TABLE IF NOT EXISTS app_access_requests (
    id TEXT PRIMARY KEY,                    -- UUID (access_request_id)
    app_client_id TEXT NOT NULL,
    flow_type TEXT NOT NULL,                -- 'redirect' | 'popup'
    redirect_uri TEXT,                      -- For redirect flow only
    status TEXT NOT NULL DEFAULT 'draft',   -- 'draft' | 'approved' | 'denied'
    tools_requested TEXT NOT NULL,          -- JSON: [{"tool_type": "builtin-exa-search"}]
    tools_approved TEXT,                    -- JSON: ["<toolset-instance-uuid>", ...] (set on approval)
    user_id TEXT,                           -- NULL until user approves
    resource_scope TEXT,                    -- KC-returned "scope_resource-xyz" (set after KC call)
    access_request_scope TEXT,              -- KC-returned "scope_access_request:<uuid>" (set after KC call)
    expires_at INTEGER NOT NULL,            -- Unix timestamp, draft TTL = 10 minutes
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_app_access_requests_status ON app_access_requests(status);
CREATE INDEX idx_app_access_requests_app_client ON app_access_requests(app_client_id);

DROP TABLE IF EXISTS app_client_toolset_configs;
```

**Create**: `crates/services/migrations/0010_app_access_requests.down.sql`
```sql
DROP TABLE IF EXISTS app_access_requests;
-- Recreate app_client_toolset_configs from 0008 (if rollback needed)
```

### 2. Domain Objects

**Create**: `crates/objs/src/access_request.rs`

```rust
pub enum AppAccessRequestStatus {
    Draft,
    Approved,
    Denied,
}

pub enum AccessRequestFlowType {
    Redirect,
    Popup,
}

pub struct ToolTypeRequest {
    pub tool_type: String,  // e.g. "builtin-exa-search"
}
```

**Modify**: `crates/services/src/db/objs.rs`

Add database row type:
```rust
pub struct AppAccessRequestRow {
    pub id: String,              // UUID (access_request_id)
    pub app_client_id: String,
    pub flow_type: String,       // "redirect" | "popup"
    pub redirect_uri: Option<String>,
    pub status: String,          // "draft" | "approved" | "denied"
    pub tools_requested: String, // JSON
    pub tools_approved: Option<String>, // JSON
    pub user_id: Option<String>,
    pub resource_scope: Option<String>,         // KC-returned scope
    pub access_request_scope: Option<String>,   // KC-returned scope
    pub expires_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
```

**Modify**: `crates/services/src/lib.rs`

Update API request/response types:
```rust
pub struct AppAccessRequest {
    pub app_client_id: String,
    pub flow_type: String,        // "redirect" | "popup"
    pub redirect_uri: Option<String>,  // required if flow_type == "redirect"
    pub tools: Vec<ToolTypeRequest>,
}

pub struct AppAccessResponse {
    pub access_request_id: String,
    pub review_url: String,
    pub scopes: Vec<String>,     // ["scope_resource-<id>", "scope_access_request:<uuid>"]
}
```

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/services/migrations/0010_app_access_requests.up.sql` | Create | Create table, drop old table |
| `crates/services/migrations/0010_app_access_requests.down.sql` | Create | Rollback migration |
| `crates/objs/src/access_request.rs` | Create | Domain enums and types |
| `crates/objs/src/lib.rs` | Modify | Re-export new types |
| `crates/services/src/db/objs.rs` | Modify | Add `AppAccessRequestRow` |
| `crates/services/src/lib.rs` | Modify | Update request/response structs |

## Research Questions

1. **Migration numbering**: Verify 0010 is the next available number. Check existing migrations.
2. **JSON storage**: Confirm pattern for JSON columns (check existing examples like `toolset_configs`).
3. **Timestamp handling**: Verify `created_at`/`updated_at` patterns (check `TimeService` usage).
4. **Indexing strategy**: Are additional indexes needed for common queries (e.g., by user_id)?
5. **Down migration**: Do we need full rollback with `app_client_toolset_configs` recreation, or just drop table?

## Acceptance Criteria

- [ ] Migration files created with correct numbering
- [ ] Table schema matches specification
- [ ] Indexes created for status and app_client_id lookups
- [ ] Old `app_client_toolset_configs` table dropped
- [ ] Domain enums defined in `objs` crate
- [ ] Row struct added to `services/src/db/objs.rs`
- [ ] API request/response structs updated in `services/src/lib.rs`
- [ ] All types properly exported
- [ ] Migration runs successfully: `cargo test -p services`
- [ ] Down migration works (optional verification)

## Notes for Sub-Agent

- **Don't implement repository yet** — that's Phase 2
- Focus on schema design and type definitions
- Follow existing patterns for timestamps (see `TimeService` in MEMORY.md)
- Check if JSON serialization needs custom handling (see existing JSON columns)
- Verify migration numbering by listing `crates/services/migrations/`
- Consider whether `expires_at` should be nullable or always set
- Think about query patterns: will we need to look up by `user_id`? Add index if yes.

## Next Phase

Phase 2 will use these types to implement the repository layer and service methods.
