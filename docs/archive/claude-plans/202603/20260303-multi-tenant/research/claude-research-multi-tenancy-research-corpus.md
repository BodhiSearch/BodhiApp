# BodhiApp Multi-Tenancy Research Corpus

> **Purpose:** Reusable research reference for BodhiApp's SaaS multi-tenancy architecture.
> Preserves raw sources, decision rationale, open questions, and grouped resources
> so future requirement changes can build on this foundation without re-researching from scratch.
>
> **Created:** 2026-02-23
> **Last Updated:** 2026-03-03
> **Status:** Initial research complete. **Rev 2:** Architectural shift from org_id to
> tenant_id (derived UUID mapped 1:1 from Keycloak client_id) as primary discriminator.
> Organization demoted to optional enterprise layer. Decisions pending on billing model.

---

## Table of Contents

1. [Architecture Decisions Record (ADR) Summary](#1-architecture-decisions-record-adr-summary)
2. [Free-Tier Routing Research](#2-free-tier-routing-research)
3. [Cloudflare DNS & Edge Strategy Research](#3-cloudflare-dns--edge-strategy-research)
4. [Keycloak v26 Organizations Research](#4-keycloak-v26-organizations-research)
5. [PostgreSQL Row-Level Security Research](#5-postgresql-row-level-security-research)
6. [Tenant Provisioning Service Research](#6-tenant-provisioning-service-research)
7. [Self-Hosted → SaaS Migration Research](#7-self-hosted--saas-migration-research)
8. [Kubernetes Multi-Tenancy Research](#8-kubernetes-multi-tenancy-research)
9. [Billing Architecture Research](#9-billing-architecture-research)
10. [Industry Benchmarks & Comparable Products](#10-industry-benchmarks--comparable-products)
11. [Open Questions & Future Research](#11-open-questions--future-research)
12. [Master Reference Index](#12-master-reference-index)

---

## 1. Architecture Decisions Record (ADR) Summary

Captures all decisions made during the interview process. Each ADR links to the
relevant research section for full context.

### ADR-001: Data Isolation Strategy
- **Decision:** Shared DB, tenant discriminator column (`tenant_id` UUID on every row, mapped 1:1 from Keycloak `client_id`)
- **Status:** DECIDED (Rev 2 — changed from `org_id` to `tenant_id` derived from `client_id`)
- **Rationale:** `client_id` is already the security boundary in every JWT (`azp` claim), roles are scoped to it, redirect URIs are tied to it, and it's the same identity primitive in both self-hosted and SaaS modes. Using a derived `tenant_id` UUID decouples the DB schema from Keycloak's string-format `clientId`, so client renames don't require data migration.
- **Alternatives rejected:** `org_id` as discriminator (premature abstraction — Organization is an enterprise feature, not the universal tenant identity), Keycloak internal client UUID (opaque, not in JWT by default), `clientId` string directly (couples DB to Keycloak naming convention).
- **Related research:** [§5 PostgreSQL RLS](#5-postgresql-row-level-security-research)

### ADR-002: Compute Isolation Model
- **Decision:** Stateless Kubernetes pods in a shared pool, requests routed by tenant context from JWT `azp` claim
- **Status:** DECIDED (Rev 2 — simplified from org context to `azp` claim)
- **Rationale:** Eliminates per-tenant container orchestration. AI inference is stateless (model loaded into shared GPU/CPU pods). `azp` (authorized party = client_id) is in every JWT by default — no extra scope needed for tenant identification.
- **Related research:** [§8 Kubernetes](#8-kubernetes-multi-tenancy-research)

### ADR-003: Keycloak Realm Topology
- **Decision:** Single realm. Keycloak **Clients** are the primary tenant identity (1 client = 1 tenant). **Organizations** are an optional enterprise layer for IdP linking and managed membership.
- **Status:** DECIDED (Rev 2 — Organizations demoted from co-equal to optional enterprise layer)
- **Rationale:** Single realm = single SSO session, single user pool, simplified operations. The Client is the universal tenant primitive that works identically in self-hosted and SaaS. Organization adds enterprise features (external IdP linking, email domain matching, managed members) but is not required for tenant isolation or routing.
- **Risk:** Single realm becomes a bottleneck at extreme scale (100K+ users). Mitigation: Keycloak clustering + infinispan cache tuning.
- **Related research:** [§4 Keycloak](#4-keycloak-v26-organizations-research)

### ADR-004: Free-Tier Routing
- **Decision:** Shared domain (`app.getbodhi.ai`) with in-app org switcher
- **Status:** RECOMMENDED (pending final confirmation)
- **Rationale:** Eliminates DNS/cert overhead for 99% of tenants, follows industry standard (GitHub, Notion, Vercel pattern), simplifies Keycloak to single redirect URI for free tier.
- **Related research:** [§2 Free-Tier Routing](#2-free-tier-routing-research)

### ADR-005: Tenant Context Propagation
- **Decision:** JWT `azp` claim (= Keycloak `client_id`) as primary tenant identifier. Client-level roles under `resource_access.<client-id>.roles` as authorization source. `organization` scope is **only requested for enterprise features** (IdP-linked login, managed membership), not for basic tenant scoping.
- **Status:** DECIDED (Rev 2 — simplified from `organization` scope to `azp` claim)
- **Rationale:** `azp` is in every OIDC token by default — zero additional Keycloak configuration. Removes dependency on Organization scope for basic multi-tenancy. Clean trust boundary: app reads `azp` → maps to `tenant_id` UUID → uses for RLS and authorization. Organization scope only adds value for enterprise features.
- **Auth middleware flow:**
  1. Extract `azp` from JWT → this is the Keycloak `client_id` string
  2. Lookup `tenants` table: `SELECT tenant_id FROM tenants WHERE kc_client_id = azp`
  3. `SET LOCAL app.current_tenant = tenant_id` for RLS
  4. Extract roles from `resource_access.<azp>.roles` for authorization
- **Related research:** [§4 Keycloak](#4-keycloak-v26-organizations-research), [§5 RLS](#5-postgresql-row-level-security-research)

### ADR-006: Self-Hosted ↔ SaaS Migration
- **Decision:** Bidirectional migration supported (self-hosted → SaaS primary path)
- **Status:** DECIDED
- **Self-hosted DB:** SQLite
- **SaaS DB:** PostgreSQL
- **Bridge:** JSON export format + pgloader for schema migration
- **Related research:** [§7 Migration Research](#7-self-hosted--saas-migration-research)

### ADR-007: Tenant Data Enforcement
- **Decision:** Defense in depth — app-layer filtering + PostgreSQL RLS
- **Status:** DECIDED (Rev 2 — column changed from `org_id` to `tenant_id`)
- **Pattern:** `SET LOCAL app.current_tenant = <tenant_id_uuid>` within transactions, RLS policies reference `current_setting('app.current_tenant', true)`. The `tenant_id` is a UUID derived from the JWT `azp` claim via a lookup table.
- **Related research:** [§5 PostgreSQL RLS](#5-postgresql-row-level-security-research)

### ADR-008: Cloudflare DNS Strategy
- **Decision:** Wildcard DNS + Cloudflare Tunnel (phased approach)
- **Status:** RECOMMENDED (pending final confirmation)
- **Related research:** [§3 Cloudflare DNS](#3-cloudflare-dns--edge-strategy-research)

### ADR-009: Tenant Provisioning Service
- **Decision:** Node.js/TypeScript, Keycloak Admin API + Cloudflare API, custom saga pattern
- **Status:** DECIDED
- **Related research:** [§6 Provisioning Service](#6-tenant-provisioning-service-research)

### ADR-010: Billing Architecture
- **Decision:** PENDING
- **Leading option:** Stripe with per-tenant subscription, hybrid seat + usage model
- **Related research:** [§9 Billing Architecture](#9-billing-architecture-research)

### ADR-011: Resource Metering
- **Decision:** Deferred (future concern)
- **Status:** DECIDED (not for initial launch)
- **Future tool:** OpenMeter when needed
- **Related research:** [§9 Billing Architecture](#9-billing-architecture-research)

### ADR-012: Custom Domains for Enterprise
- **Decision:** Deferred (future consideration, not initial launch)
- **Status:** DECIDED
- **Future path:** Cloudflare for SaaS Custom Hostnames API
- **Related research:** [§3 Cloudflare DNS](#3-cloudflare-dns--edge-strategy-research)

### ADR-013: Client-ID as Primary Tenant Identity (Rev 2 Architectural Shift)
- **Decision:** Keycloak `client_id` (via JWT `azp` claim) is the primary tenant identity primitive. A derived `tenant_id` UUID in the app DB decouples storage from Keycloak naming. Keycloak Organization is demoted to an optional enterprise-only layer for IdP linking and managed membership.
- **Status:** DECIDED
- **Rationale:**
  - `client_id` is already in every JWT (`azp` claim) without any extra scope or mapper
  - Roles are already scoped to client (`resource_access.<client-id>.roles`)
  - Third-party OAuth2 integrations work natively against the client
  - Self-hosted desktop app already uses a client-id — same primitive in SaaS means one code path
  - Organization was adding a parallel identity system that duplicated what `client_id` provides
  - Introducing `org_id` prematurely couples the architecture to Keycloak's Organization feature (which still has gaps: no org-scoped roles, no client-org binding)
- **Mapping table:**
  ```sql
  CREATE TABLE tenants (
    tenant_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kc_client_id VARCHAR(255) NOT NULL UNIQUE,  -- Keycloak clientId string (= JWT azp)
    kc_client_uuid UUID,                         -- Keycloak internal UUID (for Admin API calls)
    kc_org_id UUID,                              -- NULL for free tier, set for enterprise
    slug VARCHAR(255) NOT NULL UNIQUE,            -- URL-friendly identifier
    tier VARCHAR(20) NOT NULL DEFAULT 'free',     -- 'free' | 'pro' | 'enterprise'
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
  );
  ```
- **When Organization IS still used:** Enterprise tenants that need external IdP linking, email domain matching for auto-enrollment, and managed member workflows. Created as an optional step during enterprise provisioning.
- **Future risk:** If multiple clients per tenant are ever needed (web + API + mobile), the 1:1 mapping breaks. Mitigation: `tenants` table already provides indirection — add a `tenant_clients` junction table at that point. YAGNI for now.

---

## 2. Free-Tier Routing Research

### 2.1 Problem Statement

How should free-tier users (<20 users, no external IdP) access their BodhiApp org?
Enterprise users are confirmed to use subdomain routing (`acme.getbodhi.ai`).
The free-tier decision affects cookie isolation, DNS automation, Keycloak config complexity,
multi-org UX, and the upgrade path to enterprise.

### 2.2 Options Evaluated

#### Option A: Subdomain for ALL tenants (uniform model)
```
Free:       myorg.getbodhi.ai
Enterprise: acme.getbodhi.ai
```
- **Pros:** Uniform routing logic, natural cookie isolation, clean URL identity
- **Cons:** Wildcard DNS needed for potentially thousands of free orgs, subdomain squatting
  risk, every free signup needs Keycloak redirect_uri update or wildcard redirect
  (security risk — CVE-2023-6134 Keycloak wildcard redirect vulnerability)
- **Keycloak complexity:** HIGH — either wildcard redirect URIs (insecure) or per-org
  client with unique redirect URI (same overhead as enterprise)

#### Option B: Shared domain for free + subdomain for enterprise (RECOMMENDED)
```
Free:       app.getbodhi.ai/org/{slug}/dashboard
Enterprise: acme.getbodhi.ai/dashboard
```
- **Pros:** Zero DNS overhead for free tier, single Keycloak client for all free users,
  in-app org switcher is industry standard, upgrade to enterprise is additive
- **Cons:** Two routing code paths, cookie-based org context needs careful implementation,
  no natural browser isolation between orgs
- **Keycloak complexity:** LOW — one client `bodhi-app-free` with one redirect URI
- **Security model:** JWT claims are source of truth (not cookies), RLS enforces at DB

#### Option C: Auto-generated slugs for free tier
```
Free:       a1b2c3.getbodhi.ai
Enterprise: acme.getbodhi.ai
```
- **Pros:** Cookie isolation, no squatting
- **Cons:** Unmemorable URLs, same DNS overhead as Option A, no industry precedent,
  poor UX for bookmark/sharing

### 2.3 Industry Survey

| Product | Routing Model | Org Switching | Notes |
|---------|--------------|---------------|-------|
| **GitHub** | github.com/orgs/{slug} | Dropdown in UI | Shared domain, path-based |
| **Notion** | notion.so (shared) | Sidebar switcher | Single domain, all orgs |
| **Vercel** | vercel.com/{team} | Dropdown | Path-based team context |
| **Linear** | linear.app (shared) | Workspace switcher | Single domain |
| **Figma** | figma.com/{team} | Team switcher | Single domain |
| **Discord** | discord.com (shared) | Server list sidebar | Single domain |
| **Postman** | postman.com (shared) | Workspace dropdown | Single domain |
| **Slack** | {workspace}.slack.com | Separate windows/tabs | ONLY major product using subdomains for all |
| **Jira** | {tenant}.atlassian.net | Product switcher | Subdomain model |

**Conclusion:** 7/9 major products use shared domain + in-app switching. Slack's subdomain model
is widely considered a legacy architectural decision they can't easily change.

### 2.4 Cookie Security Considerations

**Critical rule for Option B:**
- Free tier cookies: `Set-Cookie: session=...; Domain=app.getbodhi.ai; Secure; HttpOnly; SameSite=Lax`
- Enterprise cookies: `Set-Cookie: session=...; Domain=acme.getbodhi.ai; Secure; HttpOnly; SameSite=Lax`
- **NEVER** set `Domain=.getbodhi.ai` — this would leak cookies across all subdomains
- Host-only cookies (no `Domain` attribute) are scoped to the exact hostname

**Multi-org session for free tier:**
- Store active org ID in a session cookie scoped to `app.getbodhi.ai`
- JWT refresh includes `organization:<active-org>` scope
- Switching orgs: trigger new auth request with different org scope, Keycloak SSO session
  reuses authentication (no re-login needed)

### 2.5 Org Context in URL Patterns

**Recommended pattern (Vercel-style):**
```
app.getbodhi.ai/org/{slug}/dashboard
app.getbodhi.ai/org/{slug}/settings
app.getbodhi.ai/org/{slug}/models
```

**Bare URL redirect logic:**
1. User visits `app.getbodhi.ai/`
2. Check session cookie for `active_org`
3. If set → redirect to `app.getbodhi.ai/org/{active_org}/dashboard`
4. If not set → show org picker (list user's orgs from JWT `org_memberships`)

### 2.6 Key Resources

| Resource | URL | Relevance |
|----------|-----|-----------|
| Keycloak CVE-2023-6134 (wildcard redirect) | https://access.redhat.com/security/cve/CVE-2023-6134 | Why wildcard redirect URIs are dangerous |
| Slack workspace architecture retrospective | (industry knowledge) | Why per-tenant subdomains are being abandoned |
| Vercel team switching UX | https://vercel.com/docs/accounts/teams | Reference implementation |
| Next.js multi-tenant guide | https://vercel.com/guides/nextjs-multi-tenant-application | Path-based vs subdomain patterns |

---

## 3. Cloudflare DNS & Edge Strategy Research

### 3.1 Problem Statement

How to handle DNS resolution and TLS for subdomain-based enterprise tenants,
with a path to supporting thousands of tenants and eventually custom domains.

### 3.2 Options Evaluated

#### Option A: Wildcard DNS + Universal SSL (RECOMMENDED Phase 1)
```
DNS Record: *.getbodhi.ai → CNAME → <tunnel-id>.cfargotunnel.com (proxied)
TLS: Cloudflare Universal SSL (free, covers *.getbodhi.ai automatically)
```
- **Pros:** Zero per-tenant DNS provisioning, instant subdomain availability,
  $0 cost on free plan, no propagation delay for new tenants
- **Cons:** No per-tenant DNS analytics, all traffic hits same backend (need app-level routing)
- **TLS depth:** Universal SSL covers `*.getbodhi.ai` but NOT `*.*.getbodhi.ai`
  (no deep subdomains). Advanced Certificate Manager ($10/mo) for multi-level wildcards.

#### Option B: Individual CNAME records per tenant
```
DNS: acme.getbodhi.ai → CNAME → lb.getbodhi.ai (created via Cloudflare API per tenant)
```
- **Pros:** Per-tenant DNS analytics, explicit control
- **Cons:** 3,500 record limit on paid plans (hard ceiling!), API call needed per provisioning,
  1-2 minute propagation delay per new tenant, API rate limit: 1,200 req/5min
- **REJECTED** due to record limit ceiling

#### Option C: Cloudflare for SaaS (Custom Hostnames API)
```
API: POST /zones/{zone_id}/custom_hostnames { hostname: "acme.getbodhi.ai" }
TLS: Per-hostname DCV via HTTP or CNAME, auto-renewed
```
- **Pros:** Purpose-built for SaaS multi-tenancy, custom domain support built-in,
  per-hostname TLS, 100 free custom hostnames then $0.10/hostname/month
- **Cons:** Overkill for same-domain subdomains (wildcard DNS solves this for free),
  DCV validation adds provisioning complexity, $2/mo minimum for the SSL product
- **Use case:** Reserve for Phase 3 (custom domains like `ai.acme.com`)

#### Option D: Cloudflare Workers + Wildcard DNS
```
Worker route: *.getbodhi.ai/* → worker script
Worker: parse Host header, validate tenant in KV, add headers, fetch origin
```
- **Pros:** Edge-level tenant validation, per-tenant rate limiting, request transformation,
  A/B testing per tenant, geo-routing
- **Cons:** Workers Paid plan ($5/mo) required for production, adds latency (~1-5ms),
  debugging complexity
- **Use case:** Phase 2 when edge-level logic is needed

### 3.3 Phased Approach (Recommended)

```
Phase 1 (0-1K tenants, $0/mo):
  Wildcard DNS + Cloudflare Tunnel + Universal SSL
  → New subdomains work instantly, zero provisioning

Phase 2 (100-1K tenants, ~$5-25/mo):
  Add Cloudflare Worker on *.getbodhi.ai route
  → Edge validation, rate limiting, header injection

Phase 3 (1K+ tenants with custom domain requests, ~$25-500/mo):
  Add Cloudflare for SaaS for enterprise custom domains only
  → Standard subdomains still use wildcard at $0 marginal cost

Phase 4 (10K+ tenants):
  Evaluate Cloudflare Enterprise for advanced features
  → Custom pricing, dedicated support
```

### 3.4 Cloudflare Tunnel Architecture

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Browser    │────▶│  Cloudflare Edge  │────▶│  cloudflared    │
│ acme.getbodhi│     │  (TLS termination │     │  (K8s Deployment│
│   .ai       │     │   + WAF + DDoS)   │     │   in cluster)   │
└─────────────┘     └──────────────────┘     └────────┬────────┘
                                                       │
                                               ┌───────▼────────┐
                                               │  K8s Service   │
                                               │  (ClusterIP)   │
                                               └───────┬────────┘
                                                       │
                                               ┌───────▼────────┐
                                               │  App Pods      │
                                               │  (parse Host   │
                                               │   header for   │
                                               │   tenant)      │
                                               └────────────────┘
```

**Key benefits over traditional Ingress:**
- No public LoadBalancer IP needed (saves ~$15-20/mo on cloud providers)
- Built-in DDoS protection at Cloudflare edge
- No Ingress Controller to manage (nginx-ingress EOL March 2026!)
- Wildcard TLS handled by Cloudflare, not cert-manager
- `cloudflared` runs as a K8s Deployment with 2+ replicas for HA

**Tunnel configuration (config.yml):**
```yaml
tunnel: <TUNNEL_ID>
credentials-file: /etc/cloudflared/credentials.json
ingress:
  - hostname: "*.getbodhi.ai"
    service: http://bodhi-app-service.default.svc.cluster.local:8080
  - hostname: "app.getbodhi.ai"
    service: http://bodhi-app-service.default.svc.cluster.local:8080
  - service: http_status:404
```

### 3.5 Cost Projections

| Scale | Cloudflare Plan | Add-ons | Monthly Cost |
|-------|----------------|---------|-------------|
| 0-100 tenants | Free | None | **$0** |
| 100-500 | Free | Workers Paid ($5) | **$5** |
| 500-1,000 | Pro ($20) | Workers ($5) | **$25** |
| 1,000-5,000 | Pro ($20) | Workers + CF for SaaS (~$0.10/custom hostname) | **$25-$500** |
| 10,000+ | Enterprise | Full suite | **Custom** |

### 3.6 Cloudflare API Details for Provisioning Service

**TypeScript SDK:** `npm install cloudflare` (official, v5.x, auto-generated from OpenAPI)
- GitHub: https://github.com/cloudflare/cloudflare-typescript
- npm: https://www.npmjs.com/package/cloudflare

**Rate limits:** 1,200 requests per 5 minutes per API token
**DNS record limit:** 3,500 per zone on paid plans (irrelevant with wildcard approach)
**Custom Hostnames (Phase 3):** 100 free, then $0.10/hostname/month, limit raised to 50,000 in May 2025

**Key API endpoints:**
```
# DNS Records (for individual records if needed)
POST /zones/{zone_id}/dns_records
GET  /zones/{zone_id}/dns_records?name={subdomain}.getbodhi.ai

# Custom Hostnames (Phase 3)
POST /zones/{zone_id}/custom_hostnames
GET  /zones/{zone_id}/custom_hostnames/{id}

# Workers (Phase 2)
PUT  /accounts/{account_id}/workers/scripts/{script_name}
POST /zones/{zone_id}/workers/routes
```

### 3.7 Key Resources

| Resource | URL | Relevance |
|----------|-----|-----------|
| Cloudflare for SaaS docs | https://developers.cloudflare.com/cloudflare-for-platforms/cloudflare-for-saas/ | Custom hostname provisioning |
| Cloudflare WAF for SaaS blog | https://blog.cloudflare.com/waf-for-saas/ | Security patterns for SaaS |
| CF for SaaS limit increase (May 2025) | https://community.cloudflare.com/t/ssl-tls-cloudflare-for-saas-secrets-store-increased-limits-for-cloudflare-for-saas-and-secrets-store-free-and-pay-as-you-go-plans/819550 | 50K hostname limit |
| DNS record limits discussion | https://community.cloudflare.com/t/dns-records-has-any-limit-on-free-plan/431008 | 3,500 record ceiling on paid |
| Cloudflare API DNS automation | https://reintech.io/blog/automating-dns-management-cloudflare-api | API patterns |
| Cloudflare TypeScript SDK | https://github.com/cloudflare/cloudflare-typescript | Official SDK |
| Cloudflare Tunnel docs | https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/ | Tunnel setup |
| Cloudflare Workers pricing | https://developers.cloudflare.com/workers/platform/pricing/ | $5/mo paid plan |

---

## 4. Keycloak v26 Organizations Research

> **Rev 2 Note:** Organizations are demoted from co-equal tenant identity to **optional
> enterprise-only layer**. The Keycloak **Client** is the primary tenant primitive.
> `azp` claim (= `client_id`) in the JWT identifies the tenant. Organizations add value
> only for: (a) enterprise IdP linking, (b) email domain auto-enrollment, (c) managed
> member workflows. Free-tier tenants may never need an Organization at all.

### 4.1 Feature Status

**GA in Keycloak 26.0** (October 2024). Previously preview in 25.x.
Must be explicitly enabled in realm settings: "Organizations" → "Enabled".

**Key releases:**
- v26.0: Organizations GA with basic membership, token claims, org-linked IdPs
- v26.0.9 / v26.1.0: Fix for org scope mismatch bug (#35935) — CRITICAL
- v26.2.x: Stability improvements
- v26.6.0 (planned): Organization Groups with role support (#45505)

**Minimum recommended version: 26.2+** (avoids scope mismatch bug)

### 4.2 Token Enrichment — How It Works

> **Rev 2 context:** The `organization` scope flow below is only needed for **enterprise
> features** (IdP-linked login, managed membership verification). For basic tenant
> identification, the `azp` claim (= `client_id`) is always present in every OIDC token
> without any additional scope. The auth middleware reads `azp` → maps to `tenant_id` →
> uses for RLS. Organization claims are supplementary.

**Basic tenant identification (all tiers, no extra scope needed):**
```
Token always contains:
{
  "azp": "bodhi-acme-corp",                     // ← primary tenant identifier
  "resource_access": {
    "bodhi-acme-corp": {
      "roles": ["admin", "model-manager"]        // ← authorization
    }
  }
}
```

**Enterprise org enrichment (optional, via organization scope):**

**Request flow:**
```
1. Client requests: scope=openid organization:acme-corp
2. Keycloak validates user is member of "acme-corp" organization
3. Token includes organization claim:
   {
     "organization": {
       "acme-corp": {
         "id": "42c3e46f-..."    // Only if mapper configured
       }
     }
   }
4. Client roles injected separately under resource_access:
   {
     "resource_access": {
       "bodhi-acme-client": {
         "roles": ["admin", "model-manager"]
       }
     }
   }
```

**IMPORTANT CONFIGURATION:** The org ID is NOT included in the token by default.
You must enable "Add organization id" on the Organization Membership mapper
in the `organization` client scope settings.

**Tenant switching (Rev 2 — two patterns):**

**Free tier (client-based, no org scope needed):**
```
1. User has active SSO session (authenticated via client-a)
2. User selects different tenant in UI org-switcher
3. Frontend triggers new authorization request against client-b's auth endpoint
4. Keycloak reuses SSO session → issues new token with azp=client-b
5. No password prompt (seamless switch via SSO session reuse)
6. Frontend stores new token, UI reflects new tenant context
```

**Enterprise (org scope for IdP-linked features):**
```
1. User has active SSO session
2. Client triggers auth request: scope=organization:org-b (against enterprise client)
3. Keycloak reuses SSO session → issues token with org claim + client roles
4. No password prompt (seamless switch)
```

**Key insight:** Both paths leverage Keycloak's SSO session reuse. The difference is
that free-tier switching targets a different `client_id` endpoint, while enterprise
switching additionally scopes to an Organization for IdP and membership features.

### 4.3 Organization-Linked Identity Providers (Enterprise SSO)

**Setup flow for enterprise tenant:**
```
1. Create Organization "Acme Corp" with alias "acme-corp"
2. Create/import SAML or OIDC IdP configuration for Acme's IdP
3. Link IdP to Organization: POST /admin/realms/{realm}/organizations/{orgId}/identity-providers
4. Associate email domain: "acme.com" with organization
5. Enable "Redirect when email domain matches" on the IdP
```

**Login flow:**
```
1. User visits acme.getbodhi.ai → redirected to Keycloak
2. Identity-first login page shows email input
3. User enters user@acme.com
4. Keycloak matches domain → redirects to Acme's IdP
5. User authenticates at Acme's IdP
6. Returns to Keycloak → user auto-enrolled as "managed" member of Acme org
7. Token issued with organization:acme-corp scope
```

**Managed vs unmanaged members:**
- Managed: Users who authenticate via the org's linked IdP. Organization "owns" their identity.
- Unmanaged: Users invited manually or who authenticate via realm's default IdP.

### 4.4 Known Limitations & Gaps

#### Gap 1: No Organization-Scoped Roles
- **Issue:** Roles are realm-level or client-level, not org-level
- **Rev 2 impact:** **LOW** — With client-id-centric architecture, client-level roles
  ARE the tenant-scoped roles. Each tenant has its own client, so `resource_access.<client-id>.roles`
  naturally provides "admin in Tenant A, viewer in Tenant B." This gap only matters if
  you wanted to use Organizations as the primary tenant identity (which we no longer do).
- **Future relevance:** If Organization Groups (#45505, targeted 26.6.0) ships, it could
  simplify enterprise admin workflows but is not blocking.
- **GitHub issue:** https://github.com/keycloak/keycloak/issues/45505

#### Gap 2: No Client-to-Organization Binding
- **Issue:** Cannot officially link a client to an organization in Keycloak's data model
- **Rev 2 impact:** **LOW** — The `tenants` table in the app DB already maps `kc_client_id`
  to an optional `kc_org_id`. Since Organization is enterprise-only, this mapping is only
  needed for enterprise tenants. The mapping is part of the `tenants` table (see ADR-013),
  not a separate table.
- **GitHub issue:** https://github.com/keycloak/keycloak/issues/42781

#### Gap 3: Organization Scope Mismatch Bug
- **Issue:** #35935 — switching org scope returned wrong org's claims
- **Fixed in:** 26.0.9 / 26.1.0
- **Action:** Use Keycloak 26.2+ minimum
- **GitHub issue:** https://github.com/keycloak/keycloak/issues/35935

#### Gap 4: No Batch Admin API
- **Issue:** Keycloak Admin REST API has no batch/bulk operations
- **Impact:** Creating an org with client + roles + members requires 5+ sequential API calls
- **Mitigation:** Parallelize independent calls, use service account with connection pooling

### 4.5 Admin REST API — Provisioning Sequence

> **Rev 2:** Client creation is now Step 1 (the primary tenant resource).
> Organization creation is Step 4 and is **enterprise-only**.

```
# 1. Create Client (PRIMARY — all tiers)
POST /admin/realms/{realm}/clients
Body: {
  "clientId": "bodhi-acme-corp",
  "name": "Acme Corp - BodhiApp",
  "enabled": true,
  "publicClient": false,
  "standardFlowEnabled": true,
  "redirectUris": ["https://acme-corp.getbodhi.ai/auth/callback"],
  "webOrigins": ["https://acme-corp.getbodhi.ai"],
  "protocol": "openid-connect"
}
→ Returns: 201 + Location header with client UUID

# 2. Create Client Roles (all tiers)
POST /admin/realms/{realm}/clients/{clientUuid}/roles
Body: { "name": "admin" }
POST /admin/realms/{realm}/clients/{clientUuid}/roles
Body: { "name": "editor" }
POST /admin/realms/{realm}/clients/{clientUuid}/roles
Body: { "name": "viewer" }

# 3. Assign Client Role to Founding User (all tiers)
POST /admin/realms/{realm}/users/{userId}/role-mappings/clients/{clientUuid}
Body: [{ "name": "admin", "id": "{roleId}" }]

# 4. Create Organization (ENTERPRISE ONLY — skip for free tier)
POST /admin/realms/{realm}/organizations
Body: { "name": "Acme Corp", "alias": "acme-corp", "enabled": true }
→ Returns: 201 + Location header with org ID

# 5. Add User to Organization (ENTERPRISE ONLY)
POST /admin/realms/{realm}/organizations/{orgId}/members
Body: { "id": "{userId}" }

# 6. Link External IdP to Organization (ENTERPRISE ONLY)
POST /admin/realms/{realm}/organizations/{orgId}/identity-providers
Body: { "alias": "acme-saml-idp" }
```

**For free tier, only steps 1–3 are needed.** This reduces provisioning from 5+ API calls to 3.

**Authentication:** Service account with `client_credentials` grant.
Required realm roles: `manage-clients`, `manage-users`, `manage-realm` (via `realm-management` client).

### 4.6 Phase Two `keycloak-orgs` Extension

**Alternative/complement to native Organizations:**
- Vendor: Phase Two (https://phasetwo.io)
- Provides organization-scoped roles, org invitations, org-level IdP linking
- Available as open-source extension or managed Keycloak hosting
- Pre-dates Keycloak's native Organizations feature
- **Consideration:** If native org-scoped roles remain delayed past 26.6, evaluate Phase Two extension

### 4.7 Key Resources

| Resource | URL | Relevance |
|----------|-----|-----------|
| Keycloak v26 release notes | https://www.keycloak.org/docs/latest/release_notes/index.html | Organizations GA announcement |
| Organizations multi-tenancy guide (Medium) | https://medium.com/keycloak/exploring-keycloak-26-introducing-the-organization-feature-for-multi-tenancy-fb5ebaaf8fe4 | Official Keycloak team article |
| BootLabs multi-tenancy implementation | https://blog.boottechsolutions.com/2025/05/12/keycloak-multi-tenancy-with-organizations/ | Step-by-step implementation guide |
| Skycloak organizations guide | https://skycloak.io/blog/multitenancy-in-keycloak-using-the-organizations-feature/ | Token mappers, org switching patterns |
| Org scope mismatch bug #35935 | https://github.com/keycloak/keycloak/issues/35935 | Critical bug, fixed in 26.0.9 |
| Org roles discussion #36597 | https://github.com/keycloak/keycloak/discussions/36597 | Community discussion on ACL patterns |
| Org Groups API #45505 | https://github.com/keycloak/keycloak/issues/45505 | Planned org-scoped roles feature |
| Phase Two orgs extension | https://phasetwo.io/blog/organgizations-multi-tenant-update/ | Alternative org implementation |
| Keycloak Admin API reference | https://kc-api.github.io/quick-reference/ | Quick reference for API calls |
| Cloud-IAM API docs | https://documentation.cloud-iam.com/resources/keycloak-api.html | Managed Keycloak API reference |
| Keycloak realm export/import | https://www.mastertheboss.com/keycloak/how-to-export-and-import-realms-in-keycloak/ | Migration tooling |

---

## 5. PostgreSQL Row-Level Security Research

> **Rev 2:** All references updated from `org_id` / `current_org` to `tenant_id` /
> `current_tenant`. The `tenant_id` is a UUID stored in the app DB `tenants` table,
> derived from the JWT `azp` claim (= Keycloak `client_id`).

### 5.1 Core Pattern: SET LOCAL + Transaction Wrapping

```sql
-- 1. Create app-specific role (not superuser, not BYPASSRLS)
CREATE ROLE app_user LOGIN PASSWORD '...';
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO app_user;

-- 2. Enable RLS on every tenant-scoped table
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE projects FORCE ROW LEVEL SECURITY; -- applies even to table owner

-- 3. Create optimized tenant isolation function
CREATE OR REPLACE FUNCTION current_tenant_id() RETURNS uuid AS $$
  SELECT NULLIF(current_setting('app.current_tenant', true), '')::uuid
$$ LANGUAGE SQL SECURITY DEFINER STABLE;

-- 4. Create policy with initPlan optimization
CREATE POLICY tenant_isolation ON projects
  FOR ALL TO app_user
  USING (tenant_id = (SELECT current_tenant_id()))
  WITH CHECK (tenant_id = (SELECT current_tenant_id()));

-- 5. Usage in application
BEGIN;
SET LOCAL app.current_tenant = 'a1b2c3d4-...';  -- tenant_id UUID from tenants table
SELECT * FROM projects;  -- RLS filters automatically
COMMIT;  -- SET LOCAL cleared, connection safe to return to pool
```

### 5.2 Critical Implementation Details

**Why SET LOCAL, not SET:**
- `SET` (without LOCAL) persists on the connection until changed or disconnected
- In pooled environments (PgBouncer transaction mode), the connection is returned to the pool
  after COMMIT, and the NEXT request on that connection inherits the previous tenant's context
- `SET LOCAL` is automatically discarded at `COMMIT` or `ROLLBACK` — connection-safe
- Nile documented production data leaks from using `SET` instead of `SET LOCAL`

**initPlan optimization:**
- Wrapping `current_tenant_id()` in `(SELECT ...)` triggers PostgreSQL's initPlan
- Function evaluated ONCE per query, result cached for all row comparisons
- Without this: function called per-row → 100x+ performance degradation on large tables
- Source: Supabase RLS performance testing

**Indexing strategy:**
- ALWAYS create an index on `tenant_id` for every tenant-scoped table
- For large tables, consider composite primary key: `PRIMARY KEY (tenant_id, id)`
- This makes RLS-filtered queries use index seeks instead of full scans
- Partition by `tenant_id` only if individual tenants have 10M+ rows (premature otherwise)

### 5.3 ORM Integration

#### Drizzle (Best native RLS support)
```typescript
import { pgTable, pgPolicy, enableRLS, uuid, text } from 'drizzle-orm/pg-core';
import { sql } from 'drizzle-orm';

export const projects = pgTable('projects', {
  id: uuid('id').primaryKey().defaultRandom(),
  tenantId: uuid('tenant_id').notNull(),
  name: text('name').notNull(),
}, (table) => [
  enableRLS(),
  pgPolicy('tenant_isolation', {
    for: 'all',
    to: 'app_user',
    using: sql`tenant_id = (SELECT current_tenant_id())`,
    withCheck: sql`tenant_id = (SELECT current_tenant_id())`,
  }),
]);
```

#### Prisma (via Client Extensions)
```typescript
function forTenant(tenantId: string) {
  return prisma.$extends({
    query: {
      $allModels: {
        async $allOperations({ args, query }) {
          const [, result] = await prisma.$transaction([
            prisma.$executeRaw`SELECT set_config('app.current_tenant', ${tenantId}, true)`,
            query(args),
          ]);
          return result;
        },
      },
    },
  });
}

// Usage:
const tenantPrisma = forTenant('a1b2c3d4-...');
const projects = await tenantPrisma.project.findMany(); // RLS-filtered
```

**Prisma caveat:** `set_config(..., true)` sets `is_local=true` which is equivalent to
`SET LOCAL`. The third parameter MUST be `true` in pooled environments.

### 5.4 Connection Pooling Compatibility

| Pooler | Mode | SET LOCAL Safe? | Notes |
|--------|------|----------------|-------|
| PgBouncer | Transaction | **YES** | SET LOCAL cleared at COMMIT, connection reused safely |
| PgBouncer | Session | YES | Connection dedicated to client, no leak risk |
| PgBouncer | Statement | **NO** | Statements pooled individually, SET LOCAL meaningless |
| pgpool-II | All modes | YES | Session-level pooling, SET LOCAL always safe |
| Supavisor | Transaction | **YES** | Supabase's pooler, designed for RLS |

**PgBouncer transaction mode is the recommended production configuration.**

### 5.5 Testing Strategy

**Use pgTAP for CI-integrated RLS testing:**
```sql
-- Test: Tenant A cannot see Tenant B's data
BEGIN;
SELECT plan(2);

SET LOCAL app.current_tenant = 'tenant-a-uuid';
SELECT is(
  (SELECT count(*) FROM projects WHERE tenant_id = 'tenant-b-uuid'),
  0::bigint,
  'Tenant A cannot see Tenant B projects via RLS'
);

SET LOCAL app.current_tenant = 'tenant-b-uuid';
SELECT is(
  (SELECT count(*) FROM projects WHERE tenant_id = 'tenant-b-uuid'),
  3::bigint,  -- assuming 3 projects seeded for tenant-b
  'Tenant B can see its own projects'
);

SELECT finish();
ROLLBACK;
```

**Additional test cases:**
- Insert with wrong tenant_id → rejected by WITH CHECK
- No tenant set (empty current_setting) → returns zero rows (fail-closed)
- Superuser/admin role bypass → verify FORCE ROW LEVEL SECURITY
- EXPLAIN ANALYZE as app_user → verify RLS predicates in query plan

### 5.6 Migration: Adding RLS to Existing Tables

```sql
-- Step 1: Add tenant_id column if not exists
ALTER TABLE projects ADD COLUMN tenant_id UUID NOT NULL DEFAULT 'default-tenant-uuid';

-- Step 2: Backfill tenant_id for existing data
UPDATE projects SET tenant_id = (SELECT tenant_id FROM users WHERE users.id = projects.owner_id);

-- Step 3: Create index
CREATE INDEX CONCURRENTLY idx_projects_tenant_id ON projects(tenant_id);

-- Step 4: Enable RLS
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE projects FORCE ROW LEVEL SECURITY;

-- Step 5: Create policies
CREATE POLICY tenant_isolation ON projects FOR ALL TO app_user
  USING (tenant_id = (SELECT current_tenant_id()))
  WITH CHECK (tenant_id = (SELECT current_tenant_id()));

-- Step 6: Remove default (prevent accidental missing tenant_id)
ALTER TABLE projects ALTER COLUMN tenant_id DROP DEFAULT;
```

### 5.7 Key Resources

| Resource | URL | Relevance |
|----------|-----|-----------|
| Nile multi-tenant RLS guide | https://www.thenile.dev/blog/multi-tenant-rls | SET LOCAL pattern, production incidents |
| Supabase RLS performance discussion | https://github.com/orgs/supabase/discussions/14576 | initPlan optimization, 100x perf difference |
| Drizzle ORM RLS docs | https://orm.drizzle.team/docs/rls | Native RLS schema support |
| Prisma RLS extension example | https://github.com/prisma/prisma-client-extensions/tree/main/row-level-security | Official Prisma RLS pattern |
| Prisma Client Extensions blog | https://www.prisma.io/blog/client-extensions-preview-8t3w27xkrxxn | Extension API for RLS wrapping |
| Prisma + Supabase RLS discussion | https://www.answeroverflow.com/m/1326857186336575498 | Community patterns |
| Bytebase RLS reference | https://www.bytebase.com/reference/postgres/how-to/postgres-row-level-security/ | PostgreSQL RLS fundamentals |
| pgDash RLS deep dive | https://pgdash.io/blog/exploring-row-level-security-in-postgres.html | Performance analysis |
| pgTAP testing guide | https://www.endpointdev.com/blog/2022/03/using-pgtap-automate-database-testing/ | Database test framework |

---

## 6. Tenant Provisioning Service Research

> **Rev 2:** Provisioning reordered to client-first. Organization creation is now
> an optional enterprise-only step. Free-tier provisioning is 4 steps instead of 6.

### 6.1 Provisioning Flow (Happy Path)

```
User signs up / Admin creates tenant
        │
        ▼
┌─────────────────────────┐
│ 1. Create DB record      │  tenants table: status=PENDING
│    (tenant + mapping)    │  Generate tenant_id UUID
└───────────┬─────────────┘
            ▼
┌─────────────────────────┐
│ 2. Create KC Client      │  POST /admin/realms/.../clients (PRIMARY RESOURCE)
│    + default roles       │  status=CREATING_KC_CLIENT
│    Store kc_client_id    │
└───────────┬─────────────┘
            ▼
┌─────────────────────────┐
│ 3. Assign founding user  │  POST .../role-mappings/clients/{id}
│    admin role on client  │  status=CONFIGURING_USERS
└───────────┬─────────────┘
            ▼
┌─────────────────────────┐
│ 4. Seed default data     │  Default settings, welcome content
│                          │  status=READY (free tier DONE here)
└───────────┬─────────────┘
            │
            │ (enterprise only, steps 5-6)
            ▼
┌─────────────────────────┐
│ 5. Create KC Organization│  POST /admin/realms/.../organizations
│    + link to client      │  Store kc_org_id on tenants table
│    + add user as member  │  status=CREATING_KC_ORG
└───────────┬─────────────┘
            ▼
┌─────────────────────────┐
│ 6. Link external IdP     │  POST .../organizations/{id}/identity-providers
│    (if enterprise SSO)   │  status=READY
└─────────────────────────┘
```

### 6.2 Saga Pattern — Compensation Table

| Step | Execute | Compensate | Idempotency Check | Tier |
|------|---------|------------|-------------------|------|
| 1. DB Record | INSERT tenant | DELETE tenant | Check tenant exists by slug | All |
| 2. KC Client | POST clients + roles | DELETE clients/{id} | GET clients?clientId={name} | All |
| 3. User+Role | POST role-mappings | DELETE role-mappings | GET role-mappings | All |
| 4. Seed Data | INSERT seed rows | DELETE seed rows by tenant_id | Check seed marker exists | All |
| 5. KC Org | POST organizations + members | DELETE organizations/{id} | GET organizations?search={alias} | Enterprise |
| 6. IdP Link | POST org identity-providers | DELETE org identity-providers | GET org IdPs | Enterprise |

### 6.3 State Tracking Schema

```sql
CREATE TYPE provisioning_status AS ENUM (
  'PENDING',
  'CREATING_KC_CLIENT',
  'CONFIGURING_USERS',
  'SEEDING_DB',
  'CREATING_KC_ORG',      -- enterprise only
  'LINKING_IDP',           -- enterprise only
  'READY',
  'FAILED',
  'COMPENSATING',
  'ROLLED_BACK',
  'SUSPENDED',
  'DELETING'
);

CREATE TABLE tenant_provisioning (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
  status provisioning_status NOT NULL DEFAULT 'PENDING',
  current_step INTEGER NOT NULL DEFAULT 0,

  -- External resource IDs for compensation
  kc_client_id VARCHAR(255),         -- Keycloak clientId string
  kc_client_uuid UUID,               -- Keycloak internal UUID
  kc_org_id UUID,                    -- NULL for free tier
  cf_dns_record_id VARCHAR(255),     -- NULL for free tier

  -- Error tracking
  last_error TEXT,
  retry_count INTEGER DEFAULT 0,
  max_retries INTEGER DEFAULT 3,

  created_at TIMESTAMPTZ DEFAULT now(),
  updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_provisioning_stuck
  ON tenant_provisioning(status, updated_at)
  WHERE status NOT IN ('READY', 'ROLLED_BACK');
```

### 6.4 Retry & Stuck Provisioning Detection

```typescript
// Cron job: every 5 minutes
async function retryStuckProvisioning() {
  const stuck = await db.query(`
    SELECT * FROM tenant_provisioning
    WHERE status NOT IN ('READY', 'ROLLED_BACK', 'SUSPENDED')
    AND updated_at < now() - interval '10 minutes'
    AND retry_count < max_retries
    FOR UPDATE SKIP LOCKED
  `);

  for (const task of stuck.rows) {
    await resumeProvisioning(task); // Resume from current_step
  }
}
```

### 6.5 Tenant Suspension & Deletion

**Suspension (soft):**
1. Update `organizations.status = 'SUSPENDED'` in app DB
2. Disable Keycloak client: `PUT /clients/{id}` with `enabled: false`
3. Middleware checks `tenant.status` on every request → returns 403

**Deletion (hard, after retention period):**
1. Delete all tenant data from app DB (cascading deletes via tenant_id FK)
2. Delete Keycloak organization (cascades members)
3. Delete Keycloak client
4. Delete DNS record (enterprise)
5. Update provisioning record: status = 'DELETED'

**GDPR data retention:** 30 days from deletion request. Soft-delete immediately,
hard-delete via scheduled job after retention period.

### 6.6 Future Migration Path for Orchestration

```
Complexity Growth Path:
Custom saga (~200 LOC) → Inngest → Temporal.io

Custom saga: Good for <5 workflows, <10 steps each
  + Zero infrastructure overhead
  + Full control
  - Manual retry/compensation logic

Inngest: Good for 5-20 workflows
  + Serverless (no workers to manage)
  + Each step.run() independently retried
  + Built-in event replay and observability
  - Vendor dependency (self-hostable)

Temporal.io: Good for 20+ complex workflows
  + Industry standard for workflow orchestration
  + Signals, queries, timers on running workflows
  + Deterministic replay for debugging
  - Requires own infrastructure (PG + Temporal Server)
  - Temporal Cloud: ~$200+/month
  - TypeScript SDK: https://github.com/temporalio/sdk-typescript
```

### 6.7 Key Resources

| Resource | URL | Relevance |
|----------|-----|-----------|
| Saga pattern (Microsoft) | https://learn.microsoft.com/en-us/azure/architecture/patterns/saga | Orchestration vs choreography patterns |
| Saga pattern in Node.js | https://itc.im/implementing-the-saga-pattern-orchestrating-distributed-transactions-in-microservices-with-node-js/ | TypeScript implementation |
| Cloudflare TypeScript SDK | https://github.com/cloudflare/cloudflare-typescript | API automation |
| Cloudflare npm package | https://www.npmjs.com/package/cloudflare | Official SDK |
| Cloudflare API DNS management | https://www.tech-otaku.com/web-development/using-cloudflare-api-manage-dns-records/ | API patterns |
| Temporal TypeScript SDK | https://github.com/temporalio/sdk-typescript | Future orchestration |
| Inngest docs | https://www.inngest.com/docs | Serverless workflow alternative |

---

## 7. Self-Hosted → SaaS Migration Research

### 7.1 Migration Architecture

```
┌─────────────────────┐     ┌──────────────────────┐     ┌─────────────────┐
│  Self-Hosted Desktop│     │  Migration Service    │     │  SaaS Tenant    │
│  (SQLite + local KC)│     │  (server-side)        │     │  (PG + shared KC)│
│                     │     │                       │     │                 │
│  bodhi export       │────▶│  1. Validate JSON     │────▶│  PostgreSQL     │
│    --format json    │     │  2. Transform types   │     │  (tenant_id     │
│    --output backup  │     │  3. Import to PG      │     │   scoped)       │
│                     │     │  4. Import KC users   │     │  Keycloak       │
│  Keycloak export    │────▶│  5. Re-auth required  │────▶│  (shared realm) │
│    (kc.sh export)   │     │                       │     │                 │
└─────────────────────┘     └──────────────────────┘     └─────────────────┘
```

### 7.2 SQLite → PostgreSQL Type Mapping

| SQLite Type | PostgreSQL Type | Transformation |
|-------------|----------------|----------------|
| INTEGER (autoincrement) | SERIAL / BIGSERIAL | Sequence-backed identity |
| TEXT (dates) | TIMESTAMPTZ | Parse ISO 8601, add timezone |
| TEXT (JSON) | JSONB | Parse and validate JSON |
| INTEGER (boolean 0/1) | BOOLEAN | Cast to true/false |
| REAL | DOUBLE PRECISION | Direct mapping |
| BLOB | BYTEA | Direct mapping |
| TEXT (UUID strings) | UUID | Cast to UUID type |
| TEXT LIKE (case-insensitive) | ILIKE | SQLite LIKE is case-insensitive by default |

### 7.3 Tools

**pgloader** — Direct migration tool
- Command: `pgloader sqlite:///path/to/bodhi.db pgsql://user@host/bodhidb`
- Handles type conversion, constraint creation, index rebuilding
- GitHub: https://github.com/dimitri/pgloader
- SQLite docs: https://pgloader.readthedocs.io/en/latest/ref/sqlite.html
- **Caveat:** SQLite allows broken foreign keys (enforcement historically optional)
  → Clean referential integrity BEFORE migration

**Alternative: Custom JSON export/import**
- More control over transformation
- Build into desktop app: `bodhi export --format json --output backup.json`
- Include checksums (SHA-256 of each table's data) for integrity verification
- Server-side import endpoint: validates, transforms, inserts scoped by new tenant_id

### 7.4 Keycloak User Migration

**Self-hosted Keycloak has its own realm with local users.**
Migration requires moving users to the shared SaaS realm.

**Export users from self-hosted:**
```bash
kc.sh export --dir /export --realm bodhi --users realm_file
```
This produces `bodhi-users-0.json` with user records including hashed credentials.

**Import to SaaS realm via Admin API:**
```
POST /admin/realms/{realm}/users
Body: { "username": "...", "email": "...", "enabled": true, "credentials": [...] }
```

**Critical limitation:** Sessions and tokens CANNOT transfer across Keycloak instances.
Users MUST re-authenticate after migration. Design a migration wizard that:
1. Shows "Migration complete, please log in again"
2. Sends email verification if email-based auth
3. Links existing social login (Google) to new realm user

### 7.5 Industry Precedents

| Product | Migration Path | Data Format | Auth Handling |
|---------|---------------|-------------|---------------|
| **GitLab** | Self-managed → GitLab.com | JSON export per project | Re-auth required |
| **Mattermost** | Self-hosted → Cloud | bulk export (JSONL) | SAML migration or re-auth |
| **Bitwarden** | Self-hosted → Cloud | Encrypted JSON export | Master password re-entry |
| **Discourse** | Self-hosted → Hosted | PG backup + restore | Session invalidation |

**Common pattern:** JSON/JSONL export format, server-side import with validation,
re-authentication required, email notification to users.

### 7.6 Key Resources

| Resource | URL | Relevance |
|----------|-----|-----------|
| pgloader GitHub | https://github.com/dimitri/pgloader | SQLite→PG migration tool |
| pgloader SQLite docs | https://pgloader.readthedocs.io/en/latest/ref/sqlite.html | Configuration reference |
| Render migration guide | https://render.com/articles/how-to-migrate-from-sqlite-to-postgresql | Step-by-step guide |
| Keycloak realm export | https://www.mastertheboss.com/keycloak/how-to-export-and-import-realms-in-keycloak/ | Export/import procedures |
| Keycloak export (Elest) | https://docs.elest.io/books/keycloak/page/exporting-and-importing-realms | Alternative docs |

---

## 8. Kubernetes Multi-Tenancy Research

### 8.1 Recommended Architecture

```
Internet
  │
  ▼
Cloudflare Edge (TLS, WAF, DDoS)
  │
  ▼ (Cloudflare Tunnel — outbound from cluster)
cloudflared Deployment (2+ replicas)
  │
  ▼
K8s Service (ClusterIP)
  │
  ▼
App Pods (read azp from JWT → resolve tenant_id → inject into request context)
  │
  ▼
PostgreSQL (RLS filters by tenant_id from SET LOCAL)
```

### 8.2 Tenant Extraction Middleware (Application-Level)

> **Rev 2:** Primary tenant identification comes from JWT `azp` claim (= `client_id`),
> not from subdomain parsing. Subdomain is used for routing/UX, but `azp` is the
> authoritative tenant identifier.

```typescript
// Express middleware example
async function tenantMiddleware(req, res, next) {
  // Step 1: Extract client_id from JWT (authoritative tenant identity)
  const jwt = verifyJWT(req.headers.authorization);
  const clientId = jwt.azp; // e.g., "bodhi-acme-corp"

  // Step 2: Resolve tenant_id from client_id (cached in Redis/memory)
  const tenant = await tenantCache.get(clientId);
  if (!tenant || tenant.status !== 'READY') return res.status(404).send('Not found');

  // Step 3: Optionally validate subdomain matches tenant (defense in depth)
  const host = req.hostname;
  if (host !== 'app.getbodhi.ai') {
    const subdomain = host.split('.getbodhi.ai')[0];
    if (subdomain !== tenant.slug) return res.status(403).send('Forbidden');
  }

  // Step 4: Inject tenant context for downstream use
  req.tenant = tenant;        // { tenant_id, slug, tier, kc_client_id, ... }
  req.roles = jwt.resource_access?.[clientId]?.roles || [];
  next();
}
```

**Why application-level over ingress-level:**
- Ingress NGINX snippets disabled by default (security CVEs)
- Testable in unit tests without K8s cluster
- Ingress NGINX EOL March 2026 — avoid building on deprecated tech
- Cloudflare Tunnel bypasses Ingress entirely

### 8.3 Autoscaling Strategy

**API tier:** Standard HPA with CPU/memory targets
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: bodhi-api-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: bodhi-api
  minReplicas: 2
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

**AI inference tier:** KEDA (CNCF graduated) for event-driven scaling
- Scale to zero when no inference requests → dramatic cost savings
- Scale based on queue depth (Redis, RabbitMQ, Kafka)
- Or scale based on Prometheus metrics (requests/second)
- Docs: https://keda.sh/

### 8.4 Observability (Per-Tenant)

**OpenTelemetry middleware pattern:**
```typescript
import { context, propagation, trace } from '@opentelemetry/api';

function otelTenantMiddleware(req, res, next) {
  const span = trace.getActiveSpan();
  if (span && req.tenant) {
    span.setAttribute('tenant.id', req.tenant.tenant_id);
    span.setAttribute('tenant.slug', req.tenant.slug);
    span.setAttribute('tenant.tier', req.tenant.tier); // 'free' | 'enterprise'
  }

  // Propagate via baggage to all downstream services
  const baggage = propagation.createBaggage({
    'tenant.id': { value: req.tenant.tenant_id },
  });
  const ctx = propagation.setBaggage(context.active(), baggage);
  context.with(ctx, next);
}
```

**Structured logging:** Include `tenant_id` in every log entry for Grafana Loki filtering.

**Cardinality warning:** With 1,000+ tenants, `tenant_id` as a Prometheus label creates
HIGH cardinality (each label combination = new time series). Mitigations:
- Use recording rules to pre-aggregate per-tier (free/enterprise) not per-tenant
- Emit `tenant_id` only in logs/traces, not metrics
- Use histograms for latency distribution, not per-tenant gauges

### 8.5 Key Resources

| Resource | URL | Relevance |
|----------|-----|-----------|
| K8s Ingress wildcard guide | https://copyprogramming.com/howto/kubernetes-ingress-rules-how-to-use-wildcard-and-specific-subdomain-together | Wildcard + specific subdomain |
| KEDA introduction | https://devtron.ai/blog/introduction-to-kubernetes-event-driven-autoscaling-keda/ | Event-driven autoscaling |
| OTel multi-tenant instrumentation | https://oneuptime.com/blog/post/2026-02-06-instrument-saas-multi-tenant-application-opentelemetry/view | Per-tenant observability |
| OTel Baggage for business context | https://oneuptime.com/blog/post/2026-02-06-baggage-pass-business-context-across-service-boundaries/view | Cross-service tenant propagation |
| Gateway API (Ingress successor) | https://gateway-api.sigs.k8s.io/ | Future-proof ingress |
| Cloudflare Tunnel K8s docs | https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/deploy-tunnels/deployment-guides/kubernetes/ | Deployment guide |

---

## 9. Billing Architecture Research

### 9.1 Billing Model Options for AI Platforms

| Model | Pros | Cons | Examples |
|-------|------|------|----------|
| **Seat-based** ($X/user/month) | Predictable revenue, simple | Doesn't capture AI usage value | Notion, Linear |
| **Usage-based** (per token/request) | Aligns cost with value | Revenue unpredictable, hard to budget | OpenAI API |
| **Hybrid** (seat base + usage overage) | Predictable base + usage upside | More complex billing | **Recommended** |
| **Tier-based** (flat tiers) | Simple to understand | Cliff edges, poor fit for AI | Slack, basic SaaS |

### 9.2 Recommended Tier Structure

```
Free Tier:
  - ≤20 users
  - 100 inference requests/month
  - Community support
  - No external IdP
  - Shared domain (app.getbodhi.ai)
  - No Stripe Customer (zero billing overhead)

Pro Tier ($X/user/month):
  - Unlimited users
  - 1,000 requests/month included
  - $0.01 per additional request
  - Priority support
  - Dedicated subdomain
  - Stripe Customer + Subscription

Enterprise Tier (custom pricing):
  - Everything in Pro
  - External IdP integration (SAML/OIDC)
  - Custom domain support (future)
  - Volume discounts, committed spend
  - SLA guarantees
  - Stripe Customer + Custom invoice
```

### 9.3 Stripe Architecture

**One Stripe Customer per organization (created at first upgrade, not at signup).**

```
Stripe Customer (tenant: acme-corp)
  └── Subscription
       ├── Price: $15/seat/month (quantity = user count)
       └── Metered Price: $0.01/request (usage records reported hourly)
```

**Key Stripe features to use:**
- **Stripe Checkout:** Hosted payment page for upgrades (PCI-compliant)
- **Customer Portal:** Self-service subscription management
- **Usage Records API:** Report metered usage hourly
- **Webhooks:** `checkout.session.completed`, `customer.subscription.updated`,
  `customer.subscription.deleted`, `invoice.payment_failed`

**Webhook handling is critical:** NEVER trust client-side for billing state changes.
All tier upgrades/downgrades must be confirmed via webhook.

### 9.4 Metering Architecture (Future)

**Phase 1 (initial):** Custom event log
```sql
CREATE TABLE usage_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
  event_type VARCHAR(50) NOT NULL, -- 'inference_request', 'model_download', etc.
  metadata JSONB,
  created_at TIMESTAMPTZ DEFAULT now()
);

-- Hourly aggregation cron → Stripe Usage Records API
```

**Phase 2 (scale):** OpenMeter
- Open-source (Apache 2.0)
- Kafka + ClickHouse backed
- Native Stripe integration
- CloudEvents spec for event ingestion
- Real-time aggregation and dashboards
- GitHub: https://github.com/openmeterio/openmeter
- YC company (W23 batch)

### 9.5 Key Resources

| Resource | URL | Relevance |
|----------|-----|-----------|
| OpenMeter | https://www.ycombinator.com/companies/openmeter | Usage metering platform |
| OpenMeter GitHub | https://github.com/openmeterio/openmeter | Open-source metering |
| Stripe Usage Records | https://stripe.com/docs/billing/subscriptions/usage-based | Metered billing docs |
| Stripe Multi-tenant patterns | https://stripe.com/docs/connect/collect-then-transfer-guide | Platform patterns (reference only) |
| Stripe Checkout | https://stripe.com/docs/payments/checkout | Hosted payment flow |

---

## 10. Industry Benchmarks & Comparable Products

### 10.1 Multi-Tenant SaaS Products with Self-Hosted Options

| Product | Self-Hosted | SaaS | Multi-Tenancy | Auth | Migration Path |
|---------|-------------|------|---------------|------|---------------|
| **GitLab** | Docker/K8s | gitlab.com | Namespace-based | Built-in + SAML/OIDC | JSON export per project |
| **Mattermost** | Docker | cloud.mattermost.com | Workspace-based | Built-in + SAML | Bulk JSONL export |
| **Bitwarden** | Docker | bitwarden.com | Organization-based | Built-in + SSO | Encrypted JSON |
| **Discourse** | Docker | hosted.discourse.org | Site-based | Built-in + OAuth | PG backup |
| **n8n** | Docker/npm | n8n.cloud | Team-based | Built-in + SAML | JSON export |
| **Appsmith** | Docker/K8s | app.appsmith.com | Workspace-based | Built-in + SAML/OIDC | JSON export |

### 10.2 Comparable AI Platform Architectures

| Product | Routing | Auth | DB Isolation | Billing |
|---------|---------|------|-------------|---------|
| **Hugging Face** | Shared domain + org path | Built-in + SSO | Shared DB | Usage-based (compute seconds) |
| **Replicate** | Shared domain | API key + OAuth | Shared DB | Usage-based (predictions) |
| **Together AI** | Shared domain | API key | Shared DB | Token-based |
| **Vercel** | Shared domain + team path | Built-in + SSO | Shared DB | Hybrid (seat + bandwidth) |

**Pattern:** All modern AI platforms use shared-domain routing with API key or OAuth auth.
None use per-tenant subdomains for their primary product.

---

## 11. Open Questions & Future Research

### 11.1 Decisions Still Pending

| Question | Options | Blocking? | Target Date |
|----------|---------|-----------|-------------|
| Free-tier subdomain vs shared domain | Shared domain recommended | Yes — affects Keycloak config | Before dev sprint |
| Billing model specifics | Hybrid recommended, pricing TBD | No — deferred | Before launch |
| ORM choice for provisioning service | Drizzle (RLS support) vs Prisma (ecosystem) | No — parallel track | During dev |
| Keycloak deployment | Self-hosted K8s vs managed (Phase Two, Cloud-IAM) | Yes — affects ops burden | Before infra setup |

### 11.2 Research Needed When Requirements Change

**If custom domains needed sooner:**
- Research Cloudflare for SaaS Custom Hostnames API in detail
- TLS DCV validation flow (HTTP-01 vs CNAME challenge)
- Keycloak redirect URI management for arbitrary domains
- Cookie handling across custom domains

**If data isolation requirements increase (enterprise compliance):**
- PostgreSQL schema-per-tenant migration from shared-schema
- Logical replication for tenant data extraction
- Encrypted-at-rest per-tenant key management (AWS KMS / Vault)

**If Keycloak org-scoped roles ship (v26.6+):**
- Evaluate whether org-scoped roles simplify enterprise tenant management
- With client_id-centric model, this is a **nice-to-have** not a blocker
- Consider if org-scoped roles reduce the need for per-tenant Keycloak clients
  (unlikely — client-per-tenant also serves OAuth2 delegation for 3rd parties)
- No migration needed: client roles remain the canonical source

**If multiple Keycloak clients per tenant are needed:**
- Scenario: separate web, API, and mobile clients for a single tenant
- The `tenants` table already provides indirection from `tenant_id`
- Add a `tenant_clients` junction table mapping tenant_id → multiple kc_client_ids
- Update middleware to accept any of the tenant's client_ids as valid `azp`
- RLS remains unchanged (still keyed on `tenant_id`)

**If scale exceeds 10K tenants:**
- PostgreSQL partitioning by tenant_id (range or hash)
- Read replicas with tenant-aware routing
- Keycloak clustering / realm sharding
- Cloudflare Enterprise evaluation

### 11.3 Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Keycloak org scope bugs (like #35935) | Low (Rev 2) | Medium (Rev 2) | Only affects enterprise IdP flows, not basic tenancy. Pin to 26.2+ |
| RLS bypass via missed SET LOCAL | Low | Critical | Middleware audit, pgTAP tests in CI |
| Subdomain squatting (if free tier gets subdomains) | Low (Rev 2) | Low | Free tier uses shared domain, enterprise slugs validated |
| Keycloak single realm bottleneck | Low | High | Clustering, infinispan tuning, monitor |
| PgBouncer transaction mode + SET LOCAL edge case | Low | High | Integration tests with pooler in CI |
| Cloudflare API rate limit during mass provisioning | Low | Medium | Batch provisioning queue, rate limit backoff |
| azp claim missing from JWT | Very Low | High | Standard OIDC claim, always present. Validate in middleware |
| tenant_id cache stale after provisioning | Low | Medium | Cache invalidation on tenant create/update, short TTL |

---

## 12. Master Reference Index

All URLs referenced in this document, grouped by topic.

### Cloudflare
- Cloudflare for SaaS docs: https://developers.cloudflare.com/cloudflare-for-platforms/cloudflare-for-saas/
- WAF for SaaS blog: https://blog.cloudflare.com/waf-for-saas/
- CF for SaaS limit increase: https://community.cloudflare.com/t/ssl-tls-cloudflare-for-saas-secrets-store-increased-limits-for-cloudflare-for-saas-and-secrets-store-free-and-pay-as-you-go-plans/819550
- DNS record limits: https://community.cloudflare.com/t/dns-records-has-any-limit-on-free-plan/431008
- API DNS automation: https://reintech.io/blog/automating-dns-management-cloudflare-api
- API DNS management: https://www.tech-otaku.com/web-development/using-cloudflare-api-manage-dns-records/
- TypeScript SDK: https://github.com/cloudflare/cloudflare-typescript
- npm package: https://www.npmjs.com/package/cloudflare
- Tunnel docs: https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/
- Workers pricing: https://developers.cloudflare.com/workers/platform/pricing/
- Tunnel K8s guide: https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/deploy-tunnels/deployment-guides/kubernetes/

### Keycloak
- v26 release notes: https://www.keycloak.org/docs/latest/release_notes/index.html
- Organizations article (Medium): https://medium.com/keycloak/exploring-keycloak-26-introducing-the-organization-feature-for-multi-tenancy-fb5ebaaf8fe4
- BootLabs implementation guide: https://blog.boottechsolutions.com/2025/05/12/keycloak-multi-tenancy-with-organizations/
- Skycloak organizations guide: https://skycloak.io/blog/multitenancy-in-keycloak-using-the-organizations-feature/
- Org scope bug #35935: https://github.com/keycloak/keycloak/issues/35935
- Org roles discussion #36597: https://github.com/keycloak/keycloak/discussions/36597
- Org Groups API #45505: https://github.com/keycloak/keycloak/issues/45505
- Phase Two extension: https://phasetwo.io/blog/organgizations-multi-tenant-update/
- Admin API quick reference: https://kc-api.github.io/quick-reference/
- Cloud-IAM API docs: https://documentation.cloud-iam.com/resources/keycloak-api.html
- Realm export/import: https://www.mastertheboss.com/keycloak/how-to-export-and-import-realms-in-keycloak/
- Realm export (Elest): https://docs.elest.io/books/keycloak/page/exporting-and-importing-realms

### PostgreSQL RLS
- Nile multi-tenant RLS: https://www.thenile.dev/blog/multi-tenant-rls
- Supabase RLS performance: https://github.com/orgs/supabase/discussions/14576
- Drizzle ORM RLS: https://orm.drizzle.team/docs/rls
- Prisma RLS extension: https://github.com/prisma/prisma-client-extensions/tree/main/row-level-security
- Prisma Client Extensions: https://www.prisma.io/blog/client-extensions-preview-8t3w27xkrxxn
- Prisma + Supabase RLS: https://www.answeroverflow.com/m/1326857186336575498
- Bytebase RLS reference: https://www.bytebase.com/reference/postgres/how-to/postgres-row-level-security/
- pgDash RLS deep dive: https://pgdash.io/blog/exploring-row-level-security-in-postgres.html
- pgTAP testing: https://www.endpointdev.com/blog/2022/03/using-pgtap-automate-database-testing/

### Migration
- pgloader: https://github.com/dimitri/pgloader
- pgloader SQLite docs: https://pgloader.readthedocs.io/en/latest/ref/sqlite.html
- Render migration guide: https://render.com/articles/how-to-migrate-from-sqlite-to-postgresql

### Kubernetes & Observability
- K8s Ingress wildcard: https://copyprogramming.com/howto/kubernetes-ingress-rules-how-to-use-wildcard-and-specific-subdomain-together
- KEDA: https://devtron.ai/blog/introduction-to-kubernetes-event-driven-autoscaling-keda/
- OTel multi-tenant: https://oneuptime.com/blog/post/2026-02-06-instrument-saas-multi-tenant-application-opentelemetry/view
- OTel Baggage: https://oneuptime.com/blog/post/2026-02-06-baggage-pass-business-context-across-service-boundaries/view
- Gateway API: https://gateway-api.sigs.k8s.io/

### Orchestration & Workflows
- Saga pattern (Microsoft): https://learn.microsoft.com/en-us/azure/architecture/patterns/saga
- Saga in Node.js: https://itc.im/implementing-the-saga-pattern-orchestrating-distributed-transactions-in-microservices-with-node-js/
- Temporal TypeScript SDK: https://github.com/temporalio/sdk-typescript
- Inngest: https://www.inngest.com/docs

### Billing & Metering
- OpenMeter: https://github.com/openmeterio/openmeter
- OpenMeter (YC): https://www.ycombinator.com/companies/openmeter
- Stripe Usage Records: https://stripe.com/docs/billing/subscriptions/usage-based
- Stripe Checkout: https://stripe.com/docs/payments/checkout

---

## Appendix: Thinking Process & Decision Logic

### Why shared domain over subdomains for free tier
The decision tree was:
1. **Cost:** Subdomains for all = wildcard DNS (free) but Keycloak redirect URIs explode
2. **Security:** Wildcard redirect URIs have CVE history in Keycloak → must create per-org clients
3. **Per-org clients for free tier** = same provisioning overhead as enterprise = defeats the purpose of a "free tier"
4. **Industry precedent:** 7/9 major SaaS products use shared domain → this is a solved problem
5. **Upgrade path:** Shared → subdomain is additive; subdomain rename is disruptive
6. **Conclusion:** Shared domain for free, subdomains for enterprise. Two code paths, but dramatically lower operational cost.

### Why wildcard DNS over individual records
1. **3,500 record limit** on Cloudflare paid plans is an absolute ceiling
2. **Wildcard covers all subdomains instantly** — zero provisioning delay
3. **Universal SSL covers *.getbodhi.ai** for free — no per-cert management
4. **Individual records provide per-tenant DNS analytics** — nice-to-have, not worth the ceiling
5. **Conclusion:** Wildcard DNS + Cloudflare Tunnel. Layer Workers for edge logic when needed.

### Why SET LOCAL over SET for RLS
1. **SET persists on connection** → in PgBouncer transaction mode, next request inherits
2. **Nile documented production data leaks** from SET without LOCAL
3. **SET LOCAL scoped to transaction** → automatically cleared at COMMIT
4. **Conclusion:** SET LOCAL is the ONLY safe pattern for pooled connections. Non-negotiable.

### Why custom saga over Temporal for provisioning
1. **Provisioning is 5-6 steps, takes 5-10 seconds** — not complex enough for Temporal
2. **Temporal requires own infrastructure** (PG + Temporal Server) or $200+/month for Cloud
3. **Custom saga: ~200 lines of TypeScript** with PostgreSQL state tracking
4. **Idempotent steps + compensation** handles all failure modes
5. **Migration path clear:** Custom → Inngest → Temporal as complexity grows
6. **Conclusion:** Start simple, graduate tooling when warranted by complexity.

### Why client_id over org_id as primary tenant discriminator (Rev 2)
This was a mid-research architectural correction that simplified the entire stack.

1. **`azp` claim is in every JWT by default** — zero Keycloak configuration needed for basic
   tenant identification. No `organization` scope, no custom mappers, no feature flags.
2. **Roles are already client-scoped** — `resource_access.<client-id>.roles` naturally gives
   "admin in Tenant A, viewer in Tenant B" without Organization-scoped roles (which don't
   exist yet in Keycloak).
3. **Self-hosted parity** — the desktop app already uses a client-id. Using the same
   primitive in SaaS means one middleware, one RLS pattern, one code path.
4. **Organization has critical gaps** — no org-scoped roles (#45505), no client-org binding
   (#42781), scope mismatch bugs (#35935). Building on it as the primary identity means
   building on the least stable Keycloak feature.
5. **Organization IS still valuable** — but for enterprise features (IdP linking, email
   domain enrollment, managed members), not as the universal tenant identity.
6. **Derived tenant_id UUID** — decouples DB from Keycloak string format. A client rename
   (`bodhi-acme` → `bodhi-acme-corp`) updates one row in `tenants` table, not every row
   in every tenant-scoped table.
7. **YAGNI for multi-client-per-tenant** — if ever needed (web + API + mobile clients),
   the `tenants` table already provides indirection. Add a junction table then, not now.

**The cascade effect was dramatic:**
- Keycloak: Organization scope no longer needed for basic flows → simpler auth
- RLS: `tenant_id` from `azp` lookup → no dependency on Organization feature
- Provisioning: Client is Step 1, Org is optional enterprise step → faster free-tier onboarding
- Middleware: Read `azp` from JWT → single trust boundary, no header/cookie/scope juggling
- Keycloak Gaps: All three major gaps (org roles, client-org binding, scope bugs) become
  enterprise-only concerns instead of blocking issues for the entire architecture
