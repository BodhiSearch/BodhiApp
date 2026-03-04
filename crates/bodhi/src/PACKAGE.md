# bodhi/src — PACKAGE.md

> See [CLAUDE.md](./CLAUDE.md) for architecture and critical rules
> See [TESTING.md](./TESTING.md) for test patterns and fixtures

## Directory Structure

### App Router (`src/app/`)

```
src/app/
├── layout.tsx                    # Root layout with ThemeProvider, ClientProviders, NavigationProvider
├── page.tsx                      # Root redirect to /ui
├── globals.css                   # Global Tailwind styles
├── ui/                           # Main application route group
│   ├── page.tsx                  # UI entry with AppInitializer
│   ├── home/page.tsx             # Dashboard
│   ├── chat/                     # Chat interface with tool calling
│   │   ├── page.tsx              # Chat page
│   │   ├── ChatUI.tsx            # Chat UI layout
│   │   ├── ChatMessage.tsx       # Message rendering
│   │   ├── ChatHistory.tsx       # Chat sidebar
│   │   ├── ToolCallMessage.tsx   # Tool call display
│   │   ├── ToolsetsPopover.tsx   # Toolset picker for chat
│   │   └── settings/            # Chat settings sidebar
│   ├── models/                   # Model alias CRUD (list, new, edit)
│   │   ├── AliasForm.tsx         # Shared alias form component
│   │   └── components/ModelPreviewModal.tsx
│   ├── modelfiles/page.tsx       # Modelfiles
│   ├── pull/page.tsx             # Model download
│   ├── api-models/               # External API model config (new, edit)
│   ├── mcps/                     # MCP instance management
│   │   ├── new/                  # New MCP with server selector, tool selection
│   │   ├── playground/           # MCP playground
│   │   └── oauth/callback/       # OAuth callback for MCP auth
│   ├── mcp-servers/              # MCP server allowlist
│   │   ├── new/page.tsx          # New server + optional auth config
│   │   ├── view/page.tsx         # Server view + auth config list
│   │   ├── edit/page.tsx         # Edit server
│   │   └── components/AuthConfigForm.tsx
│   ├── toolsets/                  # Toolset CRUD + admin
│   ├── tokens/                   # API token management
│   ├── settings/page.tsx         # Application settings
│   ├── users/                    # User management (list, pending, access-requests)
│   ├── apps/access-requests/review/ # App access request review
│   ├── request-access/page.tsx   # User access request form
│   ├── setup/                    # Multi-step onboarding flow
│   │   ├── download-models/      # Model download step
│   │   ├── toolsets/             # Toolset setup step
│   │   ├── api-models/           # API model setup step
│   │   ├── llm-engine/           # LLM engine setup step
│   │   ├── browser-extension/    # Browser extension step
│   │   ├── complete/             # Setup complete
│   │   ├── resource-admin/       # Resource admin page
│   │   └── components/           # SetupProvider, SetupCard, etc.
│   ├── login/page.tsx            # Login page
│   └── auth/callback/page.tsx    # OAuth callback
└── docs/                         # Documentation system
    ├── [...slug]/page.tsx        # Dynamic docs routing
    └── layout.tsx                # Docs layout with sidebar
```

### Components (`src/components/`)

```
src/components/
├── ui/                          # Shadcn/ui base components (button, input, dialog, etc.)
├── navigation/                  # AppHeader, AppNavigation, AppBreadcrumb
├── api-models/                  # ApiModelForm, providers, form fields, actions
├── users/                       # UsersTable, UserRow, RoleChangeDialog, RemoveUserDialog
├── setup/                       # BrowserExtensionCard, BrowserSelector
├── AppInitializer.tsx           # App status check and routing
├── ClientProviders.tsx          # QueryClientProvider wrapper
├── ThemeProvider.tsx             # Dark/light theme
├── DataTable.tsx                # Generic data table
├── ModelSelector.tsx            # Model alias selector
├── UserManagementTabs.tsx       # User/access tabs
├── McpManagementTabs.tsx        # MCP instances/servers tabs
├── LoginMenu.tsx                # Login/logout menu
├── DeleteConfirmDialog.tsx      # Reusable delete confirmation
├── Combobox.tsx                 # Generic combobox
├── AutocompleteInput.tsx        # Autocomplete input
└── CopyButton.tsx / CopyableContent.tsx
```

