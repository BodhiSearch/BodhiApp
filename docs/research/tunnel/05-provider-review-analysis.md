# 05 — Public review analysis of unmetered VPS providers

Goal: pick a gateway host with **decent public reviews** (not just lowest price) for the frp-based tunnel fleet. Willing to pay slightly more for quality. Review ratings are **approximate sentiment** synthesized from public reviews/Trustpilot/community feedback; treat as directional, not exact.

## The core tension

**Truly "unmetered" VPS tends to be budget-tier hosts with mixed support reviews.** The highest-rated providers (Hetzner, DigitalOcean, Vultr, Linode, UpCloud) meter bandwidth but with generous, predictable allowances. For BodhiApp's traffic profile (~1–8.6 TB/month total, see `04`), a "generous included quota" is effectively unmetered and often safer than a budget host's "unlimited until we throttle you."

So the decision is really: **predictable metered quality vs. literal-unmetered budget.** Recommendation lands on a middle path (below).

## Review sentiment — the original five (incl. the OVH concern)

| Provider | Unmetered? | Regions | ~Price | Review sentiment | Notes |
|---|---|---|---|---|---|
| **OVHcloud** | Yes (US/EU); APAC quota | Many + Local Zones | $4.54–8.50 | **Mixed (~3.5/5)** | Strong network, 99.9% SLA, DDoS — but **support is the common complaint** (slow, ticket-only). Solid infra, weak service. |
| **Contabo** | Unlimited (fair-use) | 9 | ~$6.39 | **Mixed/negative (~3–3.5/5)** | Cheapest big-RAM; reviews flag **oversubscribed CPU, slow support, occasional instability**. |
| **HostHatch** | Unmetered | 14 | ~$4–8 | **Mixed (~3.5/5)** | Cheap + many regions, but **openly "minimal support"**; fine for self-sufficient admins. |
| **IONOS** | Unlimited (1 Gbps) | ~6 (EU/US) | ~$11 | **Mixed (~3.5/5)** | Big brand, 1 Gbps, but billing/support reviews are mixed. |
| **BuyVM/Frantech** | Unmetered (1 Gbps) | ~4–6 | $3.50+ | **Good (~4/5)** | Strong homelab reputation; engaged owner; **reliable but small/slow compute + often out of stock**. |

## Additional providers (10+, with review sentiment)

### Quality-first (better reviews; metered but generous/predictable)

| Provider | Unmetered? | Regions | ~Price | Review sentiment | Notes |
|---|---|---|---|---|---|
| **Hetzner Cloud** | No — **20 TB included** then ~€1/TB | EU (DE/FI) + US + SG | €4–8 (CPX21/31) | **Excellent (~4.6/5)** | Price/performance king, very reliable, great API/Terraform. Best quality-per-dollar. |
| **Netcup** | **Yes — flatrate traffic** (fair-use) | ~5 (EU-centric: DE/AT, some US) | €5.91–10 | **Very good (~4.3/5)** | German, ISO 27001, 99.9% SLA; Trustpilot ~2.9k reviews. Genuinely unmetered **and** well-reviewed. |
| **Vultr** | No — $0.01/GB egress (poolable) | 16–32 DCs | $12–24 (1–2 vCPU/2–4 GB) | **Good (~4.3/5)** | Reliable, huge global footprint; metered but predictable. |
| **DigitalOcean** | No — transfer included per tier, then $0.01/GB | 15+ | $6–12 | **Very good (~4.5/5)** | "Boring in a good way": flat pricing, great docs/support. |
| **Linode/Akamai** | No — transfer pool per plan | 25+ | $5–12 | **Good (~4.3/5)** | Reliable, mature, DDoS, predictable. |
| **UpCloud** | No — metered | EU/US/Asia | $7–13 | **Very good (~4.4/5)** | High performance (MaxIOPS), 100% uptime SLA; pricier. |
| **Scaleway** | Partly — **Dedibox (bare metal) unmetered**; Instances metered | EU (FR/NL/PL) | €11+ | **Good (~4.2/5)** | EU-focused; bare-metal Dedibox is the unmetered path. |

### Budget-unmetered (cheaper; acceptable reviews, weaker support)

| Provider | Unmetered? | Regions | ~Price | Review sentiment | Notes |
|---|---|---|---|---|---|
| **RackNerd** | Often unmetered/high-BW | Mostly US | $11–22/**yr** | **Mixed-good (~3.5–4/5)** | Very cheap annual plans, Inc. 5000; **ticket-only support**. |
| **GreenCloudVPS** | Unmetered/high-BW | 30+ (Asia-heavy) | $1.25+ | **Good (~4/5)** | Published 99.99% uptime; budget, many regions; mixed support. |
| **Kamatera** | Metered (PAYG) | Global | ~$4+ | **Mixed (~3.8/5)** | Flexible, good performance; some billing complaints. |
| **Alwyzon** | **Yes — unmetered 2.5 Gbps** | EU only (Vienna) | €6+ | **Very good (~4.5/5), niche** | Small Austrian host, transparent, direct network — but **EU-only**. |

## Shortlist by priority

1. **Hetzner Cloud** — **top recommendation.** Excellent reviews + cheap + reliable + 20 TB included/instance. For BodhiApp's text traffic, 20 TB ≈ effectively unmetered and far more predictable than budget "unlimited." Add a US and SG node for the needed regions (EU is its strength; US/SG available).
2. **Netcup** — the best **literally-unmetered** option with genuinely good reviews; ideal if the "no metered billing" requirement is strict, but its footprint is EU-centric (use a second provider for US-West/APAC).
3. **BuyVM** — best cheap unmetered with a genuinely good reputation, but small compute and frequent stock-outs; fine as APAC/US-West edge fill.
4. **Vultr / DigitalOcean / Linode** — if global footprint + support matter more than literal "unmetered"; egress for text LLM traffic is modest but not flat.

## Recommendation

Given "decent reviews, willing to pay slightly more," I'd run the fleet on a **hybrid**:

- **Hetzner Cloud** for EU-West, EU-Central, and US-East (best reviews + value; 20 TB included each is ample).
- **BuyVM (or Vultr)** for US-West and APAC if strict unmetered is desired, or **Hetzner US/SG** if 20 TB included is acceptable for those regions too.

This avoids OVH's support reputation and Contabo/HostHatch's weak-support trade-offs, while keeping the total well under ~$100/mo for a 6-region fleet. If "no metered billing, ever" is a hard requirement, **Netcup (EU) + BuyVM (US/APAC)** is the cleanest well-reviewed unmetered combination.
