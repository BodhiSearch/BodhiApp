# CLAUDE.md - bodhi/src

> See [PACKAGE.md](./PACKAGE.md) for implementation details and file references

This file provides guidance to Claude Code when working with the Next.js frontend application for BodhiApp.

## Purpose

The `bodhi/src` directory contains the Next.js 14 frontend application:

- **Modern React Web UI**: React-based web interface with TypeScript
- **App Router Architecture**: Next.js 14 App Router with nested layouts and route groups
- **Responsive Design**: TailwindCSS with Shadcn/ui component library for consistent design
- **Real-time Chat Interface**: Streaming chat completions with real-time updates
- **Model Management UI**: Complete model management interface with pull/push capabilities
- **Authentication Flow**: OAuth integration with secure session management
- **Documentation System**: Built-in documentation with MDX support

## Key Components

### Application Structure (App Router)

- **Root Layout**: Main application layout with navigation and theme provider
- **Route Groups**: UI routes organized in `/ui` route group for main application
- **Documentation Routes**: `/docs` for integrated documentation system
- **API Integration**: Client-side API calls to BodhiApp backend

### Core Features

- **Chat Interface**: Real-time chat with streaming responses, message history, and tool calling support
- **Model Management**: Create, edit, and manage model aliases and configurations
- **MCP Server Management**: CRUD for MCP server instances, tool discovery, admin enable flow (`useMcps` hook, `mcps/` pages). All API endpoints under `/bodhi/v1/mcps/` prefix. MCP Servers view page (`/ui/mcp-servers/view?id=xxx`) shows server info and auth config list; auth configs (header, OAuth) are created there. Auth type is `McpAuthType` enum: `public`, `header`, `oauth`. OAuth pre-registered vs dynamic is distinguished by `registration_type` field, not separate auth types. MCP Servers new page (`/ui/mcp-servers/new`) supports optional auth config creation with OAuth registration type sub-dropdown. MCPs new/edit page uses an auth config dropdown.
- **MCP OAuth**: Callback page validates `state` parameter, handles corrupt session data gracefully; `mcpFormStore.reset()` clears sessionStorage. `mcpFormStore.ts` uses simplified auth state (auth config selection via dropdown). MSW handlers in `test-utils/msw-v2/handlers/mcps.ts` include unified auth config handlers at `/mcps/auth-configs`. **Important**: MSW handler registration order matters — sub-path handlers (`/mcps/servers`, `/mcps/auth-configs`) must be registered before wildcard `/mcps/:id` handlers to avoid route interception.
- **Auto-DCR behavior difference between new and view pages**: New server page (`mcp-servers/new/page.tsx`) uses `enableAutoDcr={true}` — when OAuth is selected, it silently tries OAuth discovery first, then falls back to pre-registered if discovery fails (no error shown to user). View/edit server page (`mcp-servers/view/page.tsx`) uses `enableAutoDcr={false}` — when OAuth is selected, discovery errors are shown to the user so they can retry; there is no silent fallback to pre-registered.
- **Toolset Management**: Toolset CRUD, type management, tool selection for chat (`useToolsets`, `use-toolset-selection` hooks, `toolsets/` pages)
- **Access Request Management**: User access requests and app access request review/approval (`useAccessRequests`, `useAppAccessRequests` hooks, `apps/access-requests/` pages)
- **API Model Management**: External API model configuration (`useApiModels` hook, `api-models/` pages)
- **User Setup**: Guided onboarding flow for first-time users (toolsets, API models, LLM engine, browser extension)
- **Settings Management**: Application settings and API token management
- **Authentication**: OAuth login/logout with session management

### UI Components

- **Design System**: Shadcn/ui components with consistent styling
- **Theme Support**: Dark/light theme toggle with system preference detection
- **Responsive Layout**: Mobile-first responsive design
- **Accessibility**: ARIA-compliant components with proper keyboard navigation

## Dependencies

### Core Framework

- `next` (14.2.30) - React framework with App Router
- `react` (^18) - React library for UI components
- `react-dom` (^18) - React DOM rendering
- `typescript` (^5) - TypeScript for type safety

### UI Framework

- `tailwindcss` (^3.4.17) - Utility-first CSS framework
- `@radix-ui/*` - Headless UI components for accessibility
- `lucide-react` (^0.515.0) - Icon library
- `class-variance-authority` (^0.7.0) - Component variant management
- `tailwind-merge` (^2.6.0) - Conditional TailwindCSS class merging

