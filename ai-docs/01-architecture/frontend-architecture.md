# Bodhi Frontend Architecture

## Project Overview

Bodhi is a React+Vite application built with TypeScript that provides a web interface for running LLMs (Large Language Models) locally. The project uses React Router for navigation and follows modern React practices with a component-based architecture.

## Project Structure

### Root Structure

```
src/
├── components/    # React components organized by feature
├── pages/         # Page components for routing
├── hooks/         # Custom React hooks
├── lib/           # Utility functions and shared logic
├── schemas/       # Data validation schemas (Zod)
├── styles/        # Global styles and theme definitions
├── tests/         # Test files and utilities
├── types/         # TypeScript type definitions
├── docs/          # Documentation content (Markdown)
└── generated/     # Generated files (docs data)
```

### Components Structure

```
components/
├── auth/          # Authentication components
├── chat/          # Chat interface components
├── docs/          # Documentation components
├── home/          # Home page components
├── login/         # Login page components
├── modelfiles/    # Model file management
├── models/        # Model-related components
├── navigation/    # Navigation and header components
├── not-found/     # 404 page components
├── pull/          # Model download components
├── settings/      # Settings page components
├── setup/         # Setup wizard components
├── tokens/        # API tokens components
├── ui/            # Common UI components (shadcn/ui)
├── users/         # User management components
└── [shared components] # Shared utilities and providers
```

### Pages Structure

```
pages/
├── ChatPage.tsx       # Chat interface page
├── DocsPage.tsx       # Documentation page
├── HomePage.tsx       # Home/dashboard page
├── LoginPage.tsx      # Login page
├── ModelFilesPage.tsx # Model files management
├── ModelsPage.tsx     # Model management page
├── NotFoundPage.tsx   # 404 error page
├── PullPage.tsx       # Model download page
├── SettingsPage.tsx   # Application settings
├── SetupPage.tsx      # Setup wizard page
├── TokensPage.tsx     # API tokens management
├── UsersPage.tsx      # User management page
└── docs/              # Documentation sub-pages
```

## Core Technologies

### Framework & Runtime
- **Vite**
- **React**
- **TypeScript**: Programming language
- **Node.js**: Runtime environment

### UI Components & Styling
- **Tailwind CSS**: Utility-first CSS framework
- **Shadcn/ui**: Component library built on Radix UI
- **Class Variance Authority (CVA)**: Component variant management
- **clsx/tailwind-merge**: Class name management utilities
- **Lucide React**: Icon library
- **Framer Motion**: Animation library

### Data Management & API
- **React Query**: Data fetching and state management
- **Axios**: HTTP client
- **React Hook Form**: Form management
- **Zod**: Schema validation

### Testing
- **Vitest**: Testing framework
- **Testing Library**: Testing utilities
- **MSW (Mock Service Worker)**: API mocking
- **Happy DOM**: DOM environment for testing

### Development Tools
- **ESLint**: Code linting
- **Prettier**: Code formatting
- **Husky**: Git hooks
- **lint-staged**: Staged files linting

## Application Architecture

### Routing and Navigation

The application uses React Router for client-side routing with a clear page-based structure:

```typescript
// App.tsx - Main routing configuration
<Routes>
  <Route path="/" element={<Navigate to="/ui" replace />} />
  <Route path="/ui" element={<HomePage />} />
  <Route path="/ui/chat" element={<ChatPage />} />
  <Route path="/ui/models" element={<ModelsPage />} />
  <Route path="/ui/modelfiles" element={<ModelFilesPage />} />
  <Route path="/ui/pull" element={<PullPage />} />
  <Route path="/ui/login" element={<LoginPage />} />
  <Route path="/ui/settings" element={<SettingsPage />} />
  <Route path="/ui/tokens" element={<TokensPage />} />
  <Route path="/ui/users" element={<UsersPage />} />
  <Route path="/ui/setup/*" element={<SetupPage />} />
  <Route path="/docs/*" element={<DocsPage />} />
  <Route path="*" element={<NotFoundPage />} />
</Routes>
```

### Component Organization

The project follows a feature-based organization pattern:

```text
components/feature-name/
├── FeaturePage.tsx        # Main page component (in /pages)
├── FeatureComponent.tsx   # Feature-specific components
├── FeatureForm.tsx        # Forms and inputs
├── FeatureDialog.tsx      # Modals and dialogs
├── FeatureComponent.test.tsx # Component tests
└── types.ts               # Feature-specific types
```

Example from tokens feature:

```text
components/tokens/
├── TokenForm.tsx          # Token creation form
├── TokenDialog.tsx        # Token display dialog
├── TokenList.tsx          # Token listing component
├── TokenForm.test.tsx     # Form tests
└── TokenDialog.test.tsx   # Dialog tests

pages/
└── TokensPage.tsx         # Main tokens page
```

