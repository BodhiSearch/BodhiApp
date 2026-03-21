# Multi-tenancy architecture for BodhiApp SaaS

**Use the Keycloak client as the primary tenant identity — not Organization — with a DB-generated `tenant_id` UUID as the discriminator column across all tables.** Every org-account maps 1:1 to a Keycloak client, and the `azp` claim in every JWT already identifies the tenant without requiring extra scopes or mappers. A single `tenants` table maps `tenant_id ↔ keycloak_client_id`, decoupling the entire data layer from Keycloak's naming. Organization is layered on top exclusively for enterprise features (IdP linking, managed membership) — it is not the foundational identity. This model unifies self-hosted and SaaS under one code path, since self-hosted instances already authenticate via a dedicated client.

For routing, use a shared domain (`app.getbodhi.ai`) for free-tier users with an in-app org switcher, reserving subdomain routing exclusively for enterprise tenants. Pair this with Cloudflare wildcard DNS + Tunnel for zero-cost, instant subdomain resolution. PostgreSQL Row-Level Security with `SET LOCAL` on `tenant_id` provides defense-in-depth data isolation, and a custom saga orchestrator handles tenant provisioning.

---

## Foundational decision: client-id as tenant identity, DB UUID as discriminator

The most important architectural decision in this document is that **the Keycloak client is the tenant, and a DB-generated UUID is the discriminator** — not `org_id`, not the Keycloak client string, and not the Keycloak internal UUID.

**Why client, not Organization:** The `azp` (authorized party) claim appears in every JWT automatically — no extra scope request, no custom mapper. Client-level roles under `resource_access.<client-id>.roles` are already the authorization source of truth. Third-party OAuth2 integrations work natively against the client. Self-hosted desktop instances already use a dedicated client-id as their identity. Using the same primitive in SaaS means one code path, not two.

**Why a DB-generated UUID, not the Keycloak `clientId` string:** The Keycloak `clientId` string (e.g., `bodhi-acme-corp`) is a human-readable label that may need to change — org renames, slug conflicts, rebranding. If this string were the foreign key on every table, a rename would require updating millions of rows across dozens of tables. With a DB-generated `tenant_id` UUID as the discriminator, a Keycloak client rename is a single-row update in the `tenants` mapping table.

**The `tenants` table is the single source of mapping:**

```sql
CREATE TABLE tenants (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  -- THIS is tenant_id, used everywhere
  slug VARCHAR(63) NOT NULL UNIQUE,               -- URL-friendly identifier (subdomain/path)
  display_name VARCHAR(255) NOT NULL,
  tier VARCHAR(20) NOT NULL DEFAULT 'free',       -- 'free' | 'pro' | 'enterprise'
  status VARCHAR(20) NOT NULL DEFAULT 'provisioning',

  -- Keycloak references (single place to update if KC changes)
  kc_client_id VARCHAR(255) NOT NULL UNIQUE,      -- Keycloak clientId string (e.g., 'bodhi-acme-corp')
  kc_client_uuid UUID,                            -- Keycloak internal UUID
  kc_org_id UUID,                                 -- NULL for free tier, set for enterprise

  -- Cloudflare references
  cf_dns_record_id VARCHAR(255),                  -- NULL for free tier (no subdomain)

  created_at TIMESTAMPTZ DEFAULT now(),
  updated_at TIMESTAMPTZ DEFAULT now()
);
```

Every other table in the system references `tenants.id` as `tenant_id`:

```sql
CREATE TABLE projects (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL REFERENCES tenants(id),
  name TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT now()
);
```

**Request flow — how `azp` becomes `tenant_id`:**

```
1. JWT arrives with azp: "bodhi-acme-corp"
2. Middleware looks up: SELECT id FROM tenants WHERE kc_client_id = 'bodhi-acme-corp'
   (cached in Redis/memory — this mapping is near-static)
3. tenant_id UUID injected into request context
4. DB layer: SET LOCAL app.current_tenant = '<tenant_id UUID>'
5. RLS filters all queries by tenant_id
```

**Where Keycloak Organization fits (enterprise only):**

Organization is an optional enhancement, not a foundation. It provides three enterprise features that have no equivalent in the client model: linking an external IdP to a tenant so users with matching email domains are auto-redirected, managed membership enrollment (Keycloak tracks who is "managed by" the org vs. self-registered), and admin-level membership views in the Keycloak console. Free-tier tenants have a client and a `tenants` row — no Organization, no extra Keycloak complexity.

