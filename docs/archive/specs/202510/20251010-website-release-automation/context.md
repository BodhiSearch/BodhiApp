# Context: Website Release Automation

## GitHub API Response Format

Confirmed response structure from `gh api repos/BodhiSearch/BodhiApp/releases`:

```json
[
  {
    "tag_name": "app/v0.0.31",
    "created_at": "2025-10-07T10:45:42Z",
    "assets": [
      {
        "name": "Bodhi.App_0.1.0_aarch64.dmg",
        "browser_download_url": "https://github.com/BodhiSearch/BodhiApp/releases/download/app/v0.0.31/Bodhi.App_0.1.0_aarch64.dmg"
      }
    ]
  }
]
```

**Key fields for script:**
- `tag_name`: Release tag (e.g., "app/v0.0.31")
- `created_at`: ISO timestamp for date filtering
- `assets[].name`: Asset filename for pattern matching
- `assets[].browser_download_url`: Download URL to extract

**Tag patterns observed:**
- `app/v*` - Desktop application releases
- `docker/v*` - Production Docker images
- `docker-dev/v*` - Development Docker images
- `ts-client/v*` - TypeScript client releases
- `napi/v*` - NAPI bindings releases (not seen yet but expected)

**Asset naming patterns:**
- macOS ARM64: `*_aarch64.dmg`
- macOS x64: `*_x64.dmg`
- Windows: `*_x64_en-US.msi`
- Linux RPM: `*_x86_64.rpm`

## Current Download URL

Current hardcoded value in `getbodhi.app/src/lib/constants.ts`:
```
https://github.com/BodhiSearch/BodhiApp/releases/download/v0.0.19/Bodhi.App_0.1.0_aarch64.dmg
```

This should be replaced with latest `app/v*` release ARM64 .dmg asset.

## Implementation Notes

- Script will use @octokit/rest library for GitHub API
- Pagination: 100 releases per page
- Stop conditions: Found all patterns OR reached 6 months back
- Single-pass extraction: Process all tag patterns in one loop
- Optional .env.release_urls loading: Don't fail if missing
- Required validation: Fail build if NEXT_PUBLIC_DOWNLOAD_URL_MACOS_ARM64 missing
