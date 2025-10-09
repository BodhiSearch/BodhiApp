---
title: 'API Tokens'
description: 'Create and manage database-backed API tokens for secure programmatic access'
order: 240
---

# API Token Management

Create and manage database-backed API tokens for programmatic access to Bodhi App. Tokens support scope-based permissions and use cryptographic security for safe integration with external applications.

## Overview

API tokens provide persistent programmatic access to Bodhi App's API endpoints without requiring interactive browser sessions. Unlike session-based authentication (browser cookies), API tokens are designed for automated systems, scripts, CI/CD pipelines, and external integrations.

**Key Features**:

- Database-backed persistence with cryptographic security
- SHA-256 hashing for secure storage
- Scope-based permissions for fine-grained access control
- One-time display security pattern
- Active/inactive status toggle
- User-linked lifecycle (tokens invalidated when user removed)

## How API Tokens Work

Bodhi App uses a secure database-backed token architecture:

**Token Lifecycle**:

1. **Creation**: User creates token via UI with name and selected scopes
2. **Generation**: Bodhi App generates cryptographically secure token string
3. **One-Time Display**: Token displayed ONCE in modal (must copy now)
4. **Hashing**: Token hashed with SHA-256 before database storage
5. **Authentication**: Future API requests include token in Authorization header
6. **Validation**: Bodhi App hashes provided token and compares to stored hash
7. **Authorization**: Scope-based permissions checked against request endpoint

**Security Model**:

- Original token value NEVER stored (only SHA-256 hash)
- Token cannot be retrieved after creation (one-time display)
- Tokens can be activated/deactivated without deletion
- Scope restrictions limit token capabilities
- User removal automatically invalidates all user's tokens

**Architecture**:

```
User creates token → Generate secure string → Display ONCE → Hash with SHA-256 → Store hash in database
                                                     ↓
API request with token → Hash provided token → Compare hashes → Check scopes → Allow/deny
```

## Creating API Tokens

<img
  src="/doc-images/api-tokens.jpg"
  alt="API Token Creation Modal"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

### Step 1: Navigate to Tokens Page

**From Navigation**:

- Click **Settings** in sidebar
- Click **Tokens** in submenu
- Or navigate directly to: <a href="http://localhost:1135/ui/tokens/" target="_blank" rel="noopener noreferrer">http://localhost:1135/ui/tokens/</a>

### Step 2: Click New API Token

Click the **New API Token** button to open the creation dialog.

### Step 3: Enter Token Name (Optional)

Token name is optional but recommended for identifying tokens later. Provide a descriptive name to identify the token's purpose.

**Name Examples**:

- `CI/CD Pipeline - GitHub Actions`
- `Mobile App - Production`
- `Data Analytics Script`
- `Integration Test Suite`

**Naming Guidelines**:

- Choose a descriptive name to identify this token
- Include environment if applicable (prod/staging/dev)
- Examples: "Production API", "Development Access", "CI/CD Pipeline"

### Step 4: Select Scopes

Choose permissions granted to this token. Each scope enables access to specific API capabilities.

**Available Scopes**:

Currently, Bodhi App supports two token scopes:

1. **Token User** (Read-Only Access):

   - Chat completions API (`/bodhi/v1/chat/completions`)
   - Embeddings API (`/bodhi/v1/embeddings`)
   - Model listing (read-only access to `/bodhi/v1/models`)
   - Cannot download or delete models
   - Ideal for: Chat applications, embedding generation, model discovery

2. **Token Power User** (Limited Write Access):
   - All Token User capabilities
   - Download models from HuggingFace
   - Delete existing models
   - **Note**: Token Power User has more restrictions than a logged-in PowerUser role
   - Ideal for: Automated model management, CI/CD pipelines

**Important**: Token-based access has additional restrictions compared to logged-in user sessions, even with the same scope name.

> **Note on Naming**: Token scopes ("Token User" and "Token Power User") have similar names to user roles ("User" and "PowerUser") but represent different authorization systems. Token scopes control API access for programmatic clients, while user roles control UI and session-based access. A logged-in PowerUser has broader capabilities than a Token Power User scope.

**Scope Selection Tips**:

- Select minimum required scopes (principle of least privilege)
- **Scopes cannot be changed after creation** - choose carefully
- To change scope: Create new token with desired scope and deactivate old token
- Token limited to selected scopes only

### Step 5: Generate Token

Click **Generate Token** button to create the token.

### Step 6: Copy Token (CRITICAL - One-Time Display)