### Hooks (`src/hooks/`)

| Hook                         | Purpose                                                                   |
| ---------------------------- | ------------------------------------------------------------------------- |
| `use-chat.tsx`               | Chat orchestration: message submission, streaming, tool calling           |
| `use-chat-completions.ts`    | SSE streaming API integration                                             |
| `use-chat-db.tsx`            | Chat persistence via localStorage                                         |
| `use-chat-settings.tsx`      | Chat config (model, system prompt, params, API token)                     |
| `use-toolset-selection.ts`   | Toolset selection state for chat                                          |
| `useQuery.ts`                | Generic `useQuery<T>` / `useMutation` wrappers with OpenAiApiError typing |
| `useModels.ts`               | Model alias CRUD hooks                                                    |
| `useModelMetadata.ts`        | Model metadata refresh                                                    |
| `useModelCatalog.ts`         | Model catalog browsing                                                    |
| `useMcps.ts`                 | MCP instance CRUD, tool discovery                                         |
| `useToolsets.ts`             | Toolset CRUD, type management                                             |
| `useApiModels.ts`            | API model configuration hooks                                             |
| `useApiTokens.ts`            | API token management                                                      |
| `useUsers.ts`                | User management hooks                                                     |
| `useAccessRequests.ts`       | User access request management                                            |
| `useAppAccessRequests.ts`    | App access request review/approve/deny                                    |
| `useSettings.ts`             | Settings management                                                       |
| `useInfo.ts`                 | App info hook                                                             |
| `useAuth.ts`                 | OAuth authentication                                                      |
| `use-navigation.tsx`         | Navigation state provider                                                 |
| `use-toast-messages.ts`      | Toast notification helpers                                                |
| `use-toast.ts`               | Toast primitive                                                           |
| `useLocalStorage.ts`         | localStorage persistence hook                                             |
| `use-mobile.tsx`             | Mobile responsive detection                                               |
| `use-media-query.ts`         | Media query hook                                                          |
| `use-browser-detection.ts`   | Browser type detection                                                    |
| `use-extension-detection.ts` | Browser extension detection                                               |
| `use-responsive-testid.tsx`  | Responsive test ID helper                                                 |

### Schemas (`src/schemas/`)

- `alias.ts` — Model alias form schema, form↔API conversion
- `apiModel.ts` — API model form schema
- `apiModels.ts` — API models list schema
- `pull.ts` — Model pull form schema
- `objs.ts` — Common object schemas

### Types (`src/types/`)

- `chat.ts` — Chat message interfaces
- `models.ts` — Model type definitions
- `navigation.ts` — Navigation item types

### Stores (`src/stores/`)

- `mcpFormStore.ts` — MCP form state persistence via sessionStorage (used by MCP new page and OAuth callback)

### Lib (`src/lib/`)

- `apiClient.ts` — Axios instance configuration
- `queryClient.ts` — React Query client configuration
- `constants.ts` — Route constants (ROUTE_CHAT, ROUTE_SETUP, etc.)
- `roles.ts` — Role utilities
- `utils.ts` — General utilities (cn() for className merging)
- `toolsets.ts` — Toolset utility functions
- `mcpUtils.ts` — MCP utility functions
- `urlUtils.ts` — URL utility functions
- `browser-utils.ts` — Browser detection utilities

## Configuration

- `next.config.mjs` — Static export (`output: 'export'`), trailing slashes, MDX support
- `tsconfig.json` — Strict TypeScript, `@/` path alias to `src/`
- `tailwind.config.ts` — Shadcn/ui theme with CSS custom properties
- `vitest.config.ts` — happy-dom environment, setup file at `src/tests/setup.ts`
