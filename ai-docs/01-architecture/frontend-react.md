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

## Page-Based Action Handling Convention

### Principle: Pages Handle Actions, Hooks Handle Queries

For better code clarity and maintainability, **page components should handle action-based logic** (redirects, navigation, UI state changes) while **hooks should focus purely on data operations**.

### Pattern: OAuth Flow Example

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

**✅ Preferred: Page handles redirects**
```typescript
// Hook focuses on data operation only
export const useOAuthInitiate = (options?: {
  onSuccess?: (response: AuthInitiateResponse) => void;
  onError?: (message: string) => void;
}) => {
  return useMutation({
    onSuccess: (response) => {
      options?.onSuccess?.(response.data); // ✅ Just call callback
    },
  });
};

// Page component handles the redirect logic
export function LoginPage() {
  const oauthInitiate = useOAuthInitiate({
    onSuccess: (response) => {
      window.location.href = response.auth_url; // ✅ Page handles redirect
    },
    onError: (message) => {
      setError(message);
    },
  });

  return (
    <AuthCard
      actions={[{
        label: 'Sign In',
        onClick: () => oauthInitiate.mutate(), // ✅ Page triggers action
      }]}
    />
  );
}
```

### Benefits of This Pattern

1. **Clearer Separation of Concerns**: Hooks focus on data, pages focus on user experience
2. **Better Testability**: Easier to test data operations separately from UI behavior
3. **Improved Reusability**: Hooks can be reused in different contexts with different action handling
4. **Enhanced Developer Experience**: Developers can easily understand where redirects and UI changes happen

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
