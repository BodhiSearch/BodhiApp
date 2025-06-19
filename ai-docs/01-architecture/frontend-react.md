# Frontend Next.js Development

> **AI Coding Assistant Guide**: This document provides concise Next.js+TypeScript frontend development conventions for the Bodhi App. Focus on established patterns and architectural principles rather than detailed implementation examples.

## Required Documentation References

**MUST READ before any Next.js changes:**
- `ai-docs/01-architecture/api-integration.md` - API integration patterns and query hooks
- `ai-docs/01-architecture/development-conventions.md` - Naming conventions and component structure

**FOR STYLING:**
- `ai-docs/01-architecture/ui-design-system.md` - UI/UX patterns and component usage

**FOR TESTING:**
- `ai-docs/01-architecture/frontend-testing.md` - Testing patterns and utilities

## Technology Stack

### Core Technologies
- **React 18+** with TypeScript for component-based UI development
- **Next.js v14.2.6** with App Router for file-based routing and navigation
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
│   │   ├── auth/callback/ # OAuth callback page
│   │   ├── chat/        # Chat interface pages
│   │   ├── login/       # Login page
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

### Mutation Hook Usage with Variable Extraction

**✅ Preferred: Extract variables for clarity and disable states**
```typescript
export function CreateModelForm() {
  const [error, setError] = useState<string | null>(null);

  // Extract mutate function and loading state for clarity
  const { mutate: createModel, isLoading } = useCreateModel({
    onSuccess: (model) => router.push(`/ui/models/${model.id}`),
    onError: (message) => setError(message),
  });
  const handleSubmit = (data: ModelFormData) => {
    createModel(data); // Clear function name
  };

  return (
    <form onSubmit={handleSubmit}>
      {error && <ErrorAlert message={error} />}
    <Button
      type="submit"
        disabled={isLoading} // Prevent double-clicks
      >
        {isLoading ? 'Creating...' : 'Create Model'}
      </Button>
    </form>
    <Button
      type="submit"
      disabled={isLoading} // Prevent double-clicks
    >
      {isLoading ? 'Creating...' : 'Create Model'}
    </Button>
  );
}
```

**❌ Avoid**: Using mutation object directly with verbose property access.
```typescript
  return (
    <Button
      onClick={() => createModelMutation.mutate(data)} // ❌ Less clear
      disabled={createModelMutation.isLoading} // ❌ Verbose
    >
      {createModelMutation.isLoading ? 'Creating...' : 'Create Model'} // ❌ Repetitive
    </Button>
```

## OAuth Authentication Patterns

### OAuth Flow Implementation

**Current Implementation**: JSON responses (not HTTP redirects), button state management, smart URL handling.

### OAuth Hook Pattern
```typescript
export function useOAuthInitiate(options?: { onSuccess?, onError? }) {
  return useMutationQuery<AuthInitiateResponse, void>(
    ENDPOINT_AUTH_INITIATE, 'post', options,
  );
}
```

### OAuth Status Codes and Responses
- **201 Created**: New OAuth session created (user not authenticated)
- **200 OK**: User already authenticated, redirect to app
- **422 Unprocessable Entity**: Validation error
- **500 Internal Server Error**: Server error during token exchange

### Smart URL Handling Pattern
```typescript
// Use the standardized utility for consistent URL handling
import { handleSmartRedirect } from '@/lib/utils';

const { mutate: initiateOAuth } = useOAuthInitiate({
  onSuccess: (response) => {
    const location = response.data?.location;
    if (location) {
      handleSmartRedirect(location, router); // Handles same-origin vs external
    }
  },
});
```

**Implementation Details**:
- **Same-origin detection**: Compares protocol and host with current URL
- **Next.js router**: Uses `router.push()` for same-origin URLs with pathname + search + hash
- **External URLs**: Uses `window.location.href` for different origins
- **Error handling**: Treats invalid URLs as external for graceful fallback

## Routing Patterns

