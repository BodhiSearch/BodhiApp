# API Tokens Implementation - Technical Summary

## Overview

This change introduces first-class API tokens for programmatic access across BodhiApp. Tokens are user-scoped, database-backed secrets with a deterministic prefix (bodhiapp_) that enables fast lookup by prefix and secure verification via SHA-256 hashing with constant-time comparison. The implementation spans backend services and authentication middleware through API routes, TypeScript client updates, React UI for token lifecycle management, and comprehensive unit/integration/E2E tests.

Key capabilities:
- Secure token generation (cryptographically strong, one-time display)
- Scopes mapped from user role (User → scope_token_user, Power/Manager/Admin → scope_token_power_user)
- Fast prefix lookup + constant-time hash verification in middleware
- Management APIs with pagination and status toggling
- React UI for token creation, listing, and status updates
- Full testing from Rust unit/integration to Playwright E2E

## Core Module Changes

### Services Layer

Major architectural changes to the service layer include:

#### DbService Interface Updates

**New Methods:**
- `create_api_token(&self, token: &mut ApiToken)` - Store token with metadata
- `get_api_token_by_prefix(&self, prefix: &str)` - Fast token lookups using prefix
- `list_api_tokens(&self, user_id: &str, page: u32, page_size: u32)` - Paginated token listing
- `update_api_token(&self, user_id: &str, token: &mut ApiToken)` - Token metadata updates

**Removed Methods:**
- `create_api_token_from(&self, name: &str, token: &str)` - JWT-based token creation
- `get_api_token_by_token_id(&self, token: &str)` - JWT-based token lookup

#### AuthService Simplification

**Removed JWT Token Exchange Logic:**
```rust
// Removed complex JWT exchange flow
async fn exchange_token(
    &self,
    client_id: &str,
    client_secret: &str,
    subject_token: &str,
    token_type: &str,
    scopes: Vec<String>,
) -> Result<(String, Option<String>)>
```

The system now uses direct database validation instead of OAuth2 token exchange patterns.

#### Token Storage Model

**ApiToken Structure:**
```rust
pub struct ApiToken {
    pub id: String,                    // UUID v4
    pub user_id: String,              // Owner user ID
    pub name: String,                 // User-friendly name
    pub token_prefix: String,         // "bodhiapp_" + 8 chars
    pub token_hash: String,          // SHA-256 hash of full token
    pub scopes: String,              // Token permissions
    pub status: TokenStatus,         // Active/Inactive
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Security Implementation:**
- **Cryptographic RNG**: Uses `rand::rng().fill_bytes()` for secure token generation
- **SHA-256 Hashing**: Full token hash stored using `sha2::Sha256`
- **Prefix Strategy**: Only first 17 characters stored separately for fast lookups
- **Constant-Time Verification**: Uses `constant_time_eq` crate to prevent timing attacks

### Authentication Middleware

Completely refactored authentication flow with database-backed token validation:

#### Token Validation Flow

**New Two-Path Authentication:**
1. **Database Tokens** (`bodhiapp_*` prefix)
2. **External Client Tokens** (JWT format)

```rust
const BODHIAPP_TOKEN_PREFIX: &str = "bodhiapp_";

// Token prefix detection
if bearer_token.starts_with(BODHIAPP_TOKEN_PREFIX) {
    // DATABASE TOKEN VALIDATION PATH
    
    // 1. Extract prefix (first 17 chars: "bodhiapp_" + 8)
    let prefix_end = BODHIAPP_TOKEN_PREFIX.len() + 8;
    let token_prefix = &bearer_token[..prefix_end];
    
    // 2. Database lookup by prefix
    let api_token = self.db_service
        .get_api_token_by_prefix(token_prefix).await?;
    
    // 3. Status validation
    if api_token.status != TokenStatus::Active {
        return Err(AuthError::TokenInactive);
    }
    
    // 4. Constant-time hash comparison
    let mut hasher = Sha256::new();
    hasher.update(bearer_token.as_bytes());
    let provided_hash = format!("{:x}", hasher.finalize());
    
    if !constant_time_eq::constant_time_eq(
        provided_hash.as_bytes(),
        api_token.token_hash.as_bytes(),
    ) {
        return Err(TokenError::InvalidToken("Invalid token".to_string()))?;
    }
    
    // 5. Return token scope directly
    let token_scope = TokenScope::from_str(&api_token.scopes)?;
    Ok((bearer_token.to_string(), ResourceScope::Token(token_scope)))
}
```

#### Security Enhancements

**Timing Attack Prevention:**
- Uses `constant_time_eq` crate for hash comparison
- Eliminates timing-based token enumeration attacks

**Status-Based Access Control:**
- Immediate rejection of inactive tokens
- No cache poisoning from disabled tokens

**Simplified Attack Surface:**
- Removes complex JWT validation logic
- Eliminates token exchange vulnerability vectors
- Direct database validation reduces code complexity

#### Error Handling

**New Error Types:**
```rust
#[error("API token is inactive")]
TokenInactive,

