import '@fontsource-variable/inter';
import React from 'react';

import { RouterProvider, createRouter } from '@tanstack/react-router';
import ReactDOM from 'react-dom/client';

import { routeTree } from './routeTree.gen';

const router = createRouter({
  routeTree,
  basepath: '/ui',
  defaultPreload: 'intent',
  trailingSlash: 'always',
  // Wrap navigations in the native document.startViewTransition() (React-18-safe;
  // ignored on browsers without the API). CSS recipes in styles/view-transitions.css.
  defaultViewTransition: true,
});

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>
);
