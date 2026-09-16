# 14 — cloudflared binary: detect, guide install, download (2026-09-15)

Research for the "bring your own `cloudflared`" and "download on trigger" paths of the named-tunnel feature. Scope: detection/install/download only — not the tunnel-run or Keycloak-sync design (see `bodhiapp-cloudflare-tunnel-feasibility.md` and `01-bodhi-app-codebase-map.md`).

## Corrections to earlier docs in this folder

- **License is plain Apache-2.0, not "Apache-2.0 with Runtime Library Exception."** `00-consolidated-research.md` and `bodhiapp-cloudflare-tunnel-feasibility.md` both state the Runtime Library Exception applies — verified false. The raw `LICENSE` file at `github.com/cloudflare/cloudflared` is the unmodified Apache 2.0 text with no exception clause, and Homebrew's formula metadata tags it plainly `Apache-2.0`. (The Runtime Library Exception is a Swift/LLVM-ecosystem addendum — `swift.org/LICENSE.txt` — evidently confused with cloudflared in the earlier pass.) No special attribution relief; standard Apache-2.0 obligations apply (§4, see Licensing below).
- Release assets do **not** include a Cloudflare-published `checksums.txt` or signature file (verified via GitHub API on the `2026.9.1` release — 26 binary/package assets, zero checksum/signature files). The "SHA256 provided" behavior earlier tooling reported is **GitHub's own per-asset digest** (`asset.digest` field, auto-computed by GitHub for every uploaded release asset), not something Cloudflare signs or publishes independently. See Verification below.

## 1. Official install channels

| Channel | Command / asset | Notes |
|---|---|---|
| macOS Homebrew | `brew install cloudflared` | Formula `cloudflared`, license tag `Apache-2.0`. Binary lands at `/opt/homebrew/opt/cloudflared/bin/cloudflared` (Apple Silicon, symlinked from `/opt/homebrew/Cellar/cloudflared/<ver>/bin/cloudflared`) or `/usr/local/opt/cloudflared/bin/cloudflared` (Intel), and Homebrew also symlinks it into `/opt/homebrew/bin/cloudflared` / `/usr/local/bin/cloudflared` respectively — verified via `brew info cloudflared` (current stable `2026.9.1`, bottled). `brew services start cloudflared` for a login-launchd service; plain `brew install` alone does not start anything. |
| macOS .pkg | `cloudflared-amd64.pkg`, `cloudflared-arm64.pkg` | From GitHub Releases (below). Installs the binary system-wide, not into a Homebrew prefix — exact install path not confirmed in this pass; treat as **UNVERIFIED**, confirm with `pkgutil --files` before relying on a path. |
| Windows winget | `winget install Cloudflare.cloudflared` | Package id `Cloudflare.cloudflared`, manifests at `microsoft/winget-pkgs/manifests/c/Cloudflare/cloudflared/`. This is an **MSI-backed** winget package (not a "portable" package type), so it installs via the `.msi` below and does not go through winget's `Links`-symlink PATH mechanism. winget package has lagged upstream releases before and been reported unsigned/outdated by users — `github.com/cloudflare/cloudflared/issues/1576` (open, unresolved as of this research) — don't assume winget always has the newest build. |
| Windows .msi | `cloudflared-windows-amd64.msi` / `-386.msi` | Default install dir is **`C:\Program Files (x86)\cloudflared\`** even for the 64-bit (`amd64`) MSI — tracked as a bug, not fixed: `github.com/cloudflare/cloudflared/issues/992`. Don't assume `Program Files\cloudflared` (no `(x86)`) on 64-bit hosts. |
| Windows .exe | `cloudflared-windows-amd64.exe` / `-386.exe` | Standalone binary, no installer, no PATH registration — caller must place/reference it explicitly. |
| Linux apt (Debian/Ubuntu) | see block below | `pkg.cloudflare.com` repo, package `cloudflared`. |
| Linux yum/dnf (RHEL/CentOS/Amazon Linux) | see block below | Same repo family, `.repo` file. |
| Linux static binaries / .deb / .rpm | `cloudflared-linux-{386,amd64,arm,arm64,armhf}` (+ `.deb` per arch, `.rpm` for `386,arm,armhf` and separately-named `aarch64`/`x86_64`) | From GitHub Releases. No install location convention — caller chooses. |
| Docker | `docker pull cloudflare/cloudflared:latest` | Not applicable to BodhiApp's desktop scope; noted for completeness. |

### apt (Debian/Ubuntu) — verified against `pkg.cloudflare.com` docs
```bash
sudo mkdir -p --mode=0755 /usr/share/keyrings
curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg | sudo tee /usr/share/keyrings/cloudflare-main.gpg >/dev/null
echo 'deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] https://pkg.cloudflare.com/cloudflared any main' | sudo tee /etc/apt/sources.list.d/cloudflared.list
sudo apt-get update && sudo apt-get install cloudflared
```
Nightly channel swaps the second line's host to `next.pkg.cloudflare.com`. Cloudflare rotated the repo's public signing key on **2025-10-30** — a stale `/usr/share/keyrings/cloudflare-main.gpg` from before that date will fail signature verification; re-run the `curl | tee` step if apt reports a GPG error.

### yum/dnf (RHEL family)
```bash
curl -fsSL https://pkg.cloudflare.com/cloudflared.repo | sudo tee /etc/yum.repos.d/cloudflared.repo
sudo yum update && sudo yum install cloudflared
```

### GitHub "latest" download URLs — exact asset names (verified via GitHub API, release `2026.9.1`, 2026-09-11)
Pattern: `https://github.com/cloudflare/cloudflared/releases/latest/download/<asset>`

