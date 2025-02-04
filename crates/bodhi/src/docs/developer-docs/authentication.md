---
title: "Authentication"
description: "Authentication guide"
order: 401
---

# Authentication Guide

## Overview

Bodhi App provides two authentication modes and various security features to protect your LLM deployments.

## Authentication Modes

### Authenticated Mode
- Full security with user authentication
- API token support
- Role-based access control
- Secure endpoint protection

### Non-Authenticated Mode
- Open access to all endpoints
- Suitable for local development
- No token validation
- Simplified deployment

## API Token Authentication

### Token Types
- OAuth2 tokens with offline_access
- 30-day idle timeout
- Stateless design
- User-scoped by default

### Token Management
- Creation through OAuth2 token exchange
- Database-backed invalidation
- No limit on active tokens
- Permission caching for performance

## Security Best Practices

1. **Token Handling**
   - Store tokens securely
   - Rotate regularly
   - Monitor for suspicious activity
   - Use environment variables for storage

2. **Access Control**
   - Use minimum required permissions
   - Regular token audits
   - Implement token expiration
   - Monitor failed attempts

## Implementation Examples

### Creating a Token
```bash
curl -X POST https://your-bodhi-instance/api/tokens \
  -H "Content-Type: application/json" \
  -d '{"name": "my-app-token"}'
```

### Using a Token
```bash
curl -H "Authorization: Bearer your-token-here" \
  https://your-bodhi-instance/api/endpoint
``` 