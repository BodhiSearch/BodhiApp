---
title: 'MCPs Overview'
description: 'How MCP servers, instances, and tool execution fit together in Bodhi App'
order: 0
---

# MCPs in Bodhi App

**Model Context Protocol (MCP)** lets a model reach beyond text — it can list files, query databases, search the web, file pull requests — by calling tools that an MCP server exposes over a standard request/response shape. If you have not seen the protocol before, the [MCP concept page](/docs/concepts/mcp-overview) covers the mental model.

This page is the entry point to the MCP feature in Bodhi App. It explains the three roles you will play (or assign to others), where each one happens in the UI, and what to read next.

## The three roles

Bodhi splits MCP responsibilities across three actors. The same person can play all three on a desktop install — a team deployment will have them spread out.

### Admin: pre-register servers in the workspace catalog

Admins curate a shared catalog of MCP servers — the **MCP store**. Publishing a server means filling in:

- Its URL.
- An **auth template**: header-based, OAuth2 with a preregistered client, OAuth2 with Dynamic Client Registration, or none.
- A name, description, and enabled flag.

The catalog lives at `/ui/mcps/servers/`. Once a server is published, every user on the deployment can pick it from a dropdown when creating their own MCP instance — they do not need to know the URL or the auth scheme. Auth templates encode "how to connect" once, so users do not paste the same OAuth client_id over and over.

Pre-registration is optional. If admins skip it, users can still type a URL by hand when creating an instance.

> Read [Pre-registered Servers](/docs/features/mcps/pre-registered-servers) for the admin walkthrough.

### Users: create instances by picking from the store

Each user creates their own **MCP instances** at `/ui/mcps/`. An instance is a per-user connection to a server, and it carries:

- A friendly name and slug.
- The user's own credentials (header secret pasted once, or OAuth tokens stored after authorization).
- A **tool whitelist** — the user picks which of the server's tools they want available.
- An enabled / disabled toggle.

Instances are user-scoped. Your tokens never leak to other users on the same Bodhi deployment. Two users connecting to the same GitHub MCP each authorize separately and operate against their own GitHub identity.

> Read [Setup](/docs/features/mcps/setup) for instance creation, and [Auth Methods](/docs/features/mcps/auth-methods) for how the three connection types differ.

### Models: invoke tools mid-conversation

Once an instance is enabled and tools are whitelisted, models can call those tools two ways:

- **In chat**, via the MCP popover next to the message input. The user picks which MCPs and tools are active for the conversation; the model decides when to call them; Bodhi executes the call and threads the result back into the conversation. Tool calls render inline as collapsible cards. See [Usage](/docs/features/mcps/usage) and [Chat Tool Calling](/docs/features/chat/tool-calling).
- **In the playground**, via a manual form runner. Useful for sanity-checking a tool's input schema before you let a model loose on it. See [Playground](/docs/features/mcps/playground).

External apps can also invoke tools through the **MCP proxy** endpoint, which preserves the same whitelist and per-user auth. That path is for app integrators — see [App Access Management](/docs/features/auth/app-access-management) for the access-grant flow.

## When to use what

| Goal                                                   | Where to go                                                          |
| ------------------------------------------------------ | -------------------------------------------------------------------- |
| Connect to a new MCP server for the first time         | [Setup](/docs/features/mcps/setup)                                   |
| Decide which auth method fits a server                 | [Auth Methods](/docs/features/mcps/auth-methods)                     |
| Test a tool's behavior before chat                     | [Playground](/docs/features/mcps/playground)                         |
| Use MCP tools in a conversation                        | [Usage](/docs/features/mcps/usage)                                   |
| Publish a server for everyone in the workspace (Admin) | [Pre-registered Servers](/docs/features/mcps/pre-registered-servers) |

## Key guarantees

- **Per-user isolation.** Credentials and OAuth tokens never cross between users.
- **Tool whitelisting.** Tools that are not on the instance's whitelist do not fire, even if a model requests them.
- **One auth gate.** MCP traffic goes through the same auth and audit path as everything else in Bodhi.
- **No vendor lock-in.** Any MCP server that speaks the standard protocol works — no Bodhi-specific extensions required.

## Where to next

- New here? Start with [Setup](/docs/features/mcps/setup).
- Picking an auth method? Read [Auth Methods](/docs/features/mcps/auth-methods).
- Want background on the protocol? See [MCP Overview (Concepts)](/docs/concepts/mcp-overview).
