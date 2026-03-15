---
title: 'Chat UI'
description: "A comprehensive guide to using Bodhi App's Chat Interface"
order: 201
---

# Chat UI

Bodhi App's Chat UI provides a conversational AI interface with support for streaming responses, tool calling, MCP integration, and fine-grained parameter control.

## Overview

The Chat UI features a three-panel design:

- **Chat History Panel (Left):** View and manage past conversations.
- **Main Chat Panel (Center):** Interact with the AI assistant.
- **Settings Panel (Right):** Configure AI behavior using sampling parameters.

<img
  src="/doc-images/chat-ui.jpg"
  alt="Chat UI"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

Conversations and settings are stored locally in your browser's LocalStorage. Data is private but will be lost if you clear your browser data.

## Streaming Responses

Bodhi App uses real-time streaming to display AI responses as they are generated.

**How It Works**:

- When you send a message, it appears immediately in the chat
- The AI assistant's response streams token-by-token as it is generated
- You can watch the response build in real-time instead of waiting for completion

**Stop Generation**:

- Currently there is no option to stop streaming mid-way
- Navigating away from the page drops the network connection, but the backend continues processing the request until complete

**Technical Details**:

- Uses Server-Sent Events (SSE) for real-time streaming
- Responses are saved to LocalStorage as they complete

**Benefits**:

- Faster perceived response time (see output immediately)
- Better user experience (no blank waiting screen)
- More natural conversation flow

## Tool Calling

Models with tool calling support can invoke tools during a conversation. When tools are enabled, the model can decide to call a tool to gather information or perform actions before generating its final response.

**How It Works**:

1. You send a message to the model
2. The model generates a `tool_call` specifying which tool to invoke and with what arguments
3. Bodhi executes the tool call against the MCP server
4. The tool result is returned to the model
5. The model generates its final response incorporating the tool result

This loop can repeat multiple times in a single conversation turn. The maximum number of iterations is configurable in the settings panel (defaults to 5).

**Tool Call Display in Chat**:

Each tool call appears as a collapsible section in the conversation:

- **Tool name** and **source** (MCP slug) shown in the header
- **Status badge** indicates the current state:
  - "Calling..." (blue, with spinner) while the tool is executing
  - "Completed" (green, with checkmark) when execution succeeds
  - "Error" (red) when execution fails
- **Arguments**: JSON of the parameters passed to the tool, shown in a collapsible code block
- **Result**: The tool's response, shown in a collapsible code block after execution completes

Tool calls auto-expand while executing and collapse once completed. You can click the header to expand or collapse any tool call at any time.

**Parallel Tool Execution**:

When the model requests multiple tool calls in a single turn, they are executed in parallel for faster response times.

## Thinking Model Support

Models with a "thinking" capability expose their internal reasoning process. When a thinking model is used, the LLM's internal reasoning is displayed in a collapsible section within the assistant's message, allowing you to inspect how the model arrived at its response.

## MCP Integration

Model Context Protocol (MCP) servers extend the model's capabilities by providing additional tools. MCPs configured in the [MCP Setup page](/docs/features/mcps/setup) are available for use in the chat UI.

**MCPs Popover**:

The MCPs popover is accessible via the plug icon button in the chat input area. It provides:

- A list of all configured MCP instances
- A badge on the plug icon showing the count of currently enabled MCP tools
- Per-MCP enable/disable toggle (checkbox enables or disables all tools for that MCP)
- Expandable per-tool enable/disable toggles within each MCP
- Tool count display showing enabled vs. total tools per MCP (e.g., "3/5")

**MCP Availability**:

An MCP appears as available in the popover when:

- The MCP server is enabled by the administrator
- The MCP instance is enabled by the user
- Tools have been discovered (tools cache is populated)
- The tools filter is not empty (if a filter is configured)

Unavailable MCPs are shown dimmed with a tooltip explaining the reason (e.g., "Disabled by administrator", "Tools not yet discovered").

**MCP Tool Selection Persistence**:

Tool selections persist across popover open/close cycles and across new chat sessions within the same browser session. Selections are stored in browser LocalStorage.

**Agentic Loop with MCPs**:

When MCP tools are enabled and the model decides to call one:

1. User sends a message
2. Model generates a tool call targeting an MCP tool
3. Bodhi executes the tool call against the MCP server via the backend API
4. The tool result is returned to the model
5. The model generates its response (or makes additional tool calls)

The tool call UI shows collapsible sections with status badges, arguments JSON, and result display.

## The Chat History Panel

The left panel displays previous conversations grouped by time period (Today, Yesterday, Previous 7 Days). You can click on any conversation to reopen it. A delete option lets you permanently remove a conversation from browser LocalStorage.

**LocalStorage Data Structure**:

- Conversations are stored under the key `bodhi-chats`
- Current active chat ID is stored in `bodhi-current-chat`
- Sidebar state (open/closed) persists in `sidebar-history-open`

