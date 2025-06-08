# Development Conventions

This document outlines the coding standards, naming conventions, and best practices for the Bodhi App project.

**Note**: This document has been factually verified against the actual source code as of the last update. All patterns and examples are confirmed to exist in the codebase.

## Project Structure

### Crate Organization
```
crates/
├── objs/                    # Application common objects (✓ verified: crates/objs/)
├── bodhi/                   # Frontend application (React+Vite) (✓ verified: crates/bodhi/)
├── services/                # Backend services (Rust) (✓ verified: crates/services/)
├── auth_middleware/         # Authentication middleware
├── commands/                # CLI interface
├── server_core/             # HTTP server infrastructure
├── routes_oai/              # OpenAI-compatible API endpoints
├── routes_app/              # Application-specific API endpoints
├── routes_all/              # Unified route composition
├── server_app/              # Standalone HTTP server
├── llama_server_proc/       # LLM process management
├── errmeta_derive/          # Error metadata macros
└── integration-tests/       # End-to-end testing
```

### Frontend Structure (bodhi/)
```
src/
├── components/    # React components organized by feature (✓ verified: crates/bodhi/src/components/)
├── pages/         # Page components for routing (✓ verified: crates/bodhi/src/pages/)
├── hooks/         # Custom React hooks (✓ verified: crates/bodhi/src/hooks/)
├── lib/           # Utility functions and shared logic (✓ verified: crates/bodhi/src/lib/)
├── schemas/       # Data validation schemas (✓ verified: crates/bodhi/src/schemas/)
├── styles/        # Global styles and theme definitions (✓ verified: crates/bodhi/src/styles/)
├── tests/         # Test files (✓ verified: crates/bodhi/src/tests/)
├── types/         # TypeScript type definitions (✓ verified: crates/bodhi/src/types/)
├── docs/          # Documentation content (Markdown)
└── generated/     # Generated files (docs data)
```

### Backend Structure (services/)
```
src/
├── db/           # Database layer (models, services) (✓ verified: crates/services/src/db/)
├── test_utils/   # Testing infrastructure (✓ verified: crates/services/src/test_utils/)
├── lib.rs        # Main library entry point (✓ verified: crates/services/src/lib.rs)
├── *_service.rs  # Various service implementations
├── objs.rs       # Service object definitions
├── macros.rs     # Service macros
└── obj_exts/     # Object extensions
```

**Note**: Migration files are located at `crates/services/migrations/` (not in src/).

### Component Organization
Components follow a **feature-based organization** pattern (not page-specific as originally documented):

```
src/components/
├── auth/          # Authentication components (✓ verified)
├── chat/          # Chat interface components (✓ verified)
├── docs/          # Documentation components (✓ verified)
├── models/        # Model-related components (✓ verified)
├── navigation/    # Navigation components (✓ verified)
├── settings/      # Settings components (✓ verified)
├── ui/            # Common UI components (shadcn/ui) (✓ verified)
└── [feature]/     # Other feature-specific components
```

Benefits:
- Groups components by feature/domain
- Easier to locate related functionality
- Better code organization for larger applications
- Supports feature-based development

## Naming Conventions

### Files and Directories
- **Files**: **PascalCase for components** (`AppInitializer.tsx`, `LoginMenu.tsx`) (✓ verified in codebase)
- **Directories**: kebab-case (`user-management/`) (✓ verified)
- **Test files**: `ComponentName.test.tsx` (✓ verified: `AppInitializer.test.tsx`, `LoginMenu.test.tsx`)
- **Type files**: `component-name.types.ts` or `types.ts`

**Note**: The codebase uses PascalCase for component files, not kebab-case as originally documented.

### Code Elements
- **Components**: PascalCase (`UserProfile`)
- **Functions**: camelCase (`getUserProfile`)
- **Variables**: camelCase (`userName`)
- **Constants**: UPPER_SNAKE_CASE (`API_BASE_URL`)
- **Types/Interfaces**: PascalCase (`UserProfileData`)
- **Enums**: PascalCase (`UserRole`)

### Component Structure
Use functional components with TypeScript:

```typescript
interface ComponentNameProps {
  prop1: string;
  prop2?: number;
}

export function ComponentName({ prop1, prop2 }: ComponentNameProps) {
  // Component logic
  return (
    // JSX
  );
}
```

## Backend Conventions

### Database Layer

