# Deep Dive Context: Message Types and llama.cpp Integration

## Overview
This document captures critical architectural findings about how BodhiApp's frontend handles chat completions and integrates with llama.cpp responses.

## Key Finding: Dual-Type Strategy

### OpenAI Types (API Layer)
- **Purpose**: Wire protocol - communication with backend API
- **Location**: Generated from OpenAPI spec in `@bodhiapp/ts-client`
- **Scope**: Request/response for `/v1/chat/completions` endpoint
- **Structure**: OpenAI-compatible with llama.cpp extensions

### Message Type (UI Layer)
- **Purpose**: Internal UI state management
- **Location**: `crates/bodhi/src/types/chat.ts`
- **Scope**: React state, chat history, local storage
- **Extensions**: Includes `MessageMetadata` for UI display

## Message Type Analysis

### UI Message Structure
```typescript
export interface Message {
  id?: string;                    // UI-only field for React keys
  content: string;
  role: 'system' | 'user' | 'assistant';
  metadata?: MessageMetadata;     // UI-only extension
}

export interface MessageMetadata {
  model?: string;                 // Captured from response
  usage?: {
    completion_tokens?: number;
    prompt_tokens?: number;
    total_tokens?: number;
  };
  timings?: {                     // llama.cpp extension
    prompt_per_second?: number;
    predicted_per_second?: number;
  };
}
```

### Key Characteristics
- **id**: Optional, used for React list keys in UI rendering
- **metadata**: Attached post-response for UI display purposes
- **timings**: llama.cpp-specific extension captured from API response

## llama.cpp Response Structure

### Actual Server Response (Non-Streaming)
```json
{
  "id": "chatcmpl-xyz",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "llama-3.2-1b-instruct:q4_k_m",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "Hello! How can I help you today?"
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "completion_tokens": 10,
    "prompt_tokens": 5,
    "total_tokens": 15
  },
  "timings": {
    "prompt_per_second": 123.45,
    "predicted_per_second": 67.89
  }
}
```

### Critical Finding: timings Location
**IMPORTANT**: The `timings` field is a **response-level extension**, NOT a message-level field.
- ✅ `response.timings` - Correct location
- ❌ `response.choices[0].message.timings` - Does NOT exist
- ✅ `response.usage` - Correct location

## Adapter Pattern in use-chat-completions.ts

### Current Implementation Strategy
The `use-chat-completions.ts` hook acts as an **adapter layer**:

1. **Accepts**: Hand-rolled `ChatCompletionRequest` with UI `Message[]`
2. **Sends**: Hand-rolled request format to API
3. **Receives**: OpenAI-compatible response with llama.cpp extensions
4. **Transforms**: Response data into UI `Message` format
5. **Attaches**: `metadata` (including timings) to final `Message` object

### Transformation Logic (Non-Streaming)
```typescript
// Lines 149-166 in use-chat-completions.ts
const data: ChatCompletionResponse = await response.json();
if (data.choices?.[0]?.message) {
  const message = {
    ...data.choices[0].message,  // Base message from API
  };
  if (data.usage) {
    message.metadata = {
      model: data.model,
      usage: data.usage,         // Response-level field
      timings: {
        prompt_per_second: data.timings?.prompt_per_second,
        predicted_per_second: data.timings?.predicted_per_second,
      },
    };
  }
  onMessage?.(message);
  onFinish?.(message);
}
```

### Transformation Logic (Streaming)
```typescript
// Lines 119-127 in use-chat-completions.ts
if (json.choices?.[0]?.finish_reason === 'stop' && json.timings) {
  metadata = {
    model: json.model,
    usage: json.usage,
    timings: {
      prompt_per_second: json.timings?.prompt_per_second,
      predicted_per_second: json.timings?.predicted_per_second,
    },
  };
}
```

## Hand-Rolled Types (To Be Replaced)

### ChatCompletionRequest (Lines 13-23)
```typescript
interface ChatCompletionRequest {
  messages: Message[];           // Uses UI Message type
  stream?: boolean;
  model: string;
  temperature?: number;
  stop?: string[];
  max_tokens?: number;
  top_p?: number;
  frequency_penalty?: number;
  presence_penalty?: number;
}
```

### ChatCompletionResponse (Lines 25-44)
```typescript
interface ChatCompletionResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: {
    index: number;
    message: Message;            // Uses UI Message type
    finish_reason: string;
  }[];
  usage?: {
    completion_tokens?: number;
    prompt_tokens?: number;
    total_tokens?: number;
  };
  timings?: {                    // llama.cpp extension
    prompt_per_second?: number;
    predicted_per_second?: number;
  };
}
```

