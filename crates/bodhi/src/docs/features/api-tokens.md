---
title: "API Token Management"
description: "Create and manage API tokens for secure external access to Bodhi App"
---

# API Token Management

Bodhi App provides built-in support for API tokens to secure external access and enable integration with other applications.

## Understanding API Tokens

API tokens in Bodhi App are:
- **Stateless:** Function independently of user sessions
- **Secure:** Generated using OAuth2 token exchange
- **Flexible:** Can be used for programmatic access
- **Manageable:** Easy to create, monitor, and revoke

## Creating Tokens

1. Navigate to Settings > API Tokens
2. Click "Generate New Token"
3. Enter a descriptive name for the token
4. Copy and securely store the generated token
   - **Important:** The token will only be shown once

## Token Properties

- **No Expiration:** Tokens remain valid until explicitly revoked
- **Idle Timeout:** Must be used at least once every 30 days
- **Scope-Based:** Limited to specific operation types
- **Session-Independent:** Works even when user is logged out

## Security Best Practices

- Store tokens securely
- Use unique tokens for different applications
- Regularly review and revoke unused tokens
- Never share tokens in public repositories or logs

## Using Tokens

Include the token in API requests:

```bash
curl -H "Authorization: Bearer your-token-here" \
     https://your-bodhi-instance/api/endpoint
```

## Managing Existing Tokens

From the API Tokens page, you can:
- View all active tokens
- Monitor last usage dates
- Revoke tokens that are no longer needed
- Update token names and descriptions 