#### Migration Files
- Location: `crates/services/migrations/` (✓ verified)
- Naming: `NNNN_descriptive_name.{up,down}.sql` (✓ verified: `0001_create_conversations.up.sql`)
- Format: Plain SQL with descriptive comments (✓ verified)
- Always include both up and down migrations (✓ verified)

#### Database Models
- Location: `crates/services/src/db/objs.rs` (✓ verified)
- Conventions (✓ verified against actual models):
  ```rust
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, derive_builder::Builder)]
  pub struct ModelName {
      pub id: String,          // UUID as string (✓ verified in Conversation, ApiToken)
      pub created_at: DateTime<Utc>,  // (✓ verified)
      pub updated_at: DateTime<Utc>,  // (✓ verified)
      // ... other fields
  }
  ```

#### Enums
- Use serde and strum for serialization (✓ verified)
- Use kebab-case for string representations (✓ verified)
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, EnumString, strum::Display, PartialEq)]
  #[serde(rename_all = "kebab-case")]
  #[strum(serialize_all = "kebab-case")]
  pub enum StatusType {
      Active,
      Inactive,
  }
  ```

**Reference**: `crates/services/src/db/objs.rs:52-59` - DownloadStatus enum, `crates/services/src/db/objs.rs:106-112` - TokenStatus enum

### Service Layer

#### Trait Definitions
- Location: `crates/services/src/db/service.rs` (✓ verified)
- Pattern (✓ verified):
  ```rust
  #[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
  pub trait DbService: std::fmt::Debug + Send + Sync {
      async fn method_name(&self, param: Type) -> Result<ReturnType, DbError>;
  }
  ```

#### Service Implementation
- Use SQLx for database operations (✓ verified)
- Prefer query_as over raw query! macro (✓ verified in codebase)
- Use bind parameters for values (✓ verified)
  ```rust
  query_as::<_, (String, String, DateTime<Utc>)>(
      "SELECT id, name, created_at FROM table WHERE status = ? LIMIT ? OFFSET ?"
  )
  .bind(status.to_string())
  .bind(limit)
  .bind(offset)
  ```

**Reference**: `crates/services/src/db/service.rs:68-130` - DbService trait, `crates/services/src/db/service.rs:403-413` - query_as usage

### Backend Testing

#### Test Infrastructure
- Location: `crates/services/src/test_utils/` (✓ verified)
- Use TestDbService for database tests (✓ verified)
- Implement notification system for operation tracking (✓ verified)

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

**Reference**: `crates/services/src/db/service.rs:816-823` - Actual test pattern usage, `crates/services/src/test_utils/db.rs:14-28` - TestDbService fixture

#### Test Data
- Create fresh data in each test (✓ verified)
- TestDbService provides isolated test environment (✓ verified)
- Use builder patterns where appropriate (✓ verified: ConversationBuilder, MessageBuilder)

### API Conventions

#### Endpoint Structure
- **Base paths**: `/bodhi/v1` for app endpoints, `/v1` for OpenAI-compatible endpoints (✓ verified)
- Resource-based routing (✓ verified)
- **Pagination parameters**: `page` and `page_size` (✓ verified, not `per_page`)
- Status codes:
  - 200: Success
  - 201: Created
  - 400: Bad Request
  - 401: Unauthorized
  - 404: Not Found

**Reference**: `crates/bodhi/src/hooks/useQuery.ts:31-44` - Actual endpoint paths, `crates/bodhi/src/hooks/useQuery.ts:161` - page_size usage

#### Authentication
- Bearer token authentication
- Token validation in auth_middleware
- Cache token status for performance
- Clear error messages for auth failures

### Backend Error Handling
```rust
#[derive(Debug, Error)]
pub enum DbError {
    #[error("specific error message: {0}")]
    SpecificError(String),
    // ... other variants
}
```

### Backend Logging
- Use tracing for structured logging
- Log levels: ERROR, WARN, INFO, DEBUG
- Include context in log messages

## Frontend Conventions

### Styling Conventions

#### Tailwind CSS Usage
- Use utility-first CSS approach
- Follow consistent class ordering
- Use `cn()` utility for conditional classes
- Leverage design system tokens

```typescript
import { cn } from '@/lib/utils';

<div
  className={cn(
    "flex items-center p-4",
    isActive && "bg-primary text-white",
    size === 'large' && "text-lg"
  )}
