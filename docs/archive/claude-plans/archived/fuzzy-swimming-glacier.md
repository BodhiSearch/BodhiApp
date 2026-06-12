# Toolset Multi-Instance: Remaining Work Plan

## ✅ STATUS: COMPLETED

All phases successfully implemented and tested.

## Context

This plan continues from `dapper-finding-beaver.md` which implemented the UUID-based multi-instance toolset architecture. All 15 phases were completed. This plan covers:

1. Test updates for the new architecture
2. Setup wizard toolsets page implementation

## Implementation Status

### Part 1: Test Updates
- ✅ `src/hooks/useToolsets.test.ts` - Rewritten with new hooks
- ✅ `src/app/ui/toolsets/page.test.tsx` - Updated for instance-based architecture
- ✅ `src/app/ui/toolsets/edit/page.test.tsx` - Updated for UUID-based editing
- ✅ `src/app/ui/setup/toolsets/page.test.tsx` - Fully implemented and tested

### Part 2: Setup Wizard Implementation
- ✅ **Phase setup-form**: Complete rewrite of SetupToolsetForm component
- ✅ **Phase setup-page**: Minor updates to setup page integration
- ✅ **Phase setup-tests**: Complete test rewrite with MSW mocks

---

## Part 1: Test Updates (from previous session)

### Completed Test Updates:

- ✅ `src/hooks/useToolsets.test.ts` - Rewritten with new hooks
- ✅ `src/app/ui/toolsets/page.test.tsx` - Updated for instance-based architecture
- ✅ `src/app/ui/toolsets/edit/page.test.tsx` - Updated for UUID-based editing
- ✅ `src/app/ui/setup/toolsets/page.test.tsx` - Fully implemented

---

## Part 2: Setup Toolsets Page Implementation

### Summary

Rewrite the stubbed setup toolsets page to use the new instance-based architecture. The page shows a combined form with app-config toggle and toolset creation form.

### User Decisions

| Decision            | Applied                                                         |
| ------------------- | --------------------------------------------------------------- |
| UX Flow             | Single combined form - toggle + create form together            |
| Disabled UI         | Show all fields disabled/grayed until type is enabled           |
| Toggle Confirmation | Modal confirmation for enable/disable (like admin page)         |
| Existing Instances  | Show current state, prefill name, let backend handle uniqueness |
| Skip Behavior       | Always allow Skip (consistent with other setup steps)           |
| After Success       | Navigate to browser-extension step                              |

---

## Phase setup-form: Rewrite SetupToolsetForm Component

### File: `crates/bodhi/src/app/ui/setup/toolsets/SetupToolsetForm.tsx`

**Complete rewrite** of the stubbed component.

### Hooks to Use:

```typescript
import { useToolsetTypes, useCreateToolset, useEnableToolsetType, useDisableToolsetType } from '@/hooks/useToolsets';
```

### State Management:

```typescript
const [confirmDialogOpen, setConfirmDialogOpen] = useState(false);
const [pendingToggleState, setPendingToggleState] = useState<boolean | null>(null);
```

### Data Fetching:

```typescript
const { data: typesData, isLoading: typesLoading } = useToolsetTypes();
const exaType = typesData?.types?.find((t) => t.toolset_id === 'builtin-exa-web-search');
const isAppEnabled = exaType?.app_enabled ?? false;
```

### Form Schema:

```typescript
const createToolsetSchema = z.object({
  name: z
    .string()
    .min(1, 'Name is required')
    .max(24, 'Name must be 24 characters or less')
    .regex(/^[a-zA-Z0-9-]+$/, 'Only letters, numbers, and hyphens'),
  description: z.string().max(255).optional(),
  api_key: z.string().min(1, 'API key is required'),
  enabled: z.boolean().default(true),
});
```

### Form Default Values:

```typescript
defaultValues: {
  name: 'builtin-exa-web-search', // Prefill with type name
  description: '',
  api_key: '',
  enabled: true,
}
```

### UI Structure:

```
┌─────────────────────────────────────────────────────────┐
│  Configure Toolsets                                      │
│  Enhance your AI with web search capabilities           │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────────────────────────────────────────────┐│
│  │ Exa Web Search                                      ││
│  │ Search the web using Exa AI                         ││
│  │                                    [Toggle: ON/OFF] ││
│  │                                    [Enabled/Disabled]││
│  └─────────────────────────────────────────────────────┘│
│                                                          │
│  ┌─────────────────────────────────────────────────────┐│
│  │ Create Toolset Instance (disabled if toggle OFF)   ││
│  │                                                      ││
│  │ Name:        [builtin-exa-web-search         ]      ││
│  │ Description: [                                ]      ││
│  │ API Key:     [••••••••••••••••               ]      ││
│  │ Enabled:     [Toggle: ON]                           ││
│  │                                                      ││
│  │              [Create Toolset]                        ││
│  └─────────────────────────────────────────────────────┘│
│                                                          │
│  ┌─────────────────────────────────────────────────────┐│
│  │ ℹ️ Don't have an Exa API key?                        ││
│  │    Get one at exa.ai                                ││
│  └─────────────────────────────────────────────────────┘│
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Toggle Confirmation Dialog:

**Enable Dialog:**

- Title: "Enable Toolset for Server"
- Description: "This will enable Exa Web Search for all users on this server."
- Buttons: "Cancel" | "Enable"

**Disable Dialog:**

- Title: "Disable Toolset for Server"
- Description: "This will disable Exa Web Search for all users. Existing instances will stop working."
- Buttons: "Cancel" | "Disable"

### Event Handlers:

```typescript
// Toggle click
const handleToggleClick = (checked: boolean) => {
  setPendingToggleState(checked);
  setConfirmDialogOpen(true);
};

