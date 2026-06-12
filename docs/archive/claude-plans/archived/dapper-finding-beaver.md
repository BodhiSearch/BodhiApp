# Toolset Multi-Instance: UI Layer Implementation Plan

## ✅ STATUS: COMPLETED

All 15 phases successfully implemented and tested.

## Summary

Update UI from type-based (`toolset_id`) to UUID-based instance architecture. Includes backend response enrichment, hook rewrites, page updates, and chat integration changes.

## Implementation Status

### Backend Phases
- ✅ **Phase backend-naming**: Remove redundant "Instance" terminology
- ✅ **Phase backend-enrich**: Add `has_api_key`, `app_enabled`, `tools` to ToolsetResponse

### Frontend Hooks Phases
- ✅ **Phase hooks-types**: Update TypeScript types from ts-client
- ✅ **Phase hooks-crud**: Replace with new CRUD hooks (useToolsets, useToolset, useCreateToolset, useUpdateToolset, useDeleteToolset)
- ✅ **Phase hooks-types-admin**: Add admin type hooks (useToolsetTypes, useEnableToolsetType, useDisableToolsetType)
- ✅ **Phase hooks-selection**: Update selection hook documentation

### MSW & Utilities Phases
- ✅ **Phase msw-handlers**: Replace MSW handlers with instance-based mocks
- ✅ **Phase chat-encoding**: Create `lib/toolsets.ts` with encoding/decoding functions

### UI Pages Phases
- ✅ **Phase page-list**: Update toolsets list page with UUID-based architecture
- ✅ **Phase page-new**: Create new toolset page (`/ui/toolsets/new`)
- ✅ **Phase page-edit**: Update edit page with UUID-based editing
- ✅ **Phase page-admin**: Create admin types page (`/ui/toolsets/admin`)

### Chat Integration Phases
- ✅ **Phase chat-popover**: Update ToolsetsPopover with grouped toolsets
- ✅ **Phase chat-integration**: Update chat hook with name→UUID mapping
- ✅ **Phase chat-autofilter**: Auto-filter unavailable tools

### Testing Phase
- ✅ **Phase tests**: All tests updated for new architecture

## User Decisions Applied

| Decision | Applied |
|----------|---------|
| Tool name format | `toolset__{name}__{method}` (2 underscores) |
| Backwards compat | None - breaking change, clear storage for dev |
| Default selection | Inherit last chat selection from localStorage |
| Popover UX | Grouped collapsible by type |
| Admin route | Separate `/ui/toolsets/admin` with tab-like nav |
| Disabled type behavior | Read-only (no create/edit, CAN delete) |
| has_api_key in list | Not shown - backend validation only |
| Name prefill | Type name only for first instance of that type |
| Max name length | 24 chars (match backend) |
| Display names | Use backend `ToolsetTypeResponse.name` |
| Tool filtering | Pre-filter unavailable, auto-uncheck with tooltip |
| Unavailable instances | Show disabled with tooltip |
| Name→UUID mapping | Build once at chat start |
| Response enrichment | Backend adds `has_api_key`, `app_enabled`, `tools` |
| Phase order | Layers: hooks → pages → chat |
| MSW handlers | Replace entirely |
| E2E tests | Separate phase (not in this plan) |
| Test approach | Full rewrite with MSW mocks, behavior-focused |
| Progress updates | After each phase, tests run after all phases |
| Setup wizard | Out of scope - separate follow-up |

---

## Phase backend-naming: Remove Redundant "Instance" Term

### Goal
Audit and remove redundant/incorrect usage of "instance" term in backend modules for consistency.

### Files to audit:
- `crates/routes_app/src/routes_toolsets.rs` - handler/function names
- `crates/routes_app/src/toolsets_dto.rs` - comments, doc strings
- `crates/auth_middleware/src/toolset_auth_middleware.rs` - error variants, messages
- `crates/services/src/resources/en-US/messages.ftl` - error messages

