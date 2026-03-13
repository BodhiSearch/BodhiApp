# routes_app Technical Debt

## MCP OAuth flow duplication

- **Currently**: MCP OAuth handlers (`mcps_oauth_initiate`, `mcps_oauth_callback`, etc.) contain complex OAuth flow logic that partially duplicates auth module patterns
- **Skipped in CRUD refactor**: Known anomaly to address when standardizing OAuth patterns

## OAI/Ollama model conversion logic

- **Currently**: Model conversion between internal types and OpenAI/Ollama formats lives in routes_app
- **Keep as-is**: Ollama compatibility is being dropped soon

## Setup/Auth redirect URL construction

- **Currently**: `build_redirect_url` and host detection logic in setup/auth routes is HTTP-layer concern with complex edge cases
- **Keep in routes_app**: This is appropriately placed as an HTTP-layer concern

## Overlapping CORS path structure

- **Currently**: 3 URL paths have both session-only and non-session methods, preventing per-method CORS separation:
  - `/bodhi/v1/toolsets`: POST (session) + GET (oauth)
  - `/bodhi/v1/mcps`: POST (session) + GET (oauth)
  - `/bodhi/v1/mcps/{id}`: PUT/DELETE (session) + GET (exec/oauth)
- **Workaround**: Session methods at overlapping paths are in the permissive CORS group (auth still enforces session-only access)
- **Future**: Move external app endpoints to `/bodhi/v1/apps/{toolsets,mcps}`, making `/bodhi/v1/{toolsets,mcps}` fully session-only with restrictive CORS

## SessionService tower-sessions coupling

- **Currently**: `SessionService` in the services crate depends on `tower-sessions`, which is an axum-specific framework concern
- **Should be**: Abstracted behind a trait that doesn't leak web framework types
