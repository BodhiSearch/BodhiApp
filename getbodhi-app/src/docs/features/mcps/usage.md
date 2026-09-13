---
title: 'MCP Usage'
description: 'Enable MCP tools in chat conversations, control whitelists per chat, and read tool execution feedback'
order: 5
---

# MCP Usage

Once you have a working MCP instance (see [Setup](/docs/features/mcps/setup)), there are two places to use its tools:

- The **chat interface** — the model decides when to call tools mid-conversation.
- The **[Playground](/docs/features/mcps/playground)** — you call tools manually with hand-crafted inputs.

This page covers chat. For deeper chat-side mechanics — tool-call rendering, the agentic loop, abort handling — see [Tool Calling in Chat](/docs/features/chat/tool-calling).

## Enable MCPs for a conversation

Open the chat at `/ui/chat/`. Next to the message input is a **plug icon**. Click it to open the MCPs popover.

The popover lists every MCP instance you own:

- A checkbox per MCP enables or disables all of its whitelisted tools at once. The checkbox shows three states: **checked** (all tools enabled), **unchecked** (none), or **indeterminate** (some).
- An expand chevron reveals individual tools. Each tool has its own checkbox so you can pick a subset.
- A tool count indicator (e.g. `3/5`) shows how many tools are currently enabled out of those available.

A small **badge** on the plug icon counts the total number of enabled tools across all MCPs — at-a-glance visibility for what the model is allowed to call.

Selections persist across popover open/close and across new chat sessions in the same browser.

## Tool whitelisting at two layers

There are two places tools can be filtered, and it helps to keep them straight:

1. **Instance-level whitelist** (set on the [Setup](/docs/features/mcps/setup) edit form) — limits which tools the MCP exposes at all. Tools that are not on this whitelist do not appear in the popover.
2. **Per-conversation enable** (set in the chat popover) — picks, from the whitelist, which tools the model can call right now.

The popover only shows tools that pass both filters. If an instance has zero whitelisted tools, the row is dimmed with a tooltip explaining why.

### Why an MCP shows as unavailable

The popover dims a row and disables its checkboxes when the instance is not usable. Hover for the reason:

- **Disabled by administrator** — the parent server is disabled. Ask the admin to re-enable it.
- **Disabled by user** — the instance itself is toggled off. Re-enable on the [Setup](/docs/features/mcps/setup) edit page.
- **Tools not yet discovered** — you never clicked **Fetch Tools** on the edit form. Open it and discover.
- **All tools blocked by filter** — the whitelist is empty. Open the edit form and tick at least one.

## Sending a message with tools enabled

Type a prompt that needs a tool ("List the Python files in this repo", "What's the latest tag on github.com/BodhiSearch/BodhiApp?") and send it. The agentic loop runs:

1. The model receives your message plus definitions of every enabled tool.
2. It may answer in plain text, or it may emit one or more tool-call requests.
3. Bodhi executes each tool call against the right MCP instance.
4. The result is fed back to the model as a tool message.
5. The model produces its final answer (which may itself trigger more tool calls).

A single response can chain several rounds of calls before settling.

## Reading tool execution in chat

Every tool call renders as a collapsible card inside the conversation, between the user turn and the model's final answer.

The card header shows:

- The tool name and the source MCP slug.
- A status badge:
  - **Calling…** (blue, with spinner) — execution in progress. The card auto-expands while it is calling.
  - **Completed** (green, with check) — execution succeeded.
  - **Error** (red, with X) — execution failed.

The expanded body shows:

- **Arguments** — the JSON the model sent, pretty-printed.
- **Result** — the tool's response, pretty-printed. Errors render with a red label and tinted background.

Cards auto-collapse once the call completes. Click to expand or collapse at any time. If a single response triggers multiple tool calls, each renders as its own card; results are matched to the right call internally.

## Errors and failure modes

| Symptom                                           | Likely cause                             | What to do                                                                                                       |
| ------------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Card shows **Error** with a 401/403 message       | OAuth token expired or revoked           | Open the instance edit page → Disconnect → Connect to re-authorize                                               |
| Card shows **Error** with a 404                   | Tool name no longer exists on the server | Re-fetch tools on the edit form; whitelist may need updating                                                     |
| Card shows **Error** but the request looked right | Server-side issue                        | Reproduce in the [Playground](/docs/features/mcps/playground) — its raw-JSON tab makes the failure mode explicit |
| Tool not appearing in popover                     | Instance whitelist or instance disabled  | Check Setup edit page; re-fetch and re-whitelist                                                                 |
| Plug icon badge count is zero                     | No tools enabled in this conversation    | Open popover, tick the tools you want                                                                            |

For protocol-level debugging — request payloads, response shapes — the [Playground](/docs/features/mcps/playground) is the right tool. The chat UI is intentionally lossy; it shows what users need to see, not the wire-level detail.

## Switching tools mid-conversation

The popover stays accessible during a conversation. You can enable or disable tools between turns; the next message picks up the new selection. Existing turns and their tool cards stay rendered as-is.

## Where to next

- [Tool Calling in Chat](/docs/features/chat/tool-calling) — deeper detail on the agentic loop, abort handling, and how the chat UI consumes tool messages.
- [Playground](/docs/features/mcps/playground) — manual tool runs when you want to debug.
- [Setup](/docs/features/mcps/setup) — change instance whitelists or auth state.
