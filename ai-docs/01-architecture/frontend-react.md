# Frontend Next.js Development

This document provides focused guidance for Next.js+TypeScript frontend development in the Bodhi App, including component patterns, development conventions, and best practices.

## Required Documentation References

**MUST READ before any Next.js changes:**
- `ai-docs/01-architecture/api-integration.md` - API integration patterns and query hooks
- `ai-docs/01-architecture/development-conventions.md` - Naming conventions and component structure

**FOR STYLING:**
- `ai-docs/01-architecture/ui-design-system.md` - UI/UX patterns and component usage

**FOR TESTING:**
- `ai-docs/01-architecture/testing-strategy.md` - Testing patterns and utilities

## Technology Stack

### Core Technologies
- **React 18+** with TypeScript for component-based UI development
- **Next.js v14.2.6** for full-stack React framework with SSG capabilities
- **Next.js App Router** for file-based routing and navigation
- **React Query v3.39.3** for data fetching, caching, and synchronization

### UI & Styling
- **Tailwind CSS** - Utility-first CSS framework
- **Shadcn/ui** - Component library built on Radix UI
- **Lucide React** - Icon library
- **Framer Motion** - Animation library

### Form & Validation
- **React Hook Form** - Form state management
- **Zod** - Schema validation and type safety

## Project Structure

### Next.js App Directory Organization
```
src/
├── app/                 # Next.js App Router directory
│   ├── ui/              # UI pages and layouts
│   │   ├── auth/        # Authentication pages
│   │   ├── chat/        # Chat interface pages
│   │   ├── models/      # Model management pages
│   │   ├── setup/       # Setup wizard pages
│   │   ├── tokens/      # API tokens pages
│   │   └── users/       # User management pages
│   ├── docs/            # Documentation pages
│   ├── globals.css      # Global styles
│   ├── layout.tsx       # Root layout component
│   └── page.tsx         # Root page (redirects to /ui)
├── components/          # Reusable UI components
│   ├── navigation/      # Navigation components
│   ├── ui/              # Common UI components (shadcn/ui)
│   └── ThemeProvider.tsx # Theme provider
├── hooks/               # Custom React hooks
├── lib/                 # Utility functions and API clients
├── types/               # TypeScript type definitions
└── tests/               # Test files and utilities
```

### File Organization Patterns
- **Pages**: Use Next.js App Router convention (`page.tsx`, `layout.tsx`)
- **Components**: Use PascalCase (`ComponentName.tsx`) not kebab-case
- **Tests**: `ComponentName.test.tsx` in same directory as component
- **Feature-based organization**: Pages in `app/ui/<feature>/` directories
- **Custom hooks**: `hooks/` directory with proper naming (`use-feature-name.ts`)

## Component Development Patterns

### Component Structure
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

### Feature-Based Organization
```text
components/feature-name/
├── FeatureComponent.tsx      # Main feature component
├── FeatureForm.tsx          # Forms and inputs
├── FeatureDialog.tsx        # Modals and dialogs
├── FeatureComponent.test.tsx # Component tests
└── types.ts                 # Feature-specific types

pages/
└── FeaturePage.tsx          # Main page component
```

### Styling Conventions
```typescript
import { cn } from "@/lib/utils";

<div
  className={cn(
    "flex items-center p-4",
    isActive && "bg-primary text-white",
    variant === "large" && "text-lg"
  )}
>
```

## Form Handling Patterns

### Form Setup with Validation
```typescript
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

const formSchema = z.object({
  name: z.string().min(1, "Name is required"),
  email: z.string().email("Invalid email"),
});

type FormData = z.infer<typeof formSchema>;

export function MyForm() {
  const form = useForm<FormData>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: "",
      email: "",
    },
  });

  const onSubmit = async (data: FormData) => {
    // Handle form submission
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        {/* Form fields */}
      </form>
    </Form>
  );
}
```

### Form Components with Shadcn/ui
```typescript
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
```

## API Integration Standards

### Query Hooks Usage
```typescript
import { useQuery } from "react-query";
import { getModels } from "@/lib/api";

export function ModelsList() {
  const { data: models, isLoading, error } = useQuery({
    queryKey: ["models"],
    queryFn: getModels,
  });

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error loading models</div>;

  return (
    <div>
      {models?.map(model => (
        <div key={model.id}>{model.name}</div>
      ))}
    </div>
  );
}
```

### Mutation Pattern
```typescript
import { useMutation, useQueryClient } from "@tanstack/react-query";

export function useCreateModel(options?: {
  onSuccess?: (model: Model) => void;
  onError?: (message: string) => void;
}) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: createModel,
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: ["models"] });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || "Failed to create model";
      options?.onError?.(message);
    },
  });
}
```

## Routing Patterns

