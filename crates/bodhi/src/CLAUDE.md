# CLAUDE.md - bodhi/src

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
- **Chat Interface**: Real-time chat with streaming responses and message history
- **Model Management**: Create, edit, and manage model aliases and configurations
- **User Setup**: Guided onboarding flow for first-time users
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

### Content & Documentation
- `@next/mdx` (^15.3.3) - MDX support for documentation
- `react-markdown` (^9.1.0) - Markdown rendering
- `react-syntax-highlighter` (^15.6.1) - Code syntax highlighting
- `prismjs` (^1.29.0) - Additional syntax highlighting

### Development Tools
- `vitest` (^2.1.9) - Testing framework
- `@testing-library/react` (^16.0.0) - React testing utilities
- `eslint` (^8.57.1) - Code linting
- `prettier` (^3.3.3) - Code formatting

## Architecture Position

The frontend application sits at the user interface layer:
- **Provides**: Complete web-based user interface for BodhiApp
- **Communicates**: With BodhiApp HTTP API endpoints for all functionality
- **Manages**: Client-side state, authentication, and user experience
- **Renders**: Static export compatible with desktop embedding

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

  const appendMessage = useCallback(async (content: string) => {
    const existingMessages = currentChat?.messages || [];
    const newMessages = [...existingMessages, { role: 'user', content }];

    await processCompletion(newMessages);
  }, [currentChat, processCompletion]);

  const processCompletion = useCallback(async (userMessages: Message[]) => {
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
        setAssistantMessage(prev => ({
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
  }, [chatSettings, currentChat, append, createOrUpdateChat]);

  return { input, setInput, isLoading, append: appendMessage };
}
```

### API Client Configuration
```tsx
// src/lib/apiClient.ts
import axios, { AxiosInstance } from 'axios';

class ApiClient {
  private client: AxiosInstance;

  constructor(baseURL: string = '/api') {
    this.client = axios.create({
      baseURL,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Request interceptor for authentication
    this.client.interceptors.request.use(
      (config) => {
        const token = localStorage.getItem('auth_token');
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          // Handle authentication errors
          localStorage.removeItem('auth_token');
          window.location.href = '/ui/login';
        }
        return Promise.reject(error);
      }
    );
  }

  // Chat completions with streaming support
  async chatCompletion(request: ChatCompletionRequest): Promise<Response> {
    return fetch(`${this.client.defaults.baseURL}/v1/chat/completions`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${localStorage.getItem('auth_token')}`,
      },
      body: JSON.stringify(request),
    });
  }

  // Model management
  async listModels(): Promise<Model[]> {
    const response = await this.client.get('/v1/models');
    return response.data.data;
  }

  async createModel(request: CreateModelRequest): Promise<void> {
    await this.client.post('/v1/models', request);
  }
}

export const apiClient = new ApiClient();
```

### Component Development with Shadcn/ui
```tsx
// src/components/ui/button.tsx
import { Slot } from '@radix-ui/react-slot';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '@/lib/utils';

const buttonVariants = cva(
  'inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground shadow hover:bg-primary/90',
        destructive: 'bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90',
        outline: 'border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground',
        secondary: 'bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80',
        ghost: 'hover:bg-accent hover:text-accent-foreground',
        link: 'text-primary underline-offset-4 hover:underline',
      },
      size: {
        default: 'h-9 px-4 py-2',
        sm: 'h-8 rounded-md px-3 text-xs',
        lg: 'h-10 rounded-md px-8',
        icon: 'h-9 w-9',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button';
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    );
  }
);
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
src/app/
├── layout.tsx                    # Root layout with providers
├── page.tsx                      # Landing page
├── globals.css                   # Global styles
├── ui/                          # Main application routes
│   ├── home/page.tsx            # Dashboard/home page
│   ├── chat/page.tsx            # Chat interface
│   ├── models/                  # Model management
│   │   ├── page.tsx             # Models list
│   │   ├── new/page.tsx         # Create new model
│   │   └── edit/page.tsx        # Edit model
│   ├── pull/page.tsx            # Model pull interface
│   ├── settings/page.tsx        # Application settings
│   ├── tokens/page.tsx          # API token management
│   ├── login/page.tsx           # Login page
│   ├── setup/                   # Onboarding flow
│   │   ├── page.tsx             # Setup start
│   │   ├── llm-engine/page.tsx  # LLM engine selection
│   │   ├── download-models/page.tsx # Model downloads
│   │   └── complete/page.tsx    # Setup completion
│   └── auth/
│       └── callback/page.tsx    # OAuth callback
└── docs/                        # Documentation system
    ├── layout.tsx               # Docs layout
    ├── page.tsx                 # Docs index
    └── [...slug]/page.tsx       # Dynamic doc pages
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
├── use-chat.tsx                # Main chat functionality
├── use-chat-completions.ts     # Chat API integration
├── use-chat-db.tsx             # Local chat storage
├── use-chat-settings.tsx       # Chat configuration
├── use-navigation.tsx          # Navigation state
├── useOAuth.ts                 # OAuth authentication
├── useApiTokens.ts             # API token management
└── use-toast-messages.ts       # Toast notifications
```

## State Management

### Local Storage for Chat History
```tsx
// src/hooks/use-chat-db.tsx
export function useChatDB() {
  const [chats, setChats] = useLocalStorage<Chat[]>('bodhi-chats', []);
  const [currentChatId, setCurrentChatId] = useLocalStorage<string | null>('bodhi-current-chat', null);

  const createOrUpdateChat = useCallback((chat: Chat) => {
    setChats(prevChats => {
      const existingIndex = prevChats.findIndex(c => c.id === chat.id);
      if (existingIndex >= 0) {
        const newChats = [...prevChats];
        newChats[existingIndex] = chat;
        return newChats;
      }
      return [chat, ...prevChats];
    });
  }, [setChats]);

  const deleteChat = useCallback((chatId: string) => {
    setChats(prevChats => prevChats.filter(c => c.id !== chatId));
    if (currentChatId === chatId) {
      setCurrentChatId(null);
    }
  }, [setChats, currentChatId, setCurrentChatId]);

  return {
    chats,
    currentChat: chats.find(c => c.id === currentChatId) || null,
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

## Build and Deployment

### Next.js Configuration
```javascript
// next.config.mjs
const nextConfig = {
  reactStrictMode: true,
  output: 'export',                    // Static export for desktop embedding
  trailingSlash: true,
  transpilePackages: ['geist'],
  productionBrowserSourceMaps: true,
  images: {
    unoptimized: true,                 // Required for static export
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
    "format": "prettier --write ."
  }
}
```

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

### State Management Best Practices
1. Use React Query for server state management
2. Local storage for persistent client state
3. Context providers for global application state
4. Custom hooks for reusable state logic
5. Proper cleanup in useEffect hooks

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
- WebSocket integration for real-time updates
- Progressive Web App (PWA) capabilities
- Advanced chat features (file uploads, voice input)
- Enhanced accessibility features
- Internationalization (i18n) support
- Advanced analytics and user tracking