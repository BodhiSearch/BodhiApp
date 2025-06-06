# Authentication & Authorization

## Overview
The Bodhi App provides flexible authentication options during setup. Users can either run the app without authentication for personal use or configure it as an OAuth2 resource server for multi-user environments. Authentication is handled by an external OAuth2 server.

## Setup Options

### 1. No Authentication Mode
```
Features:
├── Single user mode
├── No login required
├── Direct access to all features
└── Local configuration only
```

### 2. OAuth2 Resource Mode
```
Features:
├── Multi-user support
├── Role-based access control
├── External authentication
└── Secure API access
```

## Authentication Flow

### 1. Initial Setup
```
App Start
   ↓
Setup Check
   ↓
Choose Mode:
├── No Auth → Direct Access
└── OAuth2 → Resource Setup
```

### 2. OAuth2 Configuration
```
Resource Setup
   ↓
OAuth2 Server Config
   ↓
Role Configuration
   ↓
System Initialization
```

## Implementation Details

### 1. App States
```typescript
type AppStatus = 'setup' | 'ready' | 'resource-admin';
```

- **setup**: Initial configuration required
- **resource-admin**: OAuth2 resource setup needed
- **ready**: App ready for use

### 2. Authorization Flow
```
Request → AppInitializer → Status Check → Route Guard
```

### 3. Protected Routes
- Status-based routing
- Role-based access control
- Authentication state validation

## Security Features

### 1. OAuth2 Integration
- External authentication server
- Token validation
- Secure session management
- Refresh token handling

### 2. Role-Based Access Control
```
Roles:
├── User
│   └── Basic app access
├── Manager
│   └── User management
└── Admin
    └── System configuration
```

### 3. API Security
- JWT-based authentication
- Role-based permissions
- Token validation
- Request signing

## Configuration

### 1. No Auth Setup
```typescript
{
  authz: false,
  status: 'ready'
}
```

### 2. OAuth2 Setup
```typescript
{
  authz: true,
  oauth2: {
    server_url: string,
    client_id: string,
    client_secret: string
  }
}
```

## Route Protection

### 1. Route Guards
- Status validation
- Authentication check
- Role verification
- Redirect handling

### 2. Protected Resources
- API endpoints
- UI routes
- System settings
- User management

## Implementation Components

### 1. AppInitializer
- Status checking
- Route protection
- Authentication validation
- Redirect handling

### 2. Setup Flow
- Mode selection
- OAuth2 configuration
- Role setup
- System initialization

## Error Handling

### 1. Setup Errors
- Configuration validation
- Connection testing
- Resource validation

### 2. Authentication Errors
- Token validation
- Permission denied
- Session expiration
- Server errors

## Future Enhancements

### 1. Authentication Features
- Custom OAuth2 providers
- Additional auth methods
- Enhanced session management
- Advanced token handling

### 2. Authorization Features
- Custom role definitions
- Fine-grained permissions
- Resource-level access
- Audit logging

### 3. Security Enhancements
- Certificate pinning
- Enhanced encryption
- Security monitoring
- Threat detection
