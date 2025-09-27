# Migration Plan: Parameterized Paths to openapi-msw Pattern

## Analysis Summary

After investigating all handler files, I found the following parameterized paths that need migration:

**Currently using MSW regular syntax (`:param`) - Need Migration:**

1. **access-requests.ts** (2 handlers)

   - `/bodhi/v1/access-requests/:id/approve` → `/bodhi/v1/access-requests/{id}/approve` ✅ (in OpenAPI)
   - `/bodhi/v1/access-requests/:id/reject` → `/bodhi/v1/access-requests/{id}/reject` ✅ (in OpenAPI)

2. **models.ts** (2 handlers)

   - `${ENDPOINT_MODELS}/:alias` → `/bodhi/v1/models/{alias}` ✅ (in OpenAPI)
   - PUT `${ENDPOINT_MODELS}/:alias` → PUT `/bodhi/v1/models/{alias}` ✅ (in OpenAPI)

3. **settings.ts** (7 handlers)

   - PUT/DELETE `${ENDPOINT_SETTINGS}/${settingKey}` → `/bodhi/v1/settings/{key}` ✅ (in OpenAPI)

4. **tokens.ts** (2 handlers)

   - PUT `${API_TOKENS_ENDPOINT}/${tokenId}` → `/bodhi/v1/tokens/{id}` ✅ (in OpenAPI)

5. **user.ts** (2 handlers)
   - PUT `/bodhi/v1/users/:userId/role` → `/bodhi/v1/users/{user_id}/role` ✅ (in OpenAPI)
   - DELETE `/bodhi/v1/users/:userId` → `/bodhi/v1/users/{user_id}` ✅ (in OpenAPI)

**Already using OpenAPI pattern correctly:**

- **api-models.ts** - All paths using `{id}` and `{alias}` correctly ✅

## Migration Strategy

Since ALL parameterized paths are in the OpenAPI spec, we can migrate them all to use openapi-msw's typed `http` instead of the fallback `mswHttp`.

## Detailed Migration Steps

### Step 1: Update access-requests.ts

- Change from `mswHttp.post('/bodhi/v1/access-requests/:id/approve'`
- To: `http.post('/bodhi/v1/access-requests/{id}/approve'`
- Remove `HttpResponse` usage, use response helper pattern
- Access params with typed `{ params }` destructuring

### Step 2: Update models.ts

- Change from `mswHttp.get(\`${ENDPOINT_MODELS}/:alias\``
- To: `http.get('/bodhi/v1/models/{alias}'`
- Remove `HttpResponse` usage
- Use response helper pattern with typed params

### Step 3: Update settings.ts

- Change from `mswHttp.put(\`${ENDPOINT_SETTINGS}/${settingKey}\``
- To: `http.put('/bodhi/v1/settings/{key}'`
- Change from `mswHttp.delete(\`${ENDPOINT_SETTINGS}/${settingKey}\``
- To: `http.delete('/bodhi/v1/settings/{key}'`
- Remove all `HttpResponse` usage
- Use response helper pattern

### Step 4: Update tokens.ts

- Change both `mswHttp.put(\`${API_TOKENS_ENDPOINT}/${tokenId}\``
- To: `http.put('/bodhi/v1/tokens/{id}'`
- Remove `HttpResponse` usage
- Use response helper pattern

### Step 5: Update user.ts

- Change from `http.put('/bodhi/v1/users/:userId/role'`
- To: `http.put('/bodhi/v1/users/{user_id}/role'`
- Change from `http.delete('/bodhi/v1/users/:userId'`
- To: `http.delete('/bodhi/v1/users/{user_id}'`
- Note: Already using openapi-msw `http` but with wrong syntax

### Step 6: Clean up imports

After migration, remove unnecessary imports:

- Remove `import { http as mswHttp, HttpResponse } from 'msw';`
- Keep only `import { http, type components } from '../setup';`

## Benefits After Migration

1. **Full Type Safety**: All path parameters will be typed based on OpenAPI spec
2. **Compile-time Validation**: TypeScript will catch any mismatched parameter names
3. **Response Type Checking**: Responses will be validated against OpenAPI schemas
4. **Cleaner Code**: No need for mixed MSW/openapi-msw approach
5. **Better IDE Support**: Full autocomplete for params and response structures

## Verification Steps

After migration:

1. Run `npm run build` to check TypeScript compilation
2. Run tests to ensure handlers still work correctly
3. Verify all parameterized paths are typed properly

## Files to be Modified (13 total functions across 5 files)

1. **access-requests.ts**: 2 functions (`mockAccessRequestApprove`, `mockAccessRequestReject`)
2. **models.ts**: 2 functions (`mockGetModel`, `mockUpdateModel`)
3. **settings.ts**: 7 functions (various update and delete handlers)
4. **tokens.ts**: 2 functions (both `mockUpdateToken` variants)
5. **user.ts**: 2 functions (`mockUserRoleChange`, `mockUserRemove`)

This migration will complete the full openapi-msw integration with 100% type safety for all API handlers.

## OpenAPI Paths Available

From the OpenAPI specification, these parameterized paths are available:

```
"/bodhi/v1/access-requests/{id}/approve"
"/bodhi/v1/access-requests/{id}/reject"
"/bodhi/v1/api-models/{alias}"
"/bodhi/v1/api-models/{id}"
"/bodhi/v1/modelfiles/pull/{alias}"
"/bodhi/v1/modelfiles/pull/{id}"
"/bodhi/v1/models/{alias}"
"/bodhi/v1/models/{id}"
"/bodhi/v1/settings/{key}"
"/bodhi/v1/tokens/{id}"
"/bodhi/v1/users/{user_id}"
"/bodhi/v1/users/{user_id}/role"
"/v1/models/{id}"
```

## Current Implementation Status

**Files using mixed approach (mswHttp + openapi-msw):**

- access-requests.ts ✅ (correctly implemented)
- models.ts ❌ (needs migration)
- settings.ts ❌ (needs migration)
- tokens.ts ❌ (needs migration)
- user.ts ❌ (wrong parameter syntax)

**Files using full openapi-msw:**

- api-models.ts ✅
- auth.ts ✅
- chat-completions.ts ✅
- info.ts ✅
- modelfiles.ts ✅
- setup.ts ✅

## Expected Pattern After Migration

```typescript
// Before (MSW regular)
mswHttp.get(`${ENDPOINT_MODELS}/:alias`, ({ params }) => {
  const { alias } = params;
  return HttpResponse.json({ alias });
});

// After (openapi-msw)
http.get('/bodhi/v1/models/{alias}', ({ params, response }) => {
  const { alias } = params; // ✅ Fully typed
  return response(200).json({ alias });
});
```

This migration will achieve 100% openapi-msw coverage with full type safety across all handlers.
