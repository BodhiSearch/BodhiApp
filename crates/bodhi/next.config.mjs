import withPWAInit from '@ducanh2912/next-pwa';

const withPWA = withPWAInit({
  dest: 'public',
  cacheOnFrontEndNav: true,
  aggressiveFrontEndNavCaching: true,
  register: true,
  workboxOptions: {
    disableDevLogs: true,
  }
});

/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  output: 'export',
  trailingSlash: true,
  transpilePackages: ['geist'],
  images: {
    unoptimized: true,
  },
  eslint: {
    ignoreDuringBuilds: true,
  },
  webpack: (config) => {
    config.watchOptions = {
      ignored: ['**/node_modules/', 'old-chat-app/**'],
    };
    return config;
  },
};

export default withPWA(nextConfig);
