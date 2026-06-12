# Comprehensive ChatPage Test Implementation Plan

## Overview

**Goal**: Add comprehensive scenario-based tests to verify the recent type migration to `@bodhiapp/ts-client` works correctly, focusing on:
- Core chat conversation flow (user input → AI response)
- Multi-turn conversations with state persistence
- API response metadata extraction and UI display (tokens, speed)
- Both streaming and non-streaming response handling

**Approach**: Agent-based sequential implementation with max 3 tests per agent, full verification after each step.

**Coordination Files Location**: `ai-docs/specs/20250929-repo-import/`
- `ui-test-plan.md` - This comprehensive test plan (passed to each agent)
- `ui-test-log.md` - Execution log updated by each agent
- `ui-test-ctx.md` - Technical context and patterns discovered

---

## Agent 0: Foundation & MSW Handler Updates

**Purpose**: Establish baseline, update MSW handlers with new types, create test fixtures, add missing data-testids

### Tasks

1. **Read coordination files** to understand current state:
   - `ai-docs/specs/20250929-repo-import/ui-test-plan.md` (this plan)
   - `ai-docs/specs/20250929-repo-import/ui-test-log.md`
   - `ai-docs/specs/20250929-repo-import/ui-test-ctx.md`

2. **Update MSW handlers** in `crates/bodhi/src/test-utils/msw-v2/handlers/chat-completions.ts`:
   ```typescript
   // Replace hand-rolled interfaces with generated types
   import type {
     CreateChatCompletionRequest,
     CreateChatCompletionResponse,
     CreateChatCompletionStreamResponse,
   } from '@bodhiapp/ts-client';

   // Extend for llama.cpp-specific fields
   interface LlamaCppTimings {
     prompt_per_second?: number;
     predicted_per_second?: number;
   }

   type ChatCompletionResponseWithTimings = CreateChatCompletionResponse & {
     timings?: LlamaCppTimings;
   };

   type ChatCompletionStreamResponseWithTimings = CreateChatCompletionStreamResponse & {
     timings?: LlamaCppTimings;
   };

   // Update all handler functions:
   // - mockChatCompletions(): Add usage and timings to default response
   // - mockChatCompletionsStreaming(): Add final chunk with metadata
   // - Ensure responses match actual llama.cpp server format
   // - Maintain backward compatibility with existing tests
   ```

3. **Review and add missing data-testid attributes** to source components:

   Check these files and add testids if missing:

   **ChatUI.tsx**:
   - `data-testid="chat-ui"` on main container
   - `data-testid="empty-chat-state"` on empty state
   - `data-testid="chat-input"` on input field
   - `data-testid="send-button"` on send button

   **ChatMessage.tsx**:
   - `data-testid="user-message"` on user message container
   - `data-testid="assistant-message"` on assistant message container
   - `data-testid="message-metadata"` on metadata container
   - Individual testids for token/speed displays if needed

   **ChatHistory.tsx**:
   - `data-testid="chat-history-item-{id}"` on each history item
   - `data-testid="new-chat-button"` on new chat button

   **SettingsSidebar.tsx**:
   - `data-testid="model-selector"` on model dropdown
   - `data-testid="settings-sidebar"` on sidebar container

4. **Create test fixtures** in `crates/bodhi/src/test-utils/fixtures/chat.ts`

5. **Run baseline verification**

6. **Update coordination files**

### Success Criteria
- ✅ MSW handlers updated with new types
- ✅ Test fixtures created
- ✅ Data-testids added to source components
- ✅ Baseline: 2/2 tests pass
- ✅ Build + TypeCheck successful
- ✅ Full suite passes

---

[Additional agents and tests follow the same pattern as outlined in the approved plan...]

## Plan Summary

### Test Distribution
- **Agent 0**: Setup (0 tests)
- **Agent 1**: Core response types (3 tests)
- **Agent 2**: Real usage scenarios (3 tests)
- **Agent 3**: Error scenarios (2 tests)
- **Agent 4**: Final verification (0 tests)
- **Total**: 8 new comprehensive tests (+ 2 baseline = 10 total)

### Core Focus Areas
1. ✅ **Type Verification**: New types from @bodhiapp/ts-client work correctly
2. ✅ **Metadata Display**: UI shows response data (tokens, speed) correctly
3. ✅ **Streaming vs Non-Streaming**: Both modes tested thoroughly
4. ✅ **Real Usage**: Multi-turn conversations (primary use case)
5. ✅ **Edge Cases**: Missing fields, errors, recovery
6. ✅ **Integration**: History, persistence, switching