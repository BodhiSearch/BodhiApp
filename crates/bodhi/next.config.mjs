import withPWAInit from '@ducanh2912/next-pwa';
import createMDX from '@next/mdx';

const withPWA = withPWAInit({
  dest: 'public',
  cacheOnFrontEndNav: true,
  aggressiveFrontEndNavCaching: true,
  register: true,
  workboxOptions: {
    disableDevLogs: true,
  },
});

const withMDX = createMDX({
  // Add markdown plugins here, as desired
  options: {
    remarkPlugins: [],
    rehypePlugins: [],
  },
});

/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  output: 'export',
  trailingSlash: true,
  transpilePackages: ['geist'],
  productionBrowserSourceMaps: true,
  images: {
    unoptimized: true,
  },
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

export default withPWA(withMDX(nextConfig));