### Naming guidelines:
- Use "toolset" not "instance" in most contexts
- Keep "instance" only where it disambiguates from "type" in same context
- Error variants: `ToolsetNotFound` not `InstanceNotFound`
- Comments: "toolset" or "toolset configuration"

---

## Phase backend-enrich: Enrich ToolsetResponse

### Goal
Add missing fields to `ToolsetResponse` so UI has everything in single API call.

### File: `crates/routes_app/src/toolsets_dto.rs`

**Modify `ToolsetResponse`:**
```rust
pub struct ToolsetResponse {
    pub id: String,
    pub name: String,
    pub toolset_type: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub has_api_key: bool,        // NEW
    pub app_enabled: bool,        // NEW
    pub tools: Vec<ToolDefinition>, // NEW
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### File: `crates/routes_app/src/routes_toolsets.rs`

**Update handlers** to populate new fields:
- `list_toolsets_handler`: For each instance, lookup type for `tools` and `app_enabled`
- `get_toolset_handler`: Same enrichment
- `create_toolset_handler`: Return enriched response
- `update_toolset_handler`: Return enriched response

### File: `crates/routes_app/src/openapi.rs`

Update schema registration if needed.

### Verification
```bash
cargo fmt --all
cargo test -p routes_app
cargo clippy -p routes_app
cargo run --package xtask openapi
cd ts-client && npm run generate && npm run build
```

---

## Phase hooks-types: TypeScript Types

### File: `crates/bodhi/src/hooks/useToolsets.ts`

**Remove imports:**
- `EnhancedToolsetConfigResponse`, `UpdateToolsetConfigRequest`, `UserToolsetConfig`, `UserToolsetConfigSummary`

**Add imports from ts-client:**
```typescript
import {
  ToolsetResponse,
  CreateToolsetRequest,
  UpdateToolsetRequest,
  ApiKeyUpdateDto,
  ListToolsetsResponse,
  ToolsetTypeResponse,
  ListToolsetTypesResponse,
  ToolDefinition,
} from '@bodhiapp/ts-client';
```

**Re-export types:**
```typescript
export type {
  ToolsetResponse,
  CreateToolsetRequest,
  UpdateToolsetRequest,
  ApiKeyUpdateDto,
  ToolsetTypeResponse,
  ToolDefinition,
};
```

---

## Phase hooks-crud: Toolset CRUD Hooks

### File: `crates/bodhi/src/hooks/useToolsets.ts`

**Update endpoints:**
```typescript
export const TOOLSETS_ENDPOINT = `${BODHI_API_BASE}/toolsets`;
export const TOOLSET_TYPES_ENDPOINT = `${BODHI_API_BASE}/toolset_types`;
```

**Replace hooks:**

| Old Hook | New Hook | Notes |
|----------|----------|-------|
| `useAvailableToolsets` | `useToolsets` | Returns `ListToolsetsResponse` |
| `useToolsetConfig` | `useToolset` | Takes UUID `id`, returns `ToolsetResponse` |
| `useUpdateToolsetConfig` | `useUpdateToolset` | Takes `{id, ...UpdateToolsetRequest}` |
| `useDeleteToolsetConfig` | `useDeleteToolset` | Takes UUID `id` |
| — | `useCreateToolset` | NEW: Takes `CreateToolsetRequest` |

**New hooks:**

```typescript
// List user's toolsets
export function useToolsets(options?: { enabled?: boolean }) {
  return useQuery<ListToolsetsResponse>(
    ['toolsets'],
    TOOLSETS_ENDPOINT,
    undefined,
    options
  );
}

// Get single toolset by UUID
export function useToolset(id: string | undefined, options?: { enabled?: boolean }) {
  return useQuery<ToolsetResponse>(
    ['toolsets', id],
    `${TOOLSETS_ENDPOINT}/${id}`,
    undefined,
    { ...options, enabled: options?.enabled !== false && !!id }
  );
}

// Create new toolset
export function useCreateToolset(options?: {
  onSuccess?: (toolset: ToolsetResponse) => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();
  return useMutationQuery<ToolsetResponse, CreateToolsetRequest>(
    () => TOOLSETS_ENDPOINT,
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['toolsets']);
        options?.onSuccess?.(response.data);
      },
      onError: (error) => {
        const message = error?.response?.data?.error?.message || 'Failed to create toolset';
        options?.onError?.(message);
      },
    }
  );
}

