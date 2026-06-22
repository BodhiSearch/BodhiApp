# Batch 3-6 Explore·API Models — D1 Catalog Schema (Backend 01)

**Audience:** the backend engineer dropping the catalog-ingestion tables into `api-getbodhi-app`.
**Scope:** the D1 (SQLite) schema for the unified models.dev + Portkey catalog — `sync_state`, `providers`, `models`, `model_pricing`, `slug_alias`, the FTS5 virtual table `models_fts` (external-content over `models`) and its sync triggers — plus every index the catalog API filters/sorts on, the column→source-field mapping, and the D1 limits that constrain the design.
**Source of fields:** `../data-analysis.md` §2 (models.dev TOML), §4 (Portkey), §5 (proposed `providers`/`models`/`model_pricing`), §4.7/§9 (slug map).
**Grounded in repo state:** D1 binding is `DB` (per-env, NOT inherited — dev id `393853ef-…`, prod id `45ae1a6a-…`; `wrangler.jsonc:34-57`). The repo uses **drizzle-orm `^0.45.2` + drizzle-kit `^0.31.10`** with the drizzle schema at `worker/db/schema.ts` and **raw-SQL migrations checked into `drizzle/`** (`migrations_dir 'drizzle'`). Today only `drizzle/0000_minor_mephisto.sql` exists (the generic `model_cache` blob table). These new tables are additive — `model_cache` stays untouched (logos + detail cache reuse the two-tier cache-aside).

> **Drizzle vs raw SQL — match repo convention.** The repo's convention is: author tables in `worker/db/schema.ts`, run `just db-generate` (drizzle-kit generate) to emit the numbered raw-SQL migration into `drizzle/`, then apply with `just db-migrate` (`--local`) / `db-migrate-remote-{dev,prod}`. **drizzle-kit cannot express FTS5 virtual tables or triggers** — those must live in a hand-authored raw-SQL migration. So this deliverable is split: §1 is the drizzle schema for the four relational tables + `slug_alias` (drizzle-generated migration `0001`), and §2 is a hand-authored raw-SQL migration `0002` for the FTS5 table + triggers + the partial/expression indexes drizzle can't emit. Keep the `__journal.json` ordering: `0000` → `0001` → `0002`.

---

## 0. Migration file plan (drop-in order)

| File | How produced | Contains |
|---|---|---|
| `drizzle/0000_minor_mephisto.sql` | already present | `model_cache` (untouched) |
| `drizzle/0001_<name>.sql` | `just db-generate` after editing `worker/db/schema.ts` (§1) | `sync_state`, `slug_alias`, `providers`, `models`, `model_pricing` + all column/standard indexes |
| `drizzle/0002_catalog_fts.sql` | **hand-authored raw SQL** (§2) — drizzle-kit can't emit FTS5/triggers/expression indexes | `models_fts` external-content FTS5 table, 3 sync triggers, expression/partial indexes |

> **cf-pool test caveat (from repo state):** `@cloudflare/vitest-pool-workers` does **NOT** auto-apply migrations. Every D1 test must `CREATE TABLE … / CREATE VIRTUAL TABLE …` in `beforeEach` (see `e2e-cf/detail.cache.test.ts:9`). Keep the raw DDL in §2 copy-pasteable precisely so tests can replay it. For `wrangler dev` / remote, the `drizzle/` migrations apply normally via `db-migrate*`.

---

## 1. Drizzle schema — relational tables (`worker/db/schema.ts` additions)

Append to the existing `worker/db/schema.ts` (which already exports `modelCache`). All booleans are stored as `integer({mode:'boolean'})` (SQLite has no bool); all "json array / object" fields are `text({mode:'json'})` (SQLite/D1 has **no native array type** — store JSON TEXT). Timestamps are epoch-ms `integer` to match the existing `fetched_at`/`expires_at` convention.