### State Management & API

- `react-query` (^3.39.3) - Server state management and caching
- `axios` (^1.9.0) - HTTP client for API requests
- `react-hook-form` (^7.57.0) - Form state management
- `zod` (^3.23.8) - Schema validation

### TypeScript Client Generation

- `@bodhiapp/ts-client` (file:../../ts-client) - Generated TypeScript types from OpenAPI specifications
- `@hey-api/openapi-ts` (^0.64.15) - OpenAPI to TypeScript type generation tool

### Content & Documentation

- `@next/mdx` (^15.3.3) - MDX support for documentation
- `@mdx-js/loader` (^3.1.0) - MDX webpack loader
- `@mdx-js/react` (^3.1.0) - MDX React integration
- `react-markdown` (^9.1.0) - Markdown rendering
- `react-syntax-highlighter` (^15.6.1) - Code syntax highlighting
- `prismjs` (^1.29.0) - Additional syntax highlighting
- `gray-matter` (^4.0.3) - Front matter parsing for markdown
- `unified` (^11.0.5) - Text processing pipeline
- `remark-gfm` (^4.0.0) - GitHub Flavored Markdown support
- `rehype-autolink-headings` (^7.1.0) - Automatic heading links
- `rehype-slug` (^6.0.0) - Heading slug generation

### Animation & UI Enhancement

- `framer-motion` (^11.18.2) - Animation library for React
- `vaul` (^1.1.2) - Drawer component library
- `cmdk` (^1.0.0) - Command palette component
- `simple-icons` (^15.1.0) - Icon library for brands and services

### Utilities

- `nanoid` (^5.0.9) - Unique ID generation
- `clsx` (^2.1.1) - Conditional className utility

### Development Tools

- `vitest` (^2.1.9) - Testing framework
- `@testing-library/react` (^16.0.0) - React testing utilities
- `@testing-library/dom` (^10.4.0) - DOM testing utilities
- `@testing-library/jest-dom` (^6.5.0) - Jest DOM matchers
- `@testing-library/user-event` (^14.5.2) - User interaction testing
- `happy-dom` (^15.11.7) - DOM environment for testing
- `msw` (^1.3.5) - Mock Service Worker for API mocking
- `eslint` (^8.57.1) - Code linting
- `prettier` (^3.3.3) - Code formatting
- `husky` (^9.1.5) - Git hooks for quality gates
- `lint-staged` (^15.5.2) - Run linters on staged files

## Architecture Position

The frontend application sits at the user interface layer, consuming the Rust backend API.

**Upstream dependencies** (backend APIs this consumes):

- [`routes_app`](../../routes_app/CLAUDE.md) -- all HTTP API endpoints (models, auth, toolsets, MCPs, access requests, settings, etc.)
- [`@bodhiapp/ts-client`](../../../../ts-client/) -- generated TypeScript types from OpenAPI spec

**Downstream consumers** (crates that embed this UI):

- [`bodhi/src-tauri`](../src-tauri/CLAUDE.md) -- Tauri desktop app embeds the static export via `lib_bodhiserver::EMBEDDED_UI_ASSETS`
- [`lib_bodhiserver`](../../lib_bodhiserver/CLAUDE.md) -- embeddable library serves the static export

## TypeScript Client Integration

### OpenAPI Type Generation Workflow

The frontend maintains type safety through automated TypeScript type generation from OpenAPI specifications:

1. **OpenAPI Generation**: The `xtask` crate generates `openapi.json` from Rust backend using utoipa
2. **Type Generation**: The `ts-client` package uses `@hey-api/openapi-ts` to generate TypeScript types
3. **Frontend Integration**: Generated types are imported and used throughout the frontend application

### ts-client Package Structure

The `@bodhiapp/ts-client` package provides:

- **Generated Types**: TypeScript interfaces matching backend API contracts
- **Request/Response Types**: Complete type coverage for all API endpoints
- **Error Types**: Structured error handling with `OpenAiApiError` type
- **Build Pipeline**: Automated generation and bundling of types

### Type Integration Patterns

Frontend components use generated types for:

- **API Responses**: `AliasResponse`, `SettingInfo`, `AppInfo`, `UserInfo`
- **Request Payloads**: `CreateAliasRequest`, `UpdateAliasRequest`, `SetupRequest`
- **Error Handling**: `OpenAiApiError` for consistent error processing
- **Pagination**: `PaginatedAliasResponse`, `PaginatedLocalModelResponse`

