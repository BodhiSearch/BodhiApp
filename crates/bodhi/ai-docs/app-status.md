# Application Status Guide

## Overview
The Bodhi App uses a state machine to track its setup and operational status through the `AppStatus` enum. This guide explains the different states and the flow between them.

## Status States

### 1. Setup (`setup`)
- Initial state when app is first launched
- User must choose between:
  - Authenticated mode (`authz: true`)
  - Non-authenticated mode (`authz: false`)

### 2. Resource Admin (`resource-admin`)
- Intermediate state when authenticated mode is chosen
- Requires admin user setup through login
- Only accessible when `authz: true`

### 3. Ready (`ready`)
- Final operational state
- Reached through two paths:
  - After admin setup in authenticated mode
  - Directly after choosing non-authenticated mode

## Authentication Modes

### Authenticated Mode (`authz: true`)
- Requires user authentication
- Enforces role-based access control
- Flow: `setup` → `resource-admin` → `ready`
- First user to log in becomes the admin
- Requires valid auth server configuration

### Non-Authenticated Mode (`authz: false`)
- No authentication required
- All endpoints are publicly accessible
- Flow: `setup` → `ready`
- Authorization headers are ignored
- Suitable for local development/testing

## State Transitions

```
                           ┌─────────────────┐
                           │                 │
                           │      Setup      │
                           │                 │
                           └────────┬────────┘
                                   │
                   ┌───────────────┴───────────────┐
                   │                               │
           authz: true                      authz: false
                   │                               │
                   ▼                               │
        ┌──────────────────┐                      │
        │                  │                      │
        │ Resource Admin   │                      │
        │                  │                      │
        └────────┬─────────┘                      │
                 │                                │
                 │         ┌────────────────┐     │
                 └────────►│                │◄────┘
                          │     Ready       │
                          │                │
                          └────────────────┘
```

## API Response Examples

### Setup State
```json
{
  "version": "0.1.0",
  "authz": false,
  "status": "setup"
}
```

### Resource Admin State
```json
{
  "version": "0.1.0",
  "authz": true,
  "status": "resource-admin"
}
```

### Ready State (Authenticated)
```json
{
  "version": "0.1.0",
  "authz": true,
  "status": "ready"
}
```

### Ready State (Non-Authenticated)
```json
{
  "version": "0.1.0",
  "authz": false,
  "status": "ready"
}
```

## Key Points
- Status is persisted across app restarts
- Status cannot be changed after initial setup
- Auth mode (`authz`) cannot be changed after setup
- Admin privileges are granted to first login in authenticated mode
- Non-authenticated mode skips all authorization checks 