```ts
import { index, integer, primaryKey, sqliteTable, text, uniqueIndex } from 'drizzle-orm/sqlite-core'

// ── sync_state ────────────────────────────────────────────────────────────────
// Per-source git-SHA cursor for the diff-since-last-sync ingestion (data-analysis §6.2).
export const syncState = sqliteTable('sync_state', {
  source: text('source').primaryKey(), // 'models.dev' | 'portkey'
  lastSha: text('last_sha'), // null on first run → full ingest
  syncedAt: integer('synced_at'), // epoch ms of last successful advance
  lastStatus: text('last_status'), // 'ok' | 'partial' | 'error:<msg>'
  changedCount: integer('changed_count'), // files touched in last tick
})
export type SyncStateRow = typeof syncState.$inferSelect
export type NewSyncStateRow = typeof syncState.$inferInsert

// ── slug_alias ──────────────────────────────────────────────────────────────
// models.dev slug ←→ Portkey slug map (data-analysis §4.7 / §9 risk 4).
// e.g. modelsdev 'amazon-bedrock' ↔ portkey 'bedrock'; 'xai' ↔ 'x-ai';
// 'google' fans out to portkey 'google' AND 'vertex-ai' (one row each).
// A missing alias silently drops Portkey pricing — this table is the guard.
export const slugAlias = sqliteTable(
  'slug_alias',
  {
    modelsdevSlug: text('modelsdev_slug').notNull(), // canonical providers.slug
    portkeySlug: text('portkey_slug').notNull(), // Portkey filename stem
    note: text('note'), // optional provenance, e.g. 'vertex split'
  },
  (t) => [
    primaryKey({ columns: [t.modelsdevSlug, t.portkeySlug] }),
    index('idx_slug_alias_portkey').on(t.portkeySlug), // reverse lookup at join time
  ],
)
export type SlugAliasRow = typeof slugAlias.$inferSelect
export type NewSlugAliasRow = typeof slugAlias.$inferInsert

// ── providers ────────────────────────────────────────────────────────────────
export const providers = sqliteTable(
  'providers',
  {
    slug: text('slug').primaryKey(), // models.dev dir id, canonical
    name: text('name').notNull(), // models.dev `name`
    docUrl: text('doc_url'), // models.dev `doc`
    npm: text('npm'), // models.dev `npm`
    envVars: text('env_vars', { mode: 'json' }).$type<string[]>(), // models.dev `env` (JSON array)
    apiBaseUrl: text('api_base_url'), // models.dev `api` — NULL for native providers
    providerShape: text('provider_shape'), // openai|openai-compatible|anthropic|openrouter|kiro|native
    apiFormatHint: text('api_format_hint'), // OpenAI|Anthropic|Gemini|… (bridge + API-Format facet)
    logoKvKey: text('logo_kv_key'), // KV pointer for logos/<slug>.svg
    portkeySlug: text('portkey_slug'), // primary Portkey slug if it differs (denormalized convenience)
    modelCount: integer('model_count').notNull().default(0), // computed, for provider-list ranking
    sourceSha: text('source_sha'), // ingest cursor provenance
    updatedAt: integer('updated_at').notNull(),
  },
  (t) => [
    index('idx_providers_model_count').on(t.modelCount), // provider-list rank sort (DESC at query time)
    index('idx_providers_shape').on(t.providerShape), // API-format facet
  ],
)
export type ProviderRow = typeof providers.$inferSelect
export type NewProviderRow = typeof providers.$inferInsert

// ── models ───────────────────────────────────────────────────────────────────
// PK = (provider_slug, model_id) — a model AS SERVED BY a provider (data-analysis §2.1).
export const models = sqliteTable(
  'models',
  {
    providerSlug: text('provider_slug')
      .notNull()
      .references(() => providers.slug, { onDelete: 'cascade' }),
    modelId: text('model_id').notNull(), // models.dev path id — exact API string
    canonicalId: text('canonical_id').notNull(), // logical-model grouping key (03 §2.4): the `base_model` LAB/MODEL path id, else a normalized model_id. GROUP BY this for the /models list, provider_count, served_by.
    name: text('name').notNull(), // models.dev `name`
    family: text('family'), // models.dev `family`
    status: text('status'), // alpha|beta|deprecated; NULL ⇒ stable. FACET
    // capability booleans — FACETS (stored as integer 0/1)
    attachment: integer('attachment', { mode: 'boolean' }).notNull().default(false),
    reasoning: integer('reasoning', { mode: 'boolean' }).notNull().default(false),
    toolCall: integer('tool_call', { mode: 'boolean' }).notNull().default(false),
    structuredOutput: integer('structured_output', { mode: 'boolean' }), // nullable (optional in source)
    openWeights: integer('open_weights', { mode: 'boolean' }).notNull().default(false),
    temperature: text('temperature'), // bool/number union → store raw text
    modalitiesIn: text('modalities_in', { mode: 'json' }).$type<string[]>(), // enum array. FACET
    modalitiesOut: text('modalities_out', { mode: 'json' }).$type<string[]>(), // enum array. FACET
    contextLimit: integer('context_limit'), // models.dev limit.context. FACET (band) + SORT
    outputLimit: integer('output_limit'), // models.dev limit.output
    releaseDate: text('release_date'), // YYYY-MM(-DD)
    lastUpdated: text('last_updated'), // YYYY-MM(-DD). SORT (Updated column)
    knowledgeCutoff: text('knowledge_cutoff'), // YYYY-MM(-DD)
    reasoningOptions: text('reasoning_options', { mode: 'json' }), // toggle/effort/budget_tokens
    license: text('license'), // models.json enrichment
    links: text('links', { mode: 'json' }), // models.json enrichment
    weights: text('weights', { mode: 'json' }), // models.json enrichment
    benchmarks: text('benchmarks', { mode: 'json' }), // models.json enrichment
    hasPricing: integer('has_pricing', { mode: 'boolean' }).notNull().default(false), // computed
    sourceSha: text('source_sha'),
    updatedAt: integer('updated_at').notNull(),
  },
  (t) => [
    primaryKey({ columns: [t.providerSlug, t.modelId] }),
    index('idx_models_canonical_id').on(t.canonicalId), // logical-model GROUP BY / served_by lookup (03 §2.4)
    index('idx_models_provider').on(t.providerSlug), // provider-detail "models by provider" + provider facet
    index('idx_models_context_limit').on(t.contextLimit), // context-band facet + sort
    index('idx_models_status').on(t.status), // status facet
    index('idx_models_last_updated').on(t.lastUpdated), // Updated-column sort
    index('idx_models_name').on(t.name), // name sort
    // capability facets — see §2 for partial (WHERE = 1) indexes; plain ones below
    index('idx_models_tool_call').on(t.toolCall),
    index('idx_models_reasoning').on(t.reasoning),
    index('idx_models_attachment').on(t.attachment),
    index('idx_models_structured_output').on(t.structuredOutput),
  ],
)
export type ModelRow = typeof models.$inferSelect
export type NewModelRow = typeof models.$inferInsert

// ── model_pricing ─────────────────────────────────────────────────────────────
// PK = (provider_slug, model_id) 1:1 with models. Typed columns for the common facet
// dimensions (range filter / sort), JSON blobs for the long-tail (data-analysis §5.3).
// EVERYTHING normalized to USD per 1M tokens (Portkey cents/token → $/1M; §4.2).
export const modelPricing = sqliteTable(
  'model_pricing',
  {
    providerSlug: text('provider_slug').notNull(),
    modelId: text('model_id').notNull(),
    currency: text('currency').notNull().default('USD'),
    unit: text('unit').notNull().default('usd_per_1m'), // normalization marker
    inputPerM: integer('input_per_m', { mode: 'number' }), // REAL; models.dev cost.input ∥ Portkey request_token. FACET (range slider) + SORT
    outputPerM: integer('output_per_m', { mode: 'number' }), // REAL; FACET + SORT
    cacheReadPerM: integer('cache_read_per_m', { mode: 'number' }),
    cacheWritePerM: integer('cache_write_per_m', { mode: 'number' }),
    cacheWrite1hPerM: integer('cache_write_1h_per_m', { mode: 'number' }), // Portkey gap-fill
    reasoningPerM: integer('reasoning_per_m', { mode: 'number' }),
    inputAudioPerM: integer('input_audio_per_m', { mode: 'number' }),
    outputAudioPerM: integer('output_audio_per_m', { mode: 'number' }),
    contextOver200kJson: text('context_over_200k_json', { mode: 'json' }), // tiered output
    batchJson: text('batch_json', { mode: 'json' }), // Portkey batch_config
    finetuneJson: text('finetune_json', { mode: 'json' }), // Portkey finetune_config
    toolSurchargesJson: text('tool_surcharges_json', { mode: 'json' }), // Portkey additional_units.{web_search,…}
    imagePricingJson: text('image_pricing_json', { mode: 'json' }), // Portkey image{}
    videoPricingJson: text('video_pricing_json', { mode: 'json' }), // Portkey video fields
    customPricingJson: text('custom_pricing_json', { mode: 'json' }), // OUT-OF-SPEC region/exec-mode/tier
    calculateJson: text('calculate_json', { mode: 'json' }), // Portkey calculate AST
    extraJson: text('extra_json', { mode: 'json' }), // unknown models.dev cost keys (cached_input/citation/request) — §2.4 open Q
    pricingSource: text('pricing_source').notNull(), // 'modelsdev' | 'portkey' | 'both'
    portkeyDefaultUsed: integer('portkey_default_used', { mode: 'boolean' }).notNull().default(false),
    updatedAt: integer('updated_at').notNull(),
  },
  (t) => [
    primaryKey({ columns: [t.providerSlug, t.modelId] }),
    index('idx_pricing_input_per_m').on(t.inputPerM), // pricing range slider ($0–$20/M) + sort
    index('idx_pricing_output_per_m').on(t.outputPerM),
  ],
)
export type ModelPricingRow = typeof modelPricing.$inferSelect
export type NewModelPricingRow = typeof modelPricing.$inferInsert
```