## Usage Patterns

### Application Entry Point

```tsx
// src/app/layout.tsx
'use client';

import { ThemeProvider } from '@/components/ThemeProvider';
import ClientProviders from '@/components/ClientProviders';
import { NavigationProvider } from '@/hooks/use-navigation';

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={cn('min-h-screen bg-background font-sans antialiased', fontSans.variable)}>
        <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
          <ClientProviders>
            <NavigationProvider items={defaultNavigationItems}>
              <div className="flex min-h-screen flex-col">
                <AppHeader />
                <main className="flex-1">{children}</main>
                <Toaster />
              </div>
            </NavigationProvider>
          </ClientProviders>
        </ThemeProvider>
      </body>
    </html>
  );
}
```

### Chat Interface Implementation

```tsx
// src/hooks/use-chat.tsx
export function useChat() {
  const [input, setInput] = useState('');
  const [abortController, setAbortController] = useState<AbortController | null>(null);
  const { append, isLoading } = useChatCompletion();
  const { currentChat, createOrUpdateChat } = useChatDB();
  const chatSettings = useChatSettings();

  const appendMessage = useCallback(
    async (content: string) => {
      const existingMessages = currentChat?.messages || [];
      const newMessages = [...existingMessages, { role: 'user', content }];

      await processCompletion(newMessages);
    },
    [currentChat, processCompletion]
  );

  const processCompletion = useCallback(
    async (userMessages: Message[]) => {
      const requestMessages = chatSettings.systemPrompt_enabled
        ? [{ role: 'system', content: chatSettings.systemPrompt }, ...userMessages]
        : userMessages;

      await append({
        request: {
          ...chatSettings.getRequestSettings(),
          messages: requestMessages,
        },
        onDelta: (chunk) => {
          // Handle streaming response chunks
          setAssistantMessage((prev) => ({
            role: 'assistant',
            content: prev.content + chunk,
          }));
        },
        onFinish: (message) => {
          // Save completed conversation to local storage
          createOrUpdateChat({
            id: currentChat?.id || nanoid(),
            title: messages[0]?.content.slice(0, 20) || 'New Chat',
            messages: [...userMessages, message],
            createdAt: currentChat?.createdAt || Date.now(),
            updatedAt: Date.now(),
          });
        },
      });
    },
    [chatSettings, currentChat, append, createOrUpdateChat]
  );

  return { input, setInput, isLoading, append: appendMessage };
}
```

### API Client Configuration

```tsx
// src/lib/apiClient.ts
const apiClient = axios.create({
  baseURL: isTest ? 'http://localhost:3000' : '',
  maxRedirects: 0,
});
```

Note: In tests, `baseURL` is set to `http://localhost:3000` (via `tests/setup.ts`). In production, `baseURL` is empty string `''` (relative to current origin).

### Type-Safe API Integration

```tsx
// src/hooks/useQuery.ts - Generic hooks with ts-client types
import { AliasResponse, AppInfo, SettingInfo, OpenAiApiError } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';
import { useQuery as useReactQuery, useMutation } from 'react-query';

// Type alias for consistent error handling
type ErrorResponse = OpenAiApiError;

// Generic query hook with generated types
export function useQuery<T>(
  key: string | string[],
  endpoint: string,
  params?: Record<string, any>,
  options?: UseQueryOptions<T, AxiosError<ErrorResponse>>
): UseQueryResult<T, AxiosError<ErrorResponse>> {
  return useReactQuery<T, AxiosError<ErrorResponse>>(
    key,
    async () => {
      const { data } = await apiClient.get<T>(endpoint, { params });
      return data;
    },
    options
  );
}

// Specific hooks using generated types
export function useAppInfo() {
  return useQuery<AppInfo>('appInfo', '/bodhi/v1/info');
}

export function useModels(page: number, pageSize: number, sort: string, sortOrder: string) {
  return useQuery<PaginatedAliasResponse>(
    ['models', page.toString(), pageSize.toString(), sort, sortOrder],
    '/bodhi/v1/models',
    { page, page_size: pageSize, sort, sort_order: sortOrder }
  );
}
```

### Schema Integration with Generated Types

