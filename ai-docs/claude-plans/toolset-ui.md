# Toolset Multi-Instance: UI Layer

## Context Summary

Frontend changes span API hooks, toolset management pages, and chat integration. Key shift: from single config per toolset type to multiple instances with names.

---

## API Hooks

### File: `crates/bodhi/src/hooks/useToolsets.ts`

**Reference:** Current hooks:
- `useAvailableToolsets()` - lists toolset types
- `useToolsetConfig(toolsetId)` - get single config
- `useUpdateToolsetConfig(options)` - update config
- `useDeleteToolsetConfig(options)` - delete config
- `useSetAppToolsetEnabled(options)` - admin enable

**New hook structure:**

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { BODHI_API_BASE } from './useSettings';

const TOOLSETS_ENDPOINT = `${BODHI_API_BASE}/toolsets`;
const TOOLSET_TYPES_ENDPOINT = `${BODHI_API_BASE}/toolsets/types`;

// === Types ===

interface ToolDefinition {
  type: 'function';
  function: {
    name: string;
    description: string;
    parameters: Record<string, unknown>;
  };
}

interface Toolset {
  id: string;
  name: string;
  toolset_type: string;
  description: string | null;
  enabled: boolean;
  has_api_key: boolean;
  app_enabled: boolean;
  tools: ToolDefinition[];
  created_at: string;
  updated_at: string;
}

interface CreateToolsetRequest {
  toolset_type: string;
  name: string;
  description?: string;
  enabled: boolean;
  api_key: string;
}

interface UpdateToolsetRequest {
  name?: string;
  description?: string | null;
  enabled?: boolean;
  api_key?: string;
}

interface ToolsetType {
  id: string;
  name: string;
  description: string;
  app_enabled: boolean;
  tools: ToolDefinition[];
}

// === Instance Hooks ===

/** List user's toolset instances */
export function useToolsets() {
  return useQuery<{ toolsets: Toolset[] }>({
    queryKey: ['toolsets', 'instances'],
    queryFn: async () => {
      const res = await fetch(TOOLSETS_ENDPOINT, { credentials: 'include' });
      if (!res.ok) throw new Error('Failed to fetch instances');
      return res.json();
    },
  });
}

/** Get single instance by ID */
export function useToolset(id: string | undefined) {
  return useQuery<Toolset>({
    queryKey: ['toolsets', 'instance', id],
    queryFn: async () => {
      const res = await fetch(`${TOOLSETS_ENDPOINT}/${id}`, { credentials: 'include' });
      if (!res.ok) throw new Error('Failed to fetch instance');
      return res.json();
    },
    enabled: !!id,
  });
}

/** Create new instance */
export function useCreateToolset(options?: {
  onSuccess?: (instance: Toolset) => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (req: CreateToolsetRequest) => {
      const res = await fetch(TOOLSETS_ENDPOINT, {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(req),
      });
      if (!res.ok) {
        const error = await res.json();
        throw new Error(error?.error?.message || 'Failed to create instance');
      }
      return res.json();
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['toolsets'] });
      options?.onSuccess?.(data);
    },
    onError: (error: Error) => {
      options?.onError?.(error.message);
    },
  });
}

/** Update instance */
export function useUpdateToolset(options?: {
  onSuccess?: (instance: Toolset) => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, ...req }: UpdateToolsetRequest & { id: string }) => {
      const res = await fetch(`${TOOLSETS_ENDPOINT}/${id}`, {
        method: 'PUT',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(req),
      });
      if (!res.ok) {
        const error = await res.json();
        throw new Error(error?.error?.message || 'Failed to update instance');
      }
      return res.json();
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['toolsets'] });
      options?.onSuccess?.(data);
    },
    onError: (error: Error) => {
      options?.onError?.(error.message);
    },
  });
}

/** Delete instance */
export function useDeleteToolset(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      const res = await fetch(`${TOOLSETS_ENDPOINT}/${id}`, {
        method: 'DELETE',
        credentials: 'include',
      });
      if (!res.ok) {
        const error = await res.json();
        throw new Error(error?.error?.message || 'Failed to delete instance');
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['toolsets'] });
      options?.onSuccess?.();
    },
    onError: (error: Error) => {
      options?.onError?.(error.message);
    },
  });
}

// === Admin Type Hooks ===