> **Note on `REAL` columns.** drizzle-orm has no `real()` helper that round-trips ergonomically here; `integer({mode:'number'})` is NOT correct for fractional prices. **Override the generated DDL** for the `*_per_m` columns to `REAL` (see §1.1). Alternatively use drizzle's `real('input_per_m')` from `drizzle-orm/sqlite-core` if your drizzle-orm version exposes it (`^0.45.2` does) — prefer `real(...)`. The `{mode:'number'}` above is a placeholder; **switch to `real('…')`** and regenerate. Prices like `$3.00/1M`, `$0.30/1M` need fractional storage.

### 1.1 Expected `0001_<name>.sql` (drizzle-kit output, REAL-corrected)

This is what `just db-generate` should emit after the schema above (with `*_per_m` switched to `real(...)`). Copy-pasteable verbatim:

```sql
CREATE TABLE `sync_state` (
	`source` text PRIMARY KEY NOT NULL,
	`last_sha` text,
	`synced_at` integer,
	`last_status` text,
	`changed_count` integer
);
--> statement-breakpoint
CREATE TABLE `slug_alias` (
	`modelsdev_slug` text NOT NULL,
	`portkey_slug` text NOT NULL,
	`note` text,
	PRIMARY KEY(`modelsdev_slug`, `portkey_slug`)
);
--> statement-breakpoint
CREATE INDEX `idx_slug_alias_portkey` ON `slug_alias` (`portkey_slug`);
--> statement-breakpoint
CREATE TABLE `providers` (
	`slug` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`doc_url` text,
	`npm` text,
	`env_vars` text,
	`api_base_url` text,
	`provider_shape` text,
	`api_format_hint` text,
	`logo_kv_key` text,
	`portkey_slug` text,
	`model_count` integer DEFAULT 0 NOT NULL,
	`source_sha` text,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE INDEX `idx_providers_model_count` ON `providers` (`model_count`);
--> statement-breakpoint
CREATE INDEX `idx_providers_shape` ON `providers` (`provider_shape`);
--> statement-breakpoint
CREATE TABLE `models` (
	`provider_slug` text NOT NULL,
	`model_id` text NOT NULL,
	`canonical_id` text NOT NULL,
	`name` text NOT NULL,
	`family` text,
	`status` text,
	`attachment` integer DEFAULT false NOT NULL,
	`reasoning` integer DEFAULT false NOT NULL,
	`tool_call` integer DEFAULT false NOT NULL,
	`structured_output` integer,
	`open_weights` integer DEFAULT false NOT NULL,
	`temperature` text,
	`modalities_in` text,
	`modalities_out` text,
	`context_limit` integer,
	`output_limit` integer,
	`release_date` text,
	`last_updated` text,
	`knowledge_cutoff` text,
	`reasoning_options` text,
	`license` text,
	`links` text,
	`weights` text,
	`benchmarks` text,
	`has_pricing` integer DEFAULT false NOT NULL,
	`source_sha` text,
	`updated_at` integer NOT NULL,
	PRIMARY KEY(`provider_slug`, `model_id`),
	FOREIGN KEY (`provider_slug`) REFERENCES `providers`(`slug`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE INDEX `idx_models_canonical_id` ON `models` (`canonical_id`);
--> statement-breakpoint
CREATE INDEX `idx_models_provider` ON `models` (`provider_slug`);
--> statement-breakpoint
CREATE INDEX `idx_models_context_limit` ON `models` (`context_limit`);
--> statement-breakpoint
CREATE INDEX `idx_models_status` ON `models` (`status`);
--> statement-breakpoint
CREATE INDEX `idx_models_last_updated` ON `models` (`last_updated`);
--> statement-breakpoint
CREATE INDEX `idx_models_name` ON `models` (`name`);
--> statement-breakpoint
CREATE INDEX `idx_models_tool_call` ON `models` (`tool_call`);
--> statement-breakpoint
CREATE INDEX `idx_models_reasoning` ON `models` (`reasoning`);
--> statement-breakpoint
CREATE INDEX `idx_models_attachment` ON `models` (`attachment`);
--> statement-breakpoint
CREATE INDEX `idx_models_structured_output` ON `models` (`structured_output`);
--> statement-breakpoint
CREATE TABLE `model_pricing` (
	`provider_slug` text NOT NULL,
	`model_id` text NOT NULL,
	`currency` text DEFAULT 'USD' NOT NULL,
	`unit` text DEFAULT 'usd_per_1m' NOT NULL,
	`input_per_m` real,
	`output_per_m` real,
	`cache_read_per_m` real,
	`cache_write_per_m` real,
	`cache_write_1h_per_m` real,
	`reasoning_per_m` real,
	`input_audio_per_m` real,
	`output_audio_per_m` real,
	`context_over_200k_json` text,
	`batch_json` text,
	`finetune_json` text,
	`tool_surcharges_json` text,
	`image_pricing_json` text,
	`video_pricing_json` text,
	`custom_pricing_json` text,
	`calculate_json` text,
	`extra_json` text,
	`pricing_source` text NOT NULL,
	`portkey_default_used` integer DEFAULT false NOT NULL,
	`updated_at` integer NOT NULL,
	PRIMARY KEY(`provider_slug`, `model_id`)
);
--> statement-breakpoint
CREATE INDEX `idx_pricing_input_per_m` ON `model_pricing` (`input_per_m`);
--> statement-breakpoint
CREATE INDEX `idx_pricing_output_per_m` ON `model_pricing` (`output_per_m`);
```