**Data Persistence**:

- Chat history survives page reloads and browser restarts
- Storage limits depend on your browser (typically 5-10MB)
- Data is private to your browser and not sent to any server

**Clear Chat History**:

- Chats are stored only in browser LocalStorage
- When you clear chat history, it deletes the chats from browser LocalStorage
- Conversations are never sent to the server

## The Main Chat Panel

The center panel is where the conversation happens:

- Type your message in the input field at the bottom
- Press **Enter** (or click the send icon) to submit your message
- Click the **+** icon to start a new chat
- Responses stream in real-time with rich Markdown rendering and syntax-highlighted code blocks
- Copy responses or individual code blocks using the copy button

## Chat Statistics

Bodhi App displays performance metrics for each AI response.

**Metrics Displayed**:

- **Token counts**: Query and response token counts
- **Speed**: Processing speed in tokens per second (e.g., "Speed 25.3 t/s")
- Statistics appear below each completed AI message
- Metrics are saved with chat history

**Statistics by Model Type**:

- **Local GGUF Models**: Show inference speed based on hardware (CPU/GPU)
- **API Models**: Statistics display varies by provider and network conditions

## The Settings Panel

The right panel provides configuration parameters to fine-tune AI behavior.

**Panel Controls**:

- Sidebar can be collapsed to maximize chat space
- Settings are saved globally in browser LocalStorage
- Settings apply to new messages and conversations
- Settings changes do not affect already completed conversations
- Each parameter has a tooltip explaining its purpose

### Model Selection (Required)

You must select a model before sending messages.

**Available Models**:

- **Local Model Aliases**: GGUF models configured on your device
- **API Models**: Models from providers (OpenAI, Anthropic, Groq, Together AI)
- Models are shown in a single searchable dropdown

**Model Dropdown**:

- Searchable dropdown showing all available models
- Selecting a model enables the send button
- Switching models mid-conversation starts a new context

### Temperature (0-2)

Controls randomness and creativity in responses.

**Range**: 0.0 to 2.0
**Control Type**: Slider with numeric input

**Parameter Priority** (highest to lowest):

1. Chat UI settings (if manually configured)
2. Model alias settings (if using alias)
3. GGUF file defaults (if using model file directly)

**Guidance**:

- **0.0-0.3**: Focused, deterministic output (technical tasks, factual responses)
- **0.4-0.7**: Balanced creativity and coherence (general conversation)
- **0.8-1.2**: Creative and varied responses (creative writing, brainstorming)
- **1.3-2.0**: Highly random and experimental (artistic generation)

### Max Tokens

Maximum length of the AI response in tokens.

**Current Limit**: Currently hard-coded to max to 2,048 tokens, leave it unchecked to use the model default
**Control Type**: Numeric input

**Guidance**:

- **100-500**: Short responses (quick answers, summaries)
- **500-2000**: Standard responses (detailed explanations)
- **2000-4000**: Long-form content (essays, articles)
- **4000-8192**: Extended content (long articles, documentation)

**Note**: Longer responses consume more resources and take longer to generate.

### Top P (Nucleus Sampling)

Alternative to temperature for controlling response diversity.

**Range**: 0.0 to 1.0
**Control Type**: Slider with numeric input

**Guidance**:

- **0.1-0.5**: Focused responses with limited vocabulary
- **0.6-0.9**: Balanced diversity
- **0.95-1.0**: Full vocabulary, maximum diversity

**Note**: Can be used together with Temperature for fine-grained control.

### Top K

Limits token selection to the top K most probable tokens.

**Control Type**: Numeric input

**Note**: Interacts with Top P and Temperature to control token selection.

### Frequency Penalty

Reduces repetition by penalizing tokens based on their frequency in the text so far.

**Control Type**: Slider with numeric input

**Effects**:

- **Positive values**: Reduce repetition (model less likely to repeat words)
- **Negative values**: Encourage repetition
- **Zero**: No frequency penalty applied

### Presence Penalty

Reduces repetition by penalizing tokens that have already appeared.

**Control Type**: Slider with numeric input

**Note**: Works differently from Frequency Penalty - penalizes any token that appeared, regardless of how often.

### Stop Sequences

Custom sequences that stop response generation when encountered.

**Control Type**: Tag-based input

- Press **Enter** to create a stop sequence badge
- Click on a badge to remove it
- Add multiple stop sequences as needed

**Examples**:

- `\n\n` - Stop at double newline (paragraph breaks)
- `###` - Stop at specific marker
- `Human:` - Stop before conversation turn

**Use Cases**:

- Structured output generation
- Preventing over-generation

### System Prompt

Sets instructions or context for the AI assistant.

**Format**: Multi-line text input
**Character Limit**: No limit
**Storage**: Saved in browser LocalStorage as a global user setting

**Examples**:

```
You are a helpful coding assistant specialized in Python.
Provide concise code examples with explanations.
```

```
Respond concisely in 2-3 sentences maximum.
Use simple language appropriate for beginners.
```

```
You are a creative writing assistant.
Help users brainstorm ideas and develop compelling narratives.
```

**Tips**:

- Be specific about desired behavior
- Include output format instructions if needed
- Test different prompts to find what works best

### API Token

API token for authenticated requests.

**Use Case**: Enter an API token to test authenticated access to Bodhi App APIs

- Alternative to session-based authentication
- Useful for testing API token functionality
- Token-based access to chat completions

**Storage**: Stored in LocalStorage for convenience

**Note**: API tokens grant access to Bodhi App features. Keep them secure and remove after testing.

### Parameter Controls

**Toggle Switches**:

- Each parameter has an enable/disable toggle
- When disabled, the backend default value is used
- This allows testing different configurations without manual reset

**Settings Persistence**:

- Settings persist globally in LocalStorage
- Settings apply to all new messages and conversations
- Sidebar state (open/closed) saved in `sidebar-settings-open`

### Understanding Sampling Parameters

Bodhi App uses llama.cpp as its inference engine. All sampling parameters (Temperature, Top P, Top K, Frequency Penalty, Presence Penalty) are passed directly to llama.cpp for processing.

**Parameter Interaction**:

- These parameters can be used together and interact in complex ways
- Different models may respond differently to the same parameter values
- Experimentation is key to finding optimal settings for your use case

**For Technical Details**: See [llama.cpp documentation](https://github.com/ggerganov/llama.cpp) for in-depth information on parameter ranges, defaults, and interactions.

### Reference Configurations

For advanced users, here are sample configurations for common use cases:

**Creative Writing**:

```yaml
Temperature: 0.8
Top P: 0.9
Presence Penalty: 0.6
Frequency Penalty: 0.3
Max Tokens: 2048
```

**Technical Responses**:

```yaml
Temperature: 0.2
Top P: 1.0
Presence Penalty: 0.1
Frequency Penalty: 0.1
Max Tokens: 1024
```

**Balanced Conversation**:

```yaml
Temperature: 0.5
Top P: 1.0
Presence Penalty: 0.4
Frequency Penalty: 0.4
Max Tokens: 1500
```

## Collapsible Panels & Starting a New Chat

Both the Chat History and Settings Panels are collapsible, allowing you to maximize chat space.

**Panel State Persistence**:

- Left sidebar (history): Stored in `sidebar-history-open` LocalStorage key
- Right sidebar (settings): Stored in `sidebar-settings-open` LocalStorage key
- Panel states persist across page reloads and browser sessions

You have two options to start a new conversation:

- Click the **+** button in the chat input area
- Use the new chat option in the Chat History Panel

**New Chat Behavior**:

- Clears current conversation context
- Retains your selected model and settings
- LocalStorage entry is created when the first message is sent

## Error Handling

Bodhi App includes error handling to ensure a smooth chat experience.

**Input Restoration**:

- If an error occurs before streaming starts, your input is restored to the text field
- You can edit and retry your message without retyping
- Simply press Enter to resend

**Error Display**:

- Errors are shown in a toast notification at the bottom right of the screen
- The message remains in your input field so you can retry
- Partial responses (if streaming was interrupted) are stored with error state

**Common Errors**:

- **"Network connection failed"**: Check your internet connection and try again
- **"Model unavailable"**: The selected model may be loading or unavailable. Try a different model or wait a moment
- **"Request timeout"**: The request took too long. Try with a shorter prompt or different model
- **"Rate limit exceeded"**: Too many requests in a short time. Wait a moment before trying again

## Mobile and Responsive Design

The Chat UI adapts to different screen sizes.

**Responsive Behavior**:

- **Mobile**: Sidebars become drawer overlays, single-column chat layout, simplified settings panel
- **Tablet**: Collapsible sidebars adapt to available space
- **Desktop**: Fixed dual sidebars with optional collapse, full settings panel with all controls visible

## Advanced Features

**Copy Functionality**:

- Copy entire messages with copy button
- Copy individual code blocks
- Visual feedback indicates successful copy

**Markdown Rendering**:

- Rich text formatting (bold, italic, lists, headings)
- Code blocks with syntax highlighting for common programming languages
- Tables and blockquotes
- Clean, readable formatting in light and dark themes

**Code Highlighting**:

- Automatic language detection for code blocks
- Syntax highlighting adapts to your selected theme
- Copy button on all code blocks for easy code reuse

## Related Documentation

- [Model Aliases](/docs/features/models/model-alias) - Configure local GGUF models
- [API Models](/docs/features/models/api-models) - Set up API providers
- [API Tokens](/docs/features/auth/api-tokens) - Create tokens for programmatic access
- [Application Settings](/docs/features/settings/app-settings) - System configuration
