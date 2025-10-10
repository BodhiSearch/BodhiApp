import { config } from 'dotenv';
import { existsSync } from 'fs';
import { resolve } from 'path';

// Optional load .env.release_urls - don't fail if missing
const envPath = resolve(process.cwd(), '.env.release_urls');
if (existsSync(envPath)) {
  config({ path: envPath });
}

// After optional load, validate required env vars exist
const requiredVars = [
  'NEXT_PUBLIC_DOWNLOAD_URL_MACOS',
  'NEXT_PUBLIC_DOWNLOAD_URL_WINDOWS',
  'NEXT_PUBLIC_DOWNLOAD_URL_LINUX',
];
requiredVars.forEach((varName) => {
  if (!process.env[varName]) {
    throw new Error(
      `Build failed: ${varName} is required. ` + 'Check .env.release_urls file or set environment variable.'
    );
  }
});

/** @type {import('next').NextConfig} */
const nextConfig = {
  // Enable static exports
  output: 'export',
  trailingSlash: true,

  // Optimize image handling
  images: {
    unoptimized: true, // Required for static export
  },

  // Optimize production builds
  compress: true,
  poweredByHeader: false,

  // Cache optimizations
  generateEtags: true,

  // Optimize static rendering
  reactStrictMode: true,
  swcMinify: true,

  // Experimental features for better performance
  experimental: {
    // Optimize CSS
    optimizePackageImports: ['lucide-react'],

    // Optimize bundle
    turbo: {
      rules: {
        // Add rules for static analysis
        '*.md': ['raw-loader'],
      },
    },
  },

  // Disable checks during production builds
  typescript: {
    ignoreBuildErrors: true,
  },
  eslint: {
    ignoreDuringBuilds: true,
  },
};

export default nextConfig;