> No FK on `model_pricing` → `models` is intentional: pricing can be upserted in a Portkey-first pass before the models.dev row lands within the same sync; the ingestion enforces the relationship logically and sets `models.has_pricing`. (If you prefer strict referential integrity, add the same composite FK as `models`→`providers`; D1 enforces FKs only when `PRAGMA foreign_keys=ON`, which `wrangler`/D1 sets by default.)

---

## 2. Hand-authored raw SQL — `drizzle/0002_catalog_fts.sql`

FTS5 virtual tables, triggers, and expression/partial indexes are **not expressible in drizzle-kit**. Author this file by hand and register it in `drizzle/meta/_journal.json` after `0001`. Copy-pasteable verbatim:

```sql
-- 0002_catalog_fts.sql — FTS5 search index + sync triggers + facet indexes
-- (drizzle-kit cannot emit virtual tables / triggers / expression indexes)

-- ── models_fts : external-content FTS5 over `models` ─────────────────────────
-- content='models' + content_rowid='rowid' → FTS stores only the index, not a
-- copy of the columns; we feed it via triggers. Columns are the searchable bag:
--   name, model_id, provider_slug, provider_name (denormalized at trigger time),
--   family, modalities (flattened), capabilities (keyword bag).
-- Unindexed helper columns avoid a join back for snippet display if needed.
CREATE VIRTUAL TABLE models_fts USING fts5(
  name,
  model_id,
  provider_slug,
  provider_name,
  family,
  modalities,        -- "text image pdf" flattened from modalities_in/out
  capabilities,      -- "tool_call reasoning vision attachment structured_output" keyword bag
  content='models',
  content_rowid='rowid',
  tokenize='unicode61 remove_diacritics 2'
);
--> statement-breakpoint

-- Because models_fts is external-content with denormalized provider_name +
-- computed modalities/capabilities, the triggers below build those payloads.
-- provider_name is looked up from providers; modalities/capabilities are derived
-- from the models row. The ingestion REINDEX step (data-analysis §6.2 step 8)
-- can also DELETE+INSERT a specific rowid to rebuild a single model's FTS row.

-- INSERT: index the new model row.
CREATE TRIGGER models_fts_ai AFTER INSERT ON models BEGIN
  INSERT INTO models_fts(rowid, name, model_id, provider_slug, provider_name, family, modalities, capabilities)
  VALUES (
    new.rowid,
    new.name,
    new.model_id,
    new.provider_slug,
    (SELECT name FROM providers WHERE slug = new.provider_slug),
    new.family,
    trim(
      replace(replace(replace(coalesce(new.modalities_in,'[]') || ' ' || coalesce(new.modalities_out,'[]'),
        '[',''),']',''),'"','')
    ),
    trim(
      (CASE WHEN new.tool_call = 1 THEN 'tool_call ' ELSE '' END) ||
      (CASE WHEN new.reasoning = 1 THEN 'reasoning ' ELSE '' END) ||
      (CASE WHEN new.attachment = 1 THEN 'attachment ' ELSE '' END) ||
      (CASE WHEN new.structured_output = 1 THEN 'structured_output ' ELSE '' END) ||
      (CASE WHEN instr(coalesce(new.modalities_in,''), 'image') > 0 THEN 'vision ' ELSE '' END)
    )
  );
END;
--> statement-breakpoint

-- DELETE: external-content FTS5 requires the special 'delete' command row.
CREATE TRIGGER models_fts_ad AFTER DELETE ON models BEGIN
  INSERT INTO models_fts(models_fts, rowid, name, model_id, provider_slug, provider_name, family, modalities, capabilities)
  VALUES ('delete', old.rowid, old.name, old.model_id, old.provider_slug, '', old.family, '', '');
END;
--> statement-breakpoint

-- UPDATE: delete-then-insert (the FTS5 external-content idiom).
CREATE TRIGGER models_fts_au AFTER UPDATE ON models BEGIN
  INSERT INTO models_fts(models_fts, rowid, name, model_id, provider_slug, provider_name, family, modalities, capabilities)
  VALUES ('delete', old.rowid, old.name, old.model_id, old.provider_slug, '', old.family, '', '');
  INSERT INTO models_fts(rowid, name, model_id, provider_slug, provider_name, family, modalities, capabilities)
  VALUES (
    new.rowid,
    new.name,
    new.model_id,
    new.provider_slug,
    (SELECT name FROM providers WHERE slug = new.provider_slug),
    new.family,
    trim(
      replace(replace(replace(coalesce(new.modalities_in,'[]') || ' ' || coalesce(new.modalities_out,'[]'),
        '[',''),']',''),'"','')
    ),
    trim(
      (CASE WHEN new.tool_call = 1 THEN 'tool_call ' ELSE '' END) ||
      (CASE WHEN new.reasoning = 1 THEN 'reasoning ' ELSE '' END) ||
      (CASE WHEN new.attachment = 1 THEN 'attachment ' ELSE '' END) ||
      (CASE WHEN new.structured_output = 1 THEN 'structured_output ' ELSE '' END) ||
      (CASE WHEN instr(coalesce(new.modalities_in,''), 'image') > 0 THEN 'vision ' ELSE '' END)
    )
  );
END;
--> statement-breakpoint

-- ── partial indexes for capability facets ──────────────────────────────────
-- The facet queries are WHERE tool_call = 1 (truthy filter), so a partial index
-- on the matching rows is both smaller and faster than the full-column index.
-- These supplement (or replace) the plain capability indexes from 0001.
CREATE INDEX idx_models_tool_call_true        ON models (provider_slug) WHERE tool_call = 1;
--> statement-breakpoint
CREATE INDEX idx_models_reasoning_true         ON models (provider_slug) WHERE reasoning = 1;
--> statement-breakpoint
CREATE INDEX idx_models_attachment_true        ON models (provider_slug) WHERE attachment = 1;
--> statement-breakpoint
CREATE INDEX idx_models_structured_output_true ON models (provider_slug) WHERE structured_output = 1;
--> statement-breakpoint
CREATE INDEX idx_models_open_weights_true      ON models (provider_slug) WHERE open_weights = 1;
--> statement-breakpoint

-- ── composite sort+filter covering indexes ─────────────────────────────────
-- The API-Models list filters by provider then sorts; these match the hot paths.
CREATE INDEX idx_models_status_context  ON models (status, context_limit);
--> statement-breakpoint
CREATE INDEX idx_models_provider_ctx    ON models (provider_slug, context_limit);
```