```
Free tier tenant:    tenants row + Keycloak client + client roles
Enterprise tenant:   tenants row + Keycloak client + client roles + Keycloak Organization + linked IdP
```

---

## Free-tier routing: shared domain wins decisively

The most consequential routing decision is how free-tier tenants access the platform. Enterprise users access via subdomain (`acme.getbodhi.ai`). Three options were evaluated for free-tier users against cookie isolation, Keycloak complexity, operational overhead, multi-org UX, and upgrade paths.

**Option B — shared domain `app.getbodhi.ai` for free tier, subdomains for enterprise only — is the clear recommendation.** Seven of eight major SaaS platforms (GitHub, Notion, Vercel, Linear, Figma, Discord, Postman) use shared-domain routing with in-app organization switching. Only Slack uses per-tenant subdomains, largely for historical reasons. With BodhiApp's expected 100:1 free-to-enterprise ratio, Option B eliminates all subdomain infrastructure for 99% of tenants.

| Criterion | All subdomains | Shared domain + switcher | Auto-generated slugs |
|---|---|---|---|
| Cookie isolation | Natural browser isolation | Server-side via JWT/RLS | Natural browser isolation |
| Keycloak redirect_uri | Complex (dispatcher needed) | **Single entry for free tier** | Complex (same as all-sub) |
| DNS overhead for free tier | Wildcard covers all | **Zero** | Wildcard covers all |
| Subdomain squatting risk | High | **None** | None |
| Multi-org switching UX | Multiple tabs/URLs | **In-app switcher (industry standard)** | Unmemorable URLs |
| Routing code paths | One | Two (shared + subdomain) | One |
| Upgrade free→enterprise | Subdomain rename | **Additive (provision subdomain)** | Replace slug |
| Industry precedent | Slack only | GitHub, Notion, Vercel, Linear, Figma | None |

The tenant context resolution pattern follows Vercel's approach: URL path carries context (`app.getbodhi.ai/org/{slug}/dashboard`), a session cookie stores the active tenant slug for bare-URL redirects, and the JWT `azp` claim is resolved to `tenant_id` for authorization. PostgreSQL RLS then consumes `tenant_id`, making the security model consistent regardless of routing approach.

**Keycloak simplification is dramatic.** Free tier can share a smaller number of Keycloak clients or use a single "free-tier" client with tenant context handled at the application level via the `tenants` table lookup. Enterprise tenants get per-org clients with dedicated redirect URIs. No wildcard redirect URIs (which have known CVE history in Keycloak), no dispatcher pattern.

**Cookie security** requires one critical rule: never set `Domain=.getbodhi.ai` on any cookie. All free-tier cookies should be host-only on `app.getbodhi.ai`. Enterprise subdomain cookies should also be host-only. A single misconfigured parent-domain cookie would leak to all subdomains, enabling session fixation attacks across tenants.

---

## Cloudflare strategy: wildcard DNS + Tunnel, then layer up

For enterprise subdomains, a phased hybrid approach minimizes cost and complexity while preserving a path to custom domain support.

**Phase 1 (0–1,000 tenants, $0/month):** A single wildcard DNS record (`*.getbodhi.ai → CNAME to tunnel-id.cfargotunnel.com`, proxied) handles all subdomains instantly. Cloudflare Universal SSL — free on all plans — covers `*.getbodhi.ai` automatically. Cloudflare Tunnel (`cloudflared`) deployed as a Kubernetes Deployment eliminates the need for a public LoadBalancer IP, provides built-in DDoS protection, and works seamlessly with wildcard DNS. New enterprise subdomains work immediately with zero provisioning delay.

**Phase 2 (100–1,000 tenants, ~$5–25/month):** Add a Cloudflare Worker on the `*.getbodhi.ai` route for edge-level tenant validation against Workers KV, per-tenant rate limiting, and request header injection. Workers cost $5/month base with 10M requests included.

**Phase 3 (1,000+ tenants, custom domains needed):** Layer in Cloudflare for SaaS (Custom Hostnames API) exclusively for enterprise customers who want their own domains (`ai.acme.com`). The first 100 custom hostnames are free, then $0.10/hostname/month. The pay-as-you-go limit was raised to 50,000 hostnames in May 2025. Standard SaaS subdomains continue using wildcard DNS at zero marginal cost.

| Scale | Cloudflare plan | Add-ons | Monthly cost |
|---|---|---|---|
| 0–100 tenants | Free | None | **$0** |
| 100–500 | Free | Workers Paid ($5) | **$5** |
| 500–1,000 | Pro ($20) | Workers ($5) | **$25** |
| 1,000–5,000 | Pro ($20) | Workers + CF for SaaS | **$25–$500** |
| 10,000+ | Enterprise | Full suite | Custom |

