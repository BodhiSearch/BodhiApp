---
title: 'App Access Management'
description: 'Review and manage access requests from third-party applications'
order: 244
---

# App Access Management

## Overview

When a third-party application (built with the bodhi-js-sdk) wants to access your MCP servers or API resources through Bodhi App, it creates an access request that you review and approve or deny. This page describes the user-facing review flow and the All Requests page.

**Key Points**:

- You control exactly which resources each app can access
- You can down-scope the role an app receives (e.g., app requests PowerUser, you grant User)
- Apps can only receive User or PowerUser roles -- never Manager or Admin
- Unreviewed requests expire after 10 minutes
- Completely separate from [user access requests](/docs/features/auth/user-access-requests), which handle new user onboarding

## Reviewing an App Access Request

When a third-party app initiates an access request, you are redirected to the review page at `/ui/apps/access-requests/review?id=<request-id>`.

### What You See

The review page displays:

- **App name** (or client ID if no name is set) and optional description
- **Requested tools**: Each tool type the app wants to use, with available instances you can select
- **Requested MCP servers**: Each MCP server the app wants to connect to, with available instances
- **Approved role**: A dropdown to select the role the app will receive

### Selecting Resources

Resource selection is granular -- it is not all-or-nothing:

- **Tools**: Each requested tool type has a checkbox to include or exclude it, and a dropdown to select which specific instance to grant. Only instances that are enabled and have an API key configured appear as valid options.
- **MCP servers**: Each requested MCP server has a toggle to include or exclude it, and a dropdown to select which specific instance to grant. Only enabled instances appear as valid options.

You can approve some resources and deny others within the same request. The "Approve All" button label changes to "Approve Selected" when you uncheck any resources.

### Selecting a Role

The role dropdown shows only roles you are allowed to grant:

| Your Role          | Roles You Can Grant for Apps   |
| ------------------ | ------------------------------ |
| **PowerUser**      | User, PowerUser                |
| **Manager**        | User, PowerUser                |
| **Admin**          | User, PowerUser                |

Apps are limited to **User** and **PowerUser** roles. Manager and Admin are never available for app tokens, regardless of who is reviewing.

If the app requested PowerUser but you want to grant less access, select User from the dropdown. The app receives a token scoped to the role you select.

### Approving or Denying

- **Approve**: Click "Approve All" (or "Approve Selected"). The app receives a scoped token that works only for the approved resources and role. The Approve button is disabled until all included tools and MCP servers have a valid instance selected and a role is chosen.
- **Deny**: Click "Deny". The app receives no token and is notified of the denial.

Both actions are final for this request. The app can create a new request if needed.

## Flow Types

The access request can use one of two flow types, determined by the third-party app:

### Popup Flow

The review page opens in a popup window. After you approve or deny, the window closes automatically. The third-party app detects the result through its callback mechanism.

### Redirect Flow

You are redirected to the review page in your current browser tab. After you approve or deny:

- If a redirect URL was configured, you are sent back to the third-party app
- Otherwise, a status page is shown (Approved, Denied, or Expired)

## Request Expiry

App access requests expire **10 minutes** after creation. If you do not review the request within that window:

- The review page shows an "Expired" status
- The app must create a new access request
- Expired requests cannot be approved or denied retroactively

## What Happens After Approval

When you approve a request, the third-party app receives a scoped OAuth token. This token:

- Only grants access to the specific resource instances you selected
- Operates under the role you chose (User or PowerUser)
- Is bound to the app's client identity

The app cannot escalate its permissions beyond what you granted.

## All Requests Page

Administrators and managers can view all app access requests from the User Management area.

**Navigation**: Settings > Users > All Requests tab, or navigate directly to `/ui/users/access-requests`

**Required Role**: Manager or Admin

The page displays a table with:

| Column       | Description                                                        |
| ------------ | ------------------------------------------------------------------ |
| **Username** | The user who received or is receiving the request                  |
| **Date**     | Submission date (for pending) or last update date (for processed)  |
| **Status**   | Badge showing Pending, Approved, or Rejected                       |
| **Reviewer** | Who approved or rejected the request (blank for pending)           |
| **Actions**  | Role selection, Approve, and Reject buttons (for pending requests) |

Pending requests can be acted on directly from this page. Processed requests are shown for audit purposes.

## Privilege Escalation Protection

The system enforces that you cannot grant an app more access than you have yourself:

- If you are a User, you can only grant User role
- If you are a PowerUser, Manager, or Admin, you can grant User or PowerUser
- Manager and Admin roles are never available for apps

This prevents a scenario where an app could accumulate permissions beyond what any single user intended to grant.

## Troubleshooting

### Review Page Shows "Expired"

The request was not reviewed within 10 minutes. The third-party app needs to create a new access request.

### Approve Button Disabled

All included tool types and MCP servers must have a valid instance selected, and a role must be chosen. Check that:

- Each enabled tool has an instance selected from the dropdown
- Each enabled MCP server has an instance selected
- Instances must be enabled (and for tools, must have an API key configured)
- A role is selected in the Approved Role dropdown

### No Instances Available

If a tool type or MCP server shows "No instances configured" or "All instances are disabled":

- For tools: Create and enable an instance with an API key before approving
- For MCP servers: Create and enable an MCP instance before approving

You can uncheck the resource to exclude it from the approval and approve the remaining resources.

### Review Page Shows Error

If the review page shows "Failed to load access request", the request ID may be invalid or the request may have already been processed. Check the All Requests page for its current status.

## Related Documentation

- [User Access Requests](/docs/features/auth/user-access-requests) -- Separate feature for new user onboarding
- [User Management](/docs/features/auth/user-management) -- Admin perspective on user and request management
- [API Tokens](/docs/features/auth/api-tokens) -- Database-backed tokens for direct programmatic access