> **`vision` is synthetic.** The capability chips include `vision` but models.dev has no `vision` boolean — it is derived from `image ∈ modalities_in` (data-analysis §7.4 maps the `vision` chip to `modalities_in`). The FTS trigger bakes `vision` into the capability bag so `q=vision` works; the SQL facet filter uses `instr(modalities_in,'image')` / `json_each(modalities_in)`.

### 2.1 Query shapes the indexes serve

```sql
-- free-text search (API-Models q=)
SELECT m.* FROM models_fts f JOIN models m ON m.rowid = f.rowid
WHERE models_fts MATCH ?  ORDER BY rank LIMIT ? OFFSET ?;

-- capability facet (uses idx_models_tool_call_true)
SELECT * FROM models WHERE tool_call = 1 AND context_limit >= ? ;

-- pricing range slider $0–$20/M (uses idx_pricing_input_per_m)
SELECT m.* FROM models m JOIN model_pricing p USING (provider_slug, model_id)
WHERE p.input_per_m BETWEEN 0 AND 20 ORDER BY p.input_per_m;

-- modality facet (no native arrays → json_each over the TEXT JSON)
SELECT m.* FROM models m, json_each(m.modalities_in) j WHERE j.value = ?;

-- provider-detail "models by provider" (uses idx_models_provider)
SELECT * FROM models WHERE provider_slug = ? ORDER BY name;
```

