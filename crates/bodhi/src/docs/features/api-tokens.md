---
title: "API Tokens"
description: "Manage API tokens for secure external access to Bodhi App"
order: 240
---

# API Token Management

Bodhi App provides a robust and secure API Token Management system that enables external applications to access its services programmatically. API tokens are stateless, secure, and flexible, allowing integration without continuous user sessions.

## Key Characteristics

API tokens in Bodhi App:

- **Stateless:** Function independently of user sessions.
- **Secure:** Generated via an OAuth2 token process.
- **Offline Access:** Do not expire automatically but become inactive if unused for over 30 days.
- **Session-Independent:** Remain valid even when you are logged out.
- **Flexible:** Can be easily created, monitored, updated, and revoked.
- **User Linked:** The user generated tokens are linked to a user. If user is de-activated, or removed from the system, the token is also automatically invalidated.

## How API Tokens Work

When you generate an API token, the system:

1. **Generates the Token:**  
   - Navigate to **Settings > API Tokens**.
   - Optionally provide a descriptive name.
   - **Important:** The token is displayed only once—copy and store it securely.
2. **Stores & Validates the Token:**  
   - Saves the token along with its metadata (creation date, status, etc.) in the database.
   - Validates using cryptographic signature checks.
   - Monitors usage to automatically deactivate tokens idle for over 30 days.
3. **Enables Token Usage:**  
   - Include the token in the `Authorization` header of API requests, for example:
     ```bash
     curl -H "Authorization: Bearer your-token-here" \
          https://your-bodhi-instance/api/endpoint
     ```
   - Bodhi App verifies the token's validity before processing your request.

## Creating API Tokens

To create a new API token:

1. Go to **Settings > API Tokens**.
2. Click **Generate New Token**.
3. Enter an optional descriptive name.
4. Click **Generate** and immediately copy the token.
   - **Note:** The token will not be shown again.

<p align="center">
  <img 
    src="/doc-images/api-tokens.jpeg" 
    alt="API Tokens Page" 
    class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%]"
  />
</p>

## Managing API Tokens

On the API Tokens page, you can:

- **View Tokens:** See a list with details like name, status, creation date, and last update.
- **Revoke Tokens:** Deactivate tokens that are no longer required.
- **Update Token Information:** Edit token names and descriptions.
- **Monitor Usage:** Track token activity to prevent idle timeouts.

## Security Best Practices

- **Store Tokens Securely:** Avoid exposing tokens in public repositories or logs.
- **Use Unique Tokens:** Generate distinct tokens for different integrations.
- **Review Regularly:** Monitor usage and revoke unused or potentially compromised tokens.
- **Cache Verification Data:** Token verification is cached for performance and enhanced security.

## Non-authenticated Mode
The API tokens feature is not available for non-authenticated mode. Any one or any app, with access to you over the network, can access your API endpoints as they are public and unauthenticated.

## Summary

Bodhi App's API Token Management system offers a clear and secure method for external integrations. By following best practices for creation, storage, and management, you ensure seamless and secure access to your Bodhi App instance.

Happy integrating! 