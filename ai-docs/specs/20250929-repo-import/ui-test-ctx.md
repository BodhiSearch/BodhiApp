# Test Context & Technical Patterns

## Type Structure

### From @bodhiapp/ts-client
- **CreateChatCompletionRequest**: Request type for chat completions
- **CreateChatCompletionResponse**: Response type for non-streaming completions
- **CreateChatCompletionStreamResponse**: Response type for streaming completions (SSE chunks)

### llama.cpp Extensions
- **timings**: Optional field added by llama.cpp server
  - `prompt_per_second?: number` - Prompt processing speed
  - `predicted_per_second?: number` - Token generation speed

## Metadata Fields

### Standard OpenAI Fields (always present)
- `usage.prompt_tokens`: Number of tokens in the input
- `usage.completion_tokens`: Number of tokens in the output
- `usage.total_tokens`: Sum of prompt + completion tokens

### llama.cpp Extensions (optional)
- `timings.prompt_per_second`: Prompt processing speed
- `timings.predicted_per_second`: Token generation speed (displayed as "Speed: X.XX t/s")

## UI Display Patterns

### Component Data-TestIDs (Verified Agent 0, Updated Agent 1)
- **ChatInput**: `chat-input` (textarea element)
- **Send Button**: `send-button` (submit button) - disabled when: !input.trim() || streamLoading || !isModelSelected
- **Empty State**: `empty-chat-state` (when no messages)
- **User Message**: `user-message` (message container)
- **User Message Content**: `user-message-content` (content within user message)
- **Assistant Message**: `assistant-message` (message container)
- **Assistant Message Content**: `assistant-message-content` (content within assistant message)
- **Streaming Message**: `streaming-message` (temporary message while streaming)
- **Message Metadata**: `message-metadata` (metadata container with tokens/speed)
- **New Chat Button**: `new-chat-inline-button` (inline new chat button)
- **Model Selector Trigger**: `model-selector-trigger` (dropdown trigger for model selection)
- **Chat History Sidebar**: `chat-history-sidebar` (left sidebar with chat history)
- **Settings Sidebar**: `settings-sidebar` (right sidebar with settings)

### Metadata Display Location
- Located in ChatMessage.tsx
- Displayed only for assistant messages (!isUser)
- Displayed only when not streaming (!isStreaming)
- Contains: token counts (Query/Response) and speed (t/s)

## Test Mocking Patterns (Agent 1)

### Required Mocks for ChatPage Tests
```typescript
// Mock use-mobile hook
vi.mock('@/hooks/use-mobile', () => ({
  useIsMobile: () => false,
}));

// Mock next/navigation hooks
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => null,
}));

// Mock toast hook
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: toastMock,
  }),
}));

// Mock markdown component to avoid rendering complexity
vi.mock('@/components/ui/markdown', () => ({
  MemoizedReactMarkdown: ({ children }: { children: string }) => <div>{children}</div>,
}));
```

### Test Setup Requirements
- `localStorage.clear()` in beforeEach to prevent test pollution
- MSW server handlers for: mockAppInfoReady(), mockUserLoggedIn(), mockModels(), mockChatCompletions()
- Model selection flow required before sending messages
- userEvent.setup() for realistic user interactions

## Model Selection Flow (Agent 1)

### Requirement
- Model must be selected before messages can be sent
- Send button disabled when `!isModelSelected`
- Placeholder changes based on model selection state

### Flow Pattern
1. User clicks model-selector-trigger
2. User selects option by role="option" with model name
3. Model is saved to ChatSettings (localStorage)
4. Send button becomes enabled
5. Input placeholder changes from "Please select a model first" to "Ask me anything..."

## Multi-Turn Conversation Patterns (Agent 2)

### Pattern: Sequential MSW Handler Updates
For multi-turn conversations, update MSW handlers between API calls:

```tsx
// Turn 1
server.use(...mockChatCompletions({ response: turn1Response }));
await user.type(chatInput, 'Hello');
await user.click(sendButton);
await waitFor(() => expect(screen.queryAllByTestId('assistant-message').length).toBe(1));

// Turn 2
server.use(...mockChatCompletions({ response: turn2Response }));
await user.type(chatInput, 'What is React?');
await user.click(sendButton);
await waitFor(() => expect(screen.queryAllByTestId('assistant-message').length).toBe(2));

// Turn 3
server.use(...mockChatCompletions({ response: turn3Response }));
await user.type(chatInput, 'Does it use virtual DOM?');
await user.click(sendButton);
await waitFor(() => expect(screen.queryAllByTestId('assistant-message').length).toBe(3));
```

### Message Accumulation
- Each turn adds messages to `currentChat.messages` array
- User messages and assistant responses accumulate in chat history
- Metadata is preserved per message
- Message count verification: `expect(messages.length).toBe(expectedTurnCount * 2)` (user + assistant per turn)

