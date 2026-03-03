# Plan: Make Windows Build Optional in publish-app-bindings Workflow

## Context

The `publish-app-bindings.yml` workflow fails entirely when the Windows build fails ([run #22605477953](https://github.com/BodhiSearch/BodhiApp/actions/runs/22605477953)). The `publish` job has `needs: build`, which requires ALL matrix entries (macOS, Linux, Windows) to succeed. A Windows build failure blocks macOS+Linux publishing.

Goal: Make macOS and Linux mandatory, Windows optional. If Windows passes, include it; otherwise publish without it.

## File to Modify

- `.github/workflows/publish-app-bindings.yml`

## Changes

### 1. Add `optional` flag to Windows matrix entry

Add `optional: true` to the Windows matrix settings. macOS and Linux entries get no flag (defaults to false).

```yaml
matrix:
  settings:
    - host: macos-latest
      target: aarch64-apple-darwin
      build: npm run build:release -- --target aarch64-apple-darwin
    - host: ubuntu-latest
      target: x86_64-unknown-linux-gnu
      build: npm run build:release -- --target x86_64-unknown-linux-gnu
    - host: windows-latest
      build: npm run build:release:win
      target: x86_64-pc-windows-msvc
      optional: true
```

### 2. Add `continue-on-error` to build job

```yaml
continue-on-error: ${{ matrix.settings.optional || false }}
```

This makes the Windows build failure non-blocking. The overall `build` job still succeeds if only Windows fails, so `needs: build` on the `publish` job passes.

### 3. Add step in publish job to remove npm dirs for missing artifacts

After the `Setup npm packages` step (which runs `napi create-npm-dir` and creates dirs for ALL 3 platforms), add a step to remove platform dirs that have no corresponding artifact:

```yaml
- name: Remove npm dirs for unavailable platform artifacts
  working-directory: crates/lib_bodhiserver_napi
  shell: bash
  run: |
    for dir in npm/*/; do
      platform=$(basename "$dir")
      if ! ls artifacts/*/app-bindings.${platform}.node 1>/dev/null 2>&1; then
        echo "::warning::No artifact found for $platform, removing $dir"
        rm -rf "$dir"
      fi
    done
```

This must go **between** `Setup npm packages` and `Move artifacts` steps.

### Why downstream steps work without changes

- **`npm run artifacts`** (`napi artifacts`): moves `.node` files into existing `npm/<platform>/` dirs only
- **Platform package publish loop**: iterates `npm/*/` so only publishes dirs that exist
- **`update-optional-dependencies.js`**: scans `npm/` dirs dynamically, only lists existing platforms in `optionalDependencies`
- **`verify-npm-packages.js`**: scans `npm/` dirs dynamically, only verifies packages for existing platforms

## Verification

1. Trigger the workflow manually and verify:
   - If all 3 builds pass: all 3 platform packages + main package published
   - If Windows build fails: only macOS + Linux platform packages + main package published (with 2 optionalDependencies)
2. Check the `publish` job logs for the warning message when Windows artifact is missing
3. Verify published npm package has correct `optionalDependencies` (2 or 3 depending on Windows)