| Target | Asset name |
|---|---|
| macOS Intel | `cloudflared-darwin-amd64.tgz` (contains the `cloudflared` binary; also `cloudflared-amd64.pkg` installer) |
| macOS Apple Silicon | `cloudflared-darwin-arm64.tgz` (also `cloudflared-arm64.pkg`) |
| Linux x86_64 | `cloudflared-linux-amd64` (raw binary), `cloudflared-linux-amd64.deb`, `cloudflared-linux-x86_64.rpm` |
| Linux arm64 | `cloudflared-linux-arm64` (raw binary), `cloudflared-linux-arm64.deb`, `cloudflared-linux-aarch64.rpm` |
| Windows x64 | `cloudflared-windows-amd64.exe`, `cloudflared-windows-amd64.msi` |

Full asset list for the release also includes `cloudflared-linux-386{,.deb,.rpm}`, `cloudflared-linux-arm{,.deb,.rpm}`, `cloudflared-linux-armhf{,.deb,.rpm}`, `cloudflared-windows-386{.exe,.msi}`, and FIPS-build variants (`cloudflared-fips-linux-amd64{,.deb}`, `cloudflared-fips-linux-x86_64.rpm`) — not relevant to BodhiApp's target matrix but confirms the darwin `.tgz` (not raw binary) is the only macOS non-installer asset shape, unlike Linux which ships a bare executable.

### Checksums / signatures — what's actually published
- **No `checksums.txt`, `.sig`, or cosign attestation from Cloudflare** on the release page (verified: `2026.9.1` has 26 assets, all binaries/packages, zero checksum/signature files).
- Every asset **does** carry a `sha256:<hex>` digest, but that's GitHub's platform-level `digest` field (computed by GitHub at upload time for any release asset on any repo), retrievable via `GET /repos/cloudflare/cloudflared/releases/latest` → `assets[].digest`. It proves the download matches what's currently hosted on GitHub; it is **not** an integrity attestation signed by Cloudflare, so it doesn't protect against a compromised release upload the way a maintainer-signed checksum file would.
- Practical verification path for BodhiApp: fetch the GitHub API release metadata alongside the asset, compare `sha256sum <file>` to the API's `digest` for that asset name. This is TOFU-grade (trust GitHub's API response), not cryptographic-signature-grade.
- The **apt/yum repos are GPG-signed** (the `cloudflare-main.gpg` key) — that's the one channel with real signature verification built into the OS package manager's install flow. Prefer apt/yum over raw-binary download on Linux when a package manager is available, for this reason alone.

