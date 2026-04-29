---
title: 'App Access Management'
description: 'Review and grant scoped access to third-party apps that want to use your MCPs and APIs'
order: 244
---

# App Access Management

When a third-party app — typically built with the [Bodhi JS SDK](/docs/developer/bodhi-js-sdk/getting-started) — wants to call your Bodhi instance on a user's behalf, it doesn't get a free pass. It creates an _app access request_ that you review, scope down, and either approve or deny. This page covers the review experience and the audit views.

App access requests are unrelated to [User Access Requests](/docs/features/auth/user-access-requests), which gate human onboarding. The two flows share a UI tab (All Requests) but use different request types and different review screens.

## What this gives you

- Granular consent: you pick which specific MCP servers and which API model instances the app can use.
- Scope cap: apps can only ever receive a `User` or `PowerUser` scoped token — never `Manager` or `Admin`.
- Time-boxed reviews: requests expire 10 minutes after creation if not acted on.
- Auditable: every approval / denial is recorded with the reviewer's username.

## The review flow

When the third-party app initiates the request, the user is redirected to the review page at `/ui/apps/access-requests/review?id=<request-id>`. The user reviewing is whoever is logged into Bodhi at that moment — they need a role of PowerUser or higher to approve.

### What you see on the review page

- **App identity** — the app's name (or client ID if no name is set) and an optional description provided at registration.
- **Requested tools** — for each tool type the app asked for, a checkbox to include it and a dropdown to pick the specific instance you'll grant.
- **Requested MCP servers** — for each MCP server, a toggle and a dropdown of available instances.
- **Approved role** — the role the app's token will carry once issued.

### Granular selection

Selection is _not_ all-or-nothing. You can approve some resources and exclude others within the same request:

- Each tool requires a valid instance with an API key configured. Instances without keys or that are disabled won't appear in the dropdown.
- Each MCP server requires an enabled instance. Disabled MCPs are filtered out.
- Unchecking a resource removes it from the grant. The Approve button label updates to "Approve Selected" once you've excluded anything.
- Approve is disabled until every _included_ resource has a valid instance picked.

### Picking the role

The role dropdown shows only roles you can grant. The cap is `PowerUser` regardless of who's reviewing:

| Reviewer role | Roles you can grant for apps |
| ------------- | ---------------------------- |
| **PowerUser** | User, PowerUser              |
| **Manager**   | User, PowerUser              |
| **Admin**     | User, PowerUser              |

So even an Admin cannot hand an external app Manager- or Admin-equivalent powers. This is by design — apps can only do what a programmatic User or PowerUser scope allows.

If the app asked for `PowerUser` and you only want to grant `User`, just pick `User` from the dropdown. The token is downgraded silently — the app gets less than it asked for and operates within the lower scope.

### Approve or deny

- **Approve** — click "Approve All" or "Approve Selected". The app receives an OAuth token scoped to the role you picked and the resources you checked. Done.
- **Deny** — click "Deny". No token is issued. The app is notified through its callback.

Both actions are final. The app can create a fresh request if it needs a different grant.

## Flow types

The third-party app picks one of two redirect models when it initiates the request:

### Popup flow

The review page opens in a popup window. Approval or denial closes the popup, and the app detects the result via its callback mechanism in the parent window.

### Redirect flow

The review page replaces the current tab. After action:

- If the app provided a redirect URL at registration, you're sent back to it.
- Otherwise, a status page is shown (Approved / Denied / Expired).

## Expiry

App access requests expire **10 minutes** after creation. If you don't review within that window:

- The review page shows "Expired".
- The app must create a new request to try again.
- Expired requests cannot be revived — even by an Admin.

This is intentionally short so that abandoned requests can't accumulate or be approved retroactively.

## What the app gets after approval

A scoped OAuth token bound to:

- The app's client identity (so it can't be used by anyone else).
- The role you selected (User or PowerUser).
- The specific MCP server instances and API model instances you checked.

The app cannot escalate beyond what you granted. To add resources, the app must request again and you must re-approve.

## All Requests view (audit)

Manager and Admin users can review the full history of app access requests from **Settings → Users → All Requests** (`/ui/users/access-requests`). The page lists user _and_ app access requests in a unified table:

| Column   | Notes                                                   |
| -------- | ------------------------------------------------------- |
| Username | Who the request belongs to                              |
| Date     | Submission date for pending; last update for processed  |
| Status   | Pending, Approved, or Rejected                          |
| Reviewer | Username of the approver / rejector (blank for pending) |
| Actions  | Available on pending rows; empty for processed          |

Pending rows can be acted on directly here. Processed rows are read-only audit entries.

## Privilege escalation guardrail

The cap at PowerUser is enforced server-side, not just in the UI dropdown:

- A User-role human cannot approve any app access request (they can't reach the review page).
- A PowerUser, Manager, or Admin can approve up to PowerUser scope for an app.
- No one can grant Manager or Admin scope to an app — those scopes are session-only and don't exist as token scopes.

This prevents an attacker who compromises an Admin's browser from issuing themselves a Manager-equivalent app token.

## Troubleshooting

### Review page shows "Expired"

The 10-minute window elapsed. Have the app initiate a new request.

### Approve button is disabled

Every included tool and MCP server must have a valid instance selected, and the Approved Role must be set. Look for:

- Tools without an API-keyed instance.
- MCP servers without an enabled instance.
- Empty role dropdown.

You can uncheck the unsatisfied resource(s) instead of configuring them, and approve only the rest.

### "No instances available" for a resource

Either no instance exists yet or all existing ones are disabled. To proceed:

- Configure or enable an instance from the relevant page (Models / API or MCPs / Servers), then return to the review.
- Or uncheck the resource and approve the remainder.

### "Failed to load access request"

The request ID in the URL is invalid, or the request was already processed. Check the All Requests page for current status.

## See also

- [Auth Overview](/docs/features/auth/overview) — roles vs scopes; why apps cap at PowerUser
- [User Access Requests](/docs/features/auth/user-access-requests) — the unrelated human onboarding flow
- [User Management](/docs/features/auth/user-management) — operator console, including All Requests
- [API Tokens](/docs/features/auth/api-tokens) — direct programmatic credentials (vs OAuth-issued app tokens)
- [Bodhi JS SDK](/docs/developer/bodhi-js-sdk/getting-started) — building third-party apps that use this flow