### Next.js App Router Structure
```text
src/app/
├── page.tsx                 # Root page (redirects to /ui)
├── layout.tsx              # Root layout
├── ui/
│   ├── auth/callback/page.tsx    # OAuth callback page
│   ├── login/page.tsx           # Login page
│   ├── chat/page.tsx            # Chat page
│   └── models/page.tsx          # Models page
```

### Navigation Hooks
```typescript
import { useRouter, usePathname } from "next/navigation";

const router = useRouter();
const pathname = usePathname();
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

### Benefits of Dumb Frontend Architecture

1. **Security**: All validation and business logic on backend prevents client-side bypasses
2. **Consistency**: Single source of truth for business rules and validation
3. **Maintainability**: Changes to business logic only require backend updates
4. **Testability**: Easier to test data operations separately from UI behavior
5. **Flexibility**: Frontend can be easily replaced or multiple frontends can share same backend
6. **Error Handling**: Backend provides consistent, localized error messages

#### Core Principles

1. **Backend-Driven Logic**: All business logic, validation, and flow decisions happen on the backend
2. **Frontend as Presentation Layer**: Frontend focuses on displaying data and collecting user input
3. **Minimal Frontend Validation**: Only basic UX validation - rely on backend for security
4. **Pass-Through Pattern**: Frontend sends all available data to backend without filtering

### Backend-Driven Validation and Logic Flow

**✅ Preferred**: Send all data to backend
```typescript
// OAuth callback - send ALL query parameters to backend
const { mutate: oauthCallback } = useOAuthCallback({
  onSuccess: (response) => handleRedirect(response.data?.location),
  onError: (message) => setError(message),
});

useEffect(() => {
  const allParams = Object.fromEntries(searchParams.entries());
  oauthCallback(allParams); // Backend handles all validation
}, []);
```

**❌ Avoid**: Frontend validation and filtering of business logic

### Page-Based Action Handling Convention

**Pattern**: Pages handle actions and redirects, hooks handle data operations.

```typescript
// Hook focuses on data operation only
export const useOAuthInitiate = (options) => {
  return useMutation({
    onSuccess: (response) => options?.onSuccess?.(response),
    onError: (error) => options?.onError?.(message),
  });
};

// Page handles redirect logic
export function LoginPage() {
  const { mutate: initiateOAuth, isLoading } = useOAuthInitiate({
    onSuccess: (response) => handleRedirect(response.data?.location),
    onError: (message) => setError(message),
  });
  // Page triggers action and handles UI state
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

### Error Handling Patterns

**✅ Preferred**: Display backend errors directly
```typescript
onError: (error: AxiosError<ErrorResponse>) => {
  const message = error?.response?.data?.error?.message || 'An unexpected error occurred';
  options?.onError?.(message); // Pass backend message directly
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

**✅ Frontend validation for UX only**:
```typescript
const formSchema = z.object({
  name: z.string().min(1, "Name is required"), // Basic UX validation
  email: z.string().email("Please enter a valid email"), // Format hint
});

// Always send to backend for authoritative validation
```

**❌ Avoid**: Complex frontend business logic validation

## Testing Requirements

### Component Testing
- Use `@testing-library/react` for component testing
- Use `createWrapper` from `@/tests/wrapper` for React Query setup
- Use `mockWindowLocation` from `@/tests/wrapper` for navigation testing

### OAuth Testing Patterns
```typescript
import { createWrapper, mockWindowLocation } from '@/tests/wrapper';

describe('OAuth Flow', () => {
  beforeEach(() => {
    mockWindowLocation('http://localhost:3000/ui/login');
  });
  // Test button states, URL handling, parameter passing
});
```

## Build and Development

### Commands
```bash
cd crates/bodhi
npm run dev            # Development server
npm run build          # Production build
npm run test           # Test runner
npm run format         # Code formatting
```

## Related Documentation
- **[API Integration](api-integration.md)** - Frontend-backend integration patterns
- **[UI Design System](ui-design-system.md)** - Design tokens and component usage
- **[Frontend Testing](frontend-testing.md)** - Testing patterns and OAuth testing
- **[Development Conventions](development-conventions.md)** - Coding standards

---

*For detailed implementation examples, reference actual code files in the codebase rather than this guide.*