// Update toolset
export function useUpdateToolset(options?: {
  onSuccess?: (toolset: ToolsetResponse) => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();
  return useMutationQuery<ToolsetResponse, UpdateToolsetRequest & { id: string }>(
    ({ id }) => `${TOOLSETS_ENDPOINT}/${id}`,
    'put',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['toolsets']);
        options?.onSuccess?.(response.data);
      },
      onError: (error) => {
        const message = error?.response?.data?.error?.message || 'Failed to update toolset';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, ...body }) => body }
  );
}

// Delete toolset
export function useDeleteToolset(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();
  return useMutationQuery<void, { id: string }>(
    ({ id }) => `${TOOLSETS_ENDPOINT}/${id}`,
    'delete',
    {
      onSuccess: () => {
        queryClient.invalidateQueries(['toolsets']);
        options?.onSuccess?.();
      },
      onError: (error) => {
        const message = error?.response?.data?.error?.message || 'Failed to delete toolset';
        options?.onError?.(message);
      },
    },
    { noBody: true }
  );
}
```

---

## Phase hooks-types-admin: Admin Type Hooks

### File: `crates/bodhi/src/hooks/useToolsets.ts`

**Add type listing hook:**
```typescript
// List toolset types (for admin and create form)
export function useToolsetTypes(options?: { enabled?: boolean }) {
  return useQuery<ListToolsetTypesResponse>(
    ['toolsets', 'types'],
    TOOLSET_TYPES_ENDPOINT,
    undefined,
    options
  );
}
```

**Update admin hooks** to use new endpoint:
```typescript
// Enable toolset type (admin only)
export function useEnableToolsetType(options?: {...}) {
  return useMutationQuery<AppToolsetConfigResponse, { typeId: string }>(
    ({ typeId }) => `${TOOLSET_TYPES_ENDPOINT}/${typeId}/app-config`,
    'put',
    {...},
    { noBody: true }
  );
}

// Disable toolset type (admin only)
export function useDisableToolsetType(options?: {...}) {
  return useMutationQuery<AppToolsetConfigResponse, { typeId: string }>(
    ({ typeId }) => `${TOOLSET_TYPES_ENDPOINT}/${typeId}/app-config`,
    'delete',
    {...},
    { noBody: true }
  );
}
```

---

## Phase hooks-selection: Update Selection Hook

### File: `crates/bodhi/src/hooks/use-toolset-selection.ts`

**Key changes:**
- State keyed by instance `id` (UUID), not `toolset_id` (type)
- Same localStorage key for inheritance

**No structural changes needed** - the hook already uses generic `toolsetId` parameter which will now be instance UUID.

**Update type documentation:**
```typescript
// EnabledTools maps instance ID (UUID) to enabled tool names
type EnabledTools = Record<string, string[]>;
// Example: { "uuid-abc-123": ["search", "contents"] }
```

---

## Phase msw-handlers: Replace MSW Handlers

### File: `crates/bodhi/src/test-utils/msw-v2/handlers/toolsets.ts`

**Replace entirely** with instance-based handlers:

```typescript
import { http, HttpResponse } from 'msw';
import { BODHI_API_BASE } from '@/hooks/useQuery';