**WARNING: Token Shown Only Once**

The token is displayed ONLY ONCE after creation. You MUST copy it now.

**Critical Steps**:

1. Token appears in modal dialog
2. Click **Copy** button to copy to clipboard
3. Store token securely immediately:
   - Password manager
   - Environment variables
   - Encrypted configuration file
   - Secret management service
4. Click **Close** or **Done** to dismiss dialog
5. **Token hidden forever** - cannot be retrieved later

**If You Lose the Token**:

- Tokens cannot be retrieved after closing dialog
- Create new token with same configuration
- Deactivate lost token
- Update applications with new token

**Security Warning**:

- Token grants API access to your Bodhi App instance
- Treat like password - never share
- Never commit to version control
- Never log in plaintext
- Store in secure location

## Using API Tokens

### Authorization Header Format

Include token in `Authorization` header as Bearer token:

```bash
curl http://localhost:1135/bodhi/v1/chat/completions \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "model-alias",
    "messages": [{"role": "user", "content": "Hello, how are you?"}]
  }'
```

**Header Format**: `Authorization: Bearer <token>`

**Base URL**: `http://localhost:1135/bodhi/v1` (adjust host/port as needed)

### API Endpoints Supporting Token Authentication

API tokens work with all Bodhi App API endpoints based on their scope:

**Token User Scope**:

- `/bodhi/v1/chat/completions` - Chat completions
- `/bodhi/v1/models` - List models (read-only)
- `/bodhi/v1/embeddings` - Generate embeddings

**Token Power User Scope**:

- All Token User endpoints
- Model download endpoints
- Model deletion endpoints

See [OpenAPI Documentation](/docs/features/openapi-docs) for complete endpoint reference.

## Managing API Tokens

### Viewing Tokens

The Tokens page displays all tokens created by the current user in a table.

**Table Columns**:

- **Name**: Token identifier you provided
- **Scope**: Granted permissions (Token User or Token Power User)
- **Status**: Active (green badge) or Inactive (gray badge) with toggle switch to activate/deactivate
- **Created At**: Creation timestamp
- **Updated At**: Last modification timestamp

**Token Values Not Shown**:

- Token values are NEVER displayed after creation
- Only token metadata visible
- Cannot retrieve original token value

### Activating/Deactivating Tokens

Toggle token status without deleting it.

**Steps**:

1. Locate token in tokens table
2. Click toggle switch in Status column
3. Status changes immediately:
   - **Active** (green badge): Token can authenticate API requests
   - **Inactive** (gray badge): Token cannot authenticate
4. API requests using inactive tokens fail immediately with authentication error

**When to Deactivate**:

- Temporarily disable access without deletion
- Rotate tokens (deactivate old, create new)
- Emergency access revocation
- Testing token validation logic

**Reactivation**:

- Toggle switch again to reactivate
- Token resumes working immediately
- Same token value (hash unchanged)

### Token Lifecycle and Expiration

**Token Expiration**:

- API tokens currently **do not expire**
- They remain valid indefinitely until deactivated or deleted
- **Future Feature**: Time-based expiration is planned for future releases

**Token States**:

- **Created**: Tokens are created in **Active** status and ready for immediate use
- **Active**: Can authenticate API requests
- **Inactive**: Cannot authenticate - returns authentication error
- No automatic expiration or deactivation based on inactivity

**User Removal Impact**:

- User deletion invalidates ALL user's tokens
- Tokens automatically deactivated
- Cannot reactivate after user removal

### Token Deletion

**Current Status**:

- Delete button is not currently available in the UI
- The API endpoint for token deletion exists
- UI support for deletion is planned for future releases

**Workaround**:

- Deactivate tokens you no longer need
- Inactive tokens cannot authenticate API requests
- This provides similar security benefits to deletion

## Security Best Practices

### Secure Token Storage

✅ **Recommended Storage Methods**:

- **Password Managers**: 1Password, Bitwarden, LastPass
- **Environment Variables**: `.env` files (excluded from Git)
- **Secret Management Services**: HashiCorp Vault, AWS Secrets Manager, Azure Key Vault
- **Encrypted Configuration**: Files encrypted at rest

❌ **Never Store Tokens**:

- In Git repositories (public or private)
- In application code
- In plaintext logs
- In browser localStorage (use session-based auth for browsers)
- In shared documents or wikis

### Token Rotation Strategy

**Rotation Frequency**:

- No specific rotation frequency is recommended by Bodhi App
- Rotate tokens based on your organization's security policies and requirements
- Consider your risk tolerance and compliance needs