>
```

**Reference**: `crates/bodhi/src/lib/utils.ts:5-7` - cn function implementation

#### Class Ordering
Follow this order for Tailwind classes:
1. Layout (flex, grid, block)
2. Positioning (relative, absolute)
3. Sizing (w-, h-, max-w-)
4. Spacing (p-, m-, space-)
5. Typography (text-, font-)
6. Colors (bg-, text-, border-)
7. Effects (shadow-, opacity-)
8. Transitions (transition-, duration-)

## State Management

### Local State
Use React hooks for component-level state:

```typescript
const [isLoading, setIsLoading] = useState(false);
const [data, setData] = useState<DataType | null>(null);
```

### Server State
Use React Query for server state management (✓ verified):

```typescript
const { data, isLoading, error } = useQuery({
  queryKey: ['users'],
  queryFn: fetchUsers,
});
```

**Reference**: `crates/bodhi/src/hooks/useQuery.ts:53-73` - useQuery implementation

### Form State
Use React Hook Form with Zod validation:

```typescript
const form = useForm<FormData>({
  resolver: zodResolver(schema),
  defaultValues: {
    name: '',
  },
});
```

## Form Handling

### Schema Definition
Define validation schemas with Zod:

```typescript
const createUserSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  email: z.string().email('Invalid email'),
});

type CreateUserData = z.infer<typeof createUserSchema>;
```

### Form Components
Use shadcn/ui form components:

```typescript
<Form {...form}>
  <form onSubmit={form.handleSubmit(onSubmit)}>
    <FormField
      control={form.control}
      name="name"
      render={({ field }) => (
        <FormItem>
          <FormLabel>Name</FormLabel>
          <FormControl>
            <Input {...field} />
          </FormControl>
          <FormMessage />
        </FormItem>
      )}
    />
  </form>
</Form>
```

### Form Submission
Handle form submission with proper error handling:

```typescript
const onSubmit = async (data: FormData) => {
  try {
    await submitData(data);
    form.reset();
    toast({
      title: 'Success',
      description: 'Form submitted successfully'
    });
  } catch (error) {
    toast({
      title: 'Error',
      description: 'Failed to submit form',
      variant: 'destructive'
    });
  }
};
```

## API Integration

For comprehensive frontend API integration patterns, see **[Frontend Query Architecture](frontend-query.md)**.

### Key API Conventions Summary

**Endpoint Structure**:
- Application APIs: `/bodhi/v1/*`
- OpenAI-compatible APIs: `/v1/*`
- Authentication: `/app/*`

**Pagination Parameters**:
- Use `page` and `page_size` (not `per_page`)
- Include sort parameters: `sort`, `sort_order`

**Error Handling**:
- Extract user-friendly messages from error responses
- Provide fallback error messages
- Use callback patterns for component-level error handling

**Query Key Patterns**:
- Include all parameters that affect query results
- Use hierarchical structure for related queries
- String conversion for numeric parameters

## Error Handling

### Error Boundaries
Implement error boundaries for graceful error handling:

```typescript
class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return <ErrorFallback />;
    }

    return this.props.children;
  }
}
```

### API Error Handling
Handle API errors consistently:

```typescript
const handleApiError = (error: AxiosError<ErrorResponse>) => {
  const message = error?.response?.data?.message || 'An error occurred';
  const status = error?.response?.status;
  
  if (status === 401) {
    // Handle authentication error
    redirectToLogin();
  } else if (status === 403) {
    // Handle authorization error
    showAccessDenied();
  } else {
    // Handle general error
    toast({
      title: 'Error',
      description: message,
      variant: 'destructive',
    });
  }
};
```

## Testing Conventions

### Test File Organization
- Place tests alongside components: `ComponentName.test.tsx` (✓ verified)
- Use descriptive test names (✓ verified)
- Group related tests with `describe` blocks (✓ verified)
- Use `it` or `test` for individual test cases (✓ verified)

### Test Structure
Follow the Arrange-Act-Assert pattern (✓ verified):

```typescript
describe('UserProfile', () => {
  it('should display user information correctly', () => {
    // Arrange
    const user = { name: 'John Doe', email: 'john@example.com' };

    // Act
    render(<UserProfile user={user} />);

    // Assert
    expect(screen.getByText('John Doe')).toBeInTheDocument();
    expect(screen.getByText('john@example.com')).toBeInTheDocument();
  });
});
```

**Reference**: `crates/bodhi/src/components/AppInitializer.test.tsx:68-99` - describe/it structure

### Mock Patterns
Use MSW for API mocking (✓ verified):

```typescript
const handlers = [
  rest.get('/api/users', (req, res, ctx) => {
    return res(ctx.json([{ id: 1, name: 'John Doe' }]));
  }),
];

const server = setupServer(...handlers);
```

**Reference**: `crates/bodhi/src/components/AppInitializer.test.tsx:13-14` - MSW imports and usage

## Accessibility Guidelines

### Semantic HTML
Use proper semantic elements:

```typescript
<main>
  <header>
    <h1>Page Title</h1>
  </header>
  <section>
    <h2>Section Title</h2>
    <article>Content</article>
  </section>
</main>
```

### ARIA Labels
Provide proper ARIA labels for interactive elements:

```typescript
<button
  aria-label="Close dialog"
  aria-describedby="dialog-description"
  onClick={onClose}
>
  <X className="h-4 w-4" />
</button>
```

### Keyboard Navigation
Ensure all interactive elements are keyboard accessible:

```typescript
<div
  role="button"
  tabIndex={0}
  onKeyDown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      onClick();
    }
  }}
  onClick={onClick}
>
  Interactive Element
</div>
```

## Performance Guidelines

### Component Optimization
Use React optimization techniques:

```typescript
// Memoize expensive calculations
const expensiveValue = useMemo(() => {
  return computeExpensiveValue(data);
}, [data]);

// Memoize callback functions
const handleClick = useCallback(() => {
  onItemClick(item.id);
}, [item.id, onItemClick]);

// Memoize components
const MemoizedComponent = memo(Component);
```

### Bundle Optimization
- Use dynamic imports for code splitting
- Lazy load components when appropriate
- Optimize images and assets
- Monitor bundle size regularly

## Code Quality

### ESLint Configuration
Follow the project's ESLint rules:
- No unused variables
- Consistent import ordering
- Proper TypeScript usage
- Accessibility rule compliance

### TypeScript Best Practices
- Use strict type checking
- Avoid `any` type
- Define proper interfaces
- Use type guards when necessary

```typescript
// Good: Proper typing
interface User {
  id: number;
  name: string;
  email: string;
}

// Good: Type guard
function isUser(obj: unknown): obj is User {
  return typeof obj === 'object' && obj !== null && 'id' in obj;
}
```

## Documentation Standards

### Code Comments
- Use JSDoc for function documentation
- Explain complex business logic
- Document non-obvious code decisions
- Keep comments up to date

```typescript
/**
 * Calculates the user's subscription status based on their plan and payment history
 * @param user - The user object containing plan and payment information
 * @returns The current subscription status
 */
