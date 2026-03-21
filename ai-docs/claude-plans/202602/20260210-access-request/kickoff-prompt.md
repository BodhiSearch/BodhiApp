# Access Request Revamp — Kickoff

## Context

The current `/apps/request-access` endpoint is a server-to-server flow where apps register scopes with Keycloak via BodhiApp proxy. Users have no visibility into what tools an app requests access to.

**Problem**: Apps get access to toolsets without explicit user consent for specific tool instances. Users cannot review, narrow, or deny specific tool access.

## Goal

Transform into a user-approval flow where:
1. App creates a **draft** access request specifying tool types
2. BodhiApp returns a **review URL** + **access_request_id** + **scopes**
3. User opens URL (popup or redirect), logs in if needed, reviews requested tools, maps tool types to specific instances, and approves/denies
4. After approval, BodhiApp calls a **custom Keycloak SPI** to register the consent description
5. App launches OAuth flow with dynamic scope `scope_access_request:<uuid>`
6. Token includes `access_request_id` claim — BodhiApp's middleware validates it against local DB
7. Downstream APIs use access_request_id to look up approved tool instances

**Breaking changes**: Drop `app_client_toolset_configs` table and old Keycloak-based request-access flow entirely.

## Reference Documents

- **KC Integration Doc**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext/ai-docs/claude-plans/20260210-access-request-doc-integration.md`
- **Context Archive**: `ai-docs/claude-plans/20260210-ctx-access-request.md`

## Phases Overview

| Phase | Focus | Dependencies |
|-------|-------|--------------|
| 0 | Keycloak SPI deployment verification | None (KC team) |
| 1 | Database migration + domain objects | Phase 0 complete |
| 2 | Service layer (DB repo + AuthService) | Phase 1 |
| 3 | Backend API endpoints | Phase 2 |
| 4 | Auth middleware changes | Phase 2, 3 |
| 5 | Remove old flow code | Phase 4 |
| 6 | Frontend review/approve page | Phase 3 |
| 7 | Rust unit + API tests | Phase 3, 4 |
| 8 | Frontend component tests | Phase 6 |
| 9 | E2E tests with real Keycloak | All phases |

## Sub-Agent Workflow

Each phase follows this pattern:

1. **Receive**: Kickoff prompt + specific phase plan
2. **Research**: Explore codebase, read referenced files, understand context
3. **Clarify**: Use AskUserQuestion if requirements are unclear or multiple approaches exist
4. **Refine**: Update the phase plan based on findings
5. **Implement**: Write code, tests, documentation
6. **Verify**: Run relevant tests, validate against acceptance criteria

**Important**: These phase plans are **seeds**, not complete specifications. Expect to discover details during implementation and update the plan accordingly. Plans evolve as you learn more about the codebase.

## Key Architectural Patterns

- **Error handling**: Centralized error types with localization support (see MEMORY.md)
- **Service layer**: Trait-based dependency injection with comprehensive mocking
- **Testing**: Follow existing patterns — no testing trivial constructors or derives
- **Frontend**: TypeScript with React Query, MSW v2 for API mocking
- **Auth flow**: Multi-service coordination (AuthService, SessionService, SecretService)

## Getting Started

1. Read this kickoff prompt
2. Read your assigned phase plan
3. Review referenced context documents
4. Explore relevant codebase sections
5. Ask clarifying questions if needed
6. Finalize your approach
7. Implement and test
