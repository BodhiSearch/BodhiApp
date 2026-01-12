# Model Metadata API Specification

**Feature**: Unified model capabilities and metadata API for BodhiApp
**Date**: 2026-01-12
**Status**: Planning Complete

## Overview

This specification defines a unified model metadata API for BodhiApp that exposes rich capability information for both local GGUF models and remote API providers (OpenAI, Anthropic, OpenRouter, etc.).

The feature enables:
- **Model capability discovery**: Vision, audio, thinking/reasoning, function calling
- **Context length information**: Max input/output tokens
- **Pricing transparency**: Per-token costs with staleness tracking
- **Architecture details**: Model family, parameter count, quantization
- **Remote metadata sync**: Regular updates from `api.getbodhi.app`

## Problem Statement

Currently, BodhiApp lacks a unified way to expose model capabilities:
- No way to query which models support vision/audio/thinking
- No pricing information for cost estimation
- No context length information from local models
- Clients must hardcode model capabilities

## Solution

Implement new endpoints:
- `GET /bodhi/v1/models` - List all models with metadata
- `GET /bodhi/v1/models/{id}/meta` - Get specific model metadata
- `POST /bodhi/v1/models/sync` - Trigger metadata sync (admin only)

Metadata stored in SQLite, seeded at install, synced periodically from remote API.

## Document Structure

- **[01-research-report.md](01-research-report.md)** - Comprehensive research on existing APIs (llama.cpp, OpenRouter, Ollama, LiteLLM, AI Gateway)
- **[02-design-spec.md](02-design-spec.md)** - Final schema design, user preferences, API contracts
- **[03-implementation-plan.md](03-implementation-plan.md)** - Phased implementation strategy with 7 incremental PRs

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Endpoint path** | `/bodhi/v1/models/{id}/meta` | Separate from OpenAI-compatible `/v1/models` |
| **Data storage** | SQLite table | Consistent with existing data layer |
| **Local GGUF** | Extract from GGUF files | No dependency on running llama.cpp server |
| **Remote models** | Hybrid: proxy + static mapping | Best of both approaches |
| **Model IDs** | Canonical (e.g., "claude-sonnet-4") | Provider field separate |
| **Sync mechanism** | Poll `api.getbodhi.app` | Dedicated metadata service |
| **Delivery** | 7 incremental PRs | Easier review, parallel work |

## Dependencies

- Existing GGUF parser in `crates/objs/src/gguf` (to be enhanced)
- SQLite database via `DbService`
- HTTP client for remote sync
- `api.getbodhi.app` metadata service (external, maintained separately)

## Timeline

Phased delivery with PRs that can be developed in parallel:
1. **Phase db-schema**: Database schema + domain types
2. **Phase gguf-enhance**: GGUF parser enhancements (parallel with #1)
3. **Phase db-service**: Repository layer
4. **Phase sync-service**: Remote sync
5. **Phase routes**: API endpoints
6. **Phase seed-data**: Initial seed data
7. **Phase integration**: Alias system integration

## Success Criteria

- [ ] `/bodhi/v1/models` returns metadata for all models
- [ ] `/bodhi/v1/models/{id}/meta` returns correct metadata
- [ ] Local GGUF models have extracted capabilities (vision/audio/context)
- [ ] Remote sync updates database with new models
- [ ] Pricing information included with staleness warning
- [ ] UI can consume endpoints for capability-based features
