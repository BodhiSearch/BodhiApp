import { useEffect, useMemo, useState } from 'react';

import { useGetAppInfo } from '@/hooks/info';

/**
 * MCP server avatar: shows the catalog `logo_url` when it loads, otherwise the monogram tint tile
 * (same `cat-logo cat-tint-N` styling as the catalog pages). Unlike the API-model catalog (whose logos
 * 404 upstream), MCP rows carry real logo URLs — try the image first, fall back on error/missing.
 *
 * `logo_url` may be a RELATIVE path (`/api/v1/mcp-servers/logos/{id}`, served by the reference API's
 * own domain) — resolve it against `reference_api_url` so it loads from there, not the BodhiApp origin.
 */
export function McpServerLogo({
  src,
  className,
  fallback,
}: {
  src: string | null;
  className: string;
  fallback: string;
}) {
  const { data: appInfo } = useGetAppInfo();
  const [broken, setBroken] = useState(false);
  useEffect(() => setBroken(false), [src]);

  const resolved = useMemo(() => {
    if (!src) return null;
    if (/^https?:\/\//i.test(src)) return src; // already absolute
    const base = appInfo?.reference_api_url;
    if (!base) return null; // relative path with no base yet → fall back to monogram
    try {
      return new URL(src, base).toString();
    } catch {
      return null;
    }
  }, [src, appInfo?.reference_api_url]);

  if (!resolved || broken) {
    return (
      <div className={className} aria-hidden="true">
        {fallback}
      </div>
    );
  }
  return (
    <div className={`${className} cat-logo--img`} aria-hidden="true">
      <img src={resolved} alt="" loading="lazy" onError={() => setBroken(true)} />
    </div>
  );
}