// Toggle confirm
const handleToggleConfirm = () => {
  if (pendingToggleState === true) {
    enableMutation.mutate({ typeId: 'builtin-exa-web-search' });
  } else {
    disableMutation.mutate({ typeId: 'builtin-exa-web-search' });
  }
  setConfirmDialogOpen(false);
  setPendingToggleState(null);
};

// Form submit
const onSubmit = (data: CreateToolsetFormData) => {
  createMutation.mutate({
    toolset_type: 'builtin-exa-web-search',
    name: data.name,
    description: data.description || undefined,
    api_key: data.api_key,
    enabled: data.enabled,
  });
};
```

### Mutation Callbacks:

```typescript
const enableMutation = useEnableToolsetType({
  onSuccess: () => {
    toast({ title: 'Success', description: 'Toolset enabled for server' });
  },
  onError: (message) => {
    toast({ title: 'Error', description: message, variant: 'destructive' });
  },
});

const createMutation = useCreateToolset({
  onSuccess: (toolset) => {
    toast({ title: 'Success', description: `Created ${toolset.name}` });
    onSuccess?.();
  },
  onError: (message) => {
    toast({ title: 'Error', description: message, variant: 'destructive' });
  },
});
```

### Component Props:

```typescript
interface SetupToolsetFormProps {
  onSuccess?: () => void;
}
```

### Data-TestIds:

- `setup-toolset-form`
- `app-enabled-toggle`
- `toolset-name-input`
- `toolset-description-input`
- `toolset-api-key-input`
- `toolset-enabled-toggle`
- `create-toolset-button`
- `enable-confirm-dialog`
- `disable-confirm-dialog`

---

## Phase setup-page: Update Setup Page

### File: `crates/bodhi/src/app/ui/setup/toolsets/page.tsx`

**Changes:**

1. Keep existing structure with `SetupContainer` and `SetupFooter`
2. Pass `onSuccess` callback that navigates to browser-extension
3. Keep skip functionality in `SetupFooter`

**No major changes needed** - the page structure is already correct.

---

## Phase setup-tests: Update Tests

### File: `crates/bodhi/src/app/ui/setup/toolsets/page.test.tsx`

**Rewrite tests** to match new implementation:

**Test Categories:**

1. **Loading state**: Shows skeleton while types loading
2. **Form display**: Shows toggle and form fields
3. **Toggle disabled state**: Form fields disabled when app-config off
4. **Enable flow**: Toggle → dialog → confirm → toast
5. **Create flow**: Fill form → submit → success toast → navigation
6. **Error handling**: Backend error shows in toast
7. **Skip flow**: Skip button navigates to browser-extension

**MSW Handlers Needed:**

- `mockListTypes([mockType])` - for useToolsetTypes
- `mockEnableType()` - for enable mutation
- `mockDisableType()` - for disable mutation
- `mockCreateToolset()` - for create mutation
- `mockCreateToolsetError({ message: 'Name already exists' })` - for uniqueness error

---

## Files Modified Summary

| File                                                          | Action                 |
| ------------------------------------------------------------- | ---------------------- |
| `crates/bodhi/src/app/ui/setup/toolsets/SetupToolsetForm.tsx` | Complete rewrite       |
| `crates/bodhi/src/app/ui/setup/toolsets/page.tsx`             | Minor updates (if any) |
| `crates/bodhi/src/app/ui/setup/toolsets/page.test.tsx`        | Complete rewrite       |

---

## Verification

**After implementation:**

```bash
# Format and lint
cd crates/bodhi && npm run format && npm run lint

# Run tests
cd crates/bodhi && npm test -- --run

# Build check
cd crates/bodhi && npm run build
```

**Manual testing flow:**

1. Start fresh app in setup mode
2. Navigate to toolsets step
3. Verify toggle is OFF and form is disabled
4. Click toggle → verify confirmation dialog
5. Confirm enable → verify toast and form enables
6. Fill form and submit → verify success toast and navigation
7. Test skip button → verify navigation
8. Test error case (duplicate name) → verify error toast
