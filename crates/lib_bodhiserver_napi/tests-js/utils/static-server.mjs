import path from 'node:path';
import { fileURLToPath } from 'node:url';
import express from 'express';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Creates and manages a static Express server for serving test pages
 */
export function createStaticServer(port) {
  const app = express();
  let server = null;

  // Serve static files from test-pages directory
  const testPagesDir = path.join(__dirname, '../test-pages');
  app.use(express.static(testPagesDir));

  // Add CORS headers for OAuth redirects
  app.use((req, res, next) => {
    res.header('Access-Control-Allow-Origin', '*');
    res.header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
    res.header(
      'Access-Control-Allow-Headers',
      'Origin, X-Requested-With, Content-Type, Accept, Authorization'
    );
    next();
  });

  const startServer = () => {
    return new Promise((resolve, reject) => {
      server = app.listen(port, 'localhost', (error) => {
        if (error) {
          reject(error);
        } else {
          const baseUrl = `http://localhost:${port}`;
          console.log(`✅ Static server started at: ${baseUrl}`);
          resolve(baseUrl);
        }
      });
    });
  };

  const stopServer = () => {
    return new Promise((resolve) => {
      if (server) {
        server.close(() => {
          console.log('✅ Static server stopped');
          resolve();
        });
      } else {
        resolve();
      }
    });
  };

  return {
    startServer,
    stopServer,
  };
}
