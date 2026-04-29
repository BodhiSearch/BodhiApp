---
title: 'Tool Calling'
description: 'How MCP tools surface in chat, per-conversation tool selection, and the agentic loop that executes tool calls inline'
order: 1
---

# Tool Calling in Chat

Bodhi App's chat UI is **agentic**: when you connect an MCP server, the chat input grows a tool selector, the model can decide to invoke tools, and the resulting tool calls and results are rendered inline as the conversation unfolds.

This page covers the chat-side experience. For setting up an MCP server (auth, base URL, allowed tools), see [MCPs → Setup](/docs/features/mcps/setup).

## What you need

Before tools become useful in chat:

1. The model you select must support tool/function calling. Most modern instruction-tuned models do; reasoning models like the GPT-OSS series do too.
2. At least one MCP server must be configured and enabled. See [MCPs → Overview](/docs/features/mcps/overview).
3. The MCP server's tool list must be discovered — Bodhi caches this; older instances may need a refresh from the MCPs page.

If any of these is missing, the tool selector still appears but will be empty or dimmed.

## The MCPs popover

Look for the **plug icon** in the chat input row. Click it to open the **MCPs popover**.

The popover lists every MCP instance configured for your account. For each:

- A checkbox enables or disables **all** tools from that server.
- An expand chevron reveals the per-tool toggles.
- A counter shows enabled tools out of total (e.g. `3/5`).
- A small spinner appears while the chat UI is connecting to the server.

When at least one tool is enabled, a numeric badge appears on the plug icon showing how many tools are active across all servers.

### Why a server might be unavailable

Servers that you cannot use right now are dimmed, with a tooltip explaining why:

- **Disabled by administrator** — the platform admin disabled this MCP server entry. Ask them to re-enable it.
- **Disabled by user** — you've turned this MCP off on the MCPs page. Re-enable it there.
- **Tools not yet discovered** — Bodhi hasn't been able to enumerate this server's tool list. Open the [MCP Playground](/docs/features/mcps/playground) and run a refresh.

## Per-conversation tool selection

Tool selections are scoped **per chat**:

- The set of enabled tools you pick is saved alongside the conversation.
- Reopening an old chat restores its exact tool selection — past conversations replay with the same toolset they were generated under.
- New chats start with the last-used selection, so you don't have to retick the same tools every time.
- Switching models mid-chat does not clear your selection, but if the new model doesn't support tools, the popover becomes inert.

The selection lives in the same browser-local IndexedDB store as your chat history. There is no server-side per-conversation tool config.

## The agentic loop

Once tools are enabled and you send a message, here's what happens:

1. You send your message.
2. Bodhi sends the message **plus the JSON schema of every enabled tool** to the model.
3. The model either replies normally, or returns one or more `tool_call` requests.
4. For each tool call, Bodhi:
   - Locates the right MCP instance from the tool name (Bodhi prefixes tool names with the MCP slug to keep them unique across servers).
   - Calls the tool via the MCP client, passing the model's arguments.
   - Captures the tool's response (or error).
5. Bodhi sends the tool results back to the model.
6. The model continues — it may answer the user, or call more tools.

Steps 3–6 can repeat. The maximum number of rounds is bounded by the **Max Tool Iterations** setting (default 5; configurable in the Settings panel). Hitting the limit forces the model to produce a final answer instead of calling another tool.

When the model requests multiple tool calls in one turn, Bodhi executes them in **parallel** for lower latency.

## How tool calls render in the conversation

Each tool invocation appears as a collapsible card embedded in the assistant message:

- **Header** shows a wrench icon, the tool name, and the source MCP slug.
- **Status badge** updates live:
  - `Calling...` (blue, spinner) while the tool is running.
  - `Completed` (green, check) on success.
  - `Error` (red, cross) on failure.
- **Arguments** (collapsible): the JSON arguments the model produced, pretty-printed.
- **Result** (collapsible): the tool's response, also pretty-printed. Errors are tinted red.

Cards auto-expand while the tool is executing so you can watch arguments and results land. They collapse once execution finishes — click the header to re-expand any card. If the result is JSON it's formatted; otherwise it's shown verbatim.

Tool calls and their results are saved into the chat history so re-opening a conversation shows the full audit trail.

## Error states

The most common failures and what they mean:

- **Tool call returns `Error`**: the MCP server itself reported a failure. The error text is shown in the result panel and forwarded to the model on the next turn — the model can choose to retry, call a different tool, or apologize.
- **MCP unreachable**: if Bodhi can't connect to the server at all, the call fails with a connection error before the tool runs. Check the server's URL and auth on the MCPs page.
- **Auth expired**: for OAuth-backed MCP servers, an expired token surfaces as an error in the result. Re-authorize the server from the MCPs page and resend the message.
- **Rate-limited or timed out**: the model may retry the tool call on its next turn. If it loops, lower the **Max Tool Iterations** value or disable the offending tool.
- **Model ignored the tools**: not every model is good at choosing tools. Try a stronger model, or rephrase your message to hint at which tool to use.

## Thinking models with tools

Reasoning-capable models that also support tools (e.g. GPT-OSS-120B, certain Claude variants) interleave thinking blocks with tool calls. The chat UI renders both inline:

- A collapsible **thinking** block above or between tool cards shows the model's reasoning trace.
- Tool cards appear in execution order.
- The final assistant message follows once the model decides it has enough information.

## Tips

- **Start small.** Enable just the tools the model needs for the task. Sending fifty tool schemas to a small model wastes context and confuses tool selection.
- **Use stop sequences sparingly.** A stop sequence that lands inside a tool-call payload will truncate the call. See [Parameters and System Prompt](/docs/features/chat/parameters-and-system-prompt).
- **Test tools first.** Use the [MCP Playground](/docs/features/mcps/playground) to call a tool by hand before relying on it in chat. If it doesn't work standalone, the model can't fix it.
- **Use the system prompt to guide tool use.** A line like _"Prefer the `search_docs` tool before answering from memory"_ materially changes selection behaviour.

## Related

- [MCPs → Setup](/docs/features/mcps/setup) — register an MCP server and pick auth method
- [MCPs → Auth Methods](/docs/features/mcps/auth-methods) — Header / OAuth2 preregistered / DCR
- [MCPs → Playground](/docs/features/mcps/playground) — exercise tools by hand
- [Parameters and System Prompt](/docs/features/chat/parameters-and-system-prompt) — Max Tool Iterations and other knobs
- [Chat UI](/docs/features/chat/chat-ui) — the rest of the chat interface
