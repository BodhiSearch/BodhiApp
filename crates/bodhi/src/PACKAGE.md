# PACKAGE.md - Frontend Implementation Index

> See [CLAUDE.md](./CLAUDE.md) for architectural narrative and design rationale

## Quick Navigation

### Application Entry Points

- **Root Layout**: `crates/bodhi/src/app/layout.tsx:26-60` - Main application layout with theme providers
- **App Router Root**: `crates/bodhi/src/app/page.tsx:3-5` - Root page redirecting to `/ui`
- **Main UI Entry**: `crates/bodhi/src/app/ui/page.tsx:5-7` - UI entry point with AppInitializer

### Core Chat Implementation

- **Chat Page**: `crates/bodhi/src/app/ui/chat/page.tsx:133-139` - Main chat interface entry
- **Chat Hook**: `crates/bodhi/src/hooks/use-chat.tsx:11-26` - Primary chat state management
- **Chat Completions**: `crates/bodhi/src/hooks/use-chat-completions.ts` - Streaming API integration
- **Chat Database**: `crates/bodhi/src/hooks/use-chat-db.tsx` - Local chat persistence

### Navigation & Layout System

- **Navigation Provider**: `crates/bodhi/src/hooks/use-navigation.tsx:138-204` - Navigation state management
- **App Header**: `crates/bodhi/src/components/navigation/AppHeader.tsx:19-48` - Main header component
- **Sidebar System**: `crates/bodhi/src/components/ui/sidebar.tsx:36-43` - Comprehensive sidebar framework

## Key Implementation Files

### State Management & API Integration

**Query Hooks** - `crates/bodhi/src/hooks/useQuery.ts:53-73`

```typescript
export function useQuery<T>(
  key: string | string[],
  endpoint: string,
  params?: Record<string, any>,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>>
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  return useReactQuery<T, AxiosError<ErrorResponse>>(
    key,
    async () => {
      const { data } = await apiClient.get<T>(endpoint, {
        params,
        headers: {
          'Content-Type': 'application/json',
        },
      });
      return data;
    },
    options
  );
}
```

**API Client Configuration** - `crates/bodhi/src/lib/apiClient.ts:4-22`

```typescript
const apiClient = axios.create({
  baseURL: isTest ? 'http://localhost:3000' : '',
  maxRedirects: 0,
});

apiClient.interceptors.response.use(
  (response) => {
    return response;
  },
  (error) => {
    console.error('Error:', error.response?.status, error.config?.url);
    return Promise.reject(error);
  }
);
```

### Chat System Implementation

**Chat State Hook** - `crates/bodhi/src/hooks/use-chat.tsx:34-50`

```typescript
const processCompletion = useCallback(
  async (userMessages: Message[]) => {
    let currentAssistantMessage = '';
    const userContent = userMessages[userMessages.length - 1].content;

    try {
      const requestMessages =
        chatSettings.systemPrompt_enabled && chatSettings.systemPrompt
          ? [{ role: 'system' as const, content: chatSettings.systemPrompt }, ...userMessages]
          : userMessages;

      const headers: Record<string, string> = {};
      if (chatSettings.api_token_enabled && chatSettings.api_token) {
        headers.Authorization = `Bearer ${chatSettings.api_token}`;
      }

      await append({
        request: {
          ...chatSettings.getRequestSettings(),
          messages: requestMessages,
        },
        headers,
        onDelta: (chunk) => {
          currentAssistantMessage += chunk;
          setAssistantMessage((prevMessage) => ({
            role: 'assistant' as const,
            content: prevMessage.content + chunk,
          }));
        },
        // ... completion handlers
      });
    } catch (error) {
      // Error handling implementation
    }
  },
  [chatSettings, currentChat, append, createOrUpdateChat]
);
```

**Chat UI Structure** - `crates/bodhi/src/app/ui/chat/ChatUI.tsx:17-24`

```typescript
const EmptyState = () => (
  <div className="flex h-full items-center justify-center" data-testid="empty-chat-state">
    <div className="text-center space-y-3">
      <h3 className="text-lg font-semibold">Welcome to Chat</h3>
      <p className="text-muted-foreground">Start a conversation by typing a message below.</p>
    </div>
  </div>
);
```

### Type-Safe Schema Validation

**Alias Schema with Generated Types** - `crates/bodhi/src/schemas/alias.ts:29-36`

```typescript
export const createAliasFormSchema = z.object({
  alias: z.string().min(1, 'Alias is required'),
  repo: z.string().min(1, 'Repo is required'),
  filename: z.string().min(1, 'Filename is required'),
  snapshot: z.string().optional(),
  request_params: requestParamsSchema,
  context_params: z.string().optional(),
});
```

**Form-to-API Conversion** - `crates/bodhi/src/schemas/alias.ts:41-53`

