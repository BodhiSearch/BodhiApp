# Consolidate Post-Login Redirect: Remove Legacy localStorage, Use sessionStorage

## Context

The access-request feature recently introduced a clean redirect-after-login pattern using browser `sessionStorage` (`bodhi-return-url` key). The legacy setup flow uses a different mechanism: the backend hardcodes redirect to `/ui/setup/download-models` for resource-admin logins, and `AppInitializer` uses a `localStorage` flag (`shown-download-models-page`) to redirect first-time users. This creates two competing redirect mechanisms. We consolidate to the sessionStorage pattern and remove the legacy localStorage-based one.

## Changes

### 1. Backend: Simplify auth callback redirect (always return `/ui/chat`)

**File:** `crates/routes_app/src/routes_auth/login.rs` (lines 294-321)

- Remove the `if status_resource_admin` branch entirely
- Always construct redirect URL using `CHAT_PATH` regardless of app status
- Remove `DOWNLOAD_MODELS_PATH` from the import on line 22

The redirect logic simplifies from a 27-line if/else to ~10 lines:
```rust
let ui_setup_resume = if let Ok(parsed_url) = Url::parse(&callback_url) {
    let mut new_url = parsed_url.clone();
    new_url.set_path(CHAT_PATH);
    new_url.set_query(None);
    new_url.to_string()
} else {
    setting_service.frontend_default_url()
};
```

### 2. Backend: Remove dead constant

**File:** `crates/services/src/setting_service/mod.rs` (line 59)

- Remove `pub const DOWNLOAD_MODELS_PATH: &str = "/ui/setup/download-models";` (no longer referenced)

### 3. Frontend: Set sessionStorage in resource-admin page before OAuth

**File:** `crates/bodhi/src/app/ui/setup/resource-admin/page.tsx`

- In `handleOAuthInitiate()` (line 43), before calling `initiateOAuth()`, add:
  ```tsx
  sessionStorage.setItem('bodhi-return-url', ROUTE_SETUP_DOWNLOAD_MODELS);
  ```
- Add import for `ROUTE_SETUP_DOWNLOAD_MODELS` from `@/lib/constants`

This mirrors the pattern in `AppInitializer.tsx:81` where `bodhi-return-url` is set before login redirect.

### 4. Frontend: Remove localStorage flag from AppInitializer

**File:** `crates/bodhi/src/components/AppInitializer.tsx`

- Remove `useLocalStorage` import (line 11)
- Remove `FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED` and `ROUTE_SETUP_DOWNLOAD_MODELS` from imports (line 14, 18)
- Remove `const [hasShownModelsPage] = useLocalStorage(...)` (line 41)
- Simplify `ready` case (lines 60-65) to always: `router.push(ROUTE_DEFAULT);`
- Remove `hasShownModelsPage` from useEffect dependency array (line 73)

### 5. Frontend: Remove localStorage flag from setup page

**File:** `crates/bodhi/src/app/ui/setup/page.tsx`

- Remove `useLocalStorage` import (line 22)
- Remove `FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED` from imports (line 24)
- Remove `const [, setHasShownModelsPage] = useLocalStorage(...)` (line 71)
- Remove `useEffect` that resets flag to false (lines 73-75)

### 6. Frontend: Remove localStorage flag from download-models page

**File:** `crates/bodhi/src/app/ui/setup/download-models/page.tsx`

- Remove `useLocalStorage` import (line 12)
- Remove `FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED` from imports (line 15)
- Remove `const [, setHasShownModelsPage] = useLocalStorage(...)` (line 33)
- Remove `useEffect` that sets flag to true (lines 35-37)

### 7. Frontend: Remove the constant

**File:** `crates/bodhi/src/lib/constants.ts` (line 24)

- Remove `export const FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED = 'shown-download-models-page';`

## Test Changes

### 8. Backend test: Update redirect assertion

**File:** `crates/routes_app/src/routes_auth/tests/login_test.rs` (line 988)

- Change assertion from `"http://frontend.localhost:3000/ui/setup/download-models"` to `"http://frontend.localhost:3000/ui/chat"`

### 9. Frontend test: Update AppInitializer tests

**File:** `crates/bodhi/src/components/AppInitializer.test.tsx`

- Remove `FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED` and `ROUTE_SETUP_DOWNLOAD_MODELS` from imports
- Remove test "redirects to download models page when status is ready and models page not shown" (lines 108-115)
- Update test "redirects to ROUTE_DEFAULT when status is ready and models page was shown" - remove localStorage setup, just test `ready` redirects to `ROUTE_DEFAULT` (lines 117-125)
- In `it.each` test cases (lines 128-150): Remove the test case with `FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED: 'false'` that expects `ROUTE_SETUP_DOWNLOAD_MODELS`. Remove all `localStorage` properties from test data - they're no longer relevant.
- In status mismatch tests (lines 153-205): Remove `localStorage` properties from test data
- In role-based tests (lines 223-227): Remove `localStorageMock.setItem(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, 'true')` from beforeEach
- Remove or simplify `localStorageMock` if no longer needed by any test

### 10. Frontend test: Update download-models page test

**File:** `crates/bodhi/src/app/ui/setup/download-models/page.test.tsx`

- Remove `FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED` from imports (line 40)
- No other changes needed (tests focus on model catalog, not localStorage flag)

### 11. Frontend test: Add resource-admin sessionStorage test

**File:** `crates/bodhi/src/app/ui/setup/resource-admin/page.test.tsx`

- Add test: verify `sessionStorage.setItem('bodhi-return-url', '/ui/setup/download-models')` is called when user clicks "Continue with Login" button
- Mock `sessionStorage` in test setup

### 12. Frontend test: Check ui/page.test.tsx

**File:** `crates/bodhi/src/app/ui/page.test.tsx`

- Remove any `FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED` references if present

## Verification

1. **Backend compilation:** `cargo check -p routes_app -p services`
2. **Backend tests:** `cargo test -p routes_app` - verify login callback test passes with `/ui/chat` redirect
3. **Frontend tests:** `cd crates/bodhi && npm test` - verify all updated tests pass
4. **Build embedded UI:** `make build.ui-rebuild` (required since frontend components changed)
5. **Manual flow check:** Setup flow: setup page -> resource-admin -> OAuth login -> callback should redirect to `/ui/setup/download-models` via sessionStorage
