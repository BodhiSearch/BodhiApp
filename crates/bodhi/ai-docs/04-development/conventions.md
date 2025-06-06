# Development Conventions

This document outlines the coding standards, naming conventions, and best practices for the Bodhi App project.

## Project Structure

### Root Structure
```
src/
├── components/    # React components organized by feature
├── pages/         # Page components for routing
├── hooks/         # Custom React hooks
├── lib/           # Utility functions and shared logic
├── schemas/       # Data validation schemas
├── styles/        # Global styles and theme definitions
├── tests/         # Test files
└── types/         # TypeScript type definitions
```

### Component Organization
Components follow a co-location pattern for page-specific components:

```
src/components/page-name/
├── page.tsx               # Main page component
├── page.test.tsx         # Page tests
├── ComponentA.tsx        # Page-specific components
├── ComponentA.test.tsx   # Component tests
└── types.ts              # Page-specific types
```

Benefits:
- Keeps related code close together
- Makes it easy to find components specific to a page
- Improves maintainability by grouping related files
- Allows for better code splitting
- Simplifies testing related components

## Naming Conventions

### Files and Directories
- **Files**: kebab-case (`my-component.tsx`)
- **Directories**: kebab-case (`user-management/`)
- **Test files**: `component-name.test.tsx`
- **Type files**: `component-name.types.ts`

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

## Styling Conventions

### Tailwind CSS Usage
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

### Class Ordering
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
Use React Query for server state management:

```typescript
const { data, isLoading, error } = useQuery({
  queryKey: ['users'],
  queryFn: fetchUsers,
});
```

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

### Mutation Pattern
Use consistent mutation patterns with React Query:

```typescript
export function useCreateUser(options?: {
  onSuccess?: (user: User) => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: createUser,
    onSuccess: (user) => {
      queryClient.invalidateQueries(['users']);
      options?.onSuccess?.(user);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.message || 'Failed to create user';
      options?.onError?.(message);
    },
  });
}
```

### Component Usage
Use mutations with proper callbacks:

```typescript
const { mutate: createUser, isLoading } = useCreateUser({
  onSuccess: (user) => {
    toast({
      title: 'Success',
      description: `User ${user.name} created successfully`,
    });
  },
  onError: (message) => {
    toast({
      title: 'Error',
      description: message,
      variant: 'destructive',
    });
  },
});
```

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
- Place tests alongside components: `component.test.tsx`
- Use descriptive test names
- Group related tests with `describe` blocks
- Use `it` or `test` for individual test cases

### Test Structure
Follow the Arrange-Act-Assert pattern:

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

### Mock Patterns
Use MSW for API mocking:

```typescript
const handlers = [
  rest.get('/api/users', (req, res, ctx) => {
    return res(ctx.json([{ id: 1, name: 'John Doe' }]));
  }),
];

const server = setupServer(...handlers);
```

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

*These conventions ensure consistent, maintainable, and high-quality code across the Bodhi App project.*