### Next.js App Router Structure
```text
src/app/
├── page.tsx                 # Root page (redirects to /ui)
├── layout.tsx              # Root layout
├── ui/
│   ├── page.tsx            # UI home page
│   ├── chat/
│   │   └── page.tsx        # Chat page
│   ├── models/
│   │   └── page.tsx        # Models page
│   └── setup/
│       └── page.tsx        # Setup page
└── not-found.tsx           # 404 page
```

### Navigation Hooks
```typescript
import { useRouter, usePathname } from "next/navigation";

export function NavigationComponent() {
  const router = useRouter();
  const pathname = usePathname();

  const handleNavigation = () => {
    router.push("/ui/models");
  };

  const isActive = pathname === "/ui/models";
}
```

## State Management

### Local State
- Use `useState` for component-local state
- Use `useReducer` for complex state logic
- Keep state as close to where it's used as possible

### Server State
- Use React Query for all server state management
- Follow established query patterns from `useQuery.ts`
- Implement proper error handling with `AxiosError<ErrorResponse>`

### Global State
- Use React Context for truly global state
- Prefer composition over prop drilling
- Keep context providers focused and specific

## Frontend Architecture Philosophy

### Dumb Frontend Principle

The Bodhi App follows a **"dumb frontend"** architecture where the frontend focuses on presentation and user interaction while the backend handles all business logic, validation, and flow control decisions.

#### Core Principles

1. **Backend-Driven Logic**: All business logic, validation, and flow decisions happen on the backend
2. **Frontend as Presentation Layer**: Frontend focuses on displaying data and collecting user input
3. **Minimal Frontend Validation**: Only basic UX validation (required fields, format hints) - rely on backend for security
4. **Pass-Through Pattern**: Frontend sends all available data to backend without filtering or interpretation

### Backend-Driven Validation and Logic Flow

**✅ Preferred: Send all data to backend**
```typescript
// OAuth callback - send ALL query parameters to backend
export function OAuthCallbackPage() {
  const searchParams = useSearchParams();
  const router = useRouter();

  const oauthCallback = useOAuthCallback({
    onSuccess: (response) => {
      // Backend determines where to redirect
      const redirectUrl = response.headers?.location || '/ui/chat';
      window.location.href = redirectUrl;
    },
    onError: (message) => {
      // Display backend-provided error message
      setError(message);
      router.push('/ui/login');
    },
  });

  useEffect(() => {
    // Send ALL query params to backend - let backend validate and process
    const allParams = Object.fromEntries(searchParams.entries());
    oauthCallback.mutate(allParams); // ✅ Backend handles all validation
  }, []);

  return <div>Processing authentication...</div>;
}
```

**❌ Avoid: Frontend validation and logic**
```typescript
// Don't do this - frontend shouldn't validate OAuth parameters
export function OAuthCallbackPage() {
  const searchParams = useSearchParams();

  // ❌ Frontend shouldn't validate OAuth flow
  const code = searchParams.get('code');
  const state = searchParams.get('state');

  if (!code || !state) {
    setError('Invalid OAuth response'); // ❌ Frontend making business decisions
    return;
  }

  // ❌ Frontend filtering data before sending to backend
  oauthCallback.mutate({ code, state });
}
```

### Page-Based Action Handling Convention

**Pages handle actions, hooks handle data operations** - this separation ensures clear responsibility boundaries and better testability.

#### Pattern: OAuth Flow Example

**✅ Preferred: Page handles redirects and UI logic**
```typescript
// Hook focuses on data operation only
export const useOAuthInitiate = (options?: {
  onSuccess?: (response: AuthInitiateResponse) => void;
  onError?: (message: string) => void;
}) => {
  return useMutation({
    onSuccess: (response) => {
      options?.onSuccess?.(response); // ✅ Just call callback with full response
    },
    onError: (error) => {
      const message = error?.response?.data?.error?.message || 'Authentication failed';
      options?.onError?.(message); // ✅ Pass backend error message
    },
  });
};

// Page component handles the redirect logic
export function LoginPage() {
  const [error, setError] = useState<string | null>(null);

  const oauthInitiate = useOAuthInitiate({
    onSuccess: (response) => {
      // Backend provides redirect URL via Location header
      const authUrl = response.headers?.location;
      if (authUrl) {
        window.location.href = authUrl; // ✅ Page handles redirect
      } else {
        setError('No authentication URL provided'); // ✅ Handle missing data
      }
    },
    onError: (message) => {
      setError(message); // ✅ Display backend error
    },
  });

  return (
    <AuthCard
      error={error}
      actions={[{
        label: 'Sign In',
        onClick: () => oauthInitiate.mutate(), // ✅ Page triggers action
      }]}
    />
  );
}
```

**❌ Avoid: Hook handling redirects**
```typescript
// Don't do this - hook handles redirect logic
export const useOAuthInitiate = () => {
  return useMutation({
    onSuccess: (response) => {
      window.location.href = response.auth_url; // ❌ Hook handles redirect
    },
  });
};
```

### Benefits of Dumb Frontend Architecture

