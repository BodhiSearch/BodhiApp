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