```typescript
export const convertFormToApi = (formData: AliasFormData): CreateAliasRequest => ({
  alias: formData.alias,
  repo: formData.repo,
  filename: formData.filename,
  snapshot: formData.snapshot || null,
  request_params: formData.request_params,
  context_params: formData.context_params
    ? formData.context_params
        .split('\n')
        .map((line) => line.trim())
        .filter((line) => line.length > 0)
    : undefined,
});
```

### Application Initialization Flow

**App Initializer Logic** - `crates/bodhi/src/components/AppInitializer.tsx:49-70`

```typescript
useEffect(() => {
  if (!appLoading && appInfo) {
    const { status } = appInfo;
    if (!allowedStatus || status !== allowedStatus) {
      switch (status) {
        case 'setup':
          router.push(ROUTE_SETUP);
          break;
        case 'ready':
          if (!hasShownModelsPage) {
            router.push(ROUTE_SETUP_DOWNLOAD_MODELS);
          } else {
            router.push(ROUTE_DEFAULT);
          }
          break;
        case 'resource-admin':
          router.push(ROUTE_RESOURCE_ADMIN);
          break;
      }
    }
  }
}, [appInfo, appLoading, allowedStatus, router, hasShownModelsPage]);
```

## Directory Structure & File Organization

### App Router Organization

```
crates/bodhi/src/app/
├── layout.tsx                    # Root layout with providers
├── page.tsx                      # Root redirect to /ui
├── globals.css                   # Global Tailwind styles
├── ui/                          # Main application routes
│   ├── chat/page.tsx            # Chat interface (133-139)
│   ├── models/                  # Model management
│   │   ├── page.tsx             # Models list
│   │   ├── new/page.tsx         # Create model alias
│   │   └── edit/page.tsx        # Edit model alias
│   ├── api-models/              # API model management
│   ├── pull/page.tsx            # Model download interface
│   ├── settings/page.tsx        # Application settings
│   ├── tokens/page.tsx          # API token management
│   ├── users/page.tsx           # User management
│   ├── access-requests/         # Access control
│   ├── setup/                   # Onboarding flow
│   └── auth/callback/page.tsx   # OAuth callback
└── docs/                        # Documentation routes
    ├── [...slug]/page.tsx       # Dynamic docs routing
    └── layout.tsx               # Docs-specific layout
```

### Component Architecture

```
crates/bodhi/src/components/
├── ui/                          # Shadcn UI base components
│   ├── sidebar.tsx              # Comprehensive sidebar system (36-148)
│   ├── button.tsx               # Base button component
│   ├── input.tsx                # Form input components
│   ├── dialog.tsx               # Modal dialogs
│   ├── toast.tsx                # Toast notifications
│   └── markdown.tsx             # Markdown rendering
├── navigation/                  # Navigation-specific components
│   ├── AppHeader.tsx            # Main header (19-48)
│   ├── AppNavigation.tsx        # Navigation menu
│   └── AppBreadcrumb.tsx        # Breadcrumb navigation
├── ClientProviders.tsx          # React Query & providers
├── ThemeProvider.tsx           # Dark/light theme management
└── AppInitializer.tsx          # App state initialization (31-147)
```

### Custom Hooks

```
crates/bodhi/src/hooks/
├── use-chat.tsx                # Main chat functionality (11-200+)
├── use-chat-completions.ts     # Streaming API integration
├── use-chat-db.tsx             # Chat persistence (localStorage)
├── use-chat-settings.tsx       # Chat configuration state
├── use-navigation.tsx          # Navigation state (138-207)
├── useQuery.ts                 # Generic API hooks (53-310)
├── useLocalStorage.ts          # Persistent state hook
├── use-mobile.ts               # Responsive design hook
└── use-toast-messages.ts       # Toast notification helpers
```

### Type Definitions & Schemas

```
crates/bodhi/src/types/
├── chat.ts                     # Chat message interfaces (14-41)
├── models.ts                   # Model type definitions
└── navigation.ts               # Navigation item types

crates/bodhi/src/schemas/
├── alias.ts                    # Model alias validation (29-89)
├── apiModel.ts                 # API model schemas
├── pull.ts                     # Model download schemas
└── objs.ts                     # Common object schemas
```

### Testing Infrastructure

```
crates/bodhi/src/test-utils/
├── msw-v2/                     # MSW v2 setup and handlers
│   ├── setup.ts                # MSW server setup
│   └── handlers/               # Domain-specific mock handlers
│       ├── access-requests.ts  # User access request mocks
│       ├── app-access-requests.ts # App access request mocks
│       ├── api-models.ts       # API model mocks
│       ├── auth.ts             # Auth mocks
│       ├── chat-completions.ts # Chat completion mocks
│       ├── info.ts             # App info mocks
│       ├── mcps.ts             # MCP server mocks
│       ├── models.ts           # Model mocks
│       ├── modelfiles.ts       # Modelfile mocks
│       ├── setup.ts            # Setup flow mocks
│       ├── settings.ts         # Settings mocks
│       ├── tokens.ts           # API token mocks
│       ├── toolsets.ts         # Toolset mocks
│       └── user.ts             # User mocks
├── fixtures/chat.ts            # Chat test fixtures
├── api-model-test-utils.ts     # API model test helpers
└── mock-user.ts                # Mock user data

crates/bodhi/src/tests/
├── setup.ts                    # Vitest setup (sets apiClient.defaults.baseURL)
├── wrapper.tsx                 # Test wrapper with providers
└── mocks/framer-motion.tsx     # Framer motion mock
```

