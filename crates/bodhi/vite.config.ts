import path from 'path';

import { TanStackRouterVite } from '@tanstack/router-plugin/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const devProxyPort = Number(process.env.BODHI_DEV_PROXY_UI_PORT) || 3000;

export default defineConfig({
  plugins: [
    TanStackRouterVite({
      routesDirectory: './src/routes',
      generatedRouteTree: './src/routeTree.gen.ts',
      routeFileIgnorePattern: '.*\\.test\\..*',
    }),
    react(),
  ],
  base: '/ui/',
  resolve: {
    alias: { '@': path.resolve(__dirname, './src') },
  },
  build: {
    outDir: 'out',
    emptyOutDir: true,
    sourcemap: false,
    chunkSizeWarningLimit: 3000,
  },
  server: {
    port: devProxyPort,
    strictPort: true,
    hmr: {
      // Behind the Rust proxy (make app.run.live) only /ui/* is proxied, so HMR must
      // connect directly to Vite's port instead of the browser's page origin.
      clientPort: devProxyPort,
    },
    // Pre-bundle the SPA entry on startup so the first navigation doesn't race
    // Vite's lazy dep optimization (E2E otherwise sees a blank first page).
    warmup: {
      clientFiles: ['./src/main.tsx', './src/routeTree.gen.ts'],
    },
  },
});
