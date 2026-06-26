import { useEffect, useState } from 'react';

/**
 * MCP server avatar: shows the scraped `logo_url` when it loads, otherwise the monogram tint tile
 * (same `cat-logo cat-tint-N` styling as the catalog pages). Unlike the API-model catalog (whose logos
 * 404 upstream), MCP rows carry real logo URLs — so try the image first, fall back on error/missing.
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
  const [broken, setBroken] = useState(false);
  useEffect(() => setBroken(false), [src]);

  if (!src || broken) {
    return (
      <div className={className} aria-hidden="true">
        {fallback}
      </div>
    );
  }
  return (
    <div className={`${className} cat-logo--img`} aria-hidden="true">
      <img src={src} alt="" loading="lazy" onError={() => setBroken(true)} />
    </div>
  );
}