## Key Build & Development Commands

### Frontend Development

```bash
# Start development server with hot reload
cd crates/bodhi && npm run dev

# Build Next.js static export for embedding
cd crates/bodhi && npm run build

# Run frontend tests
cd crates/bodhi && npm test

# Format and lint frontend code
cd crates/bodhi && npm run format
cd crates/bodhi && npm run lint
```

### Type Generation Workflow

```bash
# 1. Generate OpenAPI spec from Rust backend
cargo run --package xtask openapi

# 2. Generate TypeScript types from OpenAPI
cd ts-client && npm run generate:types

# 3. Build ts-client package
cd ts-client && npm run build

# 4. Frontend automatically picks up updated types via file dependency
```

### Testing Commands

```bash
# Run all tests (backend + frontend + NAPI)
make test

# Run only frontend tests
make test.ui

# Run frontend tests with coverage
cd crates/bodhi && npm run test:coverage
```

### UI Development Workflow

```bash
# Clean embedded UI build
make clean.ui

# Build embedded UI for desktop app
make build.ui

# Format all code
make format
```

## TypeScript Client Integration

### Generated Type Usage

- **Base Package**: `@bodhiapp/ts-client` (file dependency in package.json:17)
- **Generated Types**: `ts-client/src/types/` directory
- **OpenAPI Config**: `ts-client/openapi-ts.config.ts`
- **Build Output**: `ts-client/dist/` directory

### Import Patterns

```typescript
// Direct imports from generated client
import { AppInfo, UserInfo, CreateAliasRequest, PaginatedAliasResponse, OpenAiApiError } from '@bodhiapp/ts-client';

// Re-export through schema files for consistency
import type { CreateAliasRequest } from '@bodhiapp/ts-client';
export type { CreateAliasRequest };
```

## Configuration Files

### Next.js Configuration

- **Main Config**: `crates/bodhi/next.config.mjs:12-32` - Static export configuration
- **TypeScript**: `crates/bodhi/tsconfig.json` - Strict TypeScript settings
- **Tailwind**: `crates/bodhi/tailwind.config.ts` - Design system configuration

### Testing Configuration

- **Vitest**: `crates/bodhi/vitest.config.ts` - Test environment setup
- **Test Setup**: `crates/bodhi/src/tests/setup.ts` - Global test configuration

### Package Dependencies

- **Core Framework**: Next.js 14.2.30, React ^18, TypeScript ^5
- **UI Components**: Radix UI primitives, Lucide icons, TailwindCSS
- **State Management**: React Query 3.39.3, React Hook Form
- **API Integration**: Axios 1.9.0, generated TypeScript client
- **Testing**: Vitest 2.1.9, Testing Library, MSW for API mocking

## Development Patterns

### Component Testing Pattern

```typescript
// Example: crates/bodhi/src/components/ui/__tests__/button.test.tsx
import { render, screen } from '@testing-library/react';
import { Button } from '../button';

describe('Button Component', () => {
  it('renders with data-testid attribute', () => {
    render(<Button data-testid="test-button">Click me</Button>);
    expect(screen.getByTestId('test-button')).toBeInTheDocument();
  });
});
```

### API Hook Pattern

```typescript
// crates/bodhi/src/hooks/useQuery.ts:112-114
export function useAppInfo() {
  return useQuery<AppInfo>('appInfo', ENDPOINT_APP_INFO);
}
```

### Schema Validation Pattern

```typescript
// Form validation with Zod + generated types
const schema = z.object({
  alias: z.string().min(1),
  // ... other fields
});

type FormData = z.infer<typeof schema>;
const convertToApiRequest = (data: FormData): CreateAliasRequest => ({ ... });
```

## Performance Considerations

### Code Splitting

- **Route-based splitting**: Automatic via Next.js App Router
- **Component splitting**: Dynamic imports for heavy components
- **Bundle optimization**: Static export with optimized assets

### State Optimization

- **React Query caching**: 5-minute stale time, 10-minute cache time
- **LocalStorage persistence**: Chat history and settings
- **Memo optimization**: Performance-critical components wrapped with React.memo

### Build Optimization

- **Static export**: `output: 'export'` for desktop embedding
- **Source maps**: Enabled for production debugging
- **Image optimization**: Unoptimized for static export compatibility

This frontend implementation provides a comprehensive React-based UI for BodhiApp, with type-safe API integration, responsive design, and robust testing infrastructure.
