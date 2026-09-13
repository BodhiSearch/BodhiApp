---
title: 'Reverse Proxy'
description: 'TLS termination and rate limiting in front of Docker — Nginx and Caddy examples'
order: 3
---

# Reverse Proxy

Bodhi's container does not terminate TLS and does not rate-limit requests. Both are explicitly the reverse proxy's job. This page shows minimal Caddy and Nginx configurations that cover the production basics: HTTPS, OAuth callback URLs, and rate limits on the public inference endpoints.

If you only need a Bodhi server on your LAN with no public exposure, you can skip this entirely.

## Why a reverse proxy

- **TLS termination.** Bodhi serves HTTP. Production traffic must be HTTPS — your proxy holds the certificate.
- **Rate limiting.** The Bodhi security model treats rate limiting as a perimeter concern. The app does **not** throttle requests; abusable surfaces (`/v1/*`, `/anthropic/v1/*`, `/v1beta/*`) need per-IP or per-token limits at the edge.
- **Hostname / path routing.** Run multiple services on one VM, behind one TLS cert.
- **HTTP-level access logs** that survive container restarts.

## OAuth callback URLs and `BODHI_PUBLIC_*`

When Bodhi sits behind a proxy, the URL that **users type in the browser** is different from the URL the **container binds to internally**. Bodhi needs the user-facing URL to build correct OAuth callback URLs — otherwise sign-in redirects come back to `localhost:8080` and fail.

Set these on the container:

| Variable              | Example             | Notes                                          |
| --------------------- | ------------------- | ---------------------------------------------- |
| `BODHI_PUBLIC_SCHEME` | `https`             | The scheme the user sees.                      |
| `BODHI_PUBLIC_HOST`   | `bodhi.example.com` | The hostname the user types.                   |
| `BODHI_PUBLIC_PORT`   | `443`               | The port the user reaches (443 / 80 / custom). |

`BODHI_HOST` / `BODHI_PORT` (defaults `0.0.0.0:8080`) are the **internal** bind — keep those alone unless you have a reason. Full reference: [Environment Variables](/docs/reference/env-vars).

## Caddy

Caddy is the shortest path: it auto-provisions Let's Encrypt certificates and reloads on file change.

```caddy
bodhi.example.com {
    encode gzip

    # Per-IP rate limit on abusable surfaces.
    # Requires the caddy-ratelimit plugin (xcaddy build with --with).
    @public_apis path /v1/* /anthropic/v1/* /v1beta/*
    rate_limit @public_apis {
        key {remote_host}
        events 60
        window 1m
    }

    reverse_proxy localhost:1135 {
        # Preserve the SSE stream — Caddy buffers by default.
        flush_interval -1
        header_up X-Forwarded-Proto {scheme}
        header_up X-Forwarded-Host {host}
    }
}
```

Container-side environment for this Caddy config:

```bash
-e BODHI_PUBLIC_SCHEME=https \
-e BODHI_PUBLIC_HOST=bodhi.example.com \
-e BODHI_PUBLIC_PORT=443
```

## Nginx

Nginx is more verbose but ships everywhere.

```nginx
# /etc/nginx/conf.d/bodhi.conf

# Per-IP rate limit zones for the abusable surfaces.
limit_req_zone $binary_remote_addr zone=bodhi_inference:10m rate=60r/m;

server {
    listen 80;
    server_name bodhi.example.com;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name bodhi.example.com;

    ssl_certificate     /etc/letsencrypt/live/bodhi.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/bodhi.example.com/privkey.pem;

    # SSE streaming — disable buffering and idle timeouts.
    proxy_buffering off;
    proxy_read_timeout 1h;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

    location ~ ^/(v1|anthropic/v1|v1beta)/ {
        limit_req zone=bodhi_inference burst=20 nodelay;
        proxy_pass http://127.0.0.1:1135;
    }

    location / {
        proxy_pass http://127.0.0.1:1135;
    }
}
```

Pair with the same `BODHI_PUBLIC_*` settings as the Caddy example.

## Common pitfalls

- **Trailing slashes.** Bodhi expects exact paths. Don't rewrite `/v1/chat/completions` to `/v1/chat/completions/`. Avoid `proxy_pass http://upstream/` (with trailing slash) in Nginx — use `proxy_pass http://upstream;`.
- **SSE / streaming buffering.** Both Caddy and Nginx buffer responses by default, which breaks token-by-token streaming for `/v1/chat/completions` and `/anthropic/v1/messages`. Use Caddy's `flush_interval -1` and Nginx's `proxy_buffering off;` plus a long `proxy_read_timeout`.
- **CORS preflight on `/bodhi/v1/apps/*`.** External-app and MCP-proxy endpoints rely on CORS. Don't strip `Origin`, `Access-Control-Request-Method`, or `Access-Control-Request-Headers` at the proxy. Bodhi handles CORS itself; the proxy just needs to forward unmodified.
- **Wrong `BODHI_PUBLIC_PORT`.** If you serve on `https://bodhi.example.com` (port 443) but set `BODHI_PUBLIC_PORT=1135`, OAuth callbacks come back to the wrong port. Match what the user actually types.
- **Forgotten `X-Forwarded-Proto`.** Without it, Bodhi may build redirects with `http://` even though users connect over HTTPS.

## Where to next

- **[Docker](/docs/deployment/docker)** — the container side of this setup.
- **[Reference → Environment Variables](/docs/reference/env-vars)** — full list of `BODHI_PUBLIC_*` and related variables.
- **[Security Model](/docs/advanced/security-model)** — what Bodhi does and does not protect against.