**Rotation Process**:

1. Create new token with same scopes
2. Update applications to use new token
3. Test new token works correctly
4. Deactivate old token
5. Keep deactivated token for audit trail or delete if UI available

**When to Rotate**:

- Regular schedule (quarterly, annually)
- After team member departure
- If token possibly exposed
- Security audit recommendations

### Scope Minimization

Grant only required scopes using principle of least privilege.

**Examples**:

**Read-Only Application**:

```
Scope: Token User
Use Case: Chat application, model discovery dashboard, embedding generation
Capabilities: Read-only access to models, chat completions, embeddings
```

**Model Management Application**:

```
Scope: Token Power User
Use Case: CI/CD pipeline, automated model management
Capabilities: All Token User capabilities + model download/deletion
```

**Principle of Least Privilege**:

- Use **Token User** for applications that only need to query models
- Use **Token Power User** only when model management is required
- Never use Token Power User scope for pure chat or embedding applications

### Monitoring Token Usage

**Security Monitoring**:

- Review token list regularly
- Deactivate unused tokens
- Rotate tokens on schedule
- Monitor application logs for authentication failures

## Troubleshooting

### Token Authentication Fails

**Symptoms**: API requests return authentication error

**Common Error Messages**:

- **"Invalid authentication token"**: Token is not recognized or formatted incorrectly
- **"Inactive token"**: Token exists but has been deactivated
- **"Unauthorized"**: Token is valid but lacks required permissions

**Possible Causes & Solutions**:

1. **Token Copied Incorrectly**:

   - Verify no extra spaces before/after token
   - Check entire token copied (no truncation)
   - Try copying again from secure storage

2. **Token Inactive**:

   - Check token status in Tokens page
   - Reactivate if accidentally deactivated
   - Create new token if intentionally deactivated

3. **Wrong Authorization Header Format**:

   - Correct: `Authorization: Bearer YOUR_TOKEN`
   - Incorrect: `Authorization: YOUR_TOKEN` (missing "Bearer ")
   - Incorrect: `Bearer YOUR_TOKEN` (missing "Authorization:" header)

4. **Token Expiration**:

   - Tokens currently do not expire automatically
   - If authentication fails, check token is still active in UI

5. **User Removed**:

   - Token invalidated when user deleted
   - Create new token with active user account

6. **Insufficient Permissions**:
   - Token scope does not allow requested operation
   - Check token scope in tokens list
   - Create new token with appropriate scope

### Token Lost After Creation

**Symptoms**: Forgot to copy token from one-time display

**Solution**:

- Tokens CANNOT be retrieved after creation
- Create new token:
  1. Navigate to Tokens page
  2. Click **New API Token**
  3. Enter same name (or append "-v2")
  4. Select same scopes
  5. **Copy token immediately this time**
- Deactivate lost token:
  1. Find original token in table
  2. Toggle to Inactive
- Update applications with new token

**Prevention**:

- Always copy token immediately
- Store in password manager before closing dialog
- Test token works before closing dialog

### Scope Permission Denied

**Symptoms**: API returns permission denied error with valid token

**Common Error Messages**:

- **"Insufficient permissions for this operation"**: Token scope does not allow this action
- **"Unauthorized"**: Token valid but lacks required scope

**Cause**: Token lacks required scope for API endpoint

**Solutions**:

1. **Check Token Scopes**:

   - View token scopes in the tokens list UI
   - Scopes are displayed in the table for easy identification
   - Verify required scope for endpoint

2. **Create New Token with Correct Scopes**:

   - **Scopes cannot be modified after token creation**
   - Create new token with required scopes
   - Update application configuration with new token
   - Deactivate old token

3. **Verify Endpoint Requirements**:
   - See [OpenAPI Documentation](/docs/features/openapi-docs) for scope requirements
   - Token User: Read-only endpoints (chat, embeddings, models list)
   - Token Power User: Model management endpoints (download, delete)

## Related Documentation

- [OpenAPI Reference](/docs/features/openapi-docs) - API endpoint documentation
- [TypeScript Client](/docs/developer/typescript-client) - SDK usage guide
- [Authentication](/docs/intro#authentication) - Session-based authentication
- [User Management](/docs/features/user-management) - Managing users and roles

---

**Summary**: Bodhi App's database-backed API tokens provide secure programmatic access with scope-based permissions. Use one-time display pattern, store tokens securely, apply principle of least privilege, and rotate regularly for optimal security.

Happy integrating!