## Migration Strategy Implications

### What Needs to Change
1. Replace hand-rolled `ChatCompletionRequest` with generated OpenAI request type
2. Replace hand-rolled `ChatCompletionResponse` with generated OpenAI response type
3. Update transformation logic to convert OpenAI message to UI Message
4. Ensure `metadata` attachment logic remains functional

### What Must Stay the Same
1. UI `Message` type must remain for React state management
2. `MessageMetadata` must remain for UI display purposes
3. Adapter pattern in `use-chat-completions.ts` must continue
4. Callbacks (`onDelta`, `onMessage`, `onFinish`) use UI `Message` type

### Critical Compatibility Requirements
- OpenAI request type must accept `messages` array that can be converted from UI `Message[]`
- OpenAI response type must have `timings` at response level (llama.cpp extension)
- Transformation logic must extract `usage` and `timings` from response level
- Final callback messages must be UI `Message` type with `metadata` attached

## Testing Considerations

### Baseline Tests to Run
1. **Type Checking**: `npm run test:typecheck`
2. **Unit Tests**: `npm test -- use-chat-completions.test.tsx`
3. **Build**: `npm run build` (ensures Next.js compilation succeeds)

### Expected Baseline State
- All tests should pass
- No TypeScript errors
- Transformation logic correctly attaches metadata
- Both streaming and non-streaming modes work

## Critical Discovery: Missing timings in OpenAPI Spec

### Investigation Results
Upon examination of the generated TypeScript types and OpenAPI specification:

**Finding**: The `timings` field is **NOT present** in:
- ❌ `openapi.json` `CreateChatCompletionResponse` schema
- ❌ `@bodhiapp/ts-client` generated `CreateChatCompletionResponse` type
- ❌ `CreateChatCompletionStreamResponse` type

**Root Cause**: The `timings` field is a **llama.cpp-specific extension** that is:
- ✅ Returned by the backend API at runtime
- ❌ Not documented in the OpenAPI specification
- ❌ Not part of the standard OpenAI API contract

### Implications for Migration

This creates a **type safety gap**:
1. Backend returns `timings` at runtime (llama.cpp extension)
2. OpenAPI spec does not include `timings` field
3. Generated types do not include `timings` field
4. Frontend code expects `timings` for UI display

### Migration Strategy Options

#### Option 1: Accept Type Safety Gap (Recommended for Phase 1)
- Use generated `CreateChatCompletionResponse` type
- Access `timings` via type assertion or `any` escape hatch
- Document this as known limitation
- File issue to add `timings` to OpenAPI spec

```typescript
const data = await response.json() as CreateChatCompletionResponse;
// Type assertion for llama.cpp extension
const timings = (data as any).timings as {
  prompt_per_second?: number;
  predicted_per_second?: number;
} | undefined;
```

#### Option 2: Extend Generated Type (Interim Solution)
- Create local extension type that adds `timings`
- Use intersection type for API responses
- Maintains type safety for llama.cpp extension

```typescript
type LlamaCppTimings = {
  timings?: {
    prompt_per_second?: number;
    predicted_per_second?: number;
  };
};

type ChatCompletionResponseWithTimings = CreateChatCompletionResponse & LlamaCppTimings;
```

#### Option 3: Update OpenAPI Spec (Long-term Solution)
- Add `timings` field to Rust backend utoipa annotations
- Regenerate OpenAPI spec and TypeScript types
- Full type safety for llama.cpp extensions
- **Out of scope for this migration task**

### Recommended Approach

For this migration task, use **Option 2: Extend Generated Type**:
1. Preserves type safety
2. Minimal code changes
3. Self-documenting llama.cpp extension
4. Easy to remove when OpenAPI spec is updated

## Summary

The dual-type strategy is intentional and correct:
- **OpenAI types**: API contract layer (from ts-client) - **but missing llama.cpp extensions**
- **UI Message type**: Internal state management layer
- **Adapter**: `use-chat-completions.ts` bridges the two layers
- **Key insight**: `timings` is response-level, not message-level
- **Critical gap**: `timings` not in OpenAPI spec, requires type extension

Migration must:
1. Replace hand-rolled OpenAI types with generated ones
2. Extend generated types to include llama.cpp `timings` field
3. Preserve adapter pattern and transformation logic
4. Maintain all existing tests passing

