#!/usr/bin/env python3
"""
Probe every provider's GET {base_url}/models from the Bodhi reference catalog and verify
that our `LenientOpenAIModel` parser would successfully extract models.

Our backend parser (crates/services/src/ai_apis/provider_shared.rs::fetch_openai_models):
  - reads response["data"] as an array
  - for each entry, deserializes into LenientOpenAIModel:
        id: String            (REQUIRED — entry dropped if missing/non-string)
        object: String        (#[serde(default = "model")])
        created: u32          (#[serde(default)] — must fit u32 if present)
        owned_by: String      (#[serde(default)])
  - entries missing a string `id` are silently filtered out.

So an entry parses IFF it has a string `id` AND (if `created` present) `created` fits in u32.
This script reports, per provider, how many entries parse vs. total, and flags any that would
still produce an empty/short list — i.e. anything that could still give us trouble.
"""
import json
import sys
import urllib.request
import urllib.error

CATALOG = "https://dev-api.getbodhi.app/api/v1/catalog/providers"
U32_MAX = 4294967295
TIMEOUT = 25


def http_get_json(url):
    req = urllib.request.Request(url, headers={"Accept": "application/json", "User-Agent": "bodhi-models-probe/1.0"})
    with urllib.request.urlopen(req, timeout=TIMEOUT) as resp:
        return resp.status, json.load(resp)


def entry_parses(entry):
    """Mirror LenientOpenAIModel: needs string `id`; `created` is Option<u64> (any
    non-negative int, incl. millisecond timestamps; truncated to u32 on convert)."""
    if not isinstance(entry, dict):
        return False, "entry is not an object"
    mid = entry.get("id")
    if not isinstance(mid, str) or mid == "":
        return False, "missing/non-string `id`"
    created = entry.get("created")
    if created is not None:
        if not isinstance(created, int) or isinstance(created, bool):
            return False, f"`created` not an integer ({created!r})"
        if created < 0 or created > 18446744073709551615:  # u64::MAX
            return False, f"`created` overflows u64 ({created})"
    return True, None


def analyze(base_url):
    url = base_url.rstrip("/") + "/models"
    try:
        status, body = http_get_json(url)
    except urllib.error.HTTPError as e:
        return {"url": url, "outcome": f"HTTP {e.code}", "note": e.reason}
    except Exception as e:
        return {"url": url, "outcome": "REQUEST FAILED", "note": str(e)}

    if status != 200:
        return {"url": url, "outcome": f"HTTP {status}"}

    data = body.get("data") if isinstance(body, dict) else None
    if not isinstance(data, list):
        top = list(body.keys()) if isinstance(body, dict) else type(body).__name__
        return {"url": url, "outcome": "NO `data` ARRAY", "note": f"top-level: {top}"}

    total = len(data)
    parsed = 0
    reasons = {}
    for e in data:
        ok, why = entry_parses(e)
        if ok:
            parsed += 1
        else:
            reasons[why] = reasons.get(why, 0) + 1
    return {"url": url, "outcome": "OK", "total": total, "parsed": parsed, "dropped_reasons": reasons}


def main():
    print(f"Fetching catalog: {CATALOG}\n")
    _, catalog = http_get_json(CATALOG)
    providers = catalog["items"]

    probed, skipped, problems = [], [], []

    for p in providers:
        slug = p["slug"]
        base = p.get("api_base_url")
        fmt = p.get("api_format_hint")
        shape = p.get("provider_shape")

        # Our /models parser only runs for the OpenAI wire format with a known base URL.
        if not base:
            skipped.append((slug, fmt, shape, "no api_base_url (uses well-known default / not probeable here)"))
            continue
        if fmt != "openai":
            skipped.append((slug, fmt, shape, f"api_format_hint={fmt} (not parsed by fetch_openai_models)"))
            continue

        r = analyze(base)
        r.update(slug=slug, fmt=fmt, shape=shape)
        probed.append(r)

    # Report
    print("=" * 100)
    print("PROBED (openai-format providers with a base_url)")
    print("=" * 100)
    for r in probed:
        if r["outcome"] == "OK":
            flag = "OK " if r["parsed"] == r["total"] and r["total"] > 0 else "!! "
            line = f"{flag}{r['slug']:14} parsed {r['parsed']}/{r['total']:<4} {r['url']}"
            print(line)
            if r["dropped_reasons"]:
                for why, n in r["dropped_reasons"].items():
                    print(f"      dropped {n}: {why}")
            if r["parsed"] == 0 or r["total"] == 0:
                problems.append(r)
            elif r["parsed"] < r["total"]:
                problems.append(r)
        else:
            print(f"?? {r['slug']:14} {r['outcome']:18} {r['url']}  {r.get('note','')}")
            # auth-gated endpoints (401/403) are expected without keys — not a parser bug.
            if r["outcome"].startswith("HTTP 4") and r["outcome"] not in ("HTTP 401", "HTTP 403"):
                problems.append(r)
            elif r["outcome"] in ("NO `data` ARRAY", "REQUEST FAILED"):
                problems.append(r)

    print("\n" + "=" * 100)
    print("SKIPPED (not hit by our openai /models parser)")
    print("=" * 100)
    for slug, fmt, shape, why in skipped:
        print(f"-- {slug:14} fmt={fmt:8} shape={shape:18} {why}")

    print("\n" + "=" * 100)
    print("VERDICT")
    print("=" * 100)
    full = [r for r in probed if r["outcome"] == "OK" and r["total"] > 0 and r["parsed"] == r["total"]]
    auth = [r for r in probed if r["outcome"] in ("HTTP 401", "HTTP 403")]
    print(f"Total catalog providers:        {len(providers)}")
    print(f"Probed (openai + base_url):     {len(probed)}")
    print(f"  fully parseable (100%):       {len(full)}")
    print(f"  auth-gated (401/403, expected): {len(auth)}  -> {[r['slug'] for r in auth]}")
    print(f"  PROBLEMS to review:           {len(problems)}  -> {[r['slug'] for r in problems]}")
    print(f"Skipped (non-openai / no base): {len(skipped)}")
    if not problems:
        print("\n==> No parser problems found for any reachable openai-format provider.")
    else:
        print("\n==> Review the PROBLEM providers above.")
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main()