## 2. Well-known probe locations (PATH discovery)

**Why this matters for BodhiApp specifically**: Tauri apps launched from Finder/Explorer/GUI (not a terminal) get the OS's minimal login-window `PATH`, not the user's shell-profile-augmented one — Homebrew's own Apple-Silicon install note ("add `/opt/homebrew/bin` to your PATH") is exactly the kind of shell-profile-only change a GUI launch never sees.

### macOS
Probe order (most → least specific):
1. `/opt/homebrew/bin/cloudflared` — Homebrew on Apple Silicon
2. `/opt/homebrew/opt/cloudflared/bin/cloudflared` — Homebrew keg path, Apple Silicon (same binary, in case the `bin` symlink is missing)
3. `/usr/local/bin/cloudflared` — Homebrew on Intel, or manual install
4. `/usr/local/opt/cloudflared/bin/cloudflared` — Homebrew keg path, Intel
5. `~/.local/bin/cloudflared` — common manual/pipx-style user install location
6. `/usr/bin/cloudflared` — unlikely (not Apple-shipped) but cheap to check

### Linux
1. `/usr/bin/cloudflared` — apt/yum package install (the common case)
2. `/usr/local/bin/cloudflared` — manual binary placement
3. `~/.local/bin/cloudflared` — user-local manual install

