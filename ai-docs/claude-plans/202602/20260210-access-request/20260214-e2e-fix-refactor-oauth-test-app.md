# Plan: Update OAuth Test App for Access-Request Flow

## Context

The `oauth-test-app.html` test page is used in Playwright E2E tests to simulate an external app completing an OAuth2 token-exchange flow. Previously, it did a simple OAuth PKCE flow after the test script called `requestAudienceAccess()` externally. Now, the access-request feature requires user review and approval of toolset access. The test app itself needs to orchestrate the full flow: call `/apps/request-access`, handle redirect to the BodhiApp review page, detect the callback, poll for approval status, and then start the OAuth flow with the approved scopes.

The API contract has also changed (`flow_type` is now required, response is a tagged union with `status` field, etc.), so the test app must use the new request/response format.

## Files to Modify

1. **`crates/lib_bodhiserver_napi/tests-js/test-pages/oauth-test-app.html`** - Main target. Full rewrite of the flow logic.
2. **`crates/lib_bodhiserver_napi/tests-js/pages/OAuth2TestAppPage.mjs`** - Page object. Add new selectors and methods matching the updated HTML.

## Reference Files (read-only)

- `crates/routes_app/src/routes_apps/types.rs` - API request/response types (`CreateAccessRequestBody`, `CreateAccessRequestResponse`, `AccessRequestStatusResponse`)
- `crates/routes_app/src/routes_apps/handlers.rs` - API handler logic
- `crates/lib_bodhiserver_napi/tests-js/utils/static-server.mjs` - Static server serving test pages (cross-origin from BodhiApp)
- `crates/lib_bodhiserver_napi/tests-js/utils/OAuth2ApiHelper.mjs` - Test helper (has old `requestAudienceAccess`, updating it is out of scope here)

## Design Decisions (from user interview)

| Decision | Choice |
|----------|--------|
| BodhiApp URL | New form field with default `http://localhost:1135` |
| Toolset types | Configurable JSON textarea for `requested` param |
| Client ID | Same for both `app_client_id` and OAuth `client_id` |
| Flow paths | Both auto-approve and user-review, based on response |
| Scope composition | Append approved scopes to standard scopes |
| Flow type | Redirect only (no popup) |
| After draft response | Auto-redirect to `review_url` |
| After approved callback | Auto-start OAuth flow |
| CORS | Already configured on BodhiApp server |

---

## Implementation Plan

### Step 1: Add New HTML Form Fields

Add two fields to `<form id="oauth-config">` after the Scope field:

```
BodhiApp Server URL:  <input type="url" id="bodhi-server-url" default="http://localhost:1135">
Requested Toolsets:   <textarea id="requested-toolsets" placeholder='[{"toolset_type":"builtin-exa-search"}]'>
```

Change submit button text: `"Start OAuth Flow"` -> `"Request Access & Login"`

### Step 2: Add New UI Sections

Add two loading sections (hidden by default) between `config-section` and existing `loading-section`:

- `<div id="access-request-loading">` - "Requesting access..."
- `<div id="access-callback-loading">` - "Checking access request status..."

### Step 3: Rewrite Page Load Detection

Replace current `handleOAuthCallback()` call with priority-ordered detection:

```
1. ?code=xxx & ?state=xxx  ->  handleOAuthCallback()    // OAuth callback
2. ?id=xxx                 ->  handleAccessRequestCallback(id)  // Review callback
3. No relevant params      ->  Show config form          // Fresh load
```

### Step 4: Add `showSection(sectionId)` Utility

Helper that hides all sections and shows the requested one. Reduces repetitive classList toggling across all functions.

Sections managed: `config-section`, `access-request-loading`, `access-callback-loading`, `loading-section`, `error-section`, `success-section`

### Step 5: Add `requestAccessAndLogin()` Function

Entry point on form submit (replaces direct `startOAuthFlow()` call):

1. Read all form fields including new `bodhi-server-url` and `requested-toolsets`
2. Validate required fields
3. Parse `requested-toolsets` textarea as JSON (if non-empty, wrap in `{ toolset_types: parsed }`)
4. Store base config in sessionStorage (`oauthConfig` key)
5. Show `access-request-loading` section
6. `POST {bodhiServerUrl}/bodhi/v1/apps/request-access`:
   ```json
   {
     "app_client_id": "{clientId}",
     "flow_type": "redirect",
     "redirect_url": "{window.location.origin + window.location.pathname}",
     "requested": {"toolset_types": [...]}  // omit if textarea empty
   }
   ```
7. Handle response:
   - `status === "approved"`: store `resource_scope`, append to scope, call `startOAuthFlow(updatedScope)`
   - `status === "draft"`: store `id`, auto-redirect to `review_url`
   - Error: `showError()` with details