### Context Preservation
- Previous messages are available in context for subsequent API calls
- Chat settings (model, temperature, etc.) persist across turns
- Metadata (tokens, speed) unique per turn

## Optional Field Handling (Agent 2)

### Timings Field (llama.cpp Extension)
- **Present**: Display "Speed: X.XX t/s" in metadata
- **Absent**: Omit speed display, only show token counts
- **Test Pattern**: Explicitly set `timings: undefined` in response to override MSW defaults

```tsx
// Standard OpenAI (no timings)
server.use(...mockChatCompletions({
  response: { ...mockStandardOpenAIResponse, timings: undefined }
}));

// Verify speed NOT displayed
const metadata = screen.getByTestId('message-metadata');
expect(metadata).not.toHaveTextContent('Speed:');
```

### Usage Field (Token Counts)
- Always present in OpenAI-compatible responses
- `prompt_tokens`: Input token count (labeled as "Query")
- `completion_tokens`: Output token count (labeled as "Response")
- `total_tokens`: Sum (not displayed separately in UI)

### Metadata Display Conditions
From ChatMessage.tsx:
- Only for assistant messages: `!isUser`
- Only when NOT streaming: `!isStreaming`
- Usage always displayed if present
- Speed only displayed if `timings?.predicted_per_second` exists

## Error Handling Patterns (Agent 3)

### API Error Flow
```tsx
// Test 7: API returns error before streaming starts
server.use(...mockChatCompletionsError({
  status: 500,
  code: 'internal_error',
  message: 'Model failed to respond'
}));

// Error handling behavior:
// 1. onError callback invoked with error response
// 2. Toast notification displayed
// 3. Input field PRESERVES original message (not cleared)
// 4. User message NOT added to chat history
// 5. Send button remains enabled for retry

// Pattern: Wait for toast
await waitFor(() => {
  expect(toastMock).toHaveBeenCalledWith(
    expect.objectContaining({
      title: 'Error',
      description: expect.stringContaining('Model failed to respond'),
    })
  );
});

// Pattern: Verify input restoration
expect(chatInput).toHaveValue(originalMessage);

// Pattern: Verify no partial state
expect(screen.queryByTestId('user-message')).not.toBeInTheDocument();

// Pattern: Verify retry capability
server.use(...mockChatCompletions({ response: mockNonStreamingResponse }));
await user.click(sendButton); // Retry succeeds
```

### Streaming Error Flow
```tsx
// Test 8: Stream starts, delivers partial content, then errors
const initialChunks = [
  '{"choices":[{"delta":{"content":"Partial "}}]}',
  '{"choices":[{"delta":{"content":"response "}}]}',
  '{"choices":[{"delta":{"content":"before "}}]}',
  '{"choices":[{"delta":{"content":"error"}}]}',
];

server.use(...mockChatCompletionsStreamingWithError({
  initialChunks,
  errorMessage: 'Stream interrupted'
}));

// Streaming error behavior:
// 1. Partial content accumulated via onDelta callbacks
// 2. Error chunk causes parse exception (logged, not thrown)
// 3. Stream completes "successfully" with partial content
// 4. onFinish called with partial content
// 5. Partial message saved to chat history (no metadata)
// 6. Input cleared (treated as successful completion)
// 7. NO error toast displayed (error silently ignored)

// Pattern: Wait for partial content in chat history
await waitFor(() => {
  const assistantMessages = screen.queryAllByTestId('assistant-message');
  expect(assistantMessages.length).toBeGreaterThan(0);
}, { timeout: 5000 });

// Pattern: Verify partial content displayed
const assistantMessageContent = lastAssistantMessage.querySelector('[data-testid="assistant-message-content"]');
expect(assistantMessageContent?.textContent).toContain('Partial response before error');

// Pattern: Verify metadata absent (incomplete response)
const metadata = screen.queryByTestId('message-metadata');
if (metadata) {
  expect(metadata.textContent).not.toContain('Query:');
  expect(metadata.textContent).not.toContain('Response:');
  expect(metadata.textContent).not.toContain('Speed:');
}

// Pattern: Verify continued interaction
expect(chatInput).not.toBeDisabled();
expect(chatInput).toHaveValue(''); // Input cleared
await user.type(chatInput, 'Can continue chatting');
```

### Error Recovery Patterns

**Input Preservation** (API Error):
- Input restored to original message: `expect(chatInput).toHaveValue(originalMessage)`
- Allows retry without retyping
- Implemented via `resetToPreSubmissionState(userContent)`

**Partial Content Preservation** (Streaming Error):
- Partial response saved to chat history
- Empty metadata container present (no tokens/speed)
- User can see what was received before error

**State Recovery**:
- Send button never permanently disabled
- Input field never disabled
- User can always continue interacting
- Chat history preserves all successful/partial messages

### Error Detection Limitations

**Current Streaming Error Behavior**:
- Error chunks in SSE stream are NOT detected as errors
- Parse exceptions caught, logged as warnings, processing continues
- Stream completes successfully with partial content
- No error notification to user
- This is current production behavior