1. **Security**: All validation and business logic on backend prevents client-side bypasses
2. **Consistency**: Single source of truth for business rules and validation
3. **Maintainability**: Changes to business logic only require backend updates
4. **Testability**: Easier to test data operations separately from UI behavior
5. **Flexibility**: Frontend can be easily replaced or multiple frontends can share same backend
6. **Error Handling**: Backend provides consistent, localized error messages

### Error Handling Patterns

Following the dumb frontend principle, error handling should primarily display backend-provided messages rather than interpreting or transforming errors on the frontend.

**✅ Preferred: Display backend errors directly**
```typescript
export function useCreateModel(options?: {
  onSuccess?: (model: Model) => void;
  onError?: (message: string) => void;
}) {
  return useMutation({
    mutationFn: createModel,
    onSuccess: (response) => {
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      // Use backend-provided error message
      const message = error?.response?.data?.error?.message || 'An unexpected error occurred';
      options?.onError?.(message); // ✅ Pass backend message directly
    },
  });
}

// Page handles error display
export function ModelsPage() {
  const [error, setError] = useState<string | null>(null);

  const createModel = useCreateModel({
    onSuccess: (model) => {
      router.push(`/ui/models/${model.id}`);
    },
    onError: (message) => {
      setError(message); // ✅ Display backend error as-is
    },
  });

  return (
    <div>
      {error && <ErrorAlert message={error} />}
      {/* Rest of component */}
    </div>
  );
}
```

**❌ Avoid: Frontend error interpretation**
```typescript
onError: (error: AxiosError<ErrorResponse>) => {
  // ❌ Don't interpret or transform backend errors
  if (error.response?.status === 400) {
    setError('Invalid input provided');
  } else if (error.response?.status === 500) {
    setError('Server error occurred');
  }
  // Backend should provide appropriate messages
}
```

### Data Validation Patterns

**✅ Frontend validation for UX only**
```typescript
const formSchema = z.object({
  name: z.string().min(1, "Name is required"), // ✅ Basic UX validation
  email: z.string().email("Please enter a valid email"), // ✅ Format hint
});

// Always send to backend for authoritative validation
const onSubmit = async (data: FormData) => {
  try {
    await createUser(data); // ✅ Backend does real validation
  } catch (error) {
    // Display backend validation errors
    setError(error.response?.data?.error?.message);
  }
};
```

**❌ Avoid: Complex frontend business logic**
```typescript
// ❌ Don't implement business rules on frontend
const onSubmit = async (data: FormData) => {
  if (data.role === 'admin' && !data.permissions.includes('manage_users')) {
    setError('Admins must have user management permissions');
    return; // ❌ Business logic belongs on backend
  }

  if (data.email.endsWith('@competitor.com')) {
    setError('Competitor emails not allowed');
    return; // ❌ Business rules belong on backend
  }
};
```

## Testing Requirements

### Component Testing
```typescript
import { render, screen } from "@testing-library/react";
import { ComponentName } from "./ComponentName";

describe("ComponentName", () => {
  it("renders correctly", () => {
    render(<ComponentName prop1="test" />);
    expect(screen.getByText("test")).toBeInTheDocument();
  });
});
```

### Testing with React Query
```typescript
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const createTestQueryClient = () => new QueryClient({
  defaultOptions: {
    queries: { retry: false },
    mutations: { retry: false },
  },
});

const renderWithQueryClient = (component: React.ReactElement) => {
  const testQueryClient = createTestQueryClient();
  return render(
    <QueryClientProvider client={testQueryClient}>
      {component}
    </QueryClientProvider>
  );
};
```

## Performance Considerations

### Code Splitting
```typescript
import { lazy, Suspense } from "react";

const LazyComponent = lazy(() => import("./LazyComponent"));

export function App() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <LazyComponent />
    </Suspense>
  );
}
```

### Memoization
```typescript
import { memo, useMemo, useCallback } from "react";

export const ExpensiveComponent = memo(({ data }: Props) => {
  const processedData = useMemo(() => {
    return expensiveCalculation(data);
  }, [data]);

  const handleClick = useCallback(() => {
    // Handle click
  }, []);

  return <div>{processedData}</div>;
});
```

## Build and Development

### Commands
```bash
cd crates/bodhi

# Development
npm run dev            # Next.js development server

# Build
npm run build          # Next.js production build

# Start
npm run start          # Start production server

# Testing
npm run test           # Vitest test runner

# Code quality
npm run format
npm run lint
```

## Related Documentation

- **[API Integration](api-integration.md)** - Frontend-backend integration patterns
- **[UI Design System](ui-design-system.md)** - Design tokens and component usage
- **[Testing Strategy](testing-strategy.md)** - Frontend testing patterns
- **[Development Conventions](development-conventions.md)** - Coding standards and best practices

---

*For detailed API integration patterns, see [API Integration](api-integration.md). For UI component usage, see [UI Design System](ui-design-system.md).*