## Agent Insights

### Agent 1 Findings - 2025-09-30 15:27 PST
**Type Import Verification**:
- ✅ Import path verified: `@bodhiapp/ts-client` exports `CreateChatCompletionRequest`
  - Located at: `ts-client/src/types/types.gen.ts:604`
  - Type definition: `export type CreateChatCompletionRequest = { ... }`
- ✅ Import path verified: `@bodhiapp/ts-client` exports `ChatCompletionRequestMessage`
  - Located at: `ts-client/src/types/types.gen.ts:307`
  - Type definition: `export type ChatCompletionRequestMessage = (ChatCompletionRequestDeveloperMessage & { ... })`
- ✅ Both types available without build errors
- ✅ TypeScript compilation successful with new imports
- ✅ No conflicts with existing code

**Type Compatibility Observations**:
- Both generated types are exported from the same module
- Import syntax uses `type` keyword for type-only imports (best practice)
- Deprecation comment successfully applied to hand-rolled interface
- All existing tests pass without modification (good sign for backward compatibility)

### Agent 2 Findings - 2025-09-30 15:31 PST
**Type Migration Strategy**:
- ✅ Removed deprecated `ChatCompletionRequest` interface completely
- ✅ Created `ChatCompletionRequestWithUIMessages` type to bridge UI and API types
  - Uses `Omit<CreateChatCompletionRequest, 'messages'> & { messages: Message[] }`
  - Maintains UI Message[] interface for hook consumers
  - Converts to ChatCompletionRequestMessage[] at API boundary
- ✅ Added `toApiMessage()` adapter function for message conversion
  - Handles three role types: system, user, assistant
  - Uses type assertion to match OpenAI tagged union structure
  - Simple switch statement based on role field

**Message Conversion Pattern**:
- UI Message structure: `{ role, content }` (simple)
- OpenAI ChatCompletionRequestMessage: Tagged union with role-specific structures
- Adapter converts at request construction: `messages: request.messages.map(toApiMessage)`
- Type casting needed: `as ChatCompletionRequestMessage` for role-specific structures
- Request body transformation: Spread operator + messages override

**Type Safety Achievements**:
- ✅ Hook signature uses wrapper type with UI Messages
- ✅ Conversion happens transparently in request body construction
- ✅ No changes needed to test files or consuming code
- ✅ TypeScript compiler validates message structure at compile time
- ✅ Backward compatibility maintained - all 6 tests pass without modification

**Key Design Insights**:
- Wrapper type pattern preserves existing API surface
- Adapter pattern cleanly separates UI types from wire protocol types
- Type system catches mismatches at compile time (initially caught messages type mismatch)
- Tests passing without changes confirms behavioral equivalence

### Agent 3 Findings - 2025-09-30 15:33 PST
**Response Type Import and Extension Strategy**:
- ✅ Imported all response types from @bodhiapp/ts-client:
  - `CreateChatCompletionResponse` - Non-streaming response type
  - `CreateChatCompletionStreamResponse` - Streaming response type
  - `ChatCompletionResponseMessage` - Message structure in non-streaming responses
  - `ChatCompletionStreamResponseDelta` - Delta structure in streaming responses
  - `CompletionUsage` - Token usage statistics
- ✅ Created comprehensive `LlamaCppTimings` interface covering all llama.cpp timing fields
- ✅ Extension types use intersection (`&`) to add llama.cpp fields to OpenAI types
- ✅ Both streaming and non-streaming responses get timings extension

**Type Extension Pattern**:
- OpenAI base types provide standard API contract
- Intersection types (`ChatCompletionResponseWithTimings`) add llama.cpp extensions
- `timings` field is optional (`?`) for compatibility with standard OpenAI servers
- Complete field coverage: `cache_n`, `prompt_n`, `prompt_ms`, `prompt_per_token_ms`, `prompt_per_second`, `predicted_n`, `predicted_ms`, `predicted_per_token_ms`, `predicted_per_second`

**Type Compatibility Verification**:
- ✅ All imports resolved successfully from @bodhiapp/ts-client
- ✅ No TypeScript compilation errors with new types
- ✅ Extension types compatible with existing code
- ✅ Deprecation comment added to hand-rolled ChatCompletionResponse
- ✅ All 6 unit tests pass without modification (confirms backward compatibility)