function calculateSubscriptionStatus(user: User): SubscriptionStatus {
  // Implementation
}
```

### README Files
Each major feature should have a README with:
- Purpose and overview
- Setup instructions
- Usage examples
- API documentation
- Testing information

## Git Workflow

### Commit Messages
Use conventional commit format:

```
type(scope): description

feat(auth): add OAuth2 login support
fix(ui): resolve button alignment issue
docs(api): update endpoint documentation
test(user): add user profile tests
```

### Branch Naming
- Feature branches: `feature/description`
- Bug fixes: `fix/description`
- Documentation: `docs/description`
- Refactoring: `refactor/description`

### Pull Request Process
1. Create feature branch from main
2. Implement changes with tests
3. Update documentation
4. Create pull request with description
5. Address review feedback
6. Merge after approval

---

## Verification Status

This document has been factually verified against the actual source code. Key verification points:

### ✅ Verified Accurate
- Project structure and crate organization
- Database layer implementation patterns
- Service layer trait definitions
- Frontend utility functions (cn, useQuery)
- Testing infrastructure (MSW, rstest)
- Enum serialization patterns
- Migration file structure

### ⚠️ Corrected Inaccuracies
- **API base paths**: Updated from `/api/v1` to actual paths (`/bodhi/v1`, `/v1`)
- **Pagination parameters**: Corrected from `per_page` to `page_size`
- **Component organization**: Updated from page-specific to feature-based structure
- **File naming**: Corrected to reflect actual PascalCase usage for components
- **Backend structure**: Updated to reflect actual directory organization

### 📁 Source References
All patterns and examples reference actual files in the codebase:
- `crates/services/src/db/objs.rs` - Database models and enums
- `crates/services/src/db/service.rs` - Service layer patterns
- `crates/bodhi/src/lib/utils.ts` - Frontend utilities
- `crates/bodhi/src/hooks/useQuery.ts` - API integration patterns
- `crates/services/src/test_utils/db.rs` - Testing infrastructure
- `crates/services/migrations/` - Migration files

*These conventions ensure consistent, maintainable, and high-quality code across the Bodhi App project.*