/** List toolset types (admin) */
export function useToolsetTypes() {
  return useQuery<{ types: ToolsetType[] }>({
    queryKey: ['toolsets', 'types'],
    queryFn: async () => {
      const res = await fetch(TOOLSET_TYPES_ENDPOINT, { credentials: 'include' });
      if (!res.ok) throw new Error('Failed to fetch types');
      return res.json();
    },
  });
}

/** Enable toolset type (admin) */
export function useEnableToolsetType(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (typeId: string) => {
      const res = await fetch(`${TOOLSET_TYPES_ENDPOINT}/${typeId}/app-config`, {
        method: 'PUT',
        credentials: 'include',
      });
      if (!res.ok) throw new Error('Failed to enable type');
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['toolsets'] });
      options?.onSuccess?.();
    },
    onError: (error: Error) => {
      options?.onError?.(error.message);
    },
  });
}

/** Disable toolset type (admin) */
export function useDisableToolsetType(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (typeId: string) => {
      const res = await fetch(`${TOOLSET_TYPES_ENDPOINT}/${typeId}/app-config`, {
        method: 'DELETE',
        credentials: 'include',
      });
      if (!res.ok) throw new Error('Failed to disable type');
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['toolsets'] });
      options?.onSuccess?.();
    },
    onError: (error: Error) => {
      options?.onError?.(error.message);
    },
  });
}
```

---

## Pages

### `/ui/toolsets` - Toolsets

**File:** `crates/bodhi/src/app/ui/toolsets/page.tsx`

**Reference:** Current page lists toolset types with config status.

**New structure:**
- Table with columns: Name, Type, API Key, Status, Actions
- "New Instance" button → `/ui/toolsets/new`
- Row actions: Edit, Delete
- Empty state: "No toolsets configured. Click 'New Toolset' to get started."
- Type display name from `TOOLSET_TYPE_DISPLAY_NAMES` map

```typescript
// Column definitions
const columns = [
  { header: 'Name', accessor: 'name' },
  { header: 'Type', accessor: (row) => TOOLSET_TYPE_DISPLAY_NAMES[row.toolset_type] || row.toolset_type },
  { header: 'API Key', accessor: (row) => row.has_api_key ? 'Configured' : 'Not Set' },
  {
    header: 'Status',
    accessor: (row) => {
      if (!row.app_enabled) return <Badge variant="destructive">App Disabled</Badge>;
      if (!row.enabled) return <Badge variant="secondary">Disabled</Badge>;
      return <Badge variant="success">Enabled</Badge>;
    }
  },
  {
    header: 'Actions',
    accessor: (row) => (
      <>
        {row.app_enabled && (
          <Button size="sm" variant="ghost" onClick={() => navigate(`/ui/toolsets/edit?id=${row.id}`)}>
            Edit
          </Button>
        )}
        <DeleteInstanceButton instance={row} />
      </>
    )
  },
];
```

### `/ui/toolsets/new` - Create Toolset

**File:** `crates/bodhi/src/app/ui/toolsets/new/page.tsx` (NEW)

**Form fields:**
1. **Toolset Type** (dropdown, required)
   - Always shown even with single option
   - Options from `useToolsetTypes()` filtered to app_enabled
2. **Name** (text input, required)
   - Prefill with toolset_type if user has no instances of that type
   - Validation: alphanumeric + hyphens, max 64 chars
   - Error: "Instance name 'X' already exists" on 409
3. **Description** (text input, optional)
   - Max 255 chars
4. **API Key** (password input, required)
   - Validation: required on create
5. **Enabled** (toggle, default true)

**Actions:** Save, Cancel

**On save success:** Navigate to `/ui/toolsets`

### `/ui/toolsets/edit?id={uuid}` - Edit Toolset

**File:** `crates/bodhi/src/app/ui/toolsets/edit/page.tsx`

**Reference:** Current edit page uses `?toolset_id=` query param.

**Changes:**
- Query param: `?id={uuid}` instead of `?toolset_id=`
- Fetch instance by UUID: `useToolset(id)`
- **Type** field: read-only display
- **Name**, **Description**, **Enabled**: editable
- **API Key**: password input, placeholder shows "••••••••" if configured
  - Leave empty to keep existing key
  - Enter new value to update
- **Delete button**: confirmation modal, then delete and navigate to list

**If type disabled by admin:**
- Redirect to `/ui/toolsets` (don't allow editing)
- Check `instance.app_enabled` after fetch

### `/ui/toolsets/admin` - Admin Type Config

**File:** `crates/bodhi/src/app/ui/toolsets/admin/page.tsx` (NEW)

**Access control:** Follow `AppInitializer` pattern for admin-only access.

**Reference:** `crates/bodhi/src/app/AppInitializer.tsx` - role-based rendering

**Structure:**
- Table: Type Name, Description, Status, Action
- Status: Badge showing Enabled/Disabled
- Action: Enable/Disable button (toggles)
- Confirmation modal before toggle

```typescript
function AdminToolsetsPage() {
  const { data: typesData } = useToolsetTypes();
  const enableType = useEnableToolsetType({ onSuccess: () => toast.success('Type enabled') });
  const disableType = useDisableToolsetType({ onSuccess: () => toast.success('Type disabled') });

  const types = typesData?.types || [];

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Type</TableHead>
          <TableHead>Description</TableHead>
          <TableHead>Status</TableHead>
          <TableHead>Action</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {types.map((type) => (
          <TableRow key={type.id}>
            <TableCell>{type.name}</TableCell>
            <TableCell>{type.description}</TableCell>
            <TableCell>
              <Badge variant={type.app_enabled ? 'success' : 'secondary'}>
                {type.app_enabled ? 'Enabled' : 'Disabled'}
              </Badge>
            </TableCell>
            <TableCell>
              <ConfirmButton
                onConfirm={() =>
                  type.app_enabled
                    ? disableType.mutate(type.id)
                    : enableType.mutate(type.id)
                }
                title={type.app_enabled ? 'Disable Type?' : 'Enable Type?'}
                description={
                  type.app_enabled
                    ? `Users will not be able to use ${type.name} toolsets.`
                    : `Users will be able to use ${type.name} toolsets.`
                }
              >
                {type.app_enabled ? 'Disable' : 'Enable'}
              </ConfirmButton>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
```

---

## Chat Integration

### File: `crates/bodhi/src/hooks/use-toolset-selection.ts`

**Reference:** Current state structure:
- `enabledTools: Record<string, string[]>` - toolsetId → tool names
- Tool name encoding: `toolset__{toolset_id}__{method}`

**New structure:**

```typescript
// Key by instance ID (UUID), not toolset type
type EnabledTools = Record<string, string[]>;  // instanceId -> toolNames[]

// Example state:
{
  "uuid-instance-1": ["search", "contents"],
  "uuid-instance-2": ["search"]
}
```

**Tool name encoding change:**

```typescript
// OLD: toolset__{toolset_id}__{method}
// NEW: toolset_{instance_name}__{method}

/** Encode tool name for LLM */
export function encodeToolName(instanceName: string, method: string): string {
  return `toolset_${instanceName}__${method}`;
}

/** Parse tool name from LLM response */
export function parseToolName(toolName: string): { instanceName: string; method: string } | null {
  const match = toolName.match(/^toolset_(.+?)__(.+)$/);
  if (!match) return null;
  return { instanceName: match[1], method: match[2] };
}
```

### File: `crates/bodhi/src/hooks/use-chat.tsx`

**Reference:** Current tool building and execution.

**Changes:**

1. **Cache name→UUID mapping at chat start:**

```typescript
// Build mapping when chat initializes or instances change
const toolsetNameToId = useMemo(() => {
  const map = new Map<string, string>();
  toolsets.forEach((i) => map.set(i.name, i.id));
  return map;
}, [toolsets]);
```

2. **Build tools array:**

```typescript
function buildToolsArray(
  enabledTools: Record<string, string[]>,
  instances: Toolset[]
): ToolDefinition[] {
  const result: ToolDefinition[] = [];

  for (const instance of instances) {
    const enabledToolNames = enabledTools[instance.id] || [];

    for (const tool of instance.tools) {
      if (enabledToolNames.includes(tool.function.name)) {
        result.push({
          type: 'function',
          function: {
            ...tool.function,
            // Use instance NAME for readability in LLM context
            name: encodeToolName(instance.name, tool.function.name),
          },
        });
      }
    }
  }

  return result;
}
```

3. **Execute tool call:**

```typescript
async function executeToolCall(
  toolCall: { function: { name: string; arguments: string } },
  toolsetNameToId: Map<string, string>
): Promise<ToolResult> {
  const parsed = parseToolName(toolCall.function.name);
  if (!parsed) {
    return { error: `Invalid tool name: ${toolCall.function.name}` };
  }

  const instanceId = toolsetNameToId.get(parsed.instanceName);
  if (!instanceId) {
    return { error: `Unknown instance: ${parsed.instanceName}` };
  }

  const response = await fetch(
    `/bodhi/v1/toolsets/${instanceId}/execute/${parsed.method}`,
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
    return { error: error?.error?.message || 'Tool execution failed' };
  }

  return response.json();
}
```

### File: `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx`

**Reference:** Current popover shows toolset types with checkboxes.

**New structure:**
- Group instances by `toolset_type`
- Collapsible sections per type
- Collapse if single instance of that type
- Checkbox per tool per instance

```typescript
function ToolsetsPopover({
  enabledTools,
  onToggleTool,
  onToggleToolset,
}: Props) {
  const { data } = useToolsets();
  const toolsets = data?.toolsets || [];

  // Group by toolset_type
  const grouped = useMemo(() => {
    return toolsets.reduce((acc, instance) => {
      const type = instance.toolset_type;
      if (!acc[type]) acc[type] = [];
      acc[type].push(instance);
      return acc;
    }, {} as Record<string, Toolset[]>);
  }, [toolsets]);

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button variant="outline">Tools</Button>
      </PopoverTrigger>
      <PopoverContent>
        {Object.entries(grouped).map(([type, typeInstances]) => (
          <ToolsetTypeGroup
            key={type}
            type={type}
            instances={typeInstances}
            enabledTools={enabledTools}
            onToggleTool={onToggleTool}
            onToggleToolset={onToggleToolset}
            collapsed={typeInstances.length === 1}
          />
        ))}
      </PopoverContent>
    </Popover>
  );
}

function ToolsetTypeGroup({
  type,
  instances,
  enabledTools,
  onToggleTool,
  onToggleToolset,
  collapsed,
}: GroupProps) {
  const [expanded, setExpanded] = useState(!collapsed);
  const displayName = TOOLSET_TYPE_DISPLAY_NAMES[type] || type;

  return (
    <div className="mb-2">
      <button
        className="flex items-center justify-between w-full font-medium"
        onClick={() => setExpanded(!expanded)}
      >
        <span>{displayName}</span>
        {!collapsed && <ChevronIcon expanded={expanded} />}
      </button>
      {expanded && (
        <div className="ml-2 mt-1">
          {toolsets.map((instance) => (
            <InstanceToolsSection
              key={instance.id}
              instance={instance}
              enabledTools={enabledTools[instance.id] || []}
              onToggleTool={(tool) => onToggleTool(instance.id, tool)}
              onToggleToolset={() => onToggleToolset(instance.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
```

---

## Toolset Type Display Names

**File:** `crates/bodhi/src/lib/toolsets.ts` (NEW or in hooks)

```typescript
/** Human-readable names for toolset types */
export const TOOLSET_TYPE_DISPLAY_NAMES: Record<string, string> = {
  'builtin-exa-web-search': 'Exa Web Search',
  // Add more as toolset types are added
};

/** Get display name for toolset type */
export function getToolsetTypeDisplayName(type: string): string {
  return TOOLSET_TYPE_DISPLAY_NAMES[type] || type;
}
```

---

## Files to Create/Modify

| File | Changes |
|------|---------|
| `crates/bodhi/src/hooks/useToolsets.ts` | Replace with instance-based hooks |
| `crates/bodhi/src/app/ui/toolsets/page.tsx` | Instance list with table |
| `crates/bodhi/src/app/ui/toolsets/new/page.tsx` | NEW: Create instance form |
| `crates/bodhi/src/app/ui/toolsets/edit/page.tsx` | Update for UUID, partial update |
| `crates/bodhi/src/app/ui/toolsets/admin/page.tsx` | NEW: Admin type config |
| `crates/bodhi/src/hooks/use-toolset-selection.ts` | Instance-based state, new encoding |
| `crates/bodhi/src/hooks/use-chat.tsx` | Name→UUID mapping, tool execution |
| `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx` | Grouped instances, collapsible |
| `crates/bodhi/src/lib/toolsets.ts` | NEW: Type display names |

---

## Test Considerations

### Component Tests

- InstanceList: renders instances, empty state, actions
- CreateInstanceForm: validation, prefill name, submit
- EditInstanceForm: load by ID, partial update, delete
- AdminTypesPage: renders types, toggle enable/disable
- ToolsetsPopover: grouping, collapse single instance

### Integration Tests (Playwright)

- Create instance flow
- Edit instance flow
- Delete instance
- Admin enable/disable type
- Chat tool selection with multiple instances
- Tool execution routing to correct instance

**Reference:** Existing Playwright tests in `crates/bodhi/tests/e2e/`
