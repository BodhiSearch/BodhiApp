import path from 'path';

import { TanStackRouterVite } from '@tanstack/router-plugin/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

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
    port: 3000,
    strictPort: true,
    hmr: {
      // When running behind Rust proxy (make app.run.live), the browser loads from port 1135
      // but HMR WebSocket needs to reach Vite directly since the proxy only handles /ui/* paths.
      // This tells the HMR client to connect to port 3000 (Vite) instead of 1135 (Rust proxy).
      clientPort: 3000,
    },
  },
});
