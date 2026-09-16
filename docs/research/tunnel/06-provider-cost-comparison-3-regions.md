# 06 — Three-region cost model: Hetzner vs Vultr vs BuyVM (no-HA and HA)

Detailed gateway budget for **APAC, EU, Americas** using three candidate providers. Node specs, per-region egress, and no-HA vs HA totals. Prices are 2026-snapshot approximations — re-verify before committing (Hetzner changed dedicated-vCPU pricing in June 2026; shared CX stayed cheap).

## Assumptions (same as doc 04)

- 1,000 concurrent active users, 10,000 total (mostly idle).
- Aggregate throughput: **low ~0.85 TB/mo, mid ~2.6 TB/mo, high ~8.6 TB/mo**.
- Regional split (weighted): Americas 45%, EU 35%, APAC 20%.

| Region | Low | Mid | High |
|---|---|---|---|
| Americas | 0.4 TB | 1.2 TB | 3.9 TB |
| EU | 0.3 TB | 0.9 TB | 3.0 TB |
| APAC | 0.2 TB | 0.5 TB | 1.7 TB |

`frps` needs only 1–2 vCPU / 2–4 GB per node for 1,000+ concurrent text connections.

## Provider pricing & bandwidth terms (approx.)

| Provider | Node (recommended) | ~Price | Bandwidth | Region coverage |
|---|---|---|---|---|
| **Hetzner Cloud** | CX22: 2 vCPU / 4 GB | ~€4.50/mo | **20 TB included**, then ~€1/TB | EU (DE/FI), Americas (US-E/W), APAC (Singapore) |
| **Vultr** | 2 vCPU / 4 GB | ~$24/mo | ~3 TB included, then **$0.01/GB ($10/TB)** | All three (many DCs) |
| **Vultr (budget)** | 1 vCPU / 2 GB | ~$12/mo | ~2 TB included, then $0.01/GB | All three |
| **BuyVM** | 1 GB slice | ~$3.50/mo | **Unmetered 1 Gbps (shared port)** | Americas (LV/NY/Miami) + EU (Luxembourg). **No APAC** |

Key facts driving the numbers:

- **Hetzner is dramatically cheaper for compute** (€4.50 vs Vultr $24 for the same 2 vCPU/4 GB), and 20 TB included/instance means **egress is effectively $0** for our traffic (max per-region is only ~3.9 TB).
- **Vultr is ~5× Hetzner on compute**, with smaller included egress (2–3 TB) and $10/TB overage. Its strength is the broadest global footprint + predictable support.
- **BuyVM is the cheapest and truly unmetered, but has no APAC location** — APAC must be covered by Hetzner (Singapore) or Vultr.

## Cost tables

### 1. Hetzner Cloud (CX22, €4.50/mo, 20 TB incl.)

| Scenario | Egress (all regions) | Nodes | Compute | Total |
|---|---|---|---|---|
| **No HA** (3 regions × 1) | €0 | 3 | €13.50 | **~€13.50/mo** |
| **HA** (3 regions × 2) | €0 | 6 | €27.00 | **~€27/mo** |

Egress never exceeds the 20 TB/instance allowance in any scenario (max single region ~3.9 TB). **Bandwidth cost = €0 across the board.**

### 2. Vultr (2 vCPU / 4 GB, $24/mo, 3 TB incl., $10/TB over)

| Scenario | Egress overage | Nodes | Compute | Total |
|---|---|---|---|---|
| **No HA** low/mid | $0 | 3 | $72 | **$72/mo** |
| **No HA** high | ~$9 (Americas only) | 3 | $72 | **~$81/mo** |
| **HA** low/mid | $0 | 6 | $144 | **$144/mo** |
| **HA** high | ~$9 | 6 | $144 | **~$153/mo** |

Only the Americas high case (~3.9 TB) slightly exceeds the 3 TB allowance.

### 2b. Vultr budget (1 vCPU / 2 GB, $12/mo, 2 TB incl.)

| Scenario | Egress overage | Nodes | Compute | Total |
|---|---|---|---|---|
| **No HA** low/mid | $0 | 3 | $36 | **$36/mo** |
| **No HA** high | ~$29 (Americas $19 + EU $10) | 3 | $36 | **~$65/mo** |
| **HA** low/mid | $0 | 6 | $72 | **$72/mo** |
| **HA** high | ~$29 | 6 | $72 | **~$101/mo** |

### 3. BuyVM (1 GB slice, $3.50/mo, unmetered) — Americas + EU only

| Scenario | Egress | Nodes | Compute | + APAC (Hetzner CX22) | Total |
|---|---|---|---|---|---|
| **No HA** (Am+EU) | $0 | 2 | $7 | +1 APAC = €4.50 (~$5) | **~$12/mo** |
| **HA** (Am+EU) | $0 | 4 | $14 | +2 APAC = €9 (~$10) | **~$24/mo** |

BuyVM has no APAC datacenter, so the APAC leg is filled by Hetzner Singapore (or Vultr) in the totals above.

## Summary comparison (full 3-region fleet)

| Provider | No HA (low/mid) | No HA (high) | HA (low/mid) | HA (high) |
|---|---|---|---|---|
| **Hetzner** | **~$15/mo** | **~$15/mo** | **~$30/mo** | **~$30/mo** |
| **Vultr** (2vCPU/4GB) | $72/mo | $81/mo | $144/mo | $153/mo |
| **Vultr** (budget) | $36/mo | $65/mo | $72/mo | $101/mo |
| **BuyVM + Hetzner APAC** | ~$12/mo | ~$12/mo | ~$24/mo | ~$24/mo |

**Bottom line:** Hetzner wins decisively on cost for our bandwidth profile (compute ~5× cheaper than Vultr, egress $0). BuyVM is the cheapest literal-unmetered option but can't cover APAC on its own. Vultr is the most expensive here — you're paying for the broadest global footprint and support, but for text LLM traffic its smaller included bandwidth + higher compute makes it ~5–10× Hetzner.

## Budget recommendation

Provision **$15/mo (no HA) → $30/mo (HA)** on Hetzner for the full 3-region fleet. If strict unmetered is a hard requirement and you accept a second vendor for APAC, **BuyVM (Americas+EU) + Hetzner (Singapore)** gives ~$12/mo (no HA) / ~$24/mo (HA) with zero egress math — but you lose single-vendor simplicity. Vultr is only worth it if you specifically want its footprint/support and are comfortable with ~$72–153/mo.

## What this means for product pricing (adoption-based)

The infrastructure is effectively a **fixed cost** that barely scales with users for text traffic:

| Adoption | Hetzner no-HA | Hetzner HA |
|---|---|---|
| 100 concurrent / 1k total | ~$15/mo | ~$30/mo |
| 1,000 concurrent / 10k total | ~$15/mo | ~$30/mo |
| 5,000 concurrent / 50k total (future) | ~$15–30/mo (still 1 node/region) | ~$30–60/mo |

At full 1,000 concurrent, infra is ~**$0.015/concurrent-user/mo** (Hetzner HA), and against 10,000 total users it's ~**$0.003/user/mo**. This means **pricing should be a value/market decision, not a cost-recovery decision** — a premium tier of even **$5–10/user/mo** (or a flat per-instance add-on) with a few dozen paying users covers the entire fleet many times over. Set the price against competitors (ngrok/Cloudflare Tunnel) and the "public endpoint" value, not against infra cost.

Caveat: these numbers hold only while the tunnel carries **LLM API text**. If BodhiApp later streams large files/models through the tunnel, bandwidth stops being negligible and Vultr's $10/TB overage becomes a real line item — re-run this model with a per-request-size/throughput assumption at that point.
