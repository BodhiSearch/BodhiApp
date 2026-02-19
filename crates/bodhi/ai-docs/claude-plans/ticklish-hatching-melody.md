# Plan: Migrate remaining hook files to use ts-client types

## Context

After migrating `useMcps.ts` to import types from `@bodhiapp/ts-client`, two more hook files still hand-roll types that have exact equivalents in ts-client. All other hooks either already use ts-client types or have intentional local types (hook config options, composite types, llama.cpp extensions).

## Analysis Summary

| Hook File | Local Types | ts-client Equivalent? | Action |
|-----------|------------|----------------------|--------|
| `useAppAccessRequests.ts` | 4 types (lines 20-43) | Yes - all 4 | **Migrate** |
| `useApiTokens.ts` | 1 type (lines 50-52) | Partial | **Migrate** |
| `useAuth.ts` | 4 hook config types | No - intentional | Skip |
| `useUsers.ts` | 1 composite type | No - intentional | Skip |
| `use-chat-completions.ts` | 7 extension types | No - intentional | Skip |

## Changes

### File 1: `crates/bodhi/src/hooks/useAppAccessRequests.ts`

**Replace 4 local types with ts-client imports:**

| Local Type | ts-client Type | Notes |
|-----------|---------------|-------|
| `ToolApprovalItem` | `ToolsetApproval` | Same shape, different name |
| `McpApprovalItem` | `McpApproval` | Same shape |
| `ApproveAccessRequestBody` | `ApproveAccessRequestBody` | Same name, uses `ApprovedResources` internally |
| `AccessRequestActionResponse` | `AccessRequestActionResponse` | Same name and shape |

**Steps:**
1. Add imports: `ToolsetApproval`, `McpApproval`, `ApproveAccessRequestBody`, `AccessRequestActionResponse`, `ApprovedResources` from `@bodhiapp/ts-client`
2. Remove local interface definitions (lines 20-43)
3. Add re-exports for consumers
4. Update any references from `ToolApprovalItem` to `ToolsetApproval` across consuming files

**Consumer impact** - None. `ToolApprovalItem` and `McpApprovalItem` are only used internally. `ApproveAccessRequestBody` and `AccessRequestActionResponse` keep the same names so consumers are unaffected.

### File 2: `crates/bodhi/src/hooks/useApiTokens.ts`

**Replace local `UpdateTokenRequestWithId` with inline intersection type (matching pattern used in `useMcps.ts`):**

The `useMcps.ts` hook uses `UpdateMcpRequest & { id: string }` inline rather than defining a separate interface. Apply the same pattern here.

**Steps:**
1. Remove `UpdateTokenRequestWithId` interface (lines 49-52)
2. Replace usage with `UpdateApiTokenRequest & { id: string }` inline in the mutation type signatures

## Verification

1. `cd crates/bodhi && npx tsc --noEmit` - type check
2. `cd crates/bodhi && npm test` - run UI tests
3. `cd crates/bodhi && npm run lint` - lint check