```tsx
// src/schemas/alias.ts - Zod schemas with ts-client type integration
import * as z from 'zod';
import type { AliasResponse, CreateAliasRequest, UpdateAliasRequest } from '@bodhiapp/ts-client';

// Re-export generated types for use throughout the app
export type { AliasResponse, CreateAliasRequest, UpdateAliasRequest };

// Zod schema for form validation
export const createAliasFormSchema = z.object({
  alias: z.string().min(1, 'Alias is required'),
  repo: z.string().min(1, 'Repo is required'),
  filename: z.string().min(1, 'Filename is required'),
  request_params: requestParamsSchema,
  context_params: z.string().optional(),
});

export type AliasFormData = z.infer<typeof createAliasFormSchema>;

// Conversion functions between form and API formats
export const convertFormToApi = (formData: AliasFormData): CreateAliasRequest => ({
  alias: formData.alias,
  repo: formData.repo,
  filename: formData.filename,
  request_params: formData.request_params,
  context_params: formData.context_params
    ? formData.context_params
        .split('\n')
        .map((line) => line.trim())
        .filter((line) => line.length > 0)
    : undefined,
});

export const convertApiToForm = (apiData: AliasResponse): AliasFormData => ({
  alias: apiData.alias,
  repo: apiData.repo,
  filename: apiData.filename,
  request_params: apiData.request_params || {},
  context_params: Array.isArray(apiData.context_params) ? apiData.context_params.join('\n') : '',
});
```

### Component Development with Generated Types

```tsx
// src/app/ui/models/page.tsx - Using generated types in components
import { AliasResponse } from '@bodhiapp/ts-client';
import { useModels } from '@/hooks/useQuery';

export default function ModelsPage() {
  const { data: modelsData, isLoading, error } = useModels(page, pageSize, sort, sortOrder);

  const handleEditModel = (model: AliasResponse) => {
    // Type-safe model editing with generated types
    router.push(`/ui/models/edit?alias=${encodeURIComponent(model.alias)}`);
  };

  return (
    <DataTable
      data={modelsData?.data || []}
      columns={[
        {
          key: 'alias',
          header: 'Alias',
          render: (model: AliasResponse) => model.alias,
        },
        {
          key: 'repo',
          header: 'Repository',
          render: (model: AliasResponse) => model.repo,
        },
      ]}
    />
  );
}
```

### Chat UI Implementation

```tsx
// src/app/ui/chat/page.tsx
'use client';

import { ChatUI } from './ChatUI';
import { useChatDB } from '@/hooks/use-chat-db';
import { useChat } from '@/hooks/use-chat';

export default function ChatPage() {
  const { currentChat, chats, setCurrentChatId, createNewChat } = useChatDB();
  const chat = useChat();

  return (
    <div className="flex h-full">
      <ChatHistory
        chats={chats}
        currentChatId={currentChat?.id}
        onSelectChat={setCurrentChatId}
        onNewChat={createNewChat}
      />
      <div className="flex-1">
        <ChatUI
          messages={currentChat?.messages || []}
          input={chat.input}
          setInput={chat.setInput}
          isLoading={chat.isLoading}
          onSubmit={chat.append}
          onStop={chat.stop}
          userMessage={chat.userMessage}
          assistantMessage={chat.assistantMessage}
        />
      </div>
    </div>
  );
}
```

## Page Structure and Routing

### App Router Organization

```
src/app/ui/
├── home/                        # Dashboard/home page
├── chat/                        # Chat interface with tool calling
├── models/                      # Model alias management (list, new, edit)
├── modelfiles/                  # Modelfiles page
├── pull/                        # Model pull interface
├── api-models/                  # API model config (new, edit)
├── mcps/                        # MCP server management (list, new, edit)
├── mcp-servers/                 # MCP server allowlist and auth config management
│   └── view/                    # Server view page (server info + auth config list)
├── toolsets/                    # Toolset management (list, new, edit, admin)
├── tokens/                      # API token management
├── settings/                    # Application settings
├── users/                       # User management (list, pending, access-requests)
├── apps/                        # App access request review
│   └── access-requests/review/
├── request-access/              # User access request form
├── setup/                       # Onboarding flow (download-models, toolsets, api-models, llm-engine, browser-extension, complete)
├── login/                       # Login page
└── auth/callback/               # OAuth callback
```

### Component Architecture

