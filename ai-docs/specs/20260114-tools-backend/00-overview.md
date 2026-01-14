# Tools Backend Implementation - Overview

> Status: Backend Complete (Phases 1-7) | Frontend Pending (Phases 8-9) | Updated: 2026-01-14

## Goal

Implement built-in tool support for Bodhi App backend, starting with Exa web search as the first tool. Design extensible architecture for future dynamic tool additions.

## Scope

### Phase 1 (This Implementation)
- Built-in Exa web search tool (`builtin-exa-web-search`)
- Tool configuration UI at `/ui/tools` and `/ui/tools/builtin-exa-web-search`
- API endpoints: `GET /bodhi/v1/tools`, `POST /bodhi/v1/tools/{tool_id}/execute`
- OAuth scope: `scope_tool-builtin-exa-web-search` for third-party app authorization
- Per-user encrypted API key storage for Exa

### Future Phases
- Dynamic tool registration
- MCP server integration
- User-defined tools

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Config scope | Per-user | Each user provides their own Exa API key |
| Tool visibility | Only configured | Users only see tools they've enabled with API key |
| Scope check | OAuth tokens only | First-party (session, bodhiapp_) bypass if tool configured |
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
- 49 passing tests across all backend layers
- ~2,000 lines of new code
- Full backend API ready for frontend integration

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
- [06-exa-integration.md](./06-exa-integration.md) - Exa API specifics
- [07-ui-pages.md](./07-ui-pages.md) - Frontend pages

## Open Questions

See individual spec files for open questions per layer.
