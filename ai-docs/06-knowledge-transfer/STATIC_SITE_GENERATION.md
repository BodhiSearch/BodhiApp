# Static Site Generation for React Router

This project implements static site generation (SSG) for React Router applications, enabling direct URL access without 404 errors when serving from static file servers.

## Overview

When building a React Router application, the default build process creates a single `index.html` file. This works fine for client-side routing, but causes 404 errors when users navigate directly to routes like `/ui/login` or `/ui/chat` because the server doesn't know about these routes.

Our solution generates individual `index.html` files for each route, allowing direct URL access while preserving client-side routing functionality.

## Architecture Pattern

The project follows a consistent page organization pattern:

- **Pages folder** (`src/pages/`): Contains root navigable components/pages
- **Components folder** (`src/components/`): Contains the actual page implementation
- **Lazy loading**: Pages use React.lazy() to load components dynamically

### Example Structure
```
src/pages/SettingsPage.tsx     -> Root page component (lazy loads)
src/components/settings/SettingsPage.tsx -> Actual implementation
```

This pattern ensures all navigable routes are discoverable in the pages folder.

## Implementation

### 1. Dynamic Route Discovery

The static generation script automatically discovers routes by scanning the `src/pages/` directory:

```bash
npm run build
# Equivalent to:
# vite build && npm run build:static
```

1. **Vite Build**: Creates the standard build output in `dist/`
2. **Route Discovery**: Scans `src/pages/` for `.tsx` files
3. **Route Conversion**: Converts page file names to route paths
4. **Static Generation**: Creates `index.html` files for each route

### 2. Route Conversion Logic

Page files are converted to routes using these rules:

- `HomePage.tsx` → `/ui`
- `ChatPage.tsx` → `/ui/chat`
- `OAuthCallbackPage.tsx` → `/ui/auth/callback` (special case)
- `ModelFilesPage.tsx` → `/ui/modelfiles` (special case)
- `docs/DocsMainPage.tsx` → `/docs`
- `NotFoundPage.tsx` → skipped (404 page)

Special routes are added manually for pages that serve multiple routes:
- `/ui/home` (HomePage also serves this)
- `/ui/models/new` and `/ui/models/edit` (ModelsPage serves these)

### 3. Generated Structure

After building, the `dist/` directory contains:

```
dist/
├── index.html                    # Root route
├── assets/                       # JS/CSS bundles
├── ui/
│   ├── index.html               # /ui route
│   ├── login/
│   │   └── index.html           # /ui/login route
│   ├── chat/
│   │   └── index.html           # /ui/chat route
│   ├── auth/
│   │   └── callback/
│   │       └── index.html       # /ui/auth/callback route
│   └── ...                      # Other UI routes
└── docs/
    └── index.html               # /docs route
```

### 4. Supported Routes

Routes are automatically discovered from the pages directory. Current routes include:

- `/` (root)
- `/ui` (home page)
- `/ui/home`
- `/ui/chat` (supports query parameters: `?id=<chat-id>&alias=<model-alias>`)
- `/ui/models`
- `/ui/models/new`
- `/ui/models/edit`
- `/ui/modelfiles`
- `/ui/pull`
- `/ui/login`
- `/ui/auth/callback`
- `/ui/settings`
- `/ui/tokens`
- `/ui/users`
- `/ui/setup`
- `/docs`

#### Chat URL Parameters

The chat page supports query parameters for deep linking to specific chats:

- `/ui/chat/?id=abc123` - Opens a specific chat conversation
- `/ui/chat/?alias=llama3` - Opens chat with a specific model pre-selected
- `/ui/chat/?id=abc123&alias=llama3` - Opens specific chat with model

## Scripts

### Build Scripts

- `npm run build` - Full build with static generation
- `npm run build:vite` - Vite build only
- `npm run build:static` - Static generation only (auto-discovers routes)

## How It Works

### 1. Asset References

All generated HTML files contain identical content with absolute asset paths:

```html
<script type="module" crossorigin src="/assets/index-BwzDBtwD.js"></script>
<link rel="stylesheet" crossorigin href="/assets/index-Bhqc94pR.css">
```

The absolute paths (starting with `/`) ensure assets load correctly from any subdirectory.

### 2. Client-Side Routing

Once the JavaScript loads, React Router takes over and handles navigation client-side. The static HTML files serve as entry points that bootstrap the application.

### 3. Chat URL Synchronization

The chat page automatically synchronizes the current chat with the URL:

- **When selecting a chat**: URL updates to `/ui/chat/?id=<chat-id>`
- **When accessing a chat URL**: Application loads the specified chat
- **When creating a new chat**: URL updates with the new chat ID
- **Invalid chat IDs**: Automatically removed from URL

This enables:
- **Bookmarking specific chats**
- **Sharing chat links**
- **Browser back/forward navigation**
- **Direct URL access to chats**

### 4. Server Compatibility

This approach works with any static file server:

```bash
# Python
cd dist && python -m http.server 8000

# Node.js
cd dist && npx serve -s .

# PHP
cd dist && php -S localhost:8000

# Nginx, Apache, etc.
```

## Adding New Pages

To add a new page that supports static site generation:

1. **Create the page component** in `src/pages/`:
   ```typescript
   // src/pages/NewFeaturePage.tsx
   import { lazy, Suspense } from 'react';

   const NewFeaturePageContent = lazy(() => import('@/components/new-feature/NewFeaturePage'));

   export default function NewFeaturePage() {
     return (
       <Suspense fallback={<div>Loading...</div>}>
         <NewFeaturePageContent />
       </Suspense>
     );
   }
   ```

2. **Create the implementation** in `src/components/`:
   ```typescript
   // src/components/new-feature/NewFeaturePage.tsx
   export function NewFeaturePageContent() {
     return <div>New Feature Content</div>;
   }

   export default NewFeaturePageContent;
   ```

3. **Add route to App.tsx**:
   ```typescript
   import NewFeaturePage from '@/pages/NewFeaturePage';
   // ...
   <Route path="/ui/new-feature" element={<NewFeaturePage />} />
   ```

4. **Build the project** - the route will be automatically discovered:
   ```bash
   npm run build
   ```

The static generation script will automatically detect `NewFeaturePage.tsx` and create `/ui/new-feature/index.html`.

## Benefits

1. **Direct URL Access**: Users can bookmark and share direct links to any route
2. **SEO Friendly**: Search engines can crawl individual routes
3. **Fast Loading**: No client-side redirects for initial page loads
4. **Server Agnostic**: Works with any static file server
5. **Backward Compatible**: Existing client-side routing continues to work

## Deployment

The generated `dist/` directory can be deployed to any static hosting service:

- GitHub Pages
- Netlify
- Vercel
- AWS S3
- CDN services
- Traditional web servers

No special server configuration is required.
