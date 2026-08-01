> **`BODHI_ENCRYPTION_KEY`**: must be at least 20 characters and not a placeholder, or the app
> refuses to start. Generate with `openssl rand -base64 32`. There is no previous-key fallback —
> **changing this value makes every stored secret (provider API keys, MCP OAuth tokens, tenant
> client secrets) permanently unrecoverable.** See `docs/architecture/security.md` → Key Derivation.

```env
BODHI_ENCRYPTION_KEY="{{ RUNPOD_SECRET_BODHI_ENCRYPTION_KEY }}"
BODHI_LOG_STDOUT="true"
BODHI_LOG_LEVEL="info"
BODHI_CANONICAL_REDIRECT=false
BODHI_PUBLIC_HOST="dev-server.getbodhi.app"
BODHI_PUBLIC_PORT="443"
BODHI_PUBLIC_SCHEME="https"
HF_TOKEN="{{ RUNPOD_SECRET_HF_TOKEN }}"
RAILWAY_RUN_UID="0"
```
