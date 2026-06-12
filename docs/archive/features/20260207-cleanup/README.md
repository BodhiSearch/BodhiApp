# Crate-by-Crate Cleanup

Systematic cleanup of each crate in the BodhiApp workspace, proceeding upstream-to-downstream. Each crate is cleaned in a separate Claude Code session using the templatized prompt.

## Recommended Sequence

1. **Run optimization prompt first**: Use `claude-code-optimization-prompt.md` once to set up reusable commands/agents
2. **Clean crates in order**: Use `crate-cleanup-prompt.md` for each crate, substituting placeholders

## Usage

1. Copy-paste the contents of `crate-cleanup-prompt.md` into a new Claude Code session
2. Follow up with the crate name:

```
$CRATE_NAME=services

Implement the above plan for crates/services
```

The crate path is derived automatically (`crates/$CRATE_NAME`, with special case `crates/bodhi/src-tauri` for bodhi).

## Processing Order

Process crates in this order (upstream dependencies first):

- [ ] 1. `errmeta_derive` — `crates/errmeta_derive`
- [ ] 2. `objs` — `crates/objs`
- [ ] 3. `llama_server_proc` — `crates/llama_server_proc`
- [ ] 4. `services` — `crates/services`
- [ ] 5. `commands` — `crates/commands`
- [ ] 6. `server_core` — `crates/server_core`
- [ ] 7. `auth_middleware` — `crates/auth_middleware`
- [ ] 8. `routes_oai` — `crates/routes_oai`
- [ ] 9. `routes_app` — `crates/routes_app`
- [ ] 10. `routes_all` — `crates/routes_all`
- [ ] 11. `server_app` — `crates/server_app`
- [ ] 12. `lib_bodhiserver` — `crates/lib_bodhiserver`
- [ ] 13. `bodhi` — `crates/bodhi/src-tauri`

**Skipped**: async-openai/*, xtask, lib_bodhiserver_napi, integration-tests, ci_optims

## Artifacts

After each crate cleanup, a report is generated at:
```
ai-docs/features/20260207-cleanup/$CRATE_NAME/report.md
```

CLAUDE.md and PACKAGE.md for the crate are also updated via the docs-updater agent.
