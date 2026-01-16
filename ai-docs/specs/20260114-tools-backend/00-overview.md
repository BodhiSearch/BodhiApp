# Tools Backend Implementation - Overview

> Status: Backend Phases 1-7.5 Complete, Phase 7.6 In Progress | Frontend Pending (Phases 8-9) | Updated: 2026-01-15

## Goal

Implement built-in tool support for Bodhi App backend, starting with Exa web search as the first tool. Design extensible architecture for future dynamic tool additions.

## Scope

### Phase 1 (This Implementation)
- Built-in Exa web search tool (`builtin-exa-web-search`)
- Tool configuration UI at `/ui/tools` and `/ui/tools/builtin-exa-web-search`
- API endpoints: `GET /bodhi/v1/tools`, `POST /bodhi/v1/tools/{tool_id}/execute`
- OAuth scope: `scope_tool-builtin-exa-web-search` for third-party app authorization
- Per-user encrypted API key storage for Exa
- **App-level tool enable/disable** (admin controls tool availability for all users)

### Future Phases
- Dynamic tool registration
- MCP server integration
- User-defined tools

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Config scope | Three-tier for OAuth | App-level (admin) + App-client (registered) + User-level (API keys) |
| Tool visibility | Show all, indicate status | Users see all tools; disabled tools shown but not configurable |
| Scope check | OAuth: 4 checks, Session: 2 checks | OAuth needs app, app-client, scope, user; Session needs app, user |
| App-level config | Local DB only | No Keycloak sync for app-level enable/disable |
| App-client cache | DB with version key | Cache Keycloak /resources/request-access response |
| UI navigation | Sidebar menu | New top-level "Tools" item in sidebar |
| Error detail | Detailed | Pass through Exa errors to LLM/frontend |
| Endpoint path | `/bodhi/v1/tools/{id}/execute` | RESTful resource with action verb |
| API timeout | 30 seconds | Balance responsiveness and query complexity |
| Result caching | None | Always fresh results |
| User reference | JWT sub claim | Store user_id directly, no FK to users table |

## Architecture

**Frontend orchestrates, backend executes** (per API contract):
1. Frontend receives `tool_calls` from LLM streaming response
2. Frontend calls `POST /bodhi/v1/tools/{tool_id}/execute` for each tool
3. Backend validates (OAuth scope OR first-party + configured), executes tool, returns result
4. Frontend sends tool results back to LLM

## Implementation Status

**✅ Completed (Phases 1-7)**:
- 49 passing tests across backend layers
- Full backend API ready for frontend integration

**✅ Completed (Phase 7.5)**:
- App-level tool enable/disable (admin controls) - 9 tests
- ~~Keycloak client scope sync~~ (removed in 7.6 - incorrect approach)
- Two-tier authorization model for session/first-party
- Total: 58 passing tests, ~3,300 lines of new/modified code

**🔄 In Progress (Phase 7.6)**:
- External app tool access via OAuth scopes
- Token exchange preserves `scope_tool-*`
- App-client tool config caching from Keycloak
- Four-tier authorization for OAuth tokens
- See [05.6-external-app-tool-access.md](./05.6-external-app-tool-access.md)

**📝 Pending (Phases 8-9)**:
- Frontend UI pages (`/ui/tools`)
- Integration and E2E tests

## Key Files (created/modified)

| Layer | Crate | Files | Status |
|-------|-------|-------|--------|
| Domain | `objs` | `tools.rs` (consolidated) | ✅ Complete |
| Database | `services` | `migrations/0007_tools_config.{up,down}.sql`, `db/service.rs` | ✅ Complete |
| Service | `services` | `tool_service.rs`, `exa_service.rs` | ✅ Complete |
| Routes | `routes_app` | `routes_tools.rs`, `tools_dto.rs` | ✅ Complete |
| Auth | `auth_middleware` | `tool_auth_middleware.rs` | ✅ Complete |
| Frontend | `bodhi` | `/ui/tools` pages, navigation | ⏳ Pending |

## Related Documents

- [01-domain-objects.md](./01-domain-objects.md) - Types and enums
- [02-database-schema.md](./02-database-schema.md) - Migration and storage
- [03-service-layer.md](./03-service-layer.md) - Business logic
- [04-routes-api.md](./04-routes-api.md) - HTTP endpoints
- [05-auth-scopes.md](./05-auth-scopes.md) - OAuth scope integration
- [05.5-app-level-tool-config.md](./05.5-app-level-tool-config.md) - App-level tool enable/disable (partially superseded by 05.6)
- [05.6-external-app-tool-access.md](./05.6-external-app-tool-access.md) - External app OAuth tool access (Phase 7.6)
- [06-exa-integration.md](./06-exa-integration.md) - Exa API specifics
- [07-ui-pages.md](./07-ui-pages.md) - Frontend pages
- [08-implementation-phases.md](./08-implementation-phases.md) - Phase tracking
- [09-keycloak-extension-contract.md](./09-keycloak-extension-contract.md) - Keycloak extension API contract
- [10-pending-items.md](./10-pending-items.md) - Security enhancements and pending requirements

## Security Considerations

### Authorization Model

| Auth Type | App Check | App-Client Check | Scope Check | User Config Check | Status |
|-----------|-----------|------------------|-------------|-------------------|--------|
| Session | ✅ | ❌ | ❌ | ✅ | ✅ Accepted (BodhiApp frontend only) |
| First-party Token | ✅ | ❌ | ❌ | ✅ | 🔴 **Pending restriction** |
| External OAuth | ✅ | ✅ | ✅ | ✅ | ✅ Fully secured |

**Session auth** is unrestricted by design - only BodhiApp's own frontend uses sessions, and users have explicitly configured their API keys.

**First-party tokens** need restriction - see [10-pending-items.md](./10-pending-items.md) for the requirement to block tool access by default for API tokens.

**External OAuth** is fully secured with 4-tier authorization.

## Open Questions

See individual spec files for open questions per layer.
