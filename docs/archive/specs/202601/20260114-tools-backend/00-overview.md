# Toolsets Backend Implementation - Overview

> Status: Phases 1-8.1 Complete | Phase 9 In Progress | Updated: 2026-01-18

## Goal

Implement built-in toolset support for Bodhi App backend. A **toolset** is a connector (like Exa) that provides multiple **tools** (functions like search, find_similar, get_contents, answer). Design extensible architecture for future dynamic toolset additions.

## Domain Model

```
Toolset (Connector)              Tool (Function)
builtin-exa-web-search      ->   toolset__builtin-exa-web-search__search
                                 toolset__builtin-exa-web-search__find_similar
                                 toolset__builtin-exa-web-search__get_contents
                                 toolset__builtin-exa-web-search__answer
```

**Key Terminology:**
- **Toolset**: A connector/service that provides one or more tools (e.g., "Exa Web Search")
- **Tool**: An individual function within a toolset (e.g., "search", "find_similar")
- **Toolset ID**: Unique identifier for a toolset (e.g., `builtin-exa-web-search`)
- **Tool ID**: Fully qualified tool name using Claude MCP format (e.g., `toolset__builtin-exa-web-search__search`)

## Scope

### Phase 1 (This Implementation)
- Built-in Exa web search toolset (`builtin-exa-web-search`) with 4 tools:
  - `search` - Semantic web search
  - `find_similar` - Find pages similar to a URL
  - `get_contents` - Get full page contents
  - `answer` - AI-powered answers from web
- Toolset configuration UI at `/ui/toolsets` and `/ui/toolsets/edit?toolsetid=xxx`
- Chat UI integration with per-tool selection and agentic loop
- API endpoints: `GET /bodhi/v1/toolsets`, `POST /bodhi/v1/toolsets/{toolset_id}/execute/{method}`
- OAuth scope: `scope_toolset-builtin-exa-web-search` for third-party app authorization
- Per-user encrypted API key storage at toolset level (one key for all tools in toolset)
- **App-level toolset enable/disable** (admin controls toolset availability for all users)

### Future Phases
- Dynamic toolset registration
- MCP server integration
- User-defined toolsets

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Config scope | Three-tier for OAuth | App-level (admin) + App-client (registered) + User-level (API keys) |
| Toolset visibility | Show all, indicate status | Users see all toolsets; disabled toolsets shown but not configurable |
| Scope check | OAuth: 4 checks, Session: 2 checks | OAuth needs app, app-client, scope, user; Session needs app, user |
| API token access | Blocked at route level | API tokens (`bodhiapp_*`) cannot access any toolset endpoints |
| App-level config | Local DB only | No Keycloak sync for app-level enable/disable |
| App-client cache | DB with version key | Cache Keycloak /resources/request-access response with scope_id validation |
| UI navigation | Sidebar menu | New top-level "Toolsets" item in sidebar |
| Error detail | Detailed | Pass through Exa errors to LLM/frontend |
| Endpoint path | `/bodhi/v1/toolsets/{id}/execute/{method}` | RESTful resource with method in path |
| Execute request | Method in path | `POST /toolsets/{toolset_id}/execute/{method}` with params in body |
| API timeout | 30 seconds | Balance responsiveness and query complexity |
| Result caching | None | Always fresh results |
| User reference | JWT sub claim | Store user_id directly, no FK to users table |
| API key storage | Toolset level | One API key per toolset (covers all tools in that toolset) |
| OAuth list filtering | Scope-based | `GET /toolsets` returns only toolsets matching token's `scope_toolset-*` scopes |

## Architecture

**Frontend orchestrates, backend executes** (per API contract):
1. Frontend receives `tool_calls` from LLM streaming response
2. Frontend parses tool name from format `toolset__{toolset_id}__{method}` 
3. Frontend calls `POST /bodhi/v1/toolsets/{toolset_id}/execute/{method}` for each tool
4. Backend validates (OAuth scope OR session + configured), executes tool, returns result
5. Frontend sends tool results back to LLM