```
src/components/
├── ui/                          # Shadcn/ui base components
│   ├── button.tsx
│   ├── input.tsx
│   ├── dialog.tsx
│   └── ...
├── navigation/                  # Navigation components
│   ├── AppHeader.tsx
│   ├── AppNavigation.tsx
│   └── AppBreadcrumb.tsx
├── ClientProviders.tsx          # Client-side providers
├── ThemeProvider.tsx           # Theme management
└── UserOnboarding.tsx          # Onboarding flow
```

### Custom Hooks

```
src/hooks/
├── use-chat.tsx                # Main chat functionality with tool calling
├── use-chat-completions.ts     # Chat API integration
├── use-chat-db.tsx             # Local chat storage
├── use-chat-settings.tsx       # Chat configuration
├── useMcps.ts                  # MCP server CRUD, tool discovery
├── useToolsets.ts              # Toolset CRUD, type management
├── use-toolset-selection.ts    # Toolset selection state for chat
├── useAccessRequests.ts        # User access request management
├── useAppAccessRequests.ts     # App access request review/approve/deny
├── useApiModels.ts             # API model configuration
├── useApiTokens.ts             # API token management
├── useModels.ts                # Model alias management
├── useModelMetadata.ts         # Model metadata refresh
├── useModelCatalog.ts          # Model catalog browsing
├── useUsers.ts                 # User management
├── useSettings.ts              # Settings management
├── useInfo.ts                  # App info
├── useAuth.ts                  # OAuth authentication
├── use-navigation.tsx          # Navigation state
├── use-toast-messages.ts       # Toast notifications
└── useQuery.ts                 # Generic query/mutation hooks
```

## State Management

### Local Storage for Chat History

```tsx
// src/hooks/use-chat-db.tsx
export function useChatDB() {
  const [chats, setChats] = useLocalStorage<Chat[]>('bodhi-chats', []);
  const [currentChatId, setCurrentChatId] = useLocalStorage<string | null>('bodhi-current-chat', null);

  const createOrUpdateChat = useCallback(
    (chat: Chat) => {
      setChats((prevChats) => {
        const existingIndex = prevChats.findIndex((c) => c.id === chat.id);
        if (existingIndex >= 0) {
          const newChats = [...prevChats];
          newChats[existingIndex] = chat;
          return newChats;
        }
        return [chat, ...prevChats];
      });
    },
    [setChats]
  );

  const deleteChat = useCallback(
    (chatId: string) => {
      setChats((prevChats) => prevChats.filter((c) => c.id !== chatId));
      if (currentChatId === chatId) {
        setCurrentChatId(null);
      }
    },
    [setChats, currentChatId, setCurrentChatId]
  );

  return {
    chats,
    currentChat: chats.find((c) => c.id === currentChatId) || null,
    currentChatId,
    setCurrentChatId,
    createOrUpdateChat,
    deleteChat,
    createNewChat: () => setCurrentChatId(null),
  };
}
```

### React Query for Server State

```tsx
// src/hooks/useQuery.ts
import { useQuery as useReactQuery, QueryClient } from 'react-query';
import { apiClient } from '@/lib/apiClient';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      cacheTime: 10 * 60 * 1000, // 10 minutes
    },
  },
});

export function useModels() {
  return useReactQuery(['models'], () => apiClient.listModels(), {
    refetchOnWindowFocus: false,
  });
}

export function useSettings() {
  return useReactQuery(['settings'], () => apiClient.getSettings());
}
```

## Styling and Design System

### TailwindCSS Configuration

```typescript
// tailwind.config.ts
import type { Config } from 'tailwindcss';

const config: Config = {
  darkMode: ['class'],
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        border: 'hsl(var(--border))',
        input: 'hsl(var(--input))',
        ring: 'hsl(var(--ring))',
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        primary: {
          DEFAULT: 'hsl(var(--primary))',
          foreground: 'hsl(var(--primary-foreground))',
        },
        // ... other theme colors
      },
      borderRadius: {
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
      },
    },
  },
  plugins: [require('tailwindcss-animate'), require('@tailwindcss/typography')],
};
```

### CSS Custom Properties

```css
/* src/app/globals.css */
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;
    --primary: 222.2 47.4% 11.2%;
    --primary-foreground: 210 40% 98%;
    /* ... other CSS variables */
  }

  .dark {
    --background: 222.2 84% 4.9%;
    --foreground: 210 40% 98%;
    --primary: 210 40% 98%;
    --primary-foreground: 222.2 47.4% 11.2%;
    /* ... dark theme variables */
  }
}
```