### Step 6: Add `handleAccessRequestCallback(id)` Function

Called when page loads with `?id=xxx` (after user reviews on BodhiApp):

1. Show `access-callback-loading` section
2. Retrieve `oauthConfig` from sessionStorage (error if missing)
3. `GET {bodhiServerUrl}/bodhi/v1/apps/access-requests/{id}?app_client_id={clientId}`
4. Handle status:
   - `"approved"`: collect `resource_scope` + `access_request_scope`, append to stored scope, clear URL params, call `startOAuthFlow(updatedScope)`
   - `"denied"`: `showError("Access request was denied.")`
   - `"draft"`: `showError("Still pending review.")`
   - `"failed"`: `showError()` with message
5. Error: `showError()` with details

### Step 7: Modify `startOAuthFlow(scopeOverride)`

Add optional `scopeOverride` parameter:

- When called from `requestAccessAndLogin()` or `handleAccessRequestCallback()`: receives full scope string with approved scopes appended
- Reads remaining config (authServerUrl, realm, etc.) from sessionStorage
- Generates PKCE, stores codeVerifier + state in sessionStorage
- Redirects to Keycloak authorize URL (same as current logic)

### Step 8: Refactor Existing Functions

- `showError()`: use `showSection('error-section')` internally
- `handleOAuthCallback()`: use `showSection()` for consistency
- `exchangeCodeForToken()`: use `showSection()` for loading/success
- `resetApp()`: reset new fields (`bodhi-server-url` default, clear `requested-toolsets`)

### Step 9: Update `OAuth2TestAppPage.mjs` Page Object

Add new selectors:
```javascript
bodhiServerUrlInput: '#bodhi-server-url',
requestedToolsetsInput: '#requested-toolsets',
accessRequestLoading: '#access-request-loading',
accessCallbackLoading: '#access-callback-loading',
```

Update `configureOAuthForm()`:
```javascript
async configureOAuthForm(bodhiServerUrl, authUrl, realm, clientId, redirectUri, scopes, requestedToolsets) {
  await this.page.fill('#bodhi-server-url', bodhiServerUrl);
  await this.page.fill('#auth-server-url', authUrl);
  // ... existing fields ...
  if (requestedToolsets) {
    await this.page.fill('#requested-toolsets', requestedToolsets);
  }
}
```

Add new methods:
```javascript
async waitForAccessRequestRedirect(bodhiServerUrl)  // waits for redirect to review URL
async waitForAccessRequestCallback(testAppUrl)       // waits for callback with ?id=
```

### SessionStorage Schema

Single `oauthConfig` key, progressively updated:

```
{
  bodhiServerUrl,          // "http://localhost:1135"
  authServerUrl,           // "http://localhost:8080"
  realm,                   // "bodhi"
  clientId,                // "my-app-client"
  isConfidential, clientSecret,
  redirectUri,             // "http://localhost:XXXX/oauth-test-app.html"
  scope,                   // "openid profile email" (base, updated before OAuth)
  requestedToolsets,       // raw JSON string from textarea
  accessRequestId,         // UUID (after request-access response)
  approvedScopes,          // ["scope_resource:xxx", "scope_access_request:xxx"]
  codeVerifier, state      // PKCE params (before OAuth redirect)
}
```

### Complete Flow Diagram

```
Fresh Load                     Callback ?id=xxx              Callback ?code=xxx&state=xxx
    |                               |                               |
Show Config Form            handleAccessRequestCallback()    handleOAuthCallback()
    |                               |                               |
User fills form              GET /access-requests/{id}        Exchange code for token
    |                               |                               |
requestAccessAndLogin()      approved? -> startOAuthFlow()    Show success + token
    |                        denied?  -> showError()
POST /apps/request-access
    |
status=approved? ---------> startOAuthFlow(scope)
status=draft? ------------> redirect to review_url
```

---

## Out of Scope

- Updating `OAuth2ApiHelper.requestAudienceAccess()` - the helper sends old format (`{ app_client_id }` without `flow_type`). Will be fixed when updating E2E test specs.
- Updating E2E test specs (`oauth2-token-exchange.spec.mjs`, `toolsets-auth-restrictions.spec.mjs`) - separate follow-up task.
- Creating `api.html` - existing button link kept as-is.

## Verification

1. **Manual**: Open the test page in browser, fill in BodhiApp URL and auth server URL, click "Request Access & Login"
   - Without toolsets: should auto-approve and redirect to Keycloak
   - With toolsets: should redirect to BodhiApp review page
2. **Visual**: After each redirect step, the page should show the correct loading section
3. **E2E**: The Playwright tests that use `OAuth2TestAppPage` will be updated in a follow-up task to use the new `configureOAuthForm()` signature