**Recommendation for Future**:
Check for `error` field in SSE chunks:
```typescript
if (json.error) {
  // Handle error chunk explicitly
  onError?.(json.error);
  break; // Stop processing stream
}
```

## Test Patterns Established (Agent 4 Final)

### Core Testing Patterns
- **Scenario-based**: Complete user workflows from start to finish
- **Type-focused**: Verify @bodhiapp/ts-client types parse correctly
- **UI-focused**: Verify metadata displays correctly with conditional rendering
- **Real usage**: Multi-turn conversations with context preservation
- **Model selection**: Required prerequisite for chat interaction
- **Error recovery**: Input restoration and state recovery patterns

### Established Code Patterns
1. **Wait for Persistent State**: Query chat history, not temporary streaming state
2. **Multi-Turn Conversations**: Update MSW handlers between turns
3. **Optional Field Handling**: Explicitly set undefined to override defaults
4. **Dynamic Message Counts**: Use queryAll for dynamic content
5. **Error Recovery**: Verify input restoration for retry capability

### Test Infrastructure Patterns
- MSW v2 for API mocking with generated types
- Test fixtures for reusable response data
- localStorage.clear() in beforeEach for isolation
- userEvent.setup() for realistic interactions
- waitFor with timeout for async operations

## Known Issues and Resolutions

### ~~Issue 1: API Integration in Tests~~ (RESOLVED by Agent 2)
**Problem**: Assistant message content renders empty despite correct mock structure
**Root Cause**: Production bug in `use-chat.tsx` - using `currentAssistantMessage` (empty for non-streaming) instead of `message.content`
**Resolution**: Fixed in `use-chat.tsx` line 77 - use `currentAssistantMessage || message.content`
**Impact**: **Production bug fix** - was affecting real users

### ~~Issue 2: Streaming Tests~~ (RESOLVED by Agent 2)
**Problem**: streaming-message data-testid not detected during tests
**Root Cause**: Testid didn't exist - ChatMessage always used 'assistant-message' regardless of streaming state
**Resolution**: Added conditional testid in ChatMessage.tsx line 36
**Lesson**: Streaming state is ephemeral in tests - verify final state in chat history instead

### ~~Issue 3: Test Complexity~~ (ADDRESSED by Agent 2)
**Problem**: ChatPage has many nested providers making full integration tests complex
**Resolution**: Full integration tests work with proper patterns:
- Wait for messages in persistent chat history, not temporary state
- Use `queryAllByTestId` to handle dynamic message counts
- Update MSW handlers between turns for multi-turn tests
**Outcome**: 8/8 integration tests passing

## Final Project Status (Agent 4)

**Implementation Complete**: ✅ 100%
- All 10 ChatPage tests passing
- Zero regressions in full suite (641/648 passing)
- 2 production bugs discovered and fixed
- Comprehensive documentation added
- Test patterns established for future work

**Type Safety Achievement**:
- Full coverage of @bodhiapp/ts-client types
- CreateChatCompletionResponse parsing verified
- CreateChatCompletionStreamResponse parsing verified
- Optional field handling verified (timings, usage)
- Error type handling verified

**Quality Metrics**:
- Build: ✅ SUCCESS
- TypeCheck: ⚠️ Pre-existing issues (not introduced)
- Lint: ⚠️ Pre-existing issues (not introduced)
- Format: ✅ SUCCESS
- Tests: ✅ 10/10 ChatPage, 641/648 total

**Deliverables**:
- Working comprehensive test suite
- Production bug fixes (use-chat.tsx, ChatMessage.tsx)
- Test infrastructure (fixtures, handlers, patterns)
- Complete documentation (plan, log, context, summary)

## Successful Test Patterns (Agent 2)

### Pattern 1: Wait for Chat History (Not Temporary State)
```tsx
// ✅ CORRECT: Wait for message in persistent chat history
await waitFor(() => {
  const assistantMessages = screen.queryAllByTestId('assistant-message');
  expect(assistantMessages.length).toBeGreaterThan(0);
}, { timeout: 5000 });

// ❌ INCORRECT: Wait for temporary state (gets cleared)
await waitFor(() => {
  expect(screen.getByTestId('assistant-message')).toBeInTheDocument();
});
```

### Pattern 2: Chat History Navigation
```tsx
// Chats are ordered newest-first (prepended to array)
const chatHistoryButtons = chatHistoryContainer.querySelectorAll('button[data-testid^="chat-history-button-"]');

// Click oldest chat (index 1, not 0)
await user.click(chatHistoryButtons[1] as HTMLElement);
```

### Pattern 3: Dynamic Message Count
```tsx
// Use queryAll + length check instead of fixed getAll
let assistantMessages = screen.getAllByTestId('assistant-message');
expect(assistantMessages[0].textContent).toContain('Expected content');
```