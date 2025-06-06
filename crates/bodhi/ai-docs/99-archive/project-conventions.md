# Bodhi App Project Conventions

This document outlines the coding conventions, frameworks, tools, and architectural patterns used in the Bodhi App project. It serves as a guide for maintaining consistency across the codebase.

## Project Structure

### Crate Organization
```
crates/
├── objs/          # contains the application common objects
├── bodhi/          # Frontend application
├── services/       # Backend services
└── ...modules/         # Other modules
```

## Backend Conventions

### Database Layer

#### Migration Files
- Location: `crates/services/migrations/`
- Naming: `NNNN_descriptive_name.{up,down}.sql`
- Format: Plain SQL with descriptive comments
- Always include both up and down migrations

#### Database Models
- Location: `crates/services/src/db/objs.rs`
- Conventions:
  ```rust
  #[derive(Debug, Clone, PartialEq)]
  pub struct ModelName {
      pub id: String,          // UUID as string
      pub created_at: DateTime<Utc>,
      pub updated_at: DateTime<Utc>,
      // ... other fields
  }
  ```

#### Enums
- Use serde and strum for serialization
- Use kebab-case for string representations
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq)]
  #[serde(rename_all = "kebab-case")]
  #[strum(serialize_all = "kebab-case")]
  pub enum StatusType {
      Active,
      Inactive,
  }
  ```

### Service Layer

#### Trait Definitions
- Location: `crates/services/src/db/service.rs`
- Pattern:
  ```rust
  pub trait DbService: std::fmt::Debug + Send + Sync {
      async fn method_name(&self, param: Type) -> Result<ReturnType, DbError>;
  }
  ```

#### Service Implementation
- Use SQLx for database operations
- Prefer query_as over raw query! macro
- Use bind parameters for values
  ```rust
  query_as::<_, (String, String, DateTime<Utc>)>(
      "SELECT id, name, created_at FROM table WHERE status = ? LIMIT ? OFFSET ?"
  )
  .bind(status.to_string())
  .bind(limit)
  .bind(offset)
  ```

### Testing

#### Test Infrastructure
- Location: `crates/services/src/test_utils/`
- Use TestDbService for database tests
- Implement notification system for operation tracking

#### Test Patterns
```rust
#[rstest]
#[awt]
#[tokio::test]
async fn test_name(
    #[future]
    #[from(test_db_service)]
    service: TestDbService,
) -> anyhow::Result<()> {
    // Test implementation
}
```

#### Test Data
- Create fresh data in each test
- No shared test fixtures
- Use builder patterns where appropriate

## Frontend Conventions

### React Components

#### Component Organization
```typescript
// page.tsx
export default function PageName() {
    // Main page component
}

// components/ComponentName.tsx
export function ComponentName() {
    // Reusable component
}
```

#### State Management
- Use React hooks for local state
- Custom hooks for complex logic
- MSW for API mocking in tests

### Testing Strategy

#### Component Testing
- Use React Testing Library
- Test user interactions
- Mock API calls with MSW
- Visual regression testing for UI components

#### Integration Testing
- End-to-end flows
- API integration tests
- Error handling scenarios

## API Conventions

### Endpoint Structure
- Base path: `/api/v1`
- Resource-based routing
- Pagination parameters: `page` and `per_page`
- Status codes:
  - 200: Success
  - 201: Created
  - 400: Bad Request
  - 401: Unauthorized
  - 404: Not Found

### Authentication
- Bearer token authentication
- Token validation in auth_middleware
- Cache token status for performance
- Clear error messages for auth failures

## Error Handling

### Backend Errors
```rust
#[derive(Debug, Error)]
pub enum DbError {
    #[error("specific error message: {0}")]
    SpecificError(String),
    // ... other variants
}
```

### Frontend Errors
- Toast notifications for user feedback
- Error boundaries for component errors
- Consistent error message format

## Documentation

### Code Documentation
- Clear function/method documentation
- Example usage in complex cases
- Architecture decision records in `ai-docs/`

### User Documentation
- API endpoint documentation
- UI component documentation
- Deployment guides

## Monitoring and Logging

### Backend Logging
- Use tracing for structured logging
- Log levels: ERROR, WARN, INFO, DEBUG
- Include context in log messages

### Frontend Logging
- Console errors for development
- Error tracking in production
- Performance monitoring

## Development Workflow

### Version Control
- Feature branches
- Descriptive commit messages
- PR templates
- Code review guidelines

### CI/CD
- Automated tests
- Linting
- Build verification
- Deployment pipelines

This document should be updated as new patterns and conventions are established in the project.
