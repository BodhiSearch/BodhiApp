# Plan: Integrate MCPs with Chat

## Context

BodhiApp has toolsets integrated with the chat UI — users can select tools from toolset instances, the LLM calls them, and the agentic loop executes them. MCPs (Model Context Protocol servers) have full CRUD, auth, and execution infrastructure but are **not yet wired into the chat**. This plan adds MCP-chat integration following the same pattern as toolsets.

**Scope**: Frontend only. Uses existing backend endpoints (`GET /bodhi/v1/mcps`, `POST /bodhi/v1/mcps/{id}/tools/{name}/execute`).

## Design Decisions

| Decision | Choice |
|----------|--------|
| UI layout | **Separate popover** — new `McpsPopover` (Plug icon) beside existing `ToolsetsPopover` (Wrench icon) |
| Persistence | **Separate field** — `enabledMcpTools: Record<string, string[]>` on `Chat` type, localStorage key `bodhi-last-mcp-selection` |
| Hook design | **Separate hook** — new `use-mcp-selection.ts` mirroring `use-toolset-selection.ts` |
| tools_filter | **Ceiling** — `null` = all tools, `[]` = block all, `["a","b"]` = only a,b. Whitelist seeded with ALL discovered tools on creation |
| OAuth availability | **Always show available** — no OAuth token check; errors surface at execution time |
| Tool display in chat | **No visual distinction** — MCP and toolset tool calls render identically |
| Tool name encoding | `mcp__{slug}__{toolName}` (vs toolsets' `toolset__{slug}__{methodName}`) |

## Implementation Steps

### Step 1: Create `crates/bodhi/src/lib/mcps.ts`

MCP tool name encoding/decoding utilities. Mirrors `crates/bodhi/src/lib/toolsets.ts`.

```
encodeMcpToolName(mcpSlug, toolName) → "mcp__{mcpSlug}__{toolName}"
decodeMcpToolName(encoded) → { mcpSlug, toolName } | null
isEncodedMcpToolName(name) → boolean
```

### Step 2: Modify `crates/bodhi/src/types/chat.ts`

Add one field to the `Chat` interface:

```typescript
enabledMcpTools?: Record<string, string[]>;  // MCP instance ID → enabled tool names
```

### Step 3: Create `crates/bodhi/src/hooks/use-mcp-selection.ts`

Mirrors `crates/bodhi/src/hooks/use-toolset-selection.ts` exactly with substitutions:

| Original | MCP version |
|----------|-------------|
| `LOCAL_STORAGE_KEY = 'bodhi-last-toolset-selection'` | `'bodhi-last-mcp-selection'` |
| `currentChat?.enabledTools` | `currentChat?.enabledMcpTools` |
| `useToolsetSelection()` | `useMcpSelection()` |
| `UseToolsetSelectionReturn` | `UseMcpSelectionReturn` |

Exports: `useMcpSelection()` with same shape (`enabledTools`, `toggleTool`, `toggleToolset`, etc.)

### Step 4: Create `crates/bodhi/src/app/ui/chat/McpsPopover.tsx`

Mirrors `crates/bodhi/src/app/ui/chat/ToolsetsPopover.tsx` with MCP-specific logic.

**Key differences from ToolsetsPopover**:
- Icon: `Plug` from lucide-react (not Wrench)
- Data: `useMcps()` from `@/hooks/useMcps` (not useToolsets)
- No type grouping: flat list of MCP instances (no `toolset_type` grouping)
- Availability: `mcp.mcp_server.enabled && mcp.enabled && hasTools && hasFilteredTools`
- Tool list: filtered by `tools_filter` ceiling

**Availability logic**:
```
isMcpAvailable(mcp):
  mcp.mcp_server.enabled
  && mcp.enabled
  && mcp.tools_cache != null && mcp.tools_cache.length > 0
  && (mcp.tools_filter == null || mcp.tools_filter.length > 0)
```

**Greyed-out tooltip reasons**:
- `!mcp_server.enabled` → "Disabled by administrator"
- `!mcp.enabled` → "Disabled by user"
- No tools_cache or empty → "Tools not yet discovered"
- tools_filter is `[]` → "All tools blocked by filter"

**Visible tools** (for expanded MCP):
```
getVisibleTools(mcp):
  if tools_filter == null → all tools_cache
  else → tools_cache.filter(t => tools_filter.includes(t.name))
```

**Props**: `{ enabledMcpTools, onToggleTool, onToggleMcp, disabled }`

**data-testid**: `mcps-popover-trigger`, `mcps-badge`, `mcps-popover-content`, `mcp-row-{id}`, `mcp-expand-{id}`, `mcp-checkbox-{id}`, `mcps-empty-state`

**Empty state**: "No MCPs configured" with link to `/ui/mcps`.

### Step 5: Modify `crates/bodhi/src/hooks/use-chat.tsx`

This is the core change — extending the agentic loop.

**5a. Add `buildMcpToolsArray()` function** (after existing `buildToolsArray`):
- Iterates MCPs, skips unavailable (same conditions as popover)
- Applies `tools_filter` ceiling
- Converts `McpTool` → `ToolDefinition`:
  ```
  { type: 'function', function: {
      name: encodeMcpToolName(mcp.slug, tool.name),
      description: tool.description ?? '',
      parameters: tool.input_schema ?? {}
  }}
  ```

**5b. Add `executeMcpToolCall()` function** (after existing `executeToolCall`):
- Decodes `mcp__` prefix via `decodeMcpToolName()`
- Maps slug → UUID via `mcpSlugToId` map
- Calls `POST /bodhi/v1/mcps/{mcpId}/tools/{toolName}/execute` with `{ params: ... }`
- Returns `{ role: 'tool', content: JSON.stringify(result), tool_call_id }` — same format as toolset results

**5c. Modify `executeToolCalls()`** — add `mcpSlugToId` parameter, route by prefix:
```typescript
if (tc.function.name.startsWith('mcp__')) {
  return executeMcpToolCall(tc, signal, headers, mcpSlugToId);
}
return executeToolCall(tc, signal, headers, toolsetSlugToId);
```

**5d. Extend `UseChatOptions`**:
```typescript
enabledMcpTools?: Record<string, string[]>;
mcps?: Mcp[];
```

**5e. In `useChat()` body**:
- Add `mcpSlugToId` memo (same pattern as `toolsetSlugToId`)
- Combine tools: `const tools = [...toolsetTools, ...mcpTools]`
- Pass `mcpSlugToId` to `executeToolCalls()`
- Save `enabledMcpTools` in `createOrUpdateChat()` call
- Update dependency arrays

### Step 6: Modify `crates/bodhi/src/app/ui/chat/ChatUI.tsx`

**6a. Add imports**: `McpsPopover`, `useMcpSelection`, `useMcps`

**6b. Add MCP hooks** (after toolset hooks):
```typescript
const { enabledTools: enabledMcpTools, toggleTool: toggleMcpTool,
        toggleToolset: toggleMcp, setEnabledTools: setEnabledMcpTools } = useMcpSelection();
const { data: mcpsResponse } = useMcps();
const mcps = useMemo(() => mcpsResponse?.mcps || [], [mcpsResponse?.mcps]);
```

**6c. Add auto-filter effect** (after toolset auto-filter):
- Same pattern: filter out unavailable MCPs from `enabledMcpTools`
- Availability: `mcp_server.enabled && mcp.enabled && tools_cache present && tools_filter not empty`

**6d. Pass MCP data to `useChat()`**: add `enabledMcpTools`, `mcps`

**6e. Extend `ChatInputProps`**: add `enabledMcpTools`, `onToggleMcpTool`, `onToggleMcp`

**6f. Add `McpsPopover` in ChatInput JSX** (after ToolsetsPopover):
```tsx
<McpsPopover
  enabledMcpTools={enabledMcpTools}
  onToggleTool={onToggleMcpTool}
  onToggleMcp={onToggleMcp}
  disabled={streamLoading}
/>
```

**6g. Adjust textarea padding**: `pl-24` → `pl-32` (third button added)

### Step 7: Modify `crates/bodhi/src/app/ui/chat/ToolCallMessage.tsx`

Update tool name decoding at line 81 to handle both `toolset__` and `mcp__` prefixes:

```typescript
import { decodeMcpToolName } from '@/lib/mcps';
import { decodeToolName } from '@/lib/toolsets';

// line 81: try both decoders
const toolsetDecoded = decodeToolName(toolCall.function.name);
const mcpDecoded = decodeMcpToolName(toolCall.function.name);
const toolName = toolsetDecoded?.method ?? mcpDecoded?.toolName ?? toolCall.function.name;
const sourceSlug = toolsetDecoded?.toolsetSlug ?? mcpDecoded?.mcpSlug ?? 'unknown';
```

## Files Summary

| File | Action | Pattern Source |
|------|--------|---------------|
| `crates/bodhi/src/lib/mcps.ts` | **Create** | `lib/toolsets.ts` |
| `crates/bodhi/src/types/chat.ts` | **Modify** (+1 field) | — |
| `crates/bodhi/src/hooks/use-mcp-selection.ts` | **Create** | `use-toolset-selection.ts` |
| `crates/bodhi/src/app/ui/chat/McpsPopover.tsx` | **Create** | `ToolsetsPopover.tsx` |
| `crates/bodhi/src/hooks/use-chat.tsx` | **Modify** (extend agentic loop) | — |
| `crates/bodhi/src/app/ui/chat/ChatUI.tsx` | **Modify** (wire MCP hooks + popover) | — |
| `crates/bodhi/src/app/ui/chat/ToolCallMessage.tsx` | **Modify** (dual decoder) | — |
| `crates/bodhi/src/hooks/use-chat-db.tsx` | **No changes** (Chat type flows through) | — |

## Key Type Mappings

```
McpTool { name, description?, input_schema? }
  → ToolDefinition { type: 'function', function: { name: encoded, description: string, parameters: unknown } }

McpExecuteResponse { result?, error? }
  → Message { role: 'tool', content: JSON.stringify(result|error), tool_call_id }
```

## Verification

1. **Unit tests**: `cd crates/bodhi && npm test`
   - `src/lib/mcps.test.ts` — encoding/decoding
   - `src/app/ui/chat/McpsPopover.test.tsx` — popover rendering, availability, tool filtering
   - Updates to existing `use-chat` tests for mixed tool execution

2. **Manual testing**: `cd crates/bodhi && npm run dev`
   - Create MCP instance with tools cached
   - Open chat, verify Plug icon appears next to Wrench icon
   - Open MCP popover, verify tools listed (filtered by tools_filter)
   - Select tools, send message, verify LLM receives tools
   - Verify agentic loop executes MCP tools correctly
   - Verify tool call rendering in chat messages
   - Verify selection persists across chat navigation

3. **E2E tests** (after `make build.ui-rebuild`):
   - `crates/lib_bodhiserver_napi/tests-js/` — add MCP-chat spec mirroring `chat-toolsets.spec.mjs`