Individual DNS records per tenant should be avoided. The 3,500-record limit on paid plans is a hard ceiling that blocks scaling. Wildcard DNS is superior for same-domain subdomains.

Design the application to read the `Host` header for tenant resolution — not just subdomain parsing — so Cloudflare for SaaS integration is trivial when custom domains are needed later.

---

## Keycloak v26: client as tenant, Organization as enterprise layer

Keycloak Organizations became fully GA in version 26 (October 2024). In BodhiApp's architecture, Organizations serve as an **enterprise enhancement** — not the primary tenant identity.

**Token flow with client-centric model:**

For every authenticated request, the JWT contains:

```json
{
  "azp": "bodhi-acme-corp",
  "resource_access": {
    "bodhi-acme-corp": {
      "roles": ["admin", "model-manager"]
    }
  }
}
```

The `azp` claim identifies the tenant. Client roles under `resource_access` provide authorization. This is present in every token without requesting any special scope. The middleware resolves `azp → tenant_id` via the cached `tenants` table lookup.

**Enterprise additions (Organization layer):**

For enterprise tenants that need external IdP integration, the Organization is created and linked:

```json
{
  "azp": "bodhi-acme-corp",
  "resource_access": {
    "bodhi-acme-corp": {
      "roles": ["admin"]
    }
  },
  "organization": {
    "acme-corp": {
      "id": "42c3e46f-..."
    }
  }
}
```

The `organization` claim appears only when `scope=organization:<alias>` is requested — enterprise clients include this scope, free-tier clients do not. The application code never branches on the `organization` claim for tenant identification; it always uses `azp → tenant_id`.

**Enterprise SSO via organization-linked identity providers is native.** Link an external SAML/OIDC IdP to an organization, associate the org with an email domain (e.g., `acme.com`), and enable "Redirect when email domain matches." Users entering `user@acme.com` on the identity-first login page are automatically redirected to Acme's IdP and enrolled as managed members.

**Multi-tenant switching with SSO session reuse.** When a user belongs to multiple tenants, the frontend triggers a new authorization request targeting the other tenant's client. If the user has an active SSO session, Keycloak reissues a token scoped to the new client without re-authentication. The new token has a different `azp`, which resolves to a different `tenant_id`.

**Known gaps and workarounds:**

