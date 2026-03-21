# Plan: Create context.md for Access-Request Feature

## Context

The `ai-docs/claude-plans/20260210-access-request/` folder contains 32 files spanning original plans, phase documents, implementation contexts, and post-implementation refactoring notes. An AI coding assistant working on adjacent features needs a single concise entry point to understand the access-request feature without reading all 32 files. The goal is a `context.md` that is ~80% self-contained, mostly functional, with technical depth only at inter-system communication boundaries.

## What to Create

**File**: `ai-docs/claude-plans/20260210-access-request/context.md`

## Structure

### 1. YAML Frontmatter
- Tiered file loading guide (none mandatory):
  - **Tier 1** (recommended for implementation work): `phase-0-keycloak-reqs.md`, `auth-refactor-ctx.md`
  - **Tier 2** (for specific deep dives): phase plans by topic (3, 4, 6, 9)
  - **Tier 3** (historical only): kickoff-prompt.md, checklist.md, session-specific files

### 2. Implementation Status Table
- Compact table: Component | Status (done/partial/not started)
- Components: Domain Objects, Service Layer, DB Repository, API Endpoints, Auth Middleware, Frontend Review Page, Unit Tests, E2E Tests
- Based on actual codebase verification (all backend layers are complete)

### 3. Feature Overview (Functional)
- Problem statement: apps need user-consented access to specific tool/MCP instances
- Solution: draft → review → approve/deny lifecycle with Keycloak OAuth integration
- Key actors: External App, BodhiApp Backend, User (via browser), Keycloak

### 4. End-to-End Flow (Mermaid Sequence Diagram)
- Mermaid `sequenceDiagram` showing the full multi-party interaction
- Concise supporting notes for non-obvious steps (e.g., auto-approval when no resources requested)

### 5. Three Lifecycle Sections

**Section A: Draft Creation**
- App-facing: POST /bodhi/v1/apps/request-access
- Request shape (app_client_id, flow_type, redirect_uri, requested resources)
- Response shape (access_request_id, review_url, scopes)
- Auto-approval path when no resources requested

**Section B: Review & Approval**
- User-facing: GET .../review, PUT .../approve, POST .../deny
- Review page shows requested resource types, user selects instances
- Approval triggers Keycloak SPI registration
- Post-action: redirect or window.close() based on flow_type

**Section C: OAuth & Token Usage**
- App launches OAuth with scopes from status response
- Token includes `access_request_id` claim
- `access_request_auth_middleware` validates entity against approved list in DB
- Enforcement: status=approved, not expired, user_id matches, entity in approved list

### 6. Keycloak Integration (One Level Deep)
- AuthService::register_access_request_consent() abstraction
- Underlying KC API: POST /realms/{realm}/bodhi/users/request-access
- Request/response shapes (concise, not full OpenAPI)
- Key behaviors: 201 first registration, 200 idempotent retry, 409 UUID collision

### 7. Data Model (High Level)
- Access request statuses: draft → approved/denied/failed
- Requested resources: JSON with toolset_types and mcp_servers
- Approved resources: JSON with toolset instances and MCP instances (selected by user)
- Scopes: resource_scope + access_request_scope (from KC)

### 8. Key File Paths
- Specific file paths for each layer (domain, service, repository, routes, middleware, frontend)
- No crate CLAUDE.md indirection — direct paths

### 9. Reference Index (Bottom)
- Plan files grouped by topic with one-line summaries
- For AI to load on-demand when deeper context needed

## Key Decisions (from interviews)

| Decision | Choice |
|----------|--------|
| Consumer goal | Build adjacent features (API contract + data model focus) |
| Auth state | Current codebase only (post-AuthContext refactor) |
| Temporal scope | Living implementation, focus on what's done |
| KC depth | AuthService abstraction + one level deep on KC API |
| API surface | Three sections by lifecycle stage |
| Plan references | 80% standalone, references for deep dives only |
| Code references | Specific file paths |
| Flow format | Mermaid sequence diagram + concise notes |
| Data model depth | Functional: status flow, JSON structure, no SQL/Rust types |
| Error handling | Not covered (too technical) |
| Security/middleware | Enforcement exists + what's validated (brief) |
| Frontmatter | Tiered loading, none mandatory |
| Status tracking | Compact status table |

## Verification

- Read the generated `context.md` and verify:
  1. File paths reference actual existing files (verified against codebase)
  2. API endpoints match actual route registrations
  3. Status table accurately reflects implementation state
  4. Mermaid diagram renders correctly (valid syntax)
  5. Token count is reasonable for AI context window (~800-1200 lines max target, aiming for concise)
  6. No stale references to old header-based auth (pre-AuthContext refactor)