## Testing Strategy

### Vitest Configuration

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import tsconfigPaths from 'vite-tsconfig-paths';

export default defineConfig({
  plugins: [react(), tsconfigPaths()],
  test: {
    environment: 'happy-dom',
    setupFiles: ['./src/tests/setup.ts'],
    coverage: {
      reporter: ['text', 'json', 'html'],
      exclude: ['node_modules/', 'src/tests/'],
    },
  },
});
```

### Component Testing

```tsx
// src/components/ui/button.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { Button } from './button';

describe('Button Component', () => {
  it('renders with default variant', () => {
    render(<Button>Click me</Button>);
    const button = screen.getByRole('button', { name: /click me/i });
    expect(button).toBeInTheDocument();
    expect(button).toHaveClass('bg-primary');
  });

  it('calls onClick handler when clicked', () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Click me</Button>);
    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});
```

### Hook Testing

```tsx
// src/hooks/use-chat.test.tsx
import { renderHook, act } from '@testing-library/react';
import { useChat } from './use-chat';
import { TestWrapper } from '@/tests/wrapper';

describe('useChat', () => {
  it('should handle message submission', async () => {
    const { result } = renderHook(() => useChat(), {
      wrapper: TestWrapper,
    });

    act(() => {
      result.current.setInput('Hello, world!');
    });

    expect(result.current.input).toBe('Hello, world!');

    await act(async () => {
      await result.current.append('Hello, world!');
    });

    expect(result.current.input).toBe('');
  });
});
```

## TypeScript Client Generation Workflow

### OpenAPI to TypeScript Pipeline

The frontend maintains type safety through a multi-step generation process:

1. **Backend OpenAPI Generation**:

   ```bash
   # Generate OpenAPI spec from Rust backend
   cargo run --package xtask openapi
   ```

   - Uses `utoipa` annotations in Rust code
   - Generates `openapi.json` at project root
   - Includes all API endpoints, request/response schemas, and error types

2. **TypeScript Type Generation**:

   ```bash
   # In ts-client directory
   npm run generate:types
   ```

   - Uses `@hey-api/openapi-ts` with configuration from `openapi-ts.config.ts`
   - Reads `../openapi.json` as input
   - Generates TypeScript types in `src/types/` directory

3. **Client Package Building**:

   ```bash
   # Build and bundle the ts-client package
   npm run build
   ```

   - Bundles types into ESM and CommonJS formats
   - Creates distribution files in `dist/` directory
   - Generates type declarations for consumption

4. **Frontend Integration**:
   - Frontend imports types from `@bodhiapp/ts-client` package
   - Uses file-based dependency: `"@bodhiapp/ts-client": "file:../../ts-client"`
   - Provides compile-time type safety for all API interactions

### openapi-ts Configuration

```typescript
// ts-client/openapi-ts.config.ts
import { defineConfig } from '@hey-api/openapi-ts';

