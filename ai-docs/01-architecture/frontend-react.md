# Frontend React Development

This document provides focused guidance for React+TypeScript frontend development in the Bodhi App, including component patterns, development conventions, and best practices.

## Required Documentation References

**MUST READ before any React changes:**
- `ai-docs/01-architecture/api-integration.md` - API integration patterns and query hooks
- `ai-docs/01-architecture/development-conventions.md` - Naming conventions and component structure

**FOR STYLING:**
- `ai-docs/01-architecture/ui-design-system.md` - UI/UX patterns and component usage

**FOR TESTING:**
- `ai-docs/01-architecture/testing-strategy.md` - Testing patterns and utilities

## Technology Stack

### Core Technologies
- **React 18+** with TypeScript for component-based UI development
- **Vite** for fast build tooling and development server
- **React Router** for client-side routing and navigation
- **React Query** for data fetching, caching, and synchronization

### UI & Styling
- **Tailwind CSS** - Utility-first CSS framework
- **Shadcn/ui** - Component library built on Radix UI
- **Lucide React** - Icon library
- **Framer Motion** - Animation library

### Form & Validation
- **React Hook Form** - Form state management
- **Zod** - Schema validation and type safety

## Project Structure

### Component Organization
```
src/
├── components/           # Feature-specific components
│   ├── auth/            # Authentication components
│   ├── chat/            # Chat interface components
│   ├── models/          # Model management components
│   ├── navigation/      # Navigation and header components
│   ├── setup/           # Setup wizard components
│   ├── tokens/          # API tokens components
│   ├── ui/              # Common UI components (shadcn/ui)
│   └── users/           # User management components
├── pages/               # Page components for routing
├── hooks/               # Custom React hooks
├── lib/                 # Utility functions and API clients
├── types/               # TypeScript type definitions
└── tests/               # Test files and utilities
```

### File Organization Patterns
- **Components**: Use PascalCase (`ComponentName.tsx`) not kebab-case
- **Tests**: `ComponentName.test.tsx` in same directory as component
- **Feature-based organization**: `components/` directory (auth/, chat/, models/, navigation/, etc.)
- **Page components**: Separate structure from feature components
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
import { useQuery } from "@tanstack/react-query";
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

### Route Configuration
```typescript
// App.tsx
<Routes>
  <Route path="/" element={<Navigate to="/ui" replace />} />
  <Route path="/ui" element={<HomePage />} />
  <Route path="/ui/chat" element={<ChatPage />} />
  <Route path="/ui/models" element={<ModelsPage />} />
  <Route path="/ui/setup/*" element={<SetupPage />} />
  <Route path="*" element={<NotFoundPage />} />
</Routes>
```

### Navigation Hooks
```typescript
import { useNavigate, useLocation } from "react-router-dom";

export function NavigationComponent() {
  const navigate = useNavigate();
  const location = useLocation();

  const handleNavigation = () => {
    navigate("/ui/models");
  };

  const isActive = location.pathname === "/ui/models";
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
npm run dev

# Build
npm run build

# Testing
npm run test           # Watch mode
npm run test -- --run  # CI mode

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
