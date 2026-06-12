# Model Metadata API Specification

**Feature**: Model metadata for local GGUF and remote API models in BodhiApp
**Date**: 2026-01-12
**Status**: Iteration 1 Implementation Ready, Iteration 2 Design Complete
**Scope**: Iteration 1 (Local GGUF models), Iteration 2 (Remote API models)

## Overview

This specification defines model metadata API for BodhiApp that exposes capability information for both local GGUF models and remote API models. The approach extends existing `/bodhi/v1/models` endpoints with optional metadata fields while maintaining backwards compatibility.

The feature enables:
- **Model capability discovery**: Vision, audio, thinking/reasoning, function calling support detection
- **Context length information**: Max input/output tokens from GGUF metadata or remote API specs
- **Architecture details**: Model family, parameter count, quantization level, format
- **Pricing information** (iteration 2): Detailed pricing tiers for remote API models
- **Model aliasing** (iteration 2): Flexible model ID matching for provider-specific naming

## Problem Statement

Currently, BodhiApp lacks metadata exposure for both local and remote models:
- No way to query which models support vision/audio
- No context length information from GGUF files or remote APIs
- UI cannot adapt features based on model capabilities (vision upload, context warnings)
- Clients must hardcode model capabilities
- No pricing information for remote API models
- No flexible aliasing for provider-specific model IDs

## Solution - Iteration 1

Extend existing `/bodhi/v1/models` endpoints with optional `metadata` field:
- `GET /bodhi/v1/models` - List with optional `metadata` per model
- `GET /bodhi/v1/models/{alias}` - Detail with optional `metadata`
- `POST /bodhi/v1/models/refresh` - Trigger metadata refresh for all local models (admin)
- `POST /bodhi/v1/models/{id}/refresh` - Trigger metadata refresh for single model (admin)

Metadata extracted from GGUF files via manual admin trigger, stored in SQLite.

## Document Structure

- **[01-research-report.md](01-research-report.md)** - Background research on llama.cpp, OpenRouter, Ollama, LiteLLM, AI Gateway approaches
- **[02-iteration1-design.md](02-iteration1-design.md)** - Complete iteration 1 design specification including:
  - Extensible database schema (local GGUF + future remote API support)
  - In-memory queue service with interface-based design (QueueProducer/QueueConsumer)
  - Single worker with producer-consumer pattern
  - Flattened core capabilities for efficient SQL queries
  - GGUF metadata extraction logic
  - Testing strategy and performance considerations
  - Deployment notes and configuration
- **[03-iteration2-design.md](03-iteration2-design.md)** - Complete iteration 2 design specification including:
  - api.getbodhi.app contract (cursor-based pagination)
  - Queue service extension (SyncAll/SyncSingle tasks)
  - Schema migration for remote models
  - ApiAlias enrichment with model name normalization
  - Pricing information structure
  - Minimal UI (sync button, pricing badge)

## Key Design Decisions

### Iteration 1 (Local GGUF Models)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Endpoint approach** | Extend existing `/bodhi/v1/models` | Backwards compatible, no new endpoints |
| **Metadata embedding** | Inline optional `metadata` field | OpenRouter-style, single request |
| **Extraction trigger** | Manual admin refresh | Controlled, no performance impact on discovery |
| **Execution model** | In-memory queue with single worker (producer-consumer) | Non-blocking, simple, interface-based for future DB extension |
| **Data storage** | SQLite `model_metadata` table | Extensible for local and remote models |
| **Primary key** | Auto-increment ID with composite unique (source, alias) | Supports user/model/api sources |
| **Capability storage** | Flattened columns for core capabilities | Efficient SQL queries (vision, audio, tools, context) |
| **Additional metadata** | JSON field for extensibility | Future-proof (pricing, rate limits, etc.) |
| **Schema naming** | Own consistent schema | Not llama.cpp compatible, prepares for iteration 2 |
| **UI surface** | Per-model refresh button (admin) | Granular control, admin-only feature |

