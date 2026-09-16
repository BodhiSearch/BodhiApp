# 04 — Tunnel gateway cost estimates (unmetered VPS fleet)

Rough infra budgeting for the self-hosted frp-based tunnel service. Assumes ~1,000 concurrent active users at peak, ~10,000 total (mostly idle). All figures are order-of-magnitude planning numbers; re-verify provider pricing and fair-use terms before committing.

> **Superseded on provider + region count:** docs `05-provider-review-analysis.md` and `06-provider-cost-comparison-3-regions.md` later switch the vendor to **Hetzner** (quality-first) and doc `08-deployment-plan.md` settles on a **single-region start** (expanding to 3). This doc's **bandwidth model and cost arithmetic remain valid**; only its OVH-centric vendor pick and "start with 4→6 regions" plan are outdated.

## TL;DR

LLM API traffic is **text** (prompts + token streams), so it is **bandwidth-light and latency-sensitive**. `frps` is a lightweight Go binary that handles tens of thousands of concurrent connections on 2 vCPU. The cost is therefore dominated by the **number of gateway VMs (regions × redundancy)**, not by bandwidth or CPU.

- **Low (MVP):** ~$20–25/mo — 4 regions × 1 small unmetered VPS.
- **Recommended:** ~$60/mo — 6 regions × 1 node + a small control-plane node.
- **HA:** ~$120–140/mo — 6 regions × 2 nodes + control plane.

This is cheap enough that even a small fraction of a premium tier (e.g. $5–10/user/mo) covers the fleet many times over.

## Bandwidth model (sanity check that "unmetered" is appropriate)

Assumptions:

| Parameter | Value |
|---|---|
| Concurrent active users (peak) | 1,000 |
| Requests per active user per minute | ~4 (one per 15 s) |
| Aggregate request rate | ~67 req/s |
| Data per request (in + out, incl. SSE overhead) | 5 KB (low) / 15 KB (mid) / 50 KB (high) |

Result:

| Scenario | Throughput | Aggregate / month | Per region (÷4) |
|---|---|---|---|
| Low (short responses) | ~0.33 MB/s | ~0.85 TB | ~0.2 TB |
| Mid | ~1 MB/s | ~2.6 TB | ~0.65 TB |
| High (long responses, larger prompts) | ~3.3 MB/s | ~8.6 TB | ~2.2 TB |

Even the "high" case (~8.6 TB/month aggregate) is comfortably within unmetered/fair-use limits, and per-region is only ~1–2 TB/month. **Bandwidth will not be the bottleneck**; if BodhiApp later pushes large model files/downloads through the tunnel, these numbers change drastically and should be re-modeled.

## Server sizing

`frps` per node: **2 vCPU / 4 GB RAM** is more than enough for 1,000+ concurrent connections and the text traffic above. Use the 4 vCPU / 8 GB tier if you want headroom for TLS termination bursts and future growth.

## Region strategy

For latency-sensitive streaming, start with 4 core regions and expand to 6:

- **US East** (New York / Virginia)
- **US West** (Los Angeles / Seattle)
- **Europe West** (London / Frankfurt / Paris)
- **Asia-Pacific** (Singapore / Tokyo / Mumbai)

Optional 5th/6th: **Europe Central** and **South America (São Paulo)** or **India**.

Redundancy: start with 1 node/region (DNS failover, frps restarts quickly); move to 2 nodes/region for HA.

## Unmetered VPS provider comparison (current anchors)

| Provider | Spec | Price/mo | Unmetered? | Regions | Notes |
|---|---|---|---|---|---|
| **OVHcloud VPS-1** | 2 vCPU / 4 GB / 40 GB NVMe | **$4.54** | Yes, 500 Mbps (US/EU). APAC has 1 TB quota. | Many + 15 Local Zone cities | 99.9% SLA, DDoS incl., daily backup. Confirmed price. |
| **OVHcloud VPS-2** | 4 vCPU / 8 GB / 75 GB NVMe | **$8.50** | Yes, 1 Gbps (US/EU) | Many | Best value pick for headroom. Confirmed price. |
| **OVHcloud VPS-3** | 6 vCPU / 12 GB / 100 GB NVMe | **$12.32** | Yes, 2 Gbps (US/EU) | Many | Confirmed price. |
| **Contabo Cloud VPS** | 4 vCPU / 8 GB | ~$6.39 | Unlimited (fair-use) | 9 locations | Cheapest big-RAM; "incoming traffic unlimited/unmetered." Approximate. |
| **HostHatch** | 2–4 vCPU / 4–8 GB | ~$4–8 | Unmetered | 14 locations, 4 continents | Budget, wide region coverage. Approximate. |
| **IONOS VPS** | 4 vCPU / 4 GB / 120 GB NVMe | ~$11 | Unlimited, 1 Gbps | ~6 DCs (EU/US) | 1 Gbps line. Approximate. |
| **BuyVM slice** | 1–4 GB RAM | $3.50+ | Unmetered, 1 Gbps | ~4–6 locations | Very cheap but small/slow compute. Approximate. |

Important caveat: "unmetered" is usually **fair-use**. Contabo explicitly reserves the right to throttle "exceptionally high or disruptive" traffic. OVH's US/EU VPS are unmetered, but **OVH APAC (Mumbai/Singapore/Sydney) has a 1–4 TB quota** then 10 Mbps. For APAC, use Contabo/HostHatch/IONOS to keep unmetered, or accept the quota (our APAC traffic is low).

## Monthly cost estimates

### Low — MVP ($20–25/mo)

- 4 regions × 1 × OVH VPS-1 ($4.54) = **$18.16**
- Wildcard DNS + Let's Encrypt cert via Caddy = $0
- **Total ≈ $20/mo**

Budget alternative: 4 × BuyVM/Contabo (~$4–6) ≈ $20/mo (smaller compute, slower).

### Recommended ($55–65/mo)

- 6 regions × 1 × OVH VPS-2 ($8.50) = **$51.00**
- 1 control-plane/monitoring node (VPS-2) = **$8.50**
- **Total ≈ $60/mo**

(Mix APAC on Contabo/HostHatch if staying strictly unmetered in APAC.)

### HA ($120–140/mo)

- 6 regions × 2 nodes × OVH VPS-2 ($8.50) = **$102.00**
- Control plane: 2 × VPS-2 = **$17.00**
- Monitoring/alerting host = **$8.50**
- **Total ≈ $127/mo**

Optional add-ons: OVH premium backup (~$1.40/node/mo) or snapshots (~$0.40/mo) — negligible at this scale.

## What's not included (and why it's cheap)

- **Anycast/managed DNS:** free-to-cheap (Cloudflare DNS free, or provider DNS). A wildcard A record per region with geo-DNS costs ~$0–10/mo.
- **Wildcard TLS:** free via Let's Encrypt/Caddy.
- **DDoS protection:** included with OVH; Contabo/IONOS include basic mitigation.
- **The BodhiApp control plane / provisioning service:** already part of the existing backend (not new infra).

## Recommendation

> **Superseded** — see `05-provider-review-analysis.md` (Hetzner preferred) and `08-deployment-plan.md` (single-region start). Kept below for the original arithmetic.

Start with the **Recommended tier (~$60/mo)**: 6 regions × 1 OVH VPS-2 node, using Contabo/HostHatch for APAC if you want strict unmetered there. This comfortably supports the 1k/10k user profile with headroom, and gives you a predictable, unmetered cost base. Scale horizontally (add nodes in a region) only if CPU/TLS termination or connection counts grow — bandwidth is not the constraint for text LLM traffic.