---

## 3. Index recommendations per filtered/sorted facet

| Facet / sort (API §7.4) | Backing column(s) | Index | File |
|---|---|---|---|
| `q` free-text | FTS bag | `models_fts` (FTS5) | 0002 |
| `context_min` / context band; sort `context` | `models.context_limit` | `idx_models_context_limit`; `idx_models_provider_ctx`, `idx_models_status_context` | 0001 / 0002 |
| `status` | `models.status` | `idx_models_status`; `idx_models_status_context` | 0001 / 0002 |
| capability `tool_call` | `models.tool_call=1` | `idx_models_tool_call_true` (partial) | 0002 |
| capability `reasoning` | `models.reasoning=1` | `idx_models_reasoning_true` (partial) | 0002 |
| capability `attachment` | `models.attachment=1` | `idx_models_attachment_true` (partial) | 0002 |
| capability `structured` | `models.structured_output=1` | `idx_models_structured_output_true` (partial) | 0002 |
| capability `vision` | `instr(modalities_in,'image')` / `json_each` | (no btree — small table; FTS covers `q=vision`) | — |
| `open_weights` | `models.open_weights=1` | `idx_models_open_weights_true` (partial) | 0002 |
| `modality` (in/out) | `models.modalities_in/out` JSON | scanned via `json_each` (SQLite can't index JSON-array membership) | — |
| logical-model `GROUP BY` / `provider_count` / `served_by` / `sort=providers` (03 §2.4) | `models.canonical_id` | `idx_models_canonical_id` | 0001 |
| `provider` (multi) | `models.provider_slug` | `idx_models_provider` | 0001 |
| pricing range `input_per_m`; sort `price` | `model_pricing.input_per_m` | `idx_pricing_input_per_m` | 0001 |
| pricing `output_per_m` | `model_pricing.output_per_m` | `idx_pricing_output_per_m` | 0001 |
| sort `updated` (Updated col) | `models.last_updated` | `idx_models_last_updated` | 0001 |
| sort `name` | `models.name` | `idx_models_name` | 0001 |
| provider-list rank | `providers.model_count` | `idx_providers_model_count` (query `ORDER BY model_count DESC`) | 0001 |
| API-format facet | `providers.provider_shape` / `api_format_hint` | `idx_providers_shape` | 0001 |
| Portkey join reverse lookup | `slug_alias.portkey_slug` | `idx_slug_alias_portkey` | 0001 |

> **Modality / "no native array" caveat:** SQLite/D1 cannot btree-index membership inside a JSON array. Modality filters run `json_each(modalities_in)` as a table-valued scan. The catalog is small (~5,300 serving models, ~145 providers — data-analysis §2.1), so a full-table `json_each` scan is acceptable; if it ever isn't, denormalize a `modalities_flat TEXT` column and FTS/`LIKE` it (the FTS bag already carries this).

---

## 4. Column → source-field mapping (with conflict rule)

**Conflict rule baseline (data-analysis §1, §5.3):** on any field both sources carry, **models.dev wins** (schema-validated; Portkey is crowd-sourced). Portkey only fills `NULL` gaps. `pricing_source` records `'modelsdev' | 'portkey' | 'both'`; `portkey_default_used` flags rows filled from Portkey's per-provider `default` fallback.

### `providers`
| Column | Source | Conflict rule |
|---|---|---|
| `slug` | models.dev dir id (canonical) | models.dev only |
| `name` | models.dev `name` | models.dev only |
| `doc_url` | models.dev `doc` | models.dev only |
| `npm` | models.dev `npm` | models.dev only |
| `env_vars` | models.dev `env` (JSON array) | models.dev only |
| `api_base_url` | models.dev `api` | models.dev only; NULL for native providers (bridge fills from BodhiApp preset, §8) |
| `provider_shape` | computed from models.dev `api` refinement rule (§2.2) | computed |
| `api_format_hint` | computed (shape → OpenAI/Anthropic/Gemini) | computed |
| `logo_kv_key` | fetched `logos/<slug>.svg` → KV | computed (side-channel; not in JSON) |
| `portkey_slug` | `slug_alias` map | computed (alias lookup) |
| `model_count` | computed `COUNT(models)` | computed |
| `source_sha` / `updated_at` | ingest cursor / clock | computed |

### `models`
| Column | Source | Conflict rule |
|---|---|---|
| `provider_slug` + `model_id` | models.dev (composite PK; `model_id` = exact API string) | models.dev only |
| `canonical_id` | computed: models.dev `base_model` `LAB/MODEL` path id, else normalized `model_id` | computed (logical-model grouping key, 03 §2.4) |
| `name`, `family`, `status` | models.dev | models.dev only |
| `attachment`, `reasoning`, `tool_call`, `open_weights` | models.dev (required bools) | models.dev only |
| `structured_output`, `temperature` | models.dev (optional) | models.dev only |
| `modalities_in` / `modalities_out` | models.dev `modalities.{input,output}` | models.dev only (Portkey has none) |
| `context_limit` / `output_limit` | models.dev `limit.{context,output}` | models.dev only (Portkey has no context, §4.6) |
| `release_date`, `last_updated`, `knowledge_cutoff` | models.dev | models.dev only |
| `reasoning_options` | models.dev | models.dev only |
| `license`, `links`, `weights`, `benchmarks` | models.json (enrichment via `base_model` join) | models.dev only |
| `has_pricing` | computed (does a `model_pricing` row with any non-null price exist) | computed |
| `source_sha` / `updated_at` | ingest | computed |

### `model_pricing`
| Column | Source | Conflict rule |
|---|---|---|
| `currency` | both → `USD` | constant |
| `unit` | normalized → `usd_per_1m` | constant (Portkey cents/token → $/1M, §4.2) |
| `input_per_m` | models.dev `cost.input` ∥ Portkey `pay_as_you_go.request_token` | **models.dev wins**; Portkey fills NULL |
| `output_per_m` | models.dev `cost.output` ∥ Portkey `response_token` | **models.dev wins** |
| `cache_read_per_m` | models.dev `cost.cache_read` ∥ Portkey `cache_read_input_token` | **models.dev wins** |
| `cache_write_per_m` | models.dev `cost.cache_write` ∥ Portkey `cache_write_input_token` | **models.dev wins** |
| `cache_write_1h_per_m` | Portkey `additional_units.cache_write_1h` | Portkey only (gap-fill) |
| `reasoning_per_m` | models.dev `cost.reasoning` ∥ Portkey `additional_units.thinking_token` | **models.dev wins** |
| `input_audio_per_m` / `output_audio_per_m` | models.dev `cost.{input,output}_audio` ∥ Portkey `{request,response}_audio_token` | **models.dev wins** |
| `context_over_200k_json` | models.dev `OutputCost.context_over_200k` ∥ Portkey `custom_pricing.context_tier_map` | **models.dev wins** |
| `batch_json` | Portkey `batch_config` | Portkey only |
| `finetune_json` | Portkey `finetune_config` | Portkey only |
| `tool_surcharges_json` | Portkey `additional_units.{web_search,file_search,…}` | Portkey only |
| `image_pricing_json` | Portkey `image{}` | Portkey only |
| `video_pricing_json` | Portkey video fields | Portkey only |
| `custom_pricing_json` | Portkey `custom_pricing` (region/exec-mode/tier — OUT-OF-SPEC, parse raw) | Portkey only |
| `calculate_json` | Portkey `calculate` AST | Portkey only |
| `extra_json` | models.dev **unknown** cost keys (`cached_input`/`citation`/`request`) — §2.4 open Q, tolerate don't reject | models.dev (overflow) |
| `pricing_source` | computed `modelsdev`/`portkey`/`both` | computed |
| `portkey_default_used` | computed (filled from Portkey `default`) | computed |

### `slug_alias`
| Column | Source | Conflict rule |
|---|---|---|
| `modelsdev_slug` | manually seeded + maintained (canonical) | hand-maintained |
| `portkey_slug` | manually seeded (`bedrock`, `x-ai`, `vertex-ai`, …) | hand-maintained |

> Seed rows (data-analysis §4.7/§9 risk 4): `('amazon-bedrock','bedrock')`, `('xai','x-ai')`, `('google','google')`, `('google','vertex-ai')`. Extend as new divergences are found; a missing alias silently drops Portkey pricing, so log unmatched Portkey provider files during ingestion.

---

## 5. D1 limits that matter

1. **No native array / no native boolean.** SQLite (and thus D1) has neither. Arrays (`env`, `modalities`, `links`, `benchmarks`, every `*_json`) are stored as **JSON TEXT** and read with `json_each` / `json_extract` (D1 ships the JSON1 extension). Booleans are `INTEGER` 0/1 (`{mode:'boolean'}` in drizzle). You **cannot** btree-index membership inside a JSON array — modality/membership filters scan via `json_each` (acceptable at ~5.3k rows).

2. **FTS5 IS available on D1.** D1 includes the SQLite FTS5 module — `CREATE VIRTUAL TABLE … USING fts5(…)` works on the platform. (It is just not present in the current repo — there is zero FTS usage today; this migration introduces it.) External-content FTS5 (`content='models'`) keeps storage minimal by storing only the inverted index and feeding it via the three triggers in §2. Use the special `('delete', …)` command-row idiom for deletes/updates — a plain `DELETE FROM models_fts` on external-content tables corrupts the index.

3. **Statement & response size limits.** D1 caps a single SQL statement string at **100 KB** and a query response at **~1 MB / row count limits per query**; a bound-parameter query is capped at **100 parameters** per statement (`?N`). This bounds batch-upsert fan-out (see #4) and means large JSON blobs (`benchmarks`, `custom_pricing_json`) must stay well under the statement cap — they do (single-model blobs are KB-scale), but **do not** build one mega-INSERT with thousands of rows.

4. **Batch upsert via `INSERT … ON CONFLICT`.** Idempotent ingestion (data-analysis §6.2 step 6) uses upserts keyed on the PK:

   ```sql
   INSERT INTO models (provider_slug, model_id, name, /*…*/, updated_at)
   VALUES (?, ?, ?, /*…*/, ?)
   ON CONFLICT(provider_slug, model_id) DO UPDATE SET
     name = excluded.name,
     /* …every mutable column… */
     updated_at = excluded.updated_at;
   ```

   This mirrors the existing `putCache` pattern (`worker/db/queries.ts` uses drizzle `onConflictDoUpdate`). **Chunk** multi-row upserts so each statement stays under the 100 KB / 100-param ceiling (e.g. ≤ ~50 rows per statement for the wide `models` table, fewer for `model_pricing` given its many columns), and wrap a sync batch in `db.batch([...])` (drizzle/D1 batch API) for atomicity per chunk. Re-running on the same SHA is a no-op because every write is a PK upsert (idempotency, §6.2).

5. **FK enforcement.** D1 runs with `PRAGMA foreign_keys=ON`; the `models.provider_slug → providers.slug ON DELETE CASCADE` FK is enforced, so a pruned provider cascades its models (which in turn fire `models_fts_ad` to clean the FTS index). `model_pricing` intentionally has no FK (Portkey-first ordering) — the ingestion sets `has_pricing` and prunes orphans logically.

6. **Migrations are NOT auto-applied in `@cloudflare/vitest-pool-workers`.** Tests must `CREATE` every table + the FTS virtual table + triggers in `beforeEach` (repo pattern, `e2e-cf/detail.cache.test.ts:9`). Keep §1.1 + §2 DDL verbatim so tests replay it exactly. `wrangler dev` / remote apply via `just db-migrate*` normally.