### Iteration 2 (Remote API Models)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Sync source** | Contract-first, server deferred | Design BodhiApp client + contract, build server separately |
| **Queue reuse** | Same QueueProducer/QueueConsumer | Add SyncAll/SyncSingle task types alongside RefreshAll/RefreshSingle |
| **Sync trigger** | Manual only (admin) | Consistent with iteration 1 approach |
| **Alias storage** | model_metadata table with aliases JSON array | Single source of truth, synced from api.getbodhi.app |
| **ApiAlias integration** | Enrich existing | Join ApiAlias with model_metadata by matching normalized model names |
| **Seed strategy** | No seeding | Start empty, wait for first sync from api.getbodhi.app |
| **Sync protocol** | Cursor-based paginated | Server returns cursor token, client passes for next page |
| **UI scope** | Minimal | Sync button (admin), pricing badge display |
| **Sync failure** | Silent log | Log warning, return error to caller, use stale data |
| **Endpoint design** | Query parameter `scope` | POST /bodhi/v1/models/refresh?scope=local\|remote\|all |
| **Pricing detail** | Detailed tiers as JSON | Informational only (input/output/cached/batch pricing) |
| **API auth** | No auth | Public endpoint, server handles rate limiting |

## Dependencies

### Iteration 1
- Existing GGUF parser in `crates/objs/src/gguf` (enhanced with capabilities extraction)
- SQLite database via `DbService`
- Queue service (new) with in-memory queue and single worker
- Existing `/bodhi/v1/models` routes (extended)
- Admin authentication middleware
- Interface-based design (QueueProducer/QueueConsumer) for future extensibility

### Iteration 2
- HTTP client (reqwest) for api.getbodhi.app communication
- Queue service extension (SyncAll/SyncSingle task types)
- Schema migration for remote model support
- ApiAlias enrichment logic (model name normalization)

## Success Criteria - Iteration 1

- [ ] Extensible database schema supports local GGUF and future remote API models
- [ ] Core capabilities stored in flattened columns for efficient SQL queries
- [ ] In-memory queue service with single worker for async metadata extraction
- [ ] Interface-based design (QueueProducer/QueueConsumer) for future DB-backed extension
- [ ] `GET /bodhi/v1/models` returns optional `metadata` for local GGUF models
- [ ] `GET /bodhi/v1/models/{alias}` returns optional `metadata`
- [ ] `POST /bodhi/v1/models/refresh` enqueues extraction task (admin)
- [ ] `POST /bodhi/v1/models/{id}/refresh` enqueues single model task (admin)
- [ ] Refresh endpoints return 202 Accepted immediately
- [ ] Metadata includes capabilities (vision/audio/thinking/tools), context, architecture
- [ ] Database queries support filtering by capabilities (e.g., all vision models)
- [ ] Backwards compatibility maintained (existing clients unaffected)
- [ ] UI shows refresh button (admin only) with metadata badges
- [ ] Snapshot change detection triggers re-extraction on refresh
- [ ] Worker processes tasks sequentially (FIFO order)

## Solution - Iteration 2

Extend iteration 1 to support remote API model metadata:
- `POST /bodhi/v1/models/refresh?scope=remote|all` - Sync from api.getbodhi.app (admin)
- Enrich existing ApiAlias responses with synced metadata
- Cursor-based pagination for scalable metadata sync
- Pricing information display in UI (minimal)
- Model name normalization for flexible aliasing

Metadata synced from api.getbodhi.app via manual admin trigger, stored in same `model_metadata` table.

## Success Criteria - Iteration 2

- [ ] Schema migration adds aliases, pricing, provider, name, description, synced_at columns
- [ ] Queue service extends RefreshTask with SyncAll/SyncSingle variants
- [ ] HTTP sync client implements cursor-based pagination
- [ ] Worker processes sync tasks and upserts remote model metadata
- [ ] `POST /bodhi/v1/models/refresh?scope=remote` triggers sync (admin)
- [ ] `GET /bodhi/v1/models` returns ApiAlias with enriched metadata
- [ ] Model name normalization matches ApiAlias models to synced metadata
- [ ] Pricing displayed as badge on ApiAlias entries (UI)
- [ ] Sync button visible to admin users only (UI)
- [ ] Sync failure logged gracefully, app continues with stale data
- [ ] api.getbodhi.app contract defined for future server implementation
- [ ] Backwards compatibility maintained (iteration 1 unaffected)

## Future Enhancements (Iteration 3+)

Deferred to future iterations:
- Automatic background sync (periodic)
- Sync status endpoint for progress tracking
- Seed data for popular models (offline use)
- Provider/capability filtering in API
- Full remote model catalog UI