export default defineConfig({
  input: '../openapi.json',
  output: 'src/types',
  plugins: ['@hey-api/typescript'],
});
```

### Generated Type Usage Patterns

The generated types provide comprehensive coverage:

- **Request Types**: `CreateAliasRequest`, `UpdateAliasRequest`, `SetupRequest`
- **Response Types**: `AliasResponse`, `AppInfo`, `UserInfo`, `SettingInfo`
- **Paginated Responses**: `PaginatedAliasResponse`, `PaginatedLocalModelResponse`
- **Error Types**: `OpenAiApiError` with structured error information
- **Enum Types**: Status codes, model types, and configuration options

### Contract Binding Benefits

This approach ensures:

- **Compile-Time Safety**: TypeScript catches API contract violations at build time
- **Automatic Updates**: Type changes in backend automatically propagate to frontend
- **IDE Support**: Full IntelliSense and autocomplete for API interactions
- **Refactoring Safety**: Breaking changes in API surface immediately in frontend code
- **Documentation**: Generated types serve as living API documentation

## Build and Deployment

### Next.js Configuration

```javascript
// next.config.mjs
const nextConfig = {
  reactStrictMode: true,
  output: 'export', // Static export for desktop embedding
  trailingSlash: true,
  transpilePackages: ['geist'],
  productionBrowserSourceMaps: true,
  images: {
    unoptimized: true, // Required for static export
  },
  eslint: {
    ignoreDuringBuilds: true,
  },
};
```

### Build Scripts

```json
// package.json
{
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "test": "vitest run",
    "lint": "eslint . --ext .js,.jsx,.ts,.tsx",
    "format": "prettier --write .",
    "prepare": "husky"
  }
}
```

### Development Workflow Integration

The frontend development workflow integrates with the type generation pipeline:

1. **Backend Changes**: When backend API changes, run `cargo run --package xtask openapi`
2. **Type Updates**: In `ts-client/`, run `npm run build` to regenerate and bundle types
3. **Frontend Development**: Types are automatically available in frontend through file dependency
4. **Testing**: Run `npm run test` to verify type safety and functionality
5. **Linting**: `npm run lint` and `npm run format` maintain code quality

### Production Build Process

1. **Static Export**: Next.js builds a static site for embedding in desktop app
2. **Asset Optimization**: Images and assets are optimized for production
3. **Code Splitting**: Automatic code splitting for optimal loading
4. **Source Maps**: Generated for debugging in production

## Development Guidelines

### Component Development

1. Use TypeScript for all components with proper type definitions
2. Follow Shadcn/ui patterns for consistent styling
3. Implement proper accessibility with ARIA attributes
4. Include comprehensive tests for all components
5. Use proper error boundaries for error handling

### Type-Safe API Integration

1. **Always use generated types** from `@bodhiapp/ts-client` for API interactions
2. **Re-export types** in schema files for consistent usage patterns
3. **Combine with Zod schemas** for form validation while maintaining type safety
4. **Use conversion functions** between form data and API request formats
5. **Handle errors consistently** using `OpenAiApiError` type

### State Management Best Practices

1. Use React Query for server state management with generated types
2. Local storage for persistent client state
3. Context providers for global application state
4. Custom hooks for reusable state logic
5. Proper cleanup in useEffect hooks

### API Development Workflow

1. **Backend First**: Make API changes in Rust backend with utoipa annotations
2. **Generate OpenAPI**: Run `cargo run --package xtask openapi` to update spec
3. **Update Types**: Run `npm run build` in `ts-client/` to regenerate TypeScript types
4. **Frontend Integration**: Import and use updated types in frontend components
5. **Test Integration**: Verify type safety and API contracts with comprehensive tests

### Performance Optimization

1. Lazy loading for heavy components
2. Memoization with React.memo and useMemo
3. Proper dependency arrays in useEffect and useCallback
4. Code splitting at route level
5. Image optimization and lazy loading

### Code Quality

- TypeScript strict mode enabled
- ESLint configuration with Next.js rules
- Prettier for consistent code formatting
- Husky pre-commit hooks for quality gates
- Comprehensive testing with Vitest
- Type safety enforced through generated API contracts

## Security Considerations

### Client-Side Security

- Secure token storage and transmission
- Input validation and sanitization
- XSS prevention in markdown rendering
- CSRF protection for API requests
- Secure HTTP headers implementation

### Authentication Flow

- OAuth2 integration with secure callback handling
- JWT token management with proper expiration
- Session cleanup on logout
- Automatic token refresh handling

## Future Extensions

The frontend application can be extended with:

- **WebSocket Integration**: Real-time updates for chat completions and model status
- **Progressive Web App (PWA)**: Offline capabilities and app-like experience
- **Advanced Chat Features**: File uploads, voice input, conversation branching
- **Enhanced Accessibility**: Screen reader optimization, keyboard navigation improvements
- **Internationalization (i18n)**: Multi-language support with react-i18next
- **Advanced Analytics**: User interaction tracking and performance monitoring
- **Plugin System**: Extensible architecture for third-party integrations
- **Collaborative Features**: Shared conversations and team workspaces

## Type Safety Maintenance

### Keeping Types in Sync

To maintain type safety across the application:

1. **Regular Regeneration**: Run type generation after backend changes
2. **CI/CD Integration**: Automate type generation in build pipelines
3. **Version Pinning**: Keep `@bodhiapp/ts-client` version aligned with backend
4. **Breaking Change Detection**: Use TypeScript compiler to catch API contract violations
5. **Documentation Updates**: Update component documentation when types change

### Best Practices for Type Usage

- **Import from ts-client**: Always import types from the generated package
- **Avoid Type Assertions**: Use proper type guards and validation instead
- **Schema Validation**: Combine generated types with Zod for runtime validation
- **Error Handling**: Use structured error types for consistent error processing
- **Testing**: Include type checking in test suites to catch regressions