### Windows
1. `%ProgramFiles(x86)%\cloudflared\cloudflared.exe` — MSI default (both arches, per the `Program Files (x86)` bug above)
2. `%ProgramFiles%\cloudflared\cloudflared.exe` — in case a future MSI fixes the bug, or a manual install used the "correct" dir
3. `%LOCALAPPDATA%\Microsoft\WinGet\Packages\Cloudflare.cloudflared_Microsoft.Winget.Source_8wekyb3d8bbwe\` — winget's per-package cache dir (exact hash suffix varies by publisher id; glob rather than hard-code) — **UNVERIFIED** exact subpath for this package, but this is winget's documented package-storage convention (`%LOCALAPPDATA%\Microsoft\WinGet\Packages\<PackageId>_<PublisherId>\`)
4. `%LOCALAPPDATA%\Microsoft\WinGet\Links\cloudflared.exe` — only populated for winget **portable**-type packages; cloudflared's winget manifest is MSI-backed so this likely does **not** apply, listed for completeness/future-proofing

### Cross-platform: also just try `PATH`
Always run `which cloudflared` / `where cloudflared.exe` (or an equivalent `std::process::Command` lookup) first — covers users who installed via a method not on the list, or who explicitly exported PATH in their launch environment.

### Prior art for the mechanism
- **`fix-path-env-rs`** (`github.com/tauri-apps/fix-path-env-rs`, Tauri's own crate): calling `fix_path_env::fix()` early in `main()` re-derives `PATH` on macOS/Linux by reading shell config (`.zshrc`, `.bash_profile`, etc. — exact mechanism, e.g. whether it spawns a login shell like `$SHELL -licd 'echo $PATH'`, was **not confirmed** from the README in this pass; treat as UNVERIFIED, read `src/lib.rs` before depending on the exact approach) and applies it to the current process's env, fixing PATH for every subsequent `Command::new(...)` lookup — not just one binary. Windows gets a different fix (PATH sometimes isn't inherited via `std::process::Command`).
- Given BodhiApp is a Tauri app already on the `tauri` dependency tree, adopting `fix-path-env-rs` in `native_init.rs` would fix PATH discovery for `cloudflared` (and any other externally-installed tool) globally, as an alternative/complement to hand-rolled well-known-path probing.
- **BodhiApp's own existing precedent is different and worth reusing conceptually**: `BODHI_EXEC_LOOKUP_PATH` (`crates/services/src/settings/constants.rs:22`, accessor `SettingService::exec_lookup_path()` at `crates/services/src/settings/setting_service.rs:309-314`) does **not** probe PATH at all — it points `llama_server_proc` at a BodhiApp-managed binary directory (defaulted from the Tauri resource dir in `crates/bodhi/src-tauri/src/native_init.rs`, or `CARGO_MANIFEST_DIR/bin` in dev server mode, `crates/bodhi/src-tauri/src/server_init.rs:42-44`). BodhiApp already owns and manages the llama-server binary lifecycle rather than relying on a system install. The same pattern (own the binary under `BODHI_HOME`, verified/downloaded by BodhiApp itself) is the more consistent architectural fit for `cloudflared` than a PATH-probing detector — probing the well-known locations above is best framed as "detect a system install to *avoid* redundant download," not as the primary resolution strategy.

## 3. Version handling

- **Release cadence**: frequent — the `2026.9.1` tag observed in this research was published 2026-09-11; historical tags follow `YYYY.M.PATCH` (e.g. `2025.8.1`, `2024.10.0`), not strict semver, and multiple releases per month are typical based on the tag density in `RELEASE_NOTES`.
- **Support window**: "Cloudflare supports versions of `cloudflared` that are within one year of the most recent release" — official statement on the Cloudflare One downloads/support-lifecycle docs. Breaking changes may land that only affect versions older than that window.
- **`cloudflared update`**: triggers an immediate update check against `https://update.argotunnel.com` (staging builds use `https://staging-update.argotunnel.com`) and applies any newer version; exits with code `11` on a successful update, `10` on error (per source inspection of the updater command). This is a **separate download channel from GitHub Releases** — it does not fetch the GitHub release asset directly.
- **Autoupdate default**: enabled by default in the background daemon (checks roughly every 24h — `time.Hour * 24` default frequency, per source), except: **disabled by default on Windows** ("Instances of `cloudflared` do not automatically update on Windows" — official docs), disabled for package-manager installs (apt/yum/Homebrew — those channels own their own update cycle), and inert when cloudflared is invoked as a one-shot command from a terminal rather than run as a persistent service.
- **`--no-autoupdate`**: suppresses the in-process background update loop. Known gap: it does **not** suppress the one-time version check cloudflared performs at startup (`github.com/cloudflare/cloudflared/issues/406`, unresolved as of this research) — relevant if BodhiApp wants a fully network-silent managed subprocess.
- **Update commands per channel** (official docs): macOS `brew upgrade cloudflared` + relaunch the launchd service; Linux `apt-get install --only-upgrade cloudflared` or `yum update cloudflared` + `systemctl restart cloudflared`; Windows `cloudflared update` + `net start cloudflared`. Docker guidance explicitly pairs `--no-autoupdate` with `docker run --pull always` (image-level updates replace in-process updates).
- **`cloudflared --version` output format** (confirmed against multiple cited examples, matches the task's own example exactly): `cloudflared version <VERSION> (built <YYYY-MM-DD-HHMM> UTC)`, e.g. `cloudflared version 2024.10.0 (built 2024-10-10-0949 UTC)`. Parse with a regex like `^cloudflared version (\S+) \(built ([\d-]+) UTC\)$` — capture group 1 is directly comparable against the `tag_name` from the GitHub Releases API for freshness checks.

## 4. Licensing

- **License: Apache License 2.0, unmodified** — verified from the raw `LICENSE` file at `github.com/cloudflare/cloudflared` (standard Apache 2.0 boilerplate, no exception clauses) and independently corroborated by Homebrew's formula metadata (`license: Apache-2.0`). See "Corrections" above — this repo does **not** carry the Runtime Library Exception some earlier notes attributed to it.
- **Attribution obligation if BodhiApp bundles/downloads the binary**: standard Apache-2.0 §4 — when redistributing (including as part of a larger product), BodhiApp must (a) give downstream recipients a copy of the Apache-2.0 license, (b) preserve any copyright/attribution/NOTICE content from cloudflared unmodified (state changes if any source was modified — not applicable for an unmodified binary redistribution), and (c) if cloudflared ships a `NOTICE` file, reproduce its contents in BodhiApp's own attribution surface (e.g. an in-app "third-party licenses" page). **UNVERIFIED**: whether the cloudflared repo actually ships a `NOTICE` file — confirm at `github.com/cloudflare/cloudflared` root before shipping.
- **Cloudflare's own policy on third parties embedding cloudflared**: no dedicated first-party policy page found. A 2018 GitHub issue asking exactly this (redistribution terms, compiling from source and redistributing, whether users must accept a license before download — `github.com/cloudflare/cloudflared/issues/53`) received **no substantive maintainer reply** in this research pass — treat Cloudflare's redistribution stance as governed by the Apache-2.0 license terms alone, with no additional first-party guidance to rely on. Cloudflare's general Third-Party-Products terms (`cloudflare.com/service-specific-terms-application-services/`) disclaim warranty for third-party integrations but don't speak to *being* embedded by a third party.
- **End-user terms**: running `cloudflared` still binds the end user to Cloudflare's own Self-Serve Subscription Agreement / ToS for their Cloudflare account — unrelated to the binary's Apache-2.0 license, and unchanged by BodhiApp downloading vs. the user installing it themselves.

## 5. Download-on-user-trigger design notes

- **Where to store**: `BODHI_HOME/bin/cloudflared[.exe]` is the natural fit — mirrors the existing `BODHI_EXEC_LOOKUP_PATH` convention (`crates/services/src/settings/setting_service.rs:309-314`) that already governs where BodhiApp looks for the llama-server binary. No existing `bin/` subdirectory under `BODHI_HOME` was found in the codebase at time of writing (`crates/lib_bodhiserver/src/app_dirs_builder.rs`) — this would be a new convention, not a reuse of an existing one.
- **Executable bit**: on macOS/Linux, `.tgz`/raw-binary downloads need `chmod +x` applied explicitly after writing to disk — archive/download tools don't reliably preserve it across an HTTP download. Windows doesn't need this.
- **macOS Gatekeeper/quarantine**: any file downloaded by an app via URLSession/networking APIs (which is how a Rust HTTP client's download would look to macOS) gets `com.apple.quarantine` set by the OS automatically, the same as a browser download — **not** specific to how BodhiApp fetches it. On first execution of a quarantined binary, Gatekeeper checks for a valid Apple code signature; an ad-hoc-signed or unsigned binary is rejected outright (this is distinct from the "cannot verify developer" prompt, which only appears for a binary that *is* signed but not notarized — an unsigned binary is blocked with no override dialog at all in current macOS). **UNVERIFIED in this pass**: whether Cloudflare's macOS release binaries (`cloudflared-darwin-{amd64,arm64}.tgz`, or the `.pkg` installers) are themselves code-signed/notarized by Cloudflare — this needs a direct `codesign -dv --verbose=4 <binary>` check against a downloaded artifact before shipping a download-on-trigger flow, since if they're unsigned, BodhiApp would need to strip quarantine itself (`xattr -d com.apple.quarantine <path>`) after verifying the download, which is a real trust boundary (BodhiApp is vouching for a binary Apple didn't). The Homebrew-bottle path sidesteps this entirely — Homebrew's own signing/notarization handling (or its own quarantine-stripping, since brew-built bottles are also quarantine-exempt by convention) means "prefer Homebrew when present" is the lower-risk macOS path, not just a convenience.
- **Windows SmartScreen**: parallels Gatekeeper — a browser-tagged (`Mark-of-the-Web`, `Zone.Identifier` ADS) `.exe` triggers SmartScreen's "Windows protected your PC" prompt on first run unless the binary carries a valid Authenticode signature with enough reputation. The open winget issue (`#1576`) reporting the winget-distributed build as "unsigned" is a signal to verify directly (`Get-AuthenticodeSignature <path>` in PowerShell) before assuming a downloaded `.exe`/`.msi` is Authenticode-signed; if unsigned, a first-run SmartScreen prompt should be expected and the in-app guidance should tell the user to click "More info → Run anyway" rather than treating it as a failure.
- **Linux noexec mounts**: if `BODHI_HOME` resolves under a `noexec`-mounted filesystem (common for `/tmp`, and for some container/CI ephemeral mounts), a downloaded binary can't be executed even with the `+x` bit set — the process spawn fails with `EACCES`/"Permission denied" despite correct permissions. Detect via a `mount | grep noexec` check against the target dir, or simply attempt-and-catch the exec failure with a clear error pointing the user at an alternate `BODHI_HOME`.
- **Prefer OS package manager when present**: for all three OSes, if the well-known-path probe (§2) finds an existing install (via Homebrew, apt/yum, or winget/MSI), prefer that over a fresh download — it's already correctly signed/notarized (macOS `.pkg`/Homebrew), GPG-chain-verified (Linux apt/yum), or at minimum user-consented (winget), and stays current via the OS's own update mechanism rather than BodhiApp's. Reserve "download on explicit trigger" for the case where no system install is found and the user explicitly opts in — never auto-download silently, given the unresolved signing/notarization/checksum gaps above.

## Sources

- Cloudflare Docs — [Cloudflare Tunnel downloads](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/)
- Cloudflare Docs — [Update cloudflared](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/update-cloudflared/)
- Cloudflare Docs — [Support lifecycle](https://developers.cloudflare.com/cloudflare-one/team-and-resources/devices/cloudflare-one-client/download/support-lifecycle) (one-year support-window statement)
- Cloudflare Docs — [pkg.cloudflare.com repository setup](https://pkg.cloudflare.com/index.html)
- GitHub — [cloudflare/cloudflared releases](https://github.com/cloudflare/cloudflared/releases) and [`releases/latest` API](https://api.github.com/repos/cloudflare/cloudflared/releases/latest) (verified 2026-09-15 against tag `2026.9.1`, published 2026-09-11)
- GitHub — [cloudflare/cloudflared `LICENSE`](https://raw.githubusercontent.com/cloudflare/cloudflared/master/LICENSE) (plain Apache-2.0, verified)
- GitHub — [cloudflare/cloudflared updater source](https://github.com/cloudflare/cloudflared/blob/master/cmd/cloudflared/updater/update.go) (autoupdate mechanism, `update.argotunnel.com`, exit codes)
- GitHub — [Issue #1576 — winget package outdated/unsigned](https://github.com/cloudflare/cloudflared/issues/1576)
- GitHub — [Issue #992 — Windows x64 MSI installs to `Program Files (x86)`](https://github.com/cloudflare/cloudflared/issues/992)
- GitHub — [Issue #406 — `--no-autoupdate` doesn't suppress the startup version check](https://github.com/cloudflare/cloudflared/issues/406)
- GitHub — [Issue #53 — redistribution/licensing clarification request, unanswered](https://github.com/cloudflare/cloudflared/issues/53)
- GitHub — [microsoft/winget-pkgs — `Cloudflare.cloudflared` manifests](https://github.com/microsoft/winget-pkgs/tree/master/manifests/c/Cloudflare/cloudflared) (package id confirmation)
- GitHub — [tauri-apps/fix-path-env-rs](https://github.com/tauri-apps/fix-path-env-rs) (PATH-fix crate for Tauri GUI apps)
- Homebrew — `brew info cloudflared` (local CLI query, 2026-09-15: stable `2026.9.1`, `License: Apache-2.0`, caveat text for `brew services start cloudflared`)
- Swift.org — [LICENSE.txt](https://www.swift.org/LICENSE.txt) (source of the "Runtime Library Exception" text mistakenly attributed to cloudflared in earlier research)
- Microsoft Learn / winget-cli discussions — [Links vs. Packages folder](https://github.com/microsoft/winget-cli/discussions/5720) (portable-package PATH mechanism, confirmed not applicable to MSI-backed packages like cloudflared)
- BodhiApp source — `crates/services/src/settings/constants.rs:22`, `crates/services/src/settings/setting_service.rs:309-314`, `crates/bodhi/src-tauri/src/native_init.rs`, `crates/bodhi/src-tauri/src/server_init.rs:29-49`, `crates/lib_bodhiserver/src/app_dirs_builder.rs` (existing `BODHI_EXEC_LOOKUP_PATH` / `BODHI_HOME` conventions)
