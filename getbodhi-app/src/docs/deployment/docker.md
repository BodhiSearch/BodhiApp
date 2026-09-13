---
title: 'Docker'
description: 'Deploy Bodhi App with Docker — CPU, CUDA, ROCm, Vulkan, MUSA, Intel, and CANN variants'
order: 2
---

# Docker

Run Bodhi as a single-tenant container on a server you control. One Bodhi instance serves multiple human users, each with their own login, role, and API tokens.

If you need to choose between Docker and the desktop app first, read **[Deployment Overview](/docs/deployment/overview)**. If you're putting Bodhi on the public internet, also read **[Reverse Proxy](/docs/deployment/reverse-proxy)** for TLS and rate limiting.

## Variants

All variants are published to GitHub Container Registry under `ghcr.io/bodhisearch/bodhiapp` with a `latest-<variant>` tag plus version-pinned tags (recommended for production). The container exposes port **8080**, which you map to a host port via `-p`.

| Variant  | Platforms     | Hardware                            |
| -------- | ------------- | ----------------------------------- |
| `cpu`    | AMD64 + ARM64 | Any CPU (AVX/AVX2/AVX512, ARM NEON) |
| `cuda`   | AMD64         | NVIDIA GPUs (CUDA 11+)              |
| `rocm`   | AMD64         | AMD Radeon / Instinct GPUs          |
| `vulkan` | AMD64         | Cross-vendor GPU via Vulkan         |
| `musa`   | AMD64         | Moore Threads S-series GPUs         |
| `intel`  | AMD64         | Intel Arc / Data Center GPUs (SYCL) |
| `cann`   | AMD64 + ARM64 | Huawei Ascend NPUs                  |

**Picking a variant:** match it to your hardware. NVIDIA → `cuda`. AMD GPU → `rocm`. Intel GPU → `intel`. Cross-vendor or unsure → `vulkan`. CPU-only or ARM device → `cpu`. Specialty hardware → `musa` / `cann`.