No organization-scoped roles — roles remain client-level only. This is a non-issue in the client-centric model because client roles ARE tenant-scoped roles. Organization Groups with role support are in development (GitHub issue #45505, targeted for 26.6.0) but are not needed.

No client-to-organization binding — clients cannot be owned by an organization in Keycloak's data model. The `tenants` table with `kc_client_id` and `kc_org_id` columns handles this mapping in application code.

Organization scope mismatch bug (#35935) — fixed in 26.0.9/26.1.0. Use Keycloak 26.2+ minimum.

No batch Admin API — Keycloak has no bulk operations. Provisioning requires sequential API calls. Idempotent check-before-create patterns handle retries.

---

## PostgreSQL RLS: SET LOCAL with `tenant_id` as discriminator

The RLS implementation uses `SET LOCAL` within transactions, scoping the `tenant_id` to the current transaction. This is the only approach that is both safe and compatible with PgBouncer transaction-mode pooling.

```sql
-- Tenant-aware function
CREATE OR REPLACE FUNCTION current_tenant_id() RETURNS uuid AS $$
  SELECT NULLIF(current_setting('app.current_tenant', true), '')::uuid
$$ LANGUAGE SQL SECURITY DEFINER STABLE;

-- RLS policy with initPlan optimization
CREATE POLICY tenant_isolation ON projects
  FOR ALL TO app_user
  USING (tenant_id = (SELECT current_tenant_id()))
  WITH CHECK (tenant_id = (SELECT current_tenant_id()));

-- Usage in application
BEGIN;
SET LOCAL app.current_tenant = '<tenant_id UUID from tenants table>';
SELECT * FROM projects;  -- RLS filters automatically
COMMIT;  -- SET LOCAL cleared, connection safe to return to pool
```

**Why SET LOCAL, not SET:** `SET` (without LOCAL) persists on the connection until changed. In PgBouncer transaction mode, the connection is returned to the pool after COMMIT, and the next request inherits the previous tenant's context. Nile documented production data leaks from this pattern. `SET LOCAL` is automatically discarded at COMMIT — connection-safe.

**initPlan optimization:** Wrapping `current_tenant_id()` in `(SELECT ...)` triggers PostgreSQL's initPlan, evaluating the function once per query instead of per-row. Supabase's performance testing found 100x+ improvements on large tables.

**For TypeScript ORMs, Drizzle has the best native RLS support** with `pgPolicy` definitions directly in the schema. Prisma works via Client Extensions that wrap every query in a `$transaction` with `set_config('app.current_tenant', tenantId, true)`.

**Always index `tenant_id`** on every tenant-scoped table. Consider composite primary keys `(tenant_id, id)` for large tables. Apply `FORCE ROW LEVEL SECURITY` so the table owner is also subject to policies. Use a dedicated `admin_user` with `BYPASSRLS` for migrations.

Test RLS policies in CI with pgTAP: verify that `app_user` connected with Tenant A's context cannot read Tenant B's data, cannot insert with Tenant B's `tenant_id`, and that cross-tenant operations raise errors.

---

## Tenant provisioning: client-first saga

The provisioning flow reflects the client-centric model: the Keycloak client is created first as the primary resource. Organization is a conditional step only for enterprise tenants.

**Free tier provisioning (4 steps):**

```
1. Create tenants row (status=PROVISIONING, generate tenant_id UUID)
2. Create Keycloak client + default roles (admin, editor, viewer)
   → store kc_client_id, kc_client_uuid in tenants row
3. Add founding user to client with admin role
4. Seed default data → status=READY
```

**Enterprise tier provisioning (6 steps):**

```
1. Create tenants row (tier=enterprise, status=PROVISIONING)
2. Create Keycloak client + default roles
3. Create Keycloak Organization + link to client (store kc_org_id)
4. Link external IdP to Organization (if provided)
5. Configure DNS subdomain (Cloudflare API, store cf_dns_record_id)
6. Add founding user + assign admin role → status=READY
```

The saga pattern defines each step with an `execute` and `compensate` function. On failure, completed steps are compensated in reverse order. Each step is idempotent (check-before-create for Keycloak and Cloudflare APIs) so retrying a failed provisioning from any step is safe.

The `tenants` table itself serves as the state tracker — no separate `tenant_provisioning` table needed. The `status` column tracks progress, and the `kc_client_uuid`, `kc_org_id`, `cf_dns_record_id` columns store external resource IDs for compensation. A cron job finds stuck provisioning (status not terminal, `updated_at > 10 minutes ago`) and retries.

**Tenant suspension:** Disable the Keycloak client (`enabled: false` blocks new logins), update `tenants.status = 'suspended'`, middleware checks status on every request → returns 403. Retain data for 30 days per GDPR, then hard-delete.

**Cloudflare API:** The official TypeScript SDK (`npm install cloudflare` v5.x) handles DNS record management. Rate limit is 1,200 requests per 5 minutes per token. With wildcard DNS, only enterprise tenants trigger Cloudflare API calls, keeping volume minimal.

**Future orchestration migration:** Custom saga (~200 LOC) → Inngest (serverless, no workers) → Temporal.io (when 5+ complex workflows exist).

---

## Self-hosted to SaaS migration

The migration path from SQLite/desktop to PostgreSQL/SaaS follows patterns established by GitLab, Mattermost, and Bitwarden: export to a portable JSON format, transform server-side, import into the SaaS tenant.

**pgloader** handles direct SQLite → PostgreSQL migration with type mapping (SQLite's dynamic typing → PostgreSQL's strict typing). Key type differences: `AUTOINCREMENT → SERIAL`, `TEXT dates → TIMESTAMPTZ`, `TEXT JSON → JSONB`, `INTEGER booleans → BOOLEAN`.

**Auth migration requires re-authentication.** Keycloak users can be exported via `kc.sh export` and imported into the shared SaaS realm via Admin REST API. However, sessions and tokens cannot transfer across instances — users must re-authenticate. The migration wizard in the UI makes this expected and smooth.

**Data import scoping:** All imported rows get the new `tenant_id` assigned during SaaS tenant provisioning. The self-hosted SQLite has no `tenant_id` column (single-tenant) — the import process adds it uniformly.

**Self-hosted client identity continuity:** The self-hosted instance already has a Keycloak `clientId`. During migration, a new SaaS tenant is provisioned with a new `tenant_id` UUID and a new Keycloak client in the shared realm. The old self-hosted client is retired. The `tenant_id` UUID (not the Keycloak client string) ensures this transition is invisible to the data layer.

---

## Kubernetes: app-level tenant extraction with Cloudflare Tunnel

For BodhiApp's shared-pool model (stateless pods serving all tenants), application-level tenant extraction from the `Host` header and JWT `azp` claim is the simplest and safest approach.

**Middleware chain:**

```
1. Parse Host header → determine routing mode (subdomain vs shared domain)
2. Extract JWT → read azp claim
3. Resolve azp → tenant_id via cached tenants table
4. Validate tenant status (READY, not suspended/deleted)
5. Inject tenant_id into request context
6. DB layer uses SET LOCAL app.current_tenant = tenant_id
```

Cloudflare Tunnel connects outbound from the cluster to Cloudflare's edge, handling TLS termination and wildcard routing without an Ingress resource. This sidesteps the Ingress NGINX end-of-life (March 2026) entirely.

Use KEDA (CNCF graduated) for AI inference workloads — its killer feature is scale to zero when no inference requests exist, then scale up based on queue depth or Prometheus metrics. Standard HPA handles the API tier with CPU/memory targets.

For observability, set `tenant_id` as both a span attribute and OpenTelemetry baggage at the middleware level. Use structured logging with `tenant_id` in every log entry, targeting Grafana Loki for aggregation. Cardinality warning: with 1,000+ tenants, `tenant_id` as a Prometheus label creates high cardinality — use recording rules to pre-aggregate, or emit `tenant_id` only in logs/traces rather than metric labels.

---

## Billing: hybrid seat + usage model with Stripe (future)

Use standard Stripe Billing with one Stripe Customer per tenant, created only at first upgrade from free tier — no Stripe overhead for free users.

The recommended billing model for an AI platform is hybrid: a base seat price plus metered usage overage. Free tier (≤20 users, limited inference) has no Stripe Customer. Pro tier gets per-seat pricing with included requests and overage. Enterprise gets custom pricing.

For metering, start with a custom event log in PostgreSQL with a cron job that aggregates usage and syncs to Stripe Usage Records API hourly. When volume grows, migrate to OpenMeter (open-source, Apache 2.0, Kafka + ClickHouse backed, native Stripe integration).

---

## Summary: the tenant identity model

The entire architecture flows from one principle — **the Keycloak client IS the tenant, and a DB-generated UUID IS the discriminator:**

```
┌──────────────────────────────────────────────────────────┐
│                    tenants table                          │
│  id (UUID) ←── THIS is tenant_id, referenced everywhere  │
│  slug          (URL-friendly, for routing)               │
│  kc_client_id  (Keycloak string, for JWT azp lookup)     │
│  kc_org_id     (NULL for free, set for enterprise)       │
│  tier          (free / pro / enterprise)                 │
│  status        (provisioning / ready / suspended)        │
└──────────────────────────────────────────────────────────┘
         │
         │ tenant_id FK on every table
         ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│    projects     │  │     models      │  │   chat_history  │
│  tenant_id (FK) │  │  tenant_id (FK) │  │  tenant_id (FK) │
│  ...            │  │  ...            │  │  ...            │
└─────────────────┘  └─────────────────┘  └─────────────────┘
         │
         │ RLS enforces: tenant_id = current_tenant_id()
         │ SET LOCAL app.current_tenant = <UUID from tenants lookup>
         ▼
┌─────────────────────────────────────────────────────────┐
│  Keycloak client rename?  → Update 1 row in tenants    │
│  Org slug change?         → Update 1 row in tenants    │
│  Enterprise IdP added?    → Create KC Org, set kc_org_id│
│  Free → Enterprise?       → Additive: add subdomain,   │
│                              create KC Org, update tier  │
└─────────────────────────────────────────────────────────┘
```

Three decisions deserve particular attention going forward. First, monitor Keycloak GitHub issue #45505 for Organization Groups — when it ships, evaluate whether Organization-scoped roles simplify the model, though client roles may remain sufficient. Second, Ingress NGINX's March 2026 end-of-life makes Cloudflare Tunnel the more future-proof ingress path. Third, the free-to-enterprise upgrade path is naturally additive: provision subdomain, create KC Organization, link IdP, update `tier` column — no data migration, no `tenant_id` changes.

The total infrastructure cost at launch is effectively $0 for Cloudflare (wildcard DNS + Tunnel on free plan) plus standard Kubernetes and PostgreSQL hosting. The architecture scales to 10,000+ tenants before requiring significant additional spend, with clear upgrade paths at each scaling threshold.