**Key Architectural Insight**:
- Extension pattern allows progressive migration without breaking changes
- Hand-rolled types remain functional during migration phase
- New extension types ready for use in next migration phase
- Type system ensures compile-time safety for llama.cpp-specific fields

### Agent 4 Findings - 2025-09-30 15:36 PST
**Non-Streaming Response Type Migration**:
- ✅ Replaced `ChatCompletionResponse` with `ChatCompletionResponseWithTimings`
- ✅ Converted OpenAI `ChatCompletionResponseMessage` to UI `Message` format
- ✅ Handled content nullability: OpenAI `string | null` → UI `string` with `|| ''`
- ✅ Role type casting: OpenAI `Role` enum → UI literal type `'assistant'`
- ✅ Metadata extraction preserved from response-level fields

**Message Conversion Pattern Established**:
- OpenAI format: `{ role: Role, content: string | null, ... }`
- UI format: `{ role: 'assistant' | 'user' | 'system', content: string }`
- Conversion steps:
  1. Extract `apiMessage` from OpenAI response: `data.choices[0].message`
  2. Create UI Message: `{ role: apiMessage.role as 'assistant', content: apiMessage.content || '' }`
  3. Attach metadata from response-level fields: `message.metadata = { model, usage, timings }`

**Type Safety Achievements**:
- Content handling: Properly converts nullable OpenAI content to required UI string
- Role casting: Safe type assertion from OpenAI enum to UI literal type
- Metadata extraction: Correctly accesses response-level `data.timings` (not message-level)
- Timings extraction: Uses llama.cpp extension fields from `ChatCompletionResponseWithTimings`
- Usage field: Properly typed as `CompletionUsage` from ts-client

**Implementation Insights**:
- Message conversion is straightforward: OpenAI response message → UI Message
- Content field requires null handling: `apiMessage.content || ''`
- Role field needs type assertion: `as 'assistant'`
- Metadata stays on UI Message (not standard OpenAI, but UI convenience)
- All existing tests pass without modification (confirms behavioral equivalence)

**Backward Compatibility Maintained**:
- No changes to callback signatures (still use UI `Message` type)
- No changes to metadata structure (still attached to `message.metadata`)
- No changes to timings extraction logic (still uses response-level fields)
- All 6 unit tests pass without modification

### Agent 5 Findings - 2025-09-30 15:39 PST
**Streaming Response Type Migration**:
- ✅ Replaced implicit `any` type with `ChatCompletionStreamResponseWithTimings`
- ✅ Updated metadata extraction from final chunk with proper typing
- ✅ Fixed null handling for usage field (OpenAI allows null, UI expects undefined)
- ✅ Verified delta content extraction matches `ChatCompletionStreamResponseDelta` structure
- ✅ Preserved streaming accumulation logic unchanged

**Streaming Chunk Structure**:
- Streaming sends multiple chunks with `delta` (partial content), not full `message`
- Delta structure: `{ delta: { content: "..." } }` for intermediate chunks
- Final chunk: `{ finish_reason: 'stop', ... }` with metadata (usage, timings)
- Content accumulation: Frontend concatenates `delta.content` into `fullContent`
- Metadata extraction: Only happens when `finish_reason === 'stop'`

**Type Safety Achievements**:
- Streaming chunk type: `ChatCompletionStreamResponseWithTimings`
- Delta structure: Properly typed with optional `content` field
- Null handling: `json.usage ?? undefined` converts OpenAI null to UI undefined
- Timings extraction: Optional chaining with proper type inference
- Usage field: Converted from `CompletionUsage | null` to `CompletionUsage | undefined`

**Implementation Insights**:
- Streaming chunk format: SSE format with "data: " prefix and JSON payload
- Final chunk identification: `finish_reason === 'stop'` indicates completion
- Metadata timing: Only available in final chunk, not intermediate chunks
- Finish behavior: Final chunk has empty delta and metadata fields
- Content accumulation: Frontend accumulates delta.content into fullContent
- Type safety: Proper null handling for optional fields (usage, timings)

**Backward Compatibility Maintained**:
- No changes to callback signatures (still use UI `Message` type)
- No changes to metadata structure (still attached to `message.metadata`)
- No changes to streaming accumulation logic
- All 6 unit tests pass without modification

### Agent 6 Findings - 2025-09-30 15:43 PST
**Cleanup and Finalization**:
- ✅ No remaining usage of deprecated types found (verified via grep)
- ✅ Clean removal of deprecated `ChatCompletionResponse` interface (20 lines)
- ✅ Hook-specific types correctly preserved (ChatCompletionCallbacks, RequestExts)
- ✅ Architecture fully documented in comprehensive file header
- ✅ Type system provides compile-time safety for all API interactions