> Check **[getbodhi.app](https://getbodhi.app)** for the current latest version tag and the up-to-date list of variants.

## Prerequisites

- Docker 20.10+ ([install guide](https://docs.docker.com/get-docker/))
- For GPU variants:
  - **CUDA**: NVIDIA Container Toolkit ([install](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/install-guide.html))
  - **ROCm**: host AMD GPU drivers + `/dev/kfd`, `/dev/dri`
  - **Vulkan**: GPU driver with Vulkan support + `/dev/dri`
  - **Intel**: Intel GPU drivers + `/dev/dri`
  - **MUSA**: MUSA toolkit on host + `/dev/mthreads`
  - **CANN**: CANN toolkit + `/dev/davinci*` devices

## Quick start (CPU)

```bash
docker run --name bodhiapp \
  -p 1135:8080 \
  -e BODHI_PUBLIC_HOST=localhost \
  -e BODHI_PUBLIC_PORT=1135 \
  -e BODHI_ENCRYPTION_KEY=replace-with-a-strong-random-key \
  -v bodhi-data:/data \
  ghcr.io/bodhisearch/bodhiapp:latest-cpu
```

Open `http://localhost:1135` and follow the setup wizard. The first user becomes admin.

> **`BODHI_ENCRYPTION_KEY` is required.** The container refuses to start without a real value. Use a strong random string and store it alongside your backup of the volume — both are needed to restore.

## GPU variants

The only differences from the CPU run line are the image tag and the device flags. Replace the encryption key with your own each time.

**NVIDIA (CUDA):**

```bash
docker run --name bodhiapp-cuda \
  -p 1135:8080 \
  -e BODHI_ENCRYPTION_KEY=... \
  -v bodhi-data:/data \
  --gpus all \
  ghcr.io/bodhisearch/bodhiapp:latest-cuda
```

**AMD (ROCm):**

```bash
docker run --name bodhiapp-rocm \
  -p 1135:8080 \
  -e BODHI_ENCRYPTION_KEY=... \
  -v bodhi-data:/data \
  --device=/dev/kfd --device=/dev/dri \
  ghcr.io/bodhisearch/bodhiapp:latest-rocm
```

**Cross-vendor (Vulkan) / Intel:**

```bash
docker run --name bodhiapp-vulkan \
  -p 1135:8080 \
  -e BODHI_ENCRYPTION_KEY=... \
  -v bodhi-data:/data \
  --device=/dev/dri \
  ghcr.io/bodhisearch/bodhiapp:latest-vulkan
# (swap latest-vulkan → latest-intel for Intel GPUs)
```

**Moore Threads (MUSA):**

```bash
docker run --name bodhiapp-musa \
  -p 1135:8080 \
  -e BODHI_ENCRYPTION_KEY=... \
  -v bodhi-data:/data \
  --device=/dev/mthreads \
  ghcr.io/bodhisearch/bodhiapp:latest-musa
```

**Huawei Ascend (CANN):**

```bash
docker run --name bodhiapp-cann \
  -p 1135:8080 \
  -e BODHI_ENCRYPTION_KEY=... \
  -v bodhi-data:/data \
  --device=/dev/davinci0 --device=/dev/davinci_manager \
  --device=/dev/devmm_svm --device=/dev/hisi_hdc \
  ghcr.io/bodhisearch/bodhiapp:latest-cann
```

## Volume layout

The image uses a single data volume mounted at `/data`, with two subdirectories:

| Path inside container | Purpose                                         | Bound env var |
| --------------------- | ----------------------------------------------- | ------------- |
| `/data/bodhi_home`    | Config, SQLite databases, aliases, logs         | `BODHI_HOME`  |
| `/data/hf_home`       | Downloaded GGUF model files (HuggingFace cache) | `HF_HOME`     |

Mount `/data` as either a named volume (`-v bodhi-data:/data`) or a bind mount (`-v /srv/bodhi:/data`). For backups, archive the whole `/data` volume **and** record the matching `BODHI_ENCRYPTION_KEY`.

> Model files dominate the volume size (4–80 GB per quantized model). Plan storage accordingly.

## Required environment variables

The image bakes sensible defaults via `defaults.yaml`. The variables below are the ones you almost always want to set or override at deploy time. Full alphabetical list: **[Reference → Environment Variables](/docs/reference/env-vars)**.

| Variable                                                          | Why                                                                                                                                         |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `BODHI_ENCRYPTION_KEY`                                            | **Required.** Encrypts secrets at rest. Container refuses to start without it.                                                              |
| `BODHI_HOME`                                                      | Override `/data/bodhi_home` if you want a different data path.                                                                              |
| `HF_HOME`                                                         | Override `/data/hf_home` if you store models elsewhere.                                                                                     |
| `HF_TOKEN`                                                        | HuggingFace token for downloading gated models.                                                                                             |
| `BODHI_HOST` / `BODHI_PORT`                                       | Internal bind address. Default `0.0.0.0:8080`; usually leave alone and remap with `-p`.                                                     |
| `BODHI_PUBLIC_SCHEME` / `BODHI_PUBLIC_HOST` / `BODHI_PUBLIC_PORT` | The externally-visible URL — what users type in the browser. The OAuth callback uses these. Set when running behind a reverse proxy or NAT. |
| `BODHI_AUTH_URL` / `BODHI_AUTH_REALM`                             | Override the OAuth provider. Defaults point at the managed Bodhi auth service.                                                              |
| `BODHI_APP_DB_URL` / `BODHI_SESSION_DB_URL`                       | Override the SQLite locations or point at PostgreSQL for the multi-tenant build (out of scope here).                                        |
| `BODHI_LOG_LEVEL`, `BODHI_LOG_STDOUT`                             | Increase verbosity / mirror logs to stdout. See [Observability](/docs/advanced/observability).                                              |

## docker compose

```yaml
services:
  bodhiapp:
    image: ghcr.io/bodhisearch/bodhiapp:latest-cpu
    ports:
      - '1135:8080'
    environment:
      BODHI_PUBLIC_SCHEME: https
      BODHI_PUBLIC_HOST: bodhi.example.com
      BODHI_PUBLIC_PORT: '443'
      BODHI_ENCRYPTION_KEY: ${BODHI_ENCRYPTION_KEY}
      HF_TOKEN: ${HF_TOKEN}
    volumes:
      - bodhi-data:/data
    restart: unless-stopped

volumes:
  bodhi-data:
```

For the CUDA variant, swap the image tag and add the GPU reservation:

```yaml
image: ghcr.io/bodhisearch/bodhiapp:latest-cuda
deploy:
  resources:
    reservations:
      devices:
        - driver: nvidia
          count: all
          capabilities: [gpu]
```

For other GPUs, use the appropriate `devices:` mapping (the same paths shown in the `docker run` examples above).

## Health checks

The container ships a built-in healthcheck against `/ping`. From the host:

```bash
curl http://localhost:1135/ping
```

`docker stats <name>` shows live CPU / memory / GPU usage.

## Upgrading

```bash
docker stop bodhiapp
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cpu
docker rm bodhiapp
# re-run with the same -e and -v flags as the original
```

The data volume is preserved. Database migrations run automatically on the new container's first boot. Always keep the same `BODHI_ENCRYPTION_KEY` across upgrades.

For production, **pin a version tag** (`:0.1.0-cpu`) instead of `latest-cpu` so upgrades are explicit.

## Backup

```bash
docker run --rm \
  -v bodhi-data:/data \
  -v "$(pwd)":/backup \
  alpine tar czf /backup/bodhi-$(date +%F).tgz -C / data
```

Restore is the inverse — `tar xzf` into the same volume on the new host. Remember: the archive plus the encryption key together is what restores; either alone is useless.

## Behind a reverse proxy

Production deployments typically put Caddy / Nginx / Traefik in front for TLS, hostname routing, and rate limiting. When you do that, set `BODHI_PUBLIC_SCHEME=https`, `BODHI_PUBLIC_HOST=your-domain`, `BODHI_PUBLIC_PORT=443` — otherwise OAuth callbacks come back to the wrong place.

Full guide: **[Reverse Proxy](/docs/deployment/reverse-proxy)**.

## Troubleshooting

- **Container exits immediately** — `docker logs <name>`. Most often: missing or placeholder `BODHI_ENCRYPTION_KEY`.
- **Port already in use** — change the host side of `-p 1135:8080`.
- **GPU not detected (CUDA)** — verify NVIDIA Container Toolkit installed and `--gpus all` flag present. `docker exec -it <name> nvidia-smi` should list the GPU.
- **GPU not detected (ROCm)** — verify `--device=/dev/kfd --device=/dev/dri` flags and host driver version.
- **OAuth callback loops back to `localhost`** — set `BODHI_PUBLIC_*` to your external URL.
- **Slow inference** — the variant defaults are tuned for single-request latency. For parallel workloads, override settings via env vars or the in-app settings page. See [Performance Tuning](/docs/advanced/performance-tuning).

For more, see [Troubleshooting](/docs/support/troubleshooting).

## Related

- [Reverse Proxy](/docs/deployment/reverse-proxy) — TLS, rate limiting, `BODHI_PUBLIC_*` variables
- [Reference → Environment Variables](/docs/reference/env-vars) — full alphabetical matrix
- [Reference → Settings](/docs/reference/settings) — runtime settings vs env-var defaults
- [Inference Stack](/docs/advanced/inference-stack) — how variant choice affects llama.cpp args
