# Phase 0: Keycloak Requirements & SPI Contract

## Purpose

Verify that Keycloak changes are deployed and working before starting BodhiApp implementation. This phase is primarily **validation** — the KC team owns the implementation.

## Prerequisites

**Execute first, before any BodhiApp changes.**

## Reference Documents

- **KC Integration Doc**: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext/ai-docs/claude-plans/20260210-access-request-doc-integration.md`
- **Context Archive**: `ai-docs/claude-plans/20260210-ctx-access-request.md`

## Key Requirements

### 1. Dynamic Scope Configuration (Already Configured)
- `scope_access_request` is an optional dynamic scope at realm level
- Format: `scope_access_request:<access-request-uuid>`
- All clients already have access to this scope

### 2. KC SPI: Register Access Request Consent

**Endpoint**: `POST {auth_url}/realms/{realm}/bodhi/users/request-access`

> **Critical**: Uses `/users/` path (not `/resources/`). Requires **user token** from resource client session, NOT service account token.

**Authentication**: Bearer token = user's access token from resource client session

**Request Body**:
```json
{
  "app_client_id": "app-abc123def456",
  "access_request_id": "550e8400-e29b-41d4-a716-446655440000",
  "description": "- Exa Web Search\n- ..."
}
```

**KC extracts from token + request**:
- **resource-client**: from token's audience
- **app-client**: from `app_client_id` field (must be public client)
- **user-uuid**: from token's `sub` claim
- Stores `access_request_id` + `description` in KC table for consent display
- Adds `scope_resource-*` to app client if not already added

**Response (201 Created)** — first registration:
```json
{
  "scope": "scope_resource-xyz789abc",
  "access_request_id": "550e8400-e29b-41d4-a716-446655440000",
  "access_request_scope": "scope_access_request:550e8400-e29b-41d4-a716-446655440000"
}
```

**Response (200 OK)** — idempotent retry:
```json
{
  "scope": "scope_resource-xyz789abc",
  "access_request_id": "550e8400-e29b-41d4-a716-446655440000",
  "access_request_scope": "scope_access_request:550e8400-e29b-41d4-a716-446655440000"
}
```

**Error Responses**:

| Status | Error | Reason |
|--------|-------|--------|
| 400 | `"access_request_id is required"` | Missing UUID |
| 400 | `"description is required"` | Missing description |
| 400 | `"App client not found"` | Invalid `app_client_id` |
| 400 | `"Only public app clients can request access"` | Confidential client used |
| 401 | `"invalid session"` | No/invalid bearer token |
| 401 | `"service account tokens not allowed"` | Used service account instead of user token |
| 401 | `"Token is not from a valid resource client"` | User token not from resource client |
| 409 | `"access_request_id already exists for a different context"` | UUID reused for different resource/app/user combo |

**Idempotency**: Same `access_request_id` with same context → 200 OK. Same UUID for different `resource_client_id`, `app_client_id`, or `user_id` → 409 Conflict (abort, regenerate UUID).

### 3. Token Behavior After Consent

When a client triggers OAuth flow with `scope_access_request:<uuid>`, Keycloak:
1. Looks up the registered consent description for that UUID
2. Displays it on the user consent screen (e.g., "Approved permission in App: \<uuid>")
3. If user consents, includes `access_request_id: "<uuid>"` as a claim in the issued token

### 4. Token Exchange Behavior

- When token exchange is called with a token containing `access_request_id` claim, the exchanged token MUST also include `access_request_id` claim
- Token exchange scope parameter must include `scope_access_request:<uuid>` to preserve the claim
- No additional KC changes needed — dynamic scope claims pass through token exchange

## Verification Tasks

### Manual Testing (with KC team)

1. **SPI Endpoint Accessibility**
   - Call `POST {auth_url}/realms/{realm}/bodhi/users/request-access` with valid user token
   - Verify 201 Created response with correct fields
   - Verify idempotent retry returns 200 OK with same data

2. **Error Handling**
   - Test all error cases in the table above
   - Confirm 409 Conflict for UUID collision (different context)

3. **Consent Screen Display**
   - Register an access request with description
   - Launch OAuth flow with `scope_access_request:<uuid>`
   - Verify consent screen shows the description

4. **Token Claim Inclusion**
   - Complete OAuth flow with `scope_access_request:<uuid>`
   - Decode access token
   - Verify `access_request_id` claim is present with correct UUID

5. **Token Exchange Preservation**
   - Perform token exchange with token containing `access_request_id` claim
   - Include `scope_access_request:<uuid>` in scope parameter
   - Verify exchanged token also contains `access_request_id` claim

## Acceptance Criteria

- [ ] SPI endpoint deployed at `/bodhi/users/request-access`
- [ ] 201 Created response includes `scope`, `access_request_id`, `access_request_scope`
- [ ] 200 OK response for idempotent retries
- [ ] 409 Conflict for UUID collision (different context)
- [ ] All error responses match specification
- [ ] Consent screen displays description when `scope_access_request:<uuid>` is requested
- [ ] Tokens include `access_request_id` claim after consent
- [ ] Token exchange preserves `access_request_id` claim

## Blockers for Next Phase

**Do not proceed to Phase 1 until**:
- All acceptance criteria above are verified
- KC team confirms deployment to dev environment
- Manual testing confirms all behaviors work as expected

## Notes for Sub-Agent

- This phase is mostly coordination with KC team
- Your role: verify the contract, document any deviations, report blockers
- If you discover ambiguities in the spec, ask clarifying questions
- Update this plan with actual test results and findings
