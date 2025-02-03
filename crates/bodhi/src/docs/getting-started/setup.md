---
title: "Setting Up Bodhi App"
description: "Learn how to install and configure Bodhi App"
---

# Setting Up Bodhi App

When you first launch Bodhi App, you'll need to choose between authenticated and non-authenticated modes. This guide helps you make the right choice.

## Authentication Modes

### Authenticated Mode

Best for:
- Team environments
- Production deployments
- Scenarios requiring access control
- Integration with other services

Features:
- Secure API access
- User management
- Token-based authentication
- Access control for endpoints

### Non-Authenticated Mode

Best for:
- Personal use
- Local development
- Quick testing
- Offline environments

Features:
- Quick setup
- No login required
- All endpoints publicly accessible
- Simplified configuration

## Initial Setup Steps

1. **Launch the App**
   - Open Bodhi App after installation
   - You'll see the setup screen

2. **Choose Mode**
   - Select either authenticated or non-authenticated
   - Consider your use case and security needs

3. **Configure Settings**
   - Set up HuggingFace token (optional)
   - Configure model storage location
   - Set resource limits

4. **Download Initial Models**
   - Browse available models
   - Select models to download
   - Wait for downloads to complete

## Next Steps

After setup:
- [Configure Model Aliases](../model-management/model-alias.md)
- [Start Using Chat](../features/chat-ui.md)
- [Create API Tokens](../features/api-tokens.md) (authenticated mode only) 