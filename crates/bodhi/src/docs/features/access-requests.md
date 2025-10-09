---
title: 'Access Requests'
description: 'Request access to Bodhi App and track your request status'
order: 240
---

# Access Requests

## Overview

Bodhi App uses a secure access request system. After authenticating with OAuth2, new users must request access from an administrator. This guide explains the access request workflow from a user's perspective.

**Key Points**:

- OAuth2 authentication required first
- Self-service request submission
- Admin approval required
- Track request status
- No access until approved

<img
  src="/doc-images/access-request.jpg"
  alt="Request Access Page"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

## Who Needs to Request Access?

**First User**: Automatically becomes admin (no request needed). The first user to log in via OAuth automatically becomes admin (determined by checking if any users exist in the database during OAuth login).

**All Other Users**: Must request access and wait for admin or manager approval

## Access Request Workflow

### Step 1: Log In with OAuth

Before requesting access, you must authenticate via OAuth2.

1. Navigate to Bodhi App URL (e.g., `http://localhost:1135`)
2. Click "Login" button
3. Redirected to configured OAuth provider (OAuth2 PKCE)
4. Authenticate with your credentials
5. Return to Bodhi App

**First Time Login**: You're redirected to Request Access page automatically

**OAuth Details**: See [Authentication Guide](/docs/intro#authentication)

### Step 2: Submit Access Request

On the Request Access page, submit your request to admins.

**Page Elements**:

- Explanation of access request system
- "Request Access" button
- User information is displayed (email/username from OAuth)
- No additional message field to admin

**Steps**:

1. Review information on page
2. Click "Request Access" button (no additional fields to fill)
3. Request submitted to admin/manager queue

**What Happens**:

- Request saved to database with "Pending" status
- Admins/managers must check the access requests page (no automatic notification sent)
- You see pending status message

### Step 3: Wait for Approval

After submitting, your request is "Pending" until an admin reviews it.

**Pending Screen**:

- Status: "Access Request Pending"
- Submission date displayed
- Message: "Your access request is under review by an administrator"
- No estimated approval time shown

<img
  src="/doc-images/access-pending.jpg"
  alt="Access Request Pending Status"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

**While Waiting**:

- You cannot access Bodhi App features
- You can log out and log back in to check status
- Request cannot be cancelled by you once submitted
- Approval time depends on admin availability (no specific SLA)

**Checking Status**:

1. Return to Bodhi App URL
2. Log in (if not already)
3. Redirected to Request Access page if still pending
4. Status displayed

### Step 4: Approval or Rejection

Admins review your request and either approve or reject it.

**If Approved**:

1. Your active session is invalidated (you are logged out)
2. Log in again to Bodhi App
3. Redirected to Chat page
4. You have access! Role is assigned by approver during approval (User, PowerUser, Manager, or Admin)
5. Start using Bodhi App with your assigned role

**If Rejected**:

1. No explicit rejection notification is shown
2. Log in to Bodhi App
3. System checks for pending request, finds none (request was rejected)
4. Request Access page is displayed again, allowing you to re-request
5. No cooldown period - you can submit a new request immediately
6. **Note**: More informative rejection messaging is planned for future releases

## Request Status Types

| Status       | Meaning            | What to Do                                                                                   |
| ------------ | ------------------ | -------------------------------------------------------------------------------------------- |
| **Pending**  | Under admin review | Wait for admin decision                                                                      |
| **Approved** | Access granted     | Log in and use Bodhi App                                                                     |
| **Rejected** | Access denied      | Request Access page appears again (no pending request found); submit new request immediately |

## Frequently Asked Questions

### How long does approval take?

Approval time depends on administrator availability. There is no specific SLA or estimated time. If your request is pending for an extended period, you can only wait for admin review (no direct contact method or emergency access process available in the UI).

### Can I request access again if rejected?

Yes, you can submit a new access request immediately after rejection. When your request is rejected, the system simply shows the Request Access page again (with no pending request found), allowing you to re-request. There is no cooldown period and no maximum attempt limit.

**Current Behavior**: No explicit rejection message is shown. The page checks for a pending request, finds none, and displays the Request Access form again. More informative rejection messaging with reasons is planned for future releases.

### Can I cancel my request?

No, there is no cancel button available once a request is submitted. The request remains in the admin queue as "Pending" until reviewed.

### Who are the administrators?

Admin contact information is not displayed on the request access page. There is no way to contact administrators directly through the Bodhi App interface.

### Can I use Bodhi App while waiting?

No. Until your request is approved, you cannot access Bodhi App features. You will be redirected to the Request Access page on every login.

### What role do I get when approved?

The approver (admin or manager) selects your role during the approval process. Depending on the approver's role:

- **Admin** can assign: User, PowerUser, Manager, or Admin
- **Manager** can assign: User, PowerUser, or Manager

Administrators can modify your role later if needed. See [User Roles Guide](/docs/features/user-management#understanding-user-roles).

### Why is access request required?

Bodhi App uses access requests for security:

- Prevents unauthorized access
- Allows admin control over user base
- Creates audit trail through request history (retained indefinitely)
- Supports secure multi-user deployment scenarios

## Troubleshooting

### Request Button Disabled

**Symptoms**: Cannot click "Request Access" button

**Possible Causes**:

- Button is disabled while request is in transit (being submitted)
- You already have a pending request (duplicate prevention)
- Network error during submission

**Solutions**:

- Wait a moment for the submission to complete
- Refresh the page if button remains disabled
- Check your request status - you may already have a pending request

### Already Have Pending Request

**Symptoms**: Message says request already submitted

**What to Do**:

- Wait for admin review - duplicate requests are prevented at database level
- Check request status by logging in (you'll see the pending status page)
- You can only have one pending request at a time

### Not Redirected After Approval

**Symptoms**: Approved but still see Request Access page

**Solutions**:

- Log out completely (active session was invalidated on approval)
- Clear browser cache if necessary
- Log in again - you should be redirected to Chat page
- If issue persists, check with administrator about approval status

## Related Documentation

- [User Management (Admin Guide)](/docs/features/user-management) - Admin perspective
- [Authentication](/docs/intro#authentication) - OAuth2 setup
- [User Roles](/docs/features/user-management#understanding-user-roles) - Role permissions
