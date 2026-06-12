# docs/conventions/ — CLAUDE.md

Cross-cutting practices and methodology. Code-level detail lives in crate docs; these are the durable how-we-work references.

| Doc | Covers |
|---|---|
| `testing.md` | **Read first before writing any test.** Testing hub + philosophy — the optimized E2E style (one journey = one `test()` of batched `test.step()` cycles), server decision tree, cross-layer rules; links to all canonical per-layer test docs |
| `test-utils-packaging.md` | The dual-availability test-utils feature-flag pattern (shared test objects across crates) |
| `unused-upgrade-dependencies.md` | Rust dependency hygiene: `cargo-machete` removal + systematic upgrade methodology |
| `github-workflows-context.md` | CI/CD architecture + Makefile-target conventions (points at `.github/` for the live inventory) |
| `cuda-dockerfile-optimizations.md` | CUDA llama-server tuning flags (`devops/cuda.Dockerfile`, `BODHI_LLAMACPP_ARGS_CUDA`) |
| `model-parameters.md` | llama.cpp CLI-flag conceptual reference (ownership: `llama_server_proc` + `BODHI_LLAMACPP_ARGS_*`) |
| `setup-processes.md` | The current app setup flow (`setup → resource_admin → ready`; OAuth mandatory) |
| `llm-resource-server.md` | Vision/architecture of Bodhi as an OAuth2 LLM resource server |

**Canonical test docs** `testing.md` links to (never duplicated): `crates/lib_bodhiserver/tests-js/E2E.md` + `CLAUDE.md`, `crates/bodhi/src/TESTING.md`, `crates/routes_app/TESTING.md`, `crates/services/src/test_utils/CLAUDE.md`, and the `test-services` / `test-routes-app` skills.