This organization:

- Separates page components from feature components
- Keeps related functionality grouped together
- Makes it easy to find components specific to a feature
- Improves maintainability and code reuse
- Allows for better code splitting and lazy loading

## Coding Conventions

### Component Structure
- Use functional components with TypeScript
- Follow the component-per-file pattern
- Place tests alongside components with `.test.tsx` extension
- Use named exports for components

Example component structure:

```typescript
export function ComponentName({ prop1, prop2 }: ComponentNameProps) {
  // Component logic
  return (
    // JSX
  )
}
```

### File Naming
- Use kebab-case for file names: `my-component.tsx`
- Use PascalCase for component names: `MyComponent`
- Test files: `my-component.test.tsx`
- Type files: `my-component.types.ts`

### Styling Conventions
- Use Tailwind CSS classes for styling
- Follow utility-first CSS approach
- Use `class:` syntax for conditional classes
- Leverage `cn()` utility for class name merging

Example styling:

```typescript
<div
  className={cn(
    "flex items-center p-4",
    isActive && "bg-primary text-white"
  )}
>
```

### State Management
- Use React Query for server state
- Use React hooks for local state
- Follow the container/presenter pattern
- Keep state as close to where it's used as possible

## Form Handling

- Use React Hook Form for form state management
- Use Zod for schema validation
- Leverage shadcn/ui form components
- Follow controlled component pattern

### Form Structure

Example form setup with validation:

```typescript
const createTokenSchema = z.object({
  name: z.string().optional()
});

export function TokenForm() {
  const form = useForm<TokenFormData>({
    resolver: zodResolver(createTokenSchema),
    mode: 'onSubmit',
    defaultValues: {
      name: '',
    },
  });
}
```

### Form Components

Use shadcn/ui form components for consistent styling:

```typescript
<Form {...form}>
  <form onSubmit={form.handleSubmit(onSubmit)}>
    <FormField
      control={form.control}
      name="name"
      render={({ field }) => (
        <FormItem>
          <FormLabel>Token Name</FormLabel>
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

Handle form submission with error handling:

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

## Network & API Conventions

### Mutation Pattern

The application follows a consistent pattern for handling mutations using react-query. This pattern provides better error handling, type safety, and separation of concerns.

#### Hook Definition Pattern

Mutation hooks should be defined with callback options:

```typescript
export function useSomeMutation(options?: {
  onSuccess?: (response: ResponseType) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<ResponseType>,
  AxiosError<ErrorResponse>,
  RequestType
> {
  const queryClient = useQueryClient();
  return useMutationQuery<ResponseType, RequestType>(
    ENDPOINT,
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries(['queryKey']);
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message =
          error?.response?.data?.error?.message || 'Failed to perform action';
        options?.onError?.(message);
      },
    }
  );
}
```

#### Component Usage Pattern

Components should use mutations by providing callbacks:

```typescript
const { mutate, isLoading } = useSomeMutation({
  onSuccess: (response) => {
    toast({
      title: 'Success',
      description: 'Operation completed successfully',
    });
    // Additional success handling
  },
  onError: (message) => {
    toast({
      title: 'Error',
      description: message,
      variant: 'destructive',
    });
    // Additional error handling
  },
});

const handleAction = (data: RequestType) => {
  mutate(data);
};
```

## Directory Purposes

### `/components`
React components organized by feature/page. Each folder contains components specific to that feature area.

### `/pages`
Page components that are used by React Router for routing. Each page component represents a distinct route in the application.

### `/hooks`
Custom React hooks for shared stateful logic and side effects. These hooks abstract common functionality used across components.

### `/lib`
Utility functions, API clients, and other shared logic that isn't specific to React components.

### `/schemas`
Data validation schemas, likely using libraries like Zod or Yup for type-safe data handling.

### `/styles`
Global styling configurations, theme definitions, and shared style utilities.

### `/types`
TypeScript type definitions and interfaces used throughout the application.

## Key Features

1. **Authentication System**
   - Login interface
   - Session management

2. **Model Management**
   - Model listing and details
   - Model file handling
   - Pull/sync capabilities

3. **Chat Interface**
   - Interactive chat UI
   - Message handling

4. **Setup and Configuration**
   - System setup components
   - Configuration management

## Architecture Principles

This structure suggests a feature-based organization with clear separation of concerns. For navigation and menu design, we should consider:

1. Grouping related features in the navigation
2. Creating a hierarchy based on user workflows
3. Ensuring easy access to frequently used features
4. Implementing proper access control based on user roles