// Mock data
const mockToolset: ToolsetResponse = {
  id: 'uuid-test-toolset',
  name: 'my-exa-search',
  toolset_type: 'builtin-exa-web-search',
  description: 'Test toolset',
  enabled: true,
  has_api_key: true,
  app_enabled: true,
  tools: [{ type: 'function', function: { name: 'search', description: '...', parameters: {} } }],
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

const mockType: ToolsetTypeResponse = {
  toolset_id: 'builtin-exa-web-search',
  name: 'Exa Web Search',
  description: 'Search the web using Exa AI',
  app_enabled: true,
  tools: [...],
};

// Handler factories
export function mockListToolsets(toolsets: ToolsetResponse[] = [mockToolset]) {
  return http.get(`${BODHI_API_BASE}/toolsets`, () =>
    HttpResponse.json({ toolsets })
  );
}

export function mockGetToolset(toolset: ToolsetResponse = mockToolset) {
  return http.get(`${BODHI_API_BASE}/toolsets/:id`, () =>
    HttpResponse.json(toolset)
  );
}

export function mockCreateToolset(response: ToolsetResponse = mockToolset) {
  return http.post(`${BODHI_API_BASE}/toolsets`, () =>
    HttpResponse.json(response, { status: 201 })
  );
}

export function mockUpdateToolset(response: ToolsetResponse = mockToolset) {
  return http.put(`${BODHI_API_BASE}/toolsets/:id`, () =>
    HttpResponse.json(response)
  );
}

export function mockDeleteToolset() {
  return http.delete(`${BODHI_API_BASE}/toolsets/:id`, () =>
    new HttpResponse(null, { status: 204 })
  );
}

export function mockListTypes(types: ToolsetTypeResponse[] = [mockType]) {
  return http.get(`${BODHI_API_BASE}/toolset_types`, () =>
    HttpResponse.json({ types })
  );
}

export function mockEnableType() {
  return http.put(`${BODHI_API_BASE}/toolset_types/:typeId/app-config`, () =>
    HttpResponse.json({ toolset_id: '...', enabled: true })
  );
}

export function mockDisableType() {
  return http.delete(`${BODHI_API_BASE}/toolset_types/:typeId/app-config`, () =>
    HttpResponse.json({ toolset_id: '...', enabled: false })
  );
}
```

---

## Phase page-list: Toolsets List Page

### File: `crates/bodhi/src/app/ui/toolsets/page.tsx`

**Major changes:**
1. Use `useToolsets()` instead of `useAvailableToolsets()`
2. Update columns: Name, Type, App Status, Status, Actions
3. Add "New" button → `/ui/toolsets/new`
4. Add tab navigation for admin (if admin)
5. Update row rendering for new data structure

**Column structure:**
```typescript
const columns = [
  { id: 'name', name: 'Name' },
  { id: 'type', name: 'Type' },
  { id: 'appStatus', name: 'App Status' },
  { id: 'status', name: 'Status' },
  { id: 'actions', name: '' },
];
```

**Status logic:**
```typescript
function getToolsetStatus(toolset: ToolsetResponse): { label: string; variant: string } {
  if (!toolset.app_enabled) return { label: 'App Disabled', variant: 'destructive' };
  if (!toolset.enabled) return { label: 'Disabled', variant: 'secondary' };
  if (!toolset.has_api_key) return { label: 'No API Key', variant: 'outline' };
  return { label: 'Enabled', variant: 'default' };
}
```

**Actions per row:**
- Edit button: disabled if `!app_enabled`, navigates to `/ui/toolsets/edit?id={uuid}`
- Delete button: always enabled (even if app disabled)

**Admin tab navigation:**
```typescript
const isAdmin = userInfo?.role === 'resource_admin';
// Show tabs: "My Toolsets" | "Admin" (admin only)
```

---

## Phase page-new: Create Toolset Page

### File: `crates/bodhi/src/app/ui/toolsets/new/page.tsx` (NEW)

**Form fields:**
1. **Toolset Type** (Select) - from `useToolsetTypes()`, filter to `app_enabled`
2. **Name** (Input) - prefill with type if user has 0 toolsets of that type
3. **Description** (Input) - optional, max 255 chars
4. **API Key** (Password Input) - required
5. **Enabled** (Switch) - default true

**Form schema:**
```typescript
const createToolsetSchema = z.object({
  toolset_type: z.string().min(1, 'Type is required'),
  name: z.string()
    .min(1, 'Name is required')
    .max(24, 'Name must be 24 characters or less')
    .regex(/^[a-zA-Z0-9-]+$/, 'Name can only contain letters, numbers, and hyphens'),
  description: z.string().max(255).optional(),
  api_key: z.string().min(1, 'API key is required'),
  enabled: z.boolean().default(true),
});
```

**Name prefill logic:**
```typescript
const { data: toolsetsData } = useToolsets();
const toolsets = toolsetsData?.toolsets || [];

// When type changes, check if user has toolsets of that type
const handleTypeChange = (type: string) => {
  const hasToolsetsOfType = toolsets.some(t => t.toolset_type === type);
  if (!hasToolsetsOfType) {
    form.setValue('name', type);
  }
};
```

**On success:** Navigate to `/ui/toolsets`

---

## Phase page-edit: Edit Toolset Page

### File: `crates/bodhi/src/app/ui/toolsets/edit/page.tsx`

**Changes:**
1. Query param: `id` (UUID) instead of `toolset_id`
2. Use `useToolset(id)` instead of `useToolsetConfig()`
3. Type field: read-only display (from `toolset.toolset_type`)
4. API key: password input with placeholder if configured
5. Redirect to `/ui/toolsets` if `!toolset.app_enabled`

**Form schema:**
```typescript
const updateToolsetSchema = z.object({
  name: z.string().min(1).max(24).regex(/^[a-zA-Z0-9-]+$/),
  description: z.string().max(255).optional().nullable(),
  enabled: z.boolean(),
  api_key: z.union([
    z.literal(''),  // Keep existing
    z.string().min(1),  // New value
  ]),
});
```

**API key handling:**
```typescript
// Transform form to API request
const request: UpdateToolsetRequest = {
  name: form.name,
  description: form.description || null,
  enabled: form.enabled,
  api_key: form.api_key === ''
    ? { action: 'Keep' }
    : { action: 'Set', value: form.api_key },
};
```

**Delete functionality:**
- Confirmation dialog
- Use `useDeleteToolset()`
- Navigate to `/ui/toolsets` on success

---

## Phase page-admin: Admin Types Page

### File: `crates/bodhi/src/app/ui/toolsets/admin/page.tsx` (NEW)

**Access control:**
```typescript
<AppInitializer authenticated={true} allowedStatus="ready" minRole="admin">
```

**Structure:**
- Tab navigation back to "My Instances"
- Table: Type, Description, App Status, Action
- Enable/Disable toggle with confirmation

**Data source:** `useToolsetTypes()`

---

## Phase chat-popover: Update ToolsetsPopover

### File: `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx`

**Changes:**
1. Use `useToolsets()` instead of `useAvailableToolsets()`
2. Group toolsets by `toolset_type`
3. Collapse sections with single toolset of that type
4. Update availability check for new data structure
5. Update test IDs to use toolset `id`

**Availability check:**
```typescript
function isToolsetAvailable(toolset: ToolsetResponse): boolean {
  return toolset.app_enabled && toolset.enabled && toolset.has_api_key;
}

function getUnavailableReason(toolset: ToolsetResponse): string | null {
  if (!toolset.app_enabled) return 'Disabled by administrator';
  if (!toolset.enabled) return 'Disabled in settings';
  if (!toolset.has_api_key) return 'API key not configured';
  return null;
}
```

**Grouping structure:**
```typescript
const grouped = useMemo(() => {
  const groups: Record<string, ToolsetResponse[]> = {};
  toolsets.forEach(toolset => {
    const type = toolset.toolset_type;
    if (!groups[type]) groups[type] = [];
    groups[type].push(toolset);
  });
  return groups;
}, [toolsets]);
```

**Type display name:** Use type info from backend `ToolsetTypeResponse.name`.

---

## Phase chat-encoding: Update Tool Encoding

### File: `crates/bodhi/src/hooks/use-chat.tsx` or new `crates/bodhi/src/lib/toolsets.ts`

**Tool name encoding:**
```typescript
// Encode: toolset__{toolsetName}__{methodName}
export function encodeToolName(toolsetName: string, methodName: string): string {
  return `toolset__${toolsetName}__${methodName}`;
}

// Decode: extract toolset name and method
export function decodeToolName(toolName: string): { toolsetName: string; method: string } | null {
  const match = toolName.match(/^toolset__(.+?)__(.+)$/);
  if (!match) return null;
  return { toolsetName: match[1], method: match[2] };
}
```

---

## Phase chat-integration: Update Chat Hook

### File: `crates/bodhi/src/hooks/use-chat.tsx`

**Build name→UUID mapping at chat start:**
```typescript
const { data: toolsetsData } = useToolsets();
const toolsets = toolsetsData?.toolsets || [];

const toolsetNameToId = useMemo(() => {
  const map = new Map<string, string>();
  toolsets.forEach(t => map.set(t.name, t.id));
  return map;
}, [toolsets]);
```

**Build tools array:**
```typescript
function buildToolsArray(
  enabledTools: Record<string, string[]>,
  toolsets: ToolsetResponse[]
): ToolDefinition[] {
  const result: ToolDefinition[] = [];

  for (const toolset of toolsets) {
    // Skip unavailable toolsets
    if (!toolset.app_enabled || !toolset.enabled || !toolset.has_api_key) continue;

    const enabledToolNames = enabledTools[toolset.id] || [];

    for (const tool of toolset.tools) {
      if (enabledToolNames.includes(tool.function.name)) {
        result.push({
          type: 'function',
          function: {
            ...tool.function,
            name: encodeToolName(toolset.name, tool.function.name),
          },
        });
      }
    }
  }

  return result;
}
```

**Execute tool call:**
```typescript
async function executeToolCall(
  toolCall: { id: string; function: { name: string; arguments: string } },
  toolsetNameToId: Map<string, string>
): Promise<ToolResult> {
  const parsed = decodeToolName(toolCall.function.name);
  if (!parsed) {
    return { tool_call_id: toolCall.id, error: `Invalid tool name: ${toolCall.function.name}` };
  }

  const toolsetId = toolsetNameToId.get(parsed.toolsetName);
  if (!toolsetId) {
    return { tool_call_id: toolCall.id, error: `Unknown toolset: ${parsed.toolsetName}` };
  }

  const response = await fetch(
    `${BODHI_API_BASE}/toolsets/${toolsetId}/execute/${parsed.method}`,
    {
      method: 'POST',
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        tool_call_id: toolCall.id,
        params: JSON.parse(toolCall.function.arguments),
      }),
    }
  );

  if (!response.ok) {
    const error = await response.json();
    return { tool_call_id: toolCall.id, error: error?.error?.message || 'Tool execution failed' };
  }

  return response.json();
}
```

---

## Phase chat-autofilter: Auto-filter Unavailable Tools

### File: `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx`

**Auto-uncheck unavailable toolsets:**
```typescript
useEffect(() => {
  // On toolsets change, filter out selections for unavailable toolsets
  const availableIds = new Set(
    toolsets.filter(isToolsetAvailable).map(t => t.id)
  );

  const filtered: Record<string, string[]> = {};
  for (const [id, tools] of Object.entries(enabledTools)) {
    if (availableIds.has(id)) {
      filtered[id] = tools;
    }
  }

  if (Object.keys(filtered).length !== Object.keys(enabledTools).length) {
    setEnabledTools(filtered);
  }
}, [toolsets, enabledTools, setEnabledTools]);
```

---

## Files Modified Summary

| File | Action |
|------|--------|
| `crates/routes_app/src/toolsets_dto.rs` | Add `has_api_key`, `app_enabled`, `tools`; remove "instance" term |
| `crates/routes_app/src/routes_toolsets.rs` | Enrich responses; remove "instance" term |
| `crates/auth_middleware/src/toolset_auth_middleware.rs` | Audit "instance" term usage |
| `crates/services/src/resources/en-US/messages.ftl` | Audit "instance" term in messages |
| `crates/bodhi/src/hooks/useToolsets.ts` | Replace with new hooks (useToolsets, useToolset, etc.) |
| `crates/bodhi/src/hooks/use-toolset-selection.ts` | Doc updates only |
| `crates/bodhi/src/hooks/use-chat.tsx` | Name→UUID mapping, encoding |
| `crates/bodhi/src/test-utils/msw-v2/handlers/toolsets.ts` | Replace entirely |
| `crates/bodhi/src/app/ui/toolsets/page.tsx` | Toolsets list, admin tabs |
| `crates/bodhi/src/app/ui/toolsets/new/page.tsx` | NEW: Create form |
| `crates/bodhi/src/app/ui/toolsets/edit/page.tsx` | UUID-based, partial update |
| `crates/bodhi/src/app/ui/toolsets/admin/page.tsx` | NEW: Admin types page |
| `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx` | Grouped toolsets |
| `crates/bodhi/src/lib/toolsets.ts` | NEW: Encoding functions |

---

## Verification

**After backend-enrich:**
```bash
cargo fmt --all
cargo test -p routes_app
cargo run --package xtask openapi
cd ts-client && npm run generate && npm run build
```

**After each UI phase:**
```bash
cd crates/bodhi && npm run format && npm run lint
cd crates/bodhi && npm test -- --run
```

**Final verification:**
```bash
make test.backend
make build.ui-clean && make build.ui
make run.app  # Manual testing of flows
```

---

## Phase tests: Full Test Rewrite

### Approach
- Delete existing toolsets tests, write new ones from scratch
- Use MSW mocks for all API calls
- Focus on behavior: user interactions, API calls, state changes
- Run tests only after all implementation phases complete

### File: `crates/bodhi/src/app/ui/toolsets/page.test.tsx`

**Test categories:**
1. **Loading/Error states**: Loading skeleton, error page rendering
2. **Empty state**: No toolsets message, New button
3. **Toolsets list**: Renders toolsets with correct columns
4. **Status badges**: App Disabled, Disabled, No API Key, Enabled
5. **Actions**: Edit button (disabled when app_disabled), Delete button
6. **Admin tabs**: Tab visibility based on role, navigation

### File: `crates/bodhi/src/app/ui/toolsets/new/page.test.tsx` (NEW)

**Test categories:**
1. **Form validation**: Required fields, name format, max length
2. **Type dropdown**: Filters to app_enabled types
3. **Name prefill**: Prefills type name for first toolset of that type
4. **Submit flow**: API call, success navigation
5. **Error handling**: Name exists error, API errors

### File: `crates/bodhi/src/app/ui/toolsets/edit/page.test.tsx`

**Test categories:**
1. **Loading**: Toolset fetch, loading state
2. **App disabled redirect**: Redirects if toolset.app_enabled false
3. **Form population**: Fields populated from toolset
4. **API key handling**: Keep vs Set modes
5. **Update flow**: API call, success toast
6. **Delete flow**: Confirmation dialog, delete API call, navigation

### File: `crates/bodhi/src/app/ui/toolsets/admin/page.test.tsx` (NEW)

**Test categories:**
1. **Access control**: Redirects non-admin
2. **Type list**: Renders types with status
3. **Enable/disable toggle**: Confirmation, API call, status update

### File: `crates/bodhi/src/hooks/useToolsets.test.ts`

**Test categories:**
1. **useToolsets**: Fetch, caching, error handling
2. **useToolset**: Fetch by ID, enabled state
3. **useCreateToolset**: Success callback, cache invalidation
4. **useUpdateToolset**: Partial update, cache invalidation
5. **useDeleteToolset**: Success callback, cache invalidation
6. **useToolsetTypes**: Fetch types list

### File: `crates/bodhi/src/app/ui/chat/ToolsetsPopover.test.tsx`

**Test categories:**
1. **Grouping**: Toolsets grouped by type
2. **Collapse behavior**: Single toolset sections collapsed
3. **Availability**: Disabled state, tooltip for unavailable
4. **Selection**: Toggle tool, toggle toolset
5. **Auto-filter**: Unavailable toolsets auto-unchecked

---

## Out of Scope

- Playwright e2e tests (separate phase)
- Setup wizard toolsets page (`/ui/setup/toolsets`)
- `SetupToolsetForm.tsx` component
- API documentation updates
