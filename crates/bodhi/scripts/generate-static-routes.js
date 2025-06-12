#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Converts a page file path to a route path
 * @param {string} filePath - The file path relative to pages directory
 * @returns {string} - The route path
 */
function pageFileToRoute(filePath) {
  // Remove .tsx extension
  let route = filePath.replace(/\.tsx$/, '');

  // Handle special cases
  if (route === 'HomePage') return '/ui';
  if (route === 'NotFoundPage') return null; // Skip 404 page
  if (route === 'OAuthCallbackPage') return '/ui/auth/callback';
  if (route === 'ModelFilesPage') return '/ui/modelfiles';

  // Remove 'Page' suffix first
  route = route.replace(/Page$/, '');

  // Convert PascalCase to kebab-case
  route = route
    .replace(/([A-Z])/g, (_, letter, index) => {
      return index === 0 ? letter.toLowerCase() : `-${letter.toLowerCase()}`;
    });

  // Handle nested routes (like docs/DocsMain -> docs/docs-main)
  if (route.includes('/')) {
    const parts = route.split('/').map(part =>
      part.replace(/([A-Z])/g, (_, letter, index) => {
        return index === 0 ? letter.toLowerCase() : `-${letter.toLowerCase()}`;
      })
    );

    // Special handling for docs routes
    if (parts[0] === 'docs') {
      if (parts[1] === 'docs-main') return '/docs';
      if (parts[1] === 'docs-slug') return null; // Skip dynamic route
      return null; // Skip other docs routes
    }

    return `/ui/${parts.join('/')}`;
  }

  return `/ui/${route}`;
}

/**
 * Recursively scan pages directory for .tsx files
 * @param {string} dir - Directory to scan
 * @param {string} baseDir - Base pages directory
 * @returns {string[]} - Array of file paths relative to pages directory
 */
function scanPagesDirectory(dir, baseDir = dir) {
  const files = [];
  const entries = fs.readdirSync(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);

    if (entry.isDirectory()) {
      // Recursively scan subdirectories
      files.push(...scanPagesDirectory(fullPath, baseDir));
    } else if (entry.isFile() && entry.name.endsWith('.tsx')) {
      // Get relative path from pages directory
      const relativePath = path.relative(baseDir, fullPath);
      files.push(relativePath);
    }
  }

  return files;
}

// Scan pages directory to get all routes
const pagesDir = path.resolve(__dirname, '../src/pages');
const pageFiles = scanPagesDirectory(pagesDir);

// Convert page files to routes
const routes = ['/']; // Always include root
pageFiles.forEach(filePath => {
  const route = pageFileToRoute(filePath);
  if (route && !routes.includes(route)) {
    routes.push(route);
  }
});

// Add special routes that don't have corresponding page files
routes.push('/ui/home'); // HomePage also serves /ui/home
routes.push('/ui/models/new'); // ModelsPage serves multiple routes
routes.push('/ui/models/edit');
routes.push('/ui/setup'); // SetupPage serves multiple routes

console.log('📄 Detected page files:', pageFiles);
console.log('🛣️  Generated routes:', routes);

// Path to the dist directory
const distDir = path.resolve(__dirname, '../dist');
const indexHtmlPath = path.join(distDir, 'index.html');

// Check if index.html exists
if (!fs.existsSync(indexHtmlPath)) {
  console.error('❌ index.html not found in dist directory. Please run build first.');
  process.exit(1);
}

// Read the original index.html
const indexHtmlContent = fs.readFileSync(indexHtmlPath, 'utf8');

console.log('🚀 Generating static routes...');

// Generate static files for each route
routes.forEach(route => {
  // Skip root route as it already has index.html
  if (route === '/') {
    console.log(`✅ / -> index.html (already exists)`);
    return;
  }

  // Create directory structure for the route
  const routePath = route.startsWith('/') ? route.slice(1) : route;
  const routeDir = path.join(distDir, routePath);
  const routeIndexPath = path.join(routeDir, 'index.html');

  // Create directory if it doesn't exist
  if (!fs.existsSync(routeDir)) {
    fs.mkdirSync(routeDir, { recursive: true });
  }

  // Copy index.html to the route directory
  fs.writeFileSync(routeIndexPath, indexHtmlContent);
  console.log(`✅ ${route} -> ${routePath}/index.html`);
});

console.log('🎉 Static route generation completed!');
console.log('\n📁 Generated structure:');
console.log('dist/');
console.log('├── index.html');
routes.slice(1).forEach(route => {
  const routePath = route.startsWith('/') ? route.slice(1) : route;
  console.log(`├── ${routePath}/`);
  console.log(`│   └── index.html`);
});

console.log('\n🧪 Test with:');
console.log('cd dist && python -m http.server 8000');
console.log('Then try accessing routes directly like:');
console.log('- http://localhost:8000/ui/login/');
console.log('- http://localhost:8000/ui/chat/');
console.log('- http://localhost:8000/ui/settings/');