**File Header Documentation Added**:
- Complete overview of chat completion hook functionality
- Type architecture explanation (API Layer, UI Layer, Adapters)
- llama.cpp extensions documentation
- Message conversion pattern documentation
- Reference to OpenAI API documentation

**Final Verification**:
- Build: Next.js production build succeeded (43/43 pages)
- Type Check: TypeScript compilation with zero errors
- Tests: All 6 unit tests passed (40ms)
- Grep verification: Only proper type references remain
- No deprecated types in codebase

**Architecture Now Fully Documented**:
- File header provides complete architecture overview
- Type relationships clearly explained
- Adapter pattern documented with examples
- llama.cpp extensions explicitly called out
- Message conversion flow documented

## Migration Summary

### Completed Changes
1. ✅ Request types migrated (Agent 1-2)
2. ✅ Response types imported (Agent 3)
3. ✅ Non-streaming response migrated (Agent 4)
4. ✅ Streaming response migrated (Agent 5)
5. ✅ Deprecated types removed (Agent 6)

### Final Architecture
- **API Layer**: Generated OpenAI types from @bodhiapp/ts-client
- **UI Layer**: Simplified Message type for React state
- **Adapters**: toApiMessage() and response handlers
- **Extensions**: llama.cpp timings support via intersection types

### Type Safety Achievements
- Compile-time verification of API contracts
- Proper null/undefined handling
- Tagged union support for role-based messages
- Optional field handling with type guards
- Zero hand-rolled OpenAI types remaining

### Migration Metrics
- **Total Agents**: 6 (Agent 0-6)
- **Lines Removed**: ~50 lines of hand-rolled types
- **Lines Added**: ~40 lines of adapters + documentation
- **Test Pass Rate**: 100% (6/6 tests)
- **Build Success**: All 43 pages
- **Type Errors**: 0

### Maintenance Benefits
- ✅ Automatic type updates from backend changes
- ✅ Compile-time API contract verification
- ✅ IDE autocomplete and IntelliSense support
- ✅ Self-documenting code with generated types
- ✅ Reduced maintenance burden (no manual type updates)
- ✅ Type safety across entire request/response cycle

### Agent 7 Findings - 2025-09-30 15:47 PST
**Integration Testing**: Completed successfully across all dependent components

**Components Tested**:
- use-chat-completions.test.tsx: 6/6 tests passed (61ms)
- use-chat.test.tsx: 8/8 tests passed (41ms)
- use-chat-db.test.tsx: 6/6 tests passed (18ms)
- Full test suite: 262/262 tests passed across 24 test files

**Type Compatibility Verification**:
- ✅ Message type: Used consistently across UI components
- ✅ OpenAI types: Used only in API boundary (use-chat-completions)
- ✅ No type conflicts detected
- ✅ Import paths verified and working

**Integration Point Analysis**:
1. **use-chat.tsx** (Main consumer):
   - Imports: `useChatCompletion` from use-chat-completions
   - Uses: UI Message type throughout
   - Status: All 8 tests passing, no issues

2. **use-chat-db.tsx** (Chat persistence):
   - Uses: Chat and Message types from @/types/chat
   - Status: All 6 tests passing, no issues

3. **MSW handlers** (Test infrastructure):
   - Uses: Own type definitions (no dependency on use-chat-completions types)
   - Status: No conflicts, proper separation of concerns

4. **UI Components** (ChatMessage.tsx, ChatUI.tsx):
   - Uses: Message type from @/types/chat
   - Status: Type compatibility maintained

**Regression Testing**:
- ✅ No breaking changes detected
- ✅ All existing functionality preserved
- ✅ Callback signatures unchanged (onDelta, onMessage, onFinish)
- ✅ Hook API surface identical (no breaking changes)
- ✅ Metadata attachment logic working correctly
- ✅ Streaming and non-streaming modes both functional

**Build & Type Checking**:
- ✅ npm run build: Next.js production build succeeded (43/43 pages)
- ✅ npm run test:typecheck: TypeScript compilation passed with zero errors
- ✅ All static pages generated successfully
- ✅ No runtime type errors detected

## Integration Verification Summary