**Note:** API tokens (`bodhiapp_*`) are blocked from all toolset endpoints at route level.

## Implementation Status

**✅ Completed (Phases 1-7)**:
- Full backend API ready for frontend integration

**✅ Completed (Phase 7.5)**:
- App-level toolset enable/disable (admin controls)
- Two-tier authorization model for session/first-party

**✅ Completed (Phase 7.6)**:
- External app toolset access via OAuth scopes
- Token exchange preserves `scope_toolset-*`
- App-client toolset config caching from Keycloak
- Four-tier authorization for OAuth tokens
- See [05.6-external-app-toolset-access.md](./05.6-external-app-toolset-access.md)

**✅ Completed (Phase 8)**:
- Frontend UI pages (`/ui/toolsets`)
- Setup flow integration

**✅ Completed (Phase 8.1)**:
- Chat UI toolset integration with per-tool selection
- Agentic loop with parallel tool execution
- E2E tests for OAuth + toolset scope combinations

**📝 In Progress (Phase 9)**:
- Additional E2E tests with real Exa API

## Key Files (created/modified)

| Layer | Crate | Files | Status |
|-------|-------|-------|--------|
| Domain | `objs` | `toolsets.rs` (consolidated) | ✅ Complete |
| Database | `services` | `migrations/0009_toolsets_schema.{up,down}.sql`, `db/service.rs` | ✅ Complete |
| Service | `services` | `toolset_service.rs`, `exa_service.rs` | ✅ Complete |
| Routes | `routes_app` | `routes_toolsets.rs`, `toolsets_dto.rs` | ✅ Complete |
| Auth | `auth_middleware` | `toolset_auth_middleware.rs` | ✅ Complete |
| Frontend | `bodhi` | `/ui/toolsets` pages, navigation | ✅ Complete |

## Related Documents

- [01-domain-objects.md](./01-domain-objects.md) - Types and enums
- [02-database-schema.md](./02-database-schema.md) - Migration and storage
- [03-service-layer.md](./03-service-layer.md) - Business logic
- [04-routes-api.md](./04-routes-api.md) - HTTP endpoints
- [05-auth-scopes.md](./05-auth-scopes.md) - OAuth scope integration
- [05.5-app-level-toolset-config.md](./05.5-app-level-toolset-config.md) - App-level toolset enable/disable
- [05.6-external-app-toolset-access.md](./05.6-external-app-toolset-access.md) - External app OAuth toolset access
- [06-exa-integration.md](./06-exa-integration.md) - Exa API specifics
- [07-ui-pages.md](./07-ui-pages.md) - Frontend toolset configuration pages
- [07.1-ui-chat-integration.md](./07.1-ui-chat-integration.md) - Chat UI toolsets integration
- [08-implementation-phases.md](./08-implementation-phases.md) - Phase tracking
- [09-keycloak-extension-contract.md](./09-keycloak-extension-contract.md) - Keycloak extension API contract
- [09.1-keycloak-toolset-scope-transfer.md](./09.1-keycloak-toolset-scope-transfer.md) - Keycloak feature: toolset scope transfer for token exchange
- [10-pending-items.md](./10-pending-items.md) - Security enhancements and pending requirements

## Security Considerations

### Authorization Model

| Auth Type | List | Config | Execute | Status |
|-----------|------|--------|---------|--------|
| Session | All toolsets | ✅ | ✅ | ✅ Full access |
| API Token (`bodhiapp_*`) | 401 | 401 | 401 | ✅ Blocked at route level |
| External OAuth | Filtered by scope | 401 | With scope check | ✅ Scope-filtered |

**Session auth** has full access - only BodhiApp's own frontend uses sessions, and users have explicitly configured their API keys.

**API tokens** are completely blocked from all toolset endpoints at the route level. This is simpler and more secure than granular permissions.

**External OAuth** can list toolsets (filtered by `scope_toolset-*` scopes in token) and execute tools (with scope validation), but cannot access config endpoints.

## Open Questions

See individual spec files for open questions per layer.
