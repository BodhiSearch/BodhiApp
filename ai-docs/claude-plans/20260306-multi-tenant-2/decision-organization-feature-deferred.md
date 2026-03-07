# Decision: Defer Keycloak Organization Creation

> **Created**: 2026-03-07
> **Status**: DECIDED — Organizations deferred to enterprise upgrade path
> **Context**: `kickoff-keycloak-spi.md`, `multi-tenant-flow-ctx.md`
> **Research**: `../20260303-multi-tenant/research/claude-research-multi-tenancy-research-corpus.md` (Section 4)

---

## Decision

**Delay Organization creation. Do NOT create Organizations alongside tenants.**

The current SPI design (`bodhi_clients` + `bodhi_clients_users` + Keycloak client roles/groups) fully covers all launch requirements. Organizations add no value until enterprise features (external IdP, email domain auto-enrollment, managed membership) are needed. Keycloak's APIs make retroactive Organization creation trivial — there is no migration penalty from deferring.

---

## What Organizations Actually Provide

| Capability | Needed at launch? | Current SPI equivalent |
|---|---|---|
| External IdP linking (SAML/OIDC SSO) | No | N/A — enterprise feature |
| Email domain auto-enrollment | No | N/A — enterprise feature |
| Managed vs unmanaged member distinction | No | N/A — all users are "unmanaged" for now |
| Org-scoped groups | No | Already have group-based roles per client (`users-{clientId}/admins` etc.) |
| `organization` token claim | No | `azp` claim + `resource_access` roles already identify tenant + permissions |
| Member invitation API | No | Role assignment via existing `/resources/assign-role` SPI endpoint |
| Org admin console views | No | Nice-to-have, not needed |

**Key insight**: Every capability Organization provides is either (a) enterprise-only or (b) already solved by the client-centric model.

---

## Why Pre-emptive Creation is Costly Without Benefit

### 1. Complexity at SPI level
Creating an Organization alongside each tenant in `POST /tenants` would mean:
- 3+ additional Keycloak Admin API calls per tenant registration (create org, add member, optionally link to client via app-level mapping)
- New failure/compensation paths in the saga
- Testing matrix expansion (org creation failures, org-member sync, org token claims)
- All for zero user-visible benefit at launch

### 2. Token payload bloat
Organization claims only appear when `scope=organization:<alias>` is requested. Pre-creating orgs means either:
- Not requesting the scope (making the org data dead weight in Keycloak), or
- Requesting it (adding unnecessary claims to every token for all users)

### 3. Schema coupling
The current `bodhi_clients` table cleanly tracks dashboard-to-resource-client relationships. Adding `kc_org_id` to this mapping prematurely couples SPI storage to a Keycloak feature that isn't being consumed.

### 4. No "migration penalty" exists
This is the decisive factor. Keycloak's Organizations API fully supports retroactive creation:

- **`POST /admin/realms/{realm}/organizations`** — create org at any time
- **`POST /admin/realms/{realm}/organizations/{orgId}/members`** — add existing users as "unmanaged" members; their client roles, sessions, groups are untouched
- **Bulk import** via `DefaultExportImportManager.importOrganizations()` — supports creating orgs with members/groups/IdPs from a `RealmRepresentation`
- **`POST /admin/realms/{realm}/organizations/{orgId}/identity-providers`** — link existing IdPs
- **User re-auth needed** only for `organization` token claims — which won't be consumed until enterprise features exist anyway

---

## Keycloak Organizations Feature Status (as of Keycloak 26.x)

Research via DeepWiki (keycloak/keycloak) confirms:

- **GA since Keycloak 26.0** (October 2024)
- **Org-scoped groups now available** — `POST /admin/realms/{realm}/organizations/{orgId}/groups` (contrary to earlier research that listed this as planned for 26.6.0)
- **Retroactive member addition confirmed** — existing realm users added as "unmanaged" members; their client roles and sessions are unaffected
- **Existing IdPs can be linked** to newly created organizations
- **Token claims require re-auth** — users must re-authenticate to receive `organization` claims, but this is a non-issue since those claims won't be consumed until enterprise features launch

---

## Recommended Upgrade Path: When Enterprise is Requested

When a customer requests enterprise features (external IdP, managed membership), the upgrade is purely additive:

```
Enterprise Upgrade SPI Endpoint (future):
POST /realms/{realm}/bodhi/tenants/{client_id}/upgrade-enterprise

Steps:
1. Look up resource_client_id in bodhi_clients
2. Create Keycloak Organization:
   POST /admin/realms/{realm}/organizations
   { name, alias, enabled: true }
3. Add all existing users (from bodhi_clients_users for this client_id) as org members:
   POST /admin/realms/{realm}/organizations/{orgId}/members
   { id: user_id }  -- for each user
4. Link external IdP (if provided):
   POST /admin/realms/{realm}/organizations/{orgId}/identity-providers
   { alias: "customer-saml-idp" }
5. Store kc_org_id in bodhi_clients or a new column
6. Update tenant tier in BodhiApp tenants table
```

**No data migration. No schema changes to existing tables. No downtime.**

The user's existing client roles, groups, and sessions are completely unaffected. They just gain the additional Organization layer for IdP linking and managed membership.

---

## Data Already Available for Migration

When the enterprise upgrade is triggered, everything needed is already in the SPI:

| Data needed for Org creation | Source |
|---|---|
| Org name/alias | `bodhi_clients.client_id` -> Keycloak ClientModel name |
| Member list | `bodhi_clients_users WHERE client_id = ?` -> all user_ids |
| Creator/admin | `bodhi_clients.created_by_user_id` |

The migration is a read-from-existing-tables -> write-to-Keycloak-org-API operation. Zero data is lost.

---

## Risk Assessment

| Risk | Pre-emptive Org creation | Deferred Org creation |
|---|---|---|
| Launch delay | Higher (more code, more tests, more failure paths) | **Lower** |
| Wasted effort if enterprise is rare | All org creation code exercised for 0 enterprise customers | **None** |
| Migration difficulty when needed | N/A | **Trivial** — Keycloak API handles it natively |
| Schema drift | Org data diverges from SPI tables | **None** — single source of truth until needed |
| Keycloak version dependency | Must handle org API quirks now | Can target whichever KC version is current when enterprise launches |

---

## Current SPI State (Confirmed)

The `keycloak-bodhi-ext` SPI codebase has:
- **Zero Organization references** — uses "tenant" and "dashboard" terminology
- **3 JPA entities**: `BodhiClientEntity` (bodhi_clients), `BodhiClientUserEntity` (bodhi_clients_users), `BodhiAccessRequestEntity` (bodhi_access_request)
- **3 client types**: `resource`, `app`, `multi-tenant`
- **Tenant endpoints**: `POST /tenants` (create), `GET /tenants` (list) — fully implemented
- **Dual-write**: Groups for JWT claims + tables for fast queries — already in place

No changes needed to support deferred Organization creation.
