// Download URL loaded from .env.release_urls
export const DOWNLOAD_URL = process.env.NEXT_PUBLIC_DOWNLOAD_URL_MACOS_ARM64;

// Build-time validation
if (!DOWNLOAD_URL) {
  throw new Error(
    'Missing required environment variable: NEXT_PUBLIC_DOWNLOAD_URL_MACOS_ARM64. ' +
      'Check .env.release_urls file or set environment variable.'
  );
}