### Test Coverage
- **Unit Tests**: 262 tests across 24 test files
- **Integration Tests**: use-chat.test.tsx, use-chat-db.test.tsx
- **Component Tests**: All UI components with Message type
- **Build Tests**: Production build and type checking

### Type Safety Achievements
- **API Layer**: Generated OpenAI types from @bodhiapp/ts-client
- **UI Layer**: Simplified Message type for React state
- **Adapter Layer**: toApiMessage() for request conversion
- **Extension Layer**: LlamaCppTimings for llama.cpp-specific fields

### Migration Validation
- ✅ All hand-rolled OpenAI types removed
- ✅ Generated types integrated successfully
- ✅ Adapter pattern working correctly
- ✅ No regressions in dependent components
- ✅ Type safety maintained across all layers
- ✅ Build and type checking pass

### Success Criteria Met
- ✅ use-chat.test.tsx passes (8/8 tests)
- ✅ use-chat-db.test.tsx passes (6/6 tests)
- ✅ Full test suite passes (262/262 tests)
- ✅ npm run build passes (43/43 pages)
- ✅ npm run test:typecheck passes (0 errors)
- ✅ No regressions found
- ✅ Type imports verified
- ✅ Documentation updated

## Final Migration Status

**Migration Complete**: All 8 agents (0-7 migration + verification + quality assurance) completed successfully

**Zero Issues**: No breaking changes, no regressions, no type conflicts

**Production Ready**: All tests passing, build successful, type checking clean, code formatted and linted

---

## Final Migration Summary - 2025-09-30 15:48 PST

### Architecture Established

**Type Layers**:
1. **API Layer**: Generated OpenAI-compatible types from @bodhiapp/ts-client
   - `CreateChatCompletionRequest`, `CreateChatCompletionResponse`
   - `CreateChatCompletionStreamResponse`, `ChatCompletionRequestMessage`

2. **Extension Layer**: llama.cpp-specific additions
   - `LlamaCppTimings` interface with all timing fields
   - `ChatCompletionResponseWithTimings` type (intersection)
   - `ChatCompletionStreamResponseWithTimings` type (intersection)

3. **UI Layer**: Simplified types for React state management
   - `Message` type from @/types/chat
   - `MessageMetadata` with usage and timings
   - Hook-specific types: `ChatCompletionCallbacks`, `RequestExts`

4. **Adapter Layer**: Type conversion at boundaries
   - `toApiMessage()`: UI Message → OpenAI ChatCompletionRequestMessage
   - Response handlers: OpenAI types → UI Message with metadata

**Design Patterns**:
- Intersection types for extensions (`Type1 & Type2`)
- Tagged unions for role-based messages
- Wrapper types for maintaining UI API (`ChatCompletionRequestWithUIMessages`)
- Null coalescing for OpenAI null → UI undefined conversion
- Type assertions for role narrowing

**Benefits Achieved**:
- ✅ Type safety: Compile-time verification
- ✅ Maintainability: Auto-updates from backend
- ✅ Documentation: Self-documenting with JSDoc
- ✅ Extensibility: Easy to add new fields
- ✅ Testing: Type errors caught before runtime
- ✅ Refactoring: Safe with type checking

### Lessons Learned

1. **Dual-Type Strategy Works**: Separating API and UI types provides flexibility
2. **Adapter Pattern is Key**: Converting at boundaries maintains clean separation
3. **Extension Types are Clean**: Intersection types handle custom fields elegantly
4. **Tests Validate Behavior**: All tests passing = no regressions
5. **Documentation is Critical**: File headers and inline docs prevent future confusion
6. **Incremental Migration**: Step-by-step approach caught issues early
7. **Linting Enforcement**: Remove unused imports for clean code

### Future Considerations

**When to Re-evaluate**:
- Adding multimodal support (images, audio) - may need UI Message type updates
- Implementing tool calling - adapter needs tool call handling
- Supporting additional llama.cpp extensions - update LlamaCppTimings interface

**Maintenance Notes**:
- Re-run type generation when backend API changes: `cd ts-client && npm run generate`
- Review generated types if OpenAI spec updates significantly
- Monitor for additional llama.cpp extensions in future versions
- Keep adapter logic synchronized with UI Message structure

**Pattern Reuse**:
This migration pattern can be reused for other hooks:
1. Import generated types from @bodhiapp/ts-client
2. Create extension types for server-specific additions
3. Add adapter functions for type conversion
4. Update hook implementation incrementally
5. Remove deprecated types after verification
6. Document architecture in file header
7. Format and lint for code quality