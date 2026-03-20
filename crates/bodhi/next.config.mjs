import createMDX from '@next/mdx';

const withMDX = createMDX({
  // Add markdown plugins here, as desired
  options: {
    remarkPlugins: [],
    rehypePlugins: [],
  },
});

/** @type {import('next').NextConfig} */
const nextConfig = {
  basePath: '/ui',
  reactStrictMode: true,
  output: 'export',
  trailingSlash: true,
  transpilePackages: ['geist'],
  productionBrowserSourceMaps: true,
  images: {
    unoptimized: true,
  },
  allowedDevOrigins: ['localhost:1135', '127.0.0.1:1135', '0.0.0.0:1135'],
  eslint: {
    ignoreDuringBuilds: true,
  },
  webpack: (config) => {
    config.watchOptions = {
      ignored: ['**/node_modules/', '**/old-chat-app/**'],
    };
    return config;
  },
};

export default withMDX(nextConfig);