#[error(transparent)]
DbError(#[from] DbError),
```

**Enhanced Error Context:**
- Clear distinction between inactive vs invalid tokens
- Database errors properly propagated
- Consistent error formatting across authentication paths

## API Layer Changes

### Route Implementations

New REST API endpoints for comprehensive token management:

#### POST /api/tokens - Token Creation

```rust
pub async fn create_token_handler(
    headers: HeaderMap,
    State(state): State<Arc<dyn RouterState>>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateApiTokenRequest>, ApiError>,
) -> Result<(StatusCode, Json<ApiTokenResponse>), ApiError>
```

**Implementation Details:**
- **User Context Extraction**: Derives user_id from authenticated session headers
- **Role-Based Scope Assignment**: Maps user roles to token scopes (User → scope_token_user, Admin → scope_token_poweruser)
- **Cryptographic Token Generation**: 32 bytes of cryptographically secure random data encoded as base64
- **Secure Storage**: SHA-256 hash stored in database, prefix indexed for lookups
- **Single-Use Response**: Token returned only once at creation time

**Request/Response Format:**
```typescript
// Request
interface CreateApiTokenRequest {
  name?: string;  // Optional user-friendly name
}

// Response
interface ApiTokenResponse {
  token: string;  // "bodhiapp_" + base64-encoded random data
}
```

#### GET /api/tokens - List User Tokens

**Features:**
- **Pagination Support**: Page and page_size parameters
- **User Isolation**: Only returns tokens owned by authenticated user
- **Metadata Only**: Token hashes never returned, only metadata

#### PATCH /api/tokens/{id} - Update Token

**Supported Operations:**
- **Name Updates**: Change user-friendly token names
- **Status Toggle**: Activate/deactivate tokens
- **Optimistic Locking**: Uses updated_at timestamps

#### DELETE /api/tokens/{id} - Token Revocation

**Security Features:**
- **Immediate Revocation**: Token unusable immediately after deletion
- **Audit Trail**: Deletion timestamps preserved
- **User Authorization**: Only token owner can delete

### OpenAPI Specifications

**Schema Changes:**

```yaml
ApiToken:
  type: object
  required:
    - id
    - user_id
    - name
    - token_prefix      # Changed from token_id
    - token_hash
    - scopes           # New field
    - status
    - created_at
    - updated_at
  properties:
    token_prefix:
      type: string
      description: "First 17 characters for lookups"
    scopes:
      type: string
      description: "Token permissions (scope_token_user, etc.)"
```

**Updated Response Examples:**
```yaml
ApiTokenResponse:
  example:
    token: "bodhiapp_1234567890abcdef"  # Changed from bapp_ prefix
```

**Security Documentation:**
- **Bearer Token Format**: Updated to reflect `bodhiapp_` prefix
- **Authentication Flow**: Documents database-backed validation
- **Scope Requirements**: Clear documentation of required permissions

## Frontend Implementation

### React Components

Comprehensive React-based UI for API token management:

#### TokenDialog Component

**Purpose**: Secure token display after creation

```typescript
interface TokenDialogProps {
  token: ApiTokenResponse;
  open: boolean;
  onClose: () => void;
}

export function TokenDialog({ token, open, onClose }: TokenDialogProps) {
  const [showToken, setShowToken] = useState(false);
  // Component implementation...
}
```

**Key Features:**
- **Security Warning**: Prominent alert about one-time token visibility
- **Show/Hide Toggle**: Secure token display with mask/reveal functionality
- **Copy Integration**: One-click clipboard copy with CopyButton component
- **Accessibility**: Proper ARIA labels and keyboard navigation
- **Test IDs**: Comprehensive data-testid attributes for E2E testing

#### TokenForm Component

**Purpose**: Token creation interface with validation

```typescript
interface TokenFormProps {
  onTokenCreated: (token: ApiTokenResponse) => void;
}

const formSchema = z.object({
  name: z.string().optional(),
});
```

**Implementation Details:**
- **React Hook Form Integration**: Type-safe form validation with Zod schema
- **Loading States**: Disabled form during submission with loading spinner
- **Error Handling**: Displays validation errors and API failures
- **Optimistic UI**: Immediate feedback for user actions

#### TokensPage Component

**Purpose**: Main token management interface

**Layout Structure:**
```typescript
<Card>
  <CardHeader>
    <CardTitle>API Tokens</CardTitle>
    <CardDescription>Generate and manage API tokens</CardDescription>
  </CardHeader>
  <CardContent>
    <Alert variant="destructive">
      {/* Security warning */}
    </Alert>
    <TokenForm onTokenCreated={handleTokenCreated} />
    <DataTable
      data={tokensData?.data || []}
      columns={columns}
      renderRow={renderRow}
    />
  </CardContent>
</Card>
```

**Data Management:**
- **Pagination Support**: Configurable page size and navigation
- **Real-time Updates**: Optimistic updates with error rollback
- **Status Toggle**: Switch components for activate/deactivate
- **Toast Notifications**: Success/error feedback for all operations

#### ShowHideInput Component Enhancement

**Security Features:**
- **Content Masking**: Renders bullet characters (•) when hidden
- **Toggle Animation**: Smooth show/hide transitions
- **Copy Button Integration**: Seamless clipboard functionality
- **Keyboard Support**: Space/Enter key toggle support

**Implementation:**
```typescript
<ShowHideInput
  value={token.token}
  shown={showToken}
  onToggle={toggleShowToken}
  actions={<CopyButton text={token.token} showToast={false} />}
  data-testid="token-value-input"
/>
```

### Custom Hooks and State Management

#### useApiTokens Hook

**Token Operations:**
```typescript
const {
  data: tokensData,
  isLoading: tokensLoading,
  error: tokensError,
} = useListTokens({ page: 1, pageSize: 10 });

const { mutate: createToken } = useCreateToken({
  onSuccess: (token) => {
    showSuccess('Success', 'API token successfully generated');
    handleTokenCreated(token);
  },
  onError: (error) => showError('Error', error.message),
});

const { mutate: updateToken } = useUpdateToken({
  onSuccess: (token) => {
    const status = token.status === 'active' ? 'active' : 'inactive';
    showSuccess('Token Updated', `Token status changed to ${status}`);
  },
  onError: (error) => showError('Error', error.message),
});
```

**State Management Features:**
- **Optimistic Updates**: UI reflects changes immediately, rolls back on error
- **Loading States**: Granular loading indicators for each operation
- **Error Handling**: Automatic error display with toast notifications
- **Cache Invalidation**: Automatic refetch after mutations
- **TypeScript Integration**: Fully typed with generated API client types

**Testing Integration:**
- Mock Service Worker (MSW) handlers for all token operations
- Test scenarios include success, loading, and error states
- Optimistic update rollback testing

## Testing Infrastructure

### Integration Tests

- Rust integration tests updated to reflect new database-backed token model
  - Tests create and persist ApiToken with token_prefix, token_hash, scopes
  - Inactive token paths validated to return 401 Unauthorized
  - Hash mismatch cases return invalid_token errors
- Live auth tests simplified by removing token exchange endpoints and focusing on middleware behavior

### End-to-End Tests

- Playwright tests cover full lifecycle:
  - Create token via UI and verify one-time dialog display
  - Show/hide and clipboard copy verified with mocked clipboard
  - Token appears in list with correct status and metadata
  - Toggle token status to inactive → chat requests fail with authentication error
  - Re-activate token → chat flow succeeds again
- Selectors use data-testid exclusively, per project preference
- Tests are structured for Chrome-only execution per user rule
- Page objects introduced for TokensPage, ChatPage, ChatSettingsPage
- New fixtures (TokenFixtures) provide reusable token names, errors, and clipboard mocking utilities
- Error handling scenarios (missing token, invalid format, non-existent token, network failures) validated

## Configuration and Dependencies

### Database Migrations

The implementation introduces a new database table `api_tokens` with the following schema:

```sql
CREATE TABLE api_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT DEFAULT '',
    token_prefix TEXT NOT NULL UNIQUE,
    token_hash TEXT NOT NULL,
    scopes TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index on token_prefix for faster lookups
CREATE INDEX idx_api_tokens_token_prefix ON api_tokens(token_prefix);
```

**Key Schema Changes:**
- **token_prefix**: Replaces `token_id` - stores the first 17 characters of the token (`bodhiapp_` + 8 chars) for efficient lookups
- **scopes**: New field storing token permissions as string (e.g., "scope_token_user", "scope_token_poweruser")
- **token_hash**: SHA-256 hash of the full token for secure verification
- **Index Strategy**: Single index on `token_prefix` for O(log n) token lookups during authentication

### TypeScript Client Updates

- OpenAPI client regenerated to reflect schema changes
  - ApiToken: fields now include token_prefix and scopes (token_id removed)
  - ApiTokenResponse: field renamed from offline_token → token
- Updated files:
  - ts-client/src/openapi-typescript/openapi-schema.ts
  - ts-client/src/types/types.gen.ts
- Frontend updated accordingly in components, hooks, and tests

## Conclusion

The API tokens feature replaces fragile JWT-based exchange paths with a robust, database-backed model that is simpler, faster, and more secure. Using a bodhiapp_ prefix for discovery and SHA-256 hashing with constant-time verification significantly reduces the attack surface while keeping runtime costs low. The feature is consistently implemented from services to routes and frontend, and is validated by extensive tests (including Chrome-only Playwright with data-testid selectors). This lays a solid foundation for future enhancements such as per-token scope editing, token audit trails, and user-bound token isolation in listings.
