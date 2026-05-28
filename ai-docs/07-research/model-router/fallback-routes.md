# Top 20 Open-Source AI Gateway Repositories with Fallback, Failover, and Health-Check Reference Value

## Executive summary

I reviewed open-source AI gateway, LLM proxy, and cloud-native gateway repositories that can serve as references for adding **ordered fallback routes, passive/active health checking, sticky routing, and primary recovery** behavior to the Bodhi app. The strongest direct implementation references are **LiteLLM**, **Bifrost**, **Portkey Gateway**, **TensorZero**, **FreeLLMAPI**, **MrFadiAi/free-llm-gateway**, **Ogem**, **OpenZiti llm-gateway**, and **Squirrel LLM Gateway** because their README or documentation explicitly discusses automatic fallbacks, retries, load balancing, provider failover, health checks, cooldowns, or related routing state.

For Bodhi’s likely requirement—“try model/API endpoints in a configured sequence, avoid unhealthy endpoints while they are down, but automatically return traffic to the primary when it recovers”—the most useful patterns to study are: **ordered fallback chains**, **failure classification**, **per-endpoint health state**, **cooldown/penalty decay**, **background active health checks**, **passive health updates from live traffic**, **sticky sessions/conversation affinity**, and **configuration-driven routing policies**.

> **Recommendation:** Start code-reading with **LiteLLM**, **Bifrost**, **FreeLLMAPI**, **MrFadiAi/free-llm-gateway**, **Ogem**, and **OpenZiti llm-gateway**. Use **Kong**, **APISIX**, **Higress**, and **Envoy AI Gateway** as references for robust gateway architecture, observability, and production-grade health/failover mechanisms, even where the AI-specific fallback policy is less obvious than in the smaller LLM gateways.

## Selection criteria

The list is ranked by a blend of **feature relevance** and **repository popularity/production maturity**, not by stars alone. I prioritized repositories that are open source, provide an AI/LLM gateway or proxy layer, expose an OpenAI-compatible or provider-unifying interface, and include at least one of the following: **fallback chains, failover, retries, health checks, load balancing, sticky routing, rate-limit-aware routing, or primary/backup provider routing**. Star and fork counts are approximate values observed during the review and may change.

| Rank | Repository | Approx. stars | Primary language | License | Why it is relevant for Bodhi fallback design |
|---:|---|---:|---|---|---|
| 1 | [BerriAI/litellm](https://github.com/BerriAI/litellm) | 48.5k | Python | Source-available / mixed | One of the strongest references for configurable **fallbacks, retries, load balancing, routing logic, budgets, and proxy-server behavior** across many providers. Its reliability docs explicitly cover fallback configuration and are worth reading first. |
| 2 | [Portkey-AI/gateway](https://github.com/Portkey-AI/gateway) | 11.9k | TypeScript | MIT | Production-oriented AI gateway with **fallbacks, retries, load balancing, routing config, virtual keys, and observability**. Very relevant for declarative route configuration and provider abstraction. |
| 3 | [maximhq/bifrost](https://github.com/maximhq/bifrost) | 5.3k | Go | Apache-2.0 | Explicitly markets itself as an AI gateway that “never goes down,” with **automatic failover, load balancing, retries/fallbacks, adaptive load balancer, clustering, metrics, and plugins**. Excellent for high-performance Go implementation ideas. |
| 4 | [tensorzero/tensorzero](https://github.com/tensorzero/tensorzero) | 11.4k | Rust | Apache-2.0 | Strong reference for **routing, fallbacks, retries, granular timeouts, experimentation, observability, and high-performance gateway design**. Useful if Bodhi wants routing policies tied to evaluations or model variants. |
| 5 | [tashfeenahmed/freellmapi](https://github.com/tashfeenahmed/freellmapi) | 6.0k | TypeScript | MIT | Directly matches the user’s example: OpenAI-compatible proxy that aggregates many providers with **automatic failover**. Useful for a compact reference implementation rather than enterprise gateway complexity. |
| 6 | [MrFadiAi/free-llm-gateway](https://github.com/MrFadiAi/free-llm-gateway) | 111 | Python | Not clearly stated in extracted README | Very relevant despite lower popularity: it explicitly includes **ordered fallback chains, automatic fallback on timeout/rate-limit/error, sticky sessions, dynamic penalty routing, per-key cooldown, runtime fallback editing, and fallback management endpoints**. |
| 7 | [yanolja/ogem](https://github.com/yanolja/ogem) | 41 | Go | Apache-2.0 | Small but highly relevant. README documents **latency-based routing, failover, health checks, ping intervals, retry intervals, and model-list fallback chains** such as comma-separated models. |
| 8 | [openziti/llm-gateway](https://github.com/openziti/llm-gateway) | 62 | Go | Apache-2.0 | Strong reference for **multi-endpoint load balancing, health checks, passive failover, VM sleep detection, weighted endpoints, and OpenAI-compatible backends**. Especially useful for local/self-hosted model endpoint pools. |
| 9 | [mylxsw/llm-gateway](https://github.com/mylxsw/llm-gateway) | 55 | Python | README says MIT | “Squirrel” LLM Gateway includes **priority, weight, round-robin, cost-based routing, provider failover, retries, timeout management, model mapping, and admin dashboard**. Useful for management UI plus backend routing state. |
| 10 | [Helicone/ai-gateway](https://github.com/Helicone/ai-gateway) | Not reliably fetched | TypeScript | Open source | User-provided example. Helicone’s AI Gateway is relevant for AI observability plus proxy/gateway design; inspect for fallback/routing implementation and how it integrates logging/metrics with request routing. |
| 11 | [QuantumNous/new-api](https://github.com/QuantumNous/new-api) | 35.9k | Go | AGPL-3.0 | Very popular unified model hub derived from the One API ecosystem. Strong for provider/channel management, model mapping, distribution, quota control, and gateway administration. Verify fallback behavior in code before adopting patterns. |
| 12 | [songquanpeng/one-api](https://github.com/songquanpeng/one-api) | 34.3k | JavaScript/Go ecosystem | MIT | Popular LLM API management and distribution system. README mentions **load-balanced channel access and automatic retry on failure**. Good for studying channel groups, token management, and retry behavior. |
| 13 | [Kong/kong](https://github.com/Kong/kong) | 43.5k | Lua | Apache-2.0 | Mature API and AI Gateway. Its AI gateway docs cover **LLM load balancing** and fault-tolerance patterns. Good reference for production plugin architecture and active/passive upstream health handling. |
| 14 | [apache/apisix](https://github.com/apache/apisix) | 16.6k | Lua | Apache-2.0 | Cloud-native API and AI gateway with mature upstream balancing, health checking, retries, circuit-breaking style policies, and plugin architecture. More generic than LiteLLM but valuable for production resilience patterns. |
| 15 | [higress-group/higress](https://github.com/higress-group/higress) | 8.5k | Go | Apache-2.0 | AI-native API gateway built around Envoy/Istio ideas. Useful for gateway-level routing, AI plugins, traffic management, and production deployment architecture. |
| 16 | [envoyproxy/ai-gateway](https://github.com/envoyproxy/ai-gateway) | 1.7k | Go | Apache-2.0 | Newer Envoy ecosystem AI gateway. Useful for Kubernetes/Envoy-native design and eventual policy abstractions for unified generative AI service access. |
| 17 | [APIParkLab/APIPark](https://github.com/APIParkLab/APIPark) | 1.7k | Go | Apache-2.0 | Open-source all-in-one AI gateway and API developer portal. Good for management-plane architecture, model/provider cataloging, monitoring, developer portal, and API lifecycle governance. |
| 18 | [Azure-Samples/AI-Gateway](https://github.com/Azure-Samples/AI-Gateway) | 931 | Jupyter / infra samples | MIT | Sample architecture rather than a pure gateway implementation. Useful for **Azure API Management + model load balancing + enterprise security** reference patterns. |
| 19 | [aws-solutions-library-samples/guidance-for-multi-provider-generative-ai-gateway-on-aws](https://github.com/aws-solutions-library-samples/guidance-for-multi-provider-generative-ai-gateway-on-aws) | Not reliably fetched | Terraform / infra | MIT-0 | AWS deployment reference for a multi-provider generative AI gateway based on LiteLLM. It explicitly mentions **retry/fallback routing logic across multiple providers** and adds enterprise deployment concerns such as WAF, ALB, ECS/EKS, RDS, Redis, and Secrets Manager. |
| 20 | [linto-ai/llm-gateway](https://github.com/linto-ai/llm-gateway) | 12 | Python | AGPL-3.0 | Smaller gateway that handles **chunking, queuing, retries, token tracking, and document export**. Less directly aligned than the top entries, but useful for request orchestration and retry handling around LLM services. |

## Best repositories to inspect first

The following subset is most directly aligned with the Bodhi fallback-route feature. I would inspect these before the broader API-gateway projects.

| Priority | Repository | What to inspect first | Why |
|---:|---|---|---|
| 1 | [LiteLLM](https://github.com/BerriAI/litellm) | Proxy reliability docs, fallback config parsing, router/load-balancer code, retry/error classification | It is the most established open-source LLM gateway with explicit fallback and load-balancing semantics. |
| 2 | [Bifrost](https://github.com/maximhq/bifrost) | `core`, provider registry, retry/fallback feature docs, adaptive load-balancer implementation | It is designed around high availability and fast failover, with Go code that may map well to a production service. |
| 3 | [FreeLLMAPI](https://github.com/tashfeenahmed/freellmapi) | Provider fallback order, health state, routing middleware, stream handling | Compact example close to the requested behavior and already user-validated as a reference. |
| 4 | [MrFadiAi/free-llm-gateway](https://github.com/MrFadiAi/free-llm-gateway) | Fallback management endpoints, sticky session store, cooldown/penalty routing, provider health sorting | It explicitly implements nearly every requested feature: sequence fallback, stickiness, health, cooldown, and runtime fallback edits. |
| 5 | [Ogem](https://github.com/yanolja/ogem) | Config model, ping interval, retry interval, fallback chain parsing, endpoint state | Its model string fallback chain and health-check interval are simple and likely easy to adapt. |
| 6 | [OpenZiti llm-gateway](https://github.com/openziti/llm-gateway) | Endpoint health checks, passive failover, weighted round-robin, local endpoint pool | Especially relevant if Bodhi will route between self-hosted or local inference endpoints. |

## Implementation patterns worth borrowing

A robust Bodhi implementation can combine the patterns below. The key architectural choice is to keep **routing policy**, **endpoint health state**, and **request execution** separate so the feature remains testable.

| Pattern | Description | Repositories to study |
|---|---|---|
| Ordered fallback chain | Configure a primary endpoint followed by secondaries. Try in sequence only when the current candidate fails with a retryable failure. | LiteLLM, FreeLLMAPI, MrFadiAi/free-llm-gateway, Ogem, Portkey |
| Passive health updates | Mark an endpoint degraded/unhealthy after live request failures such as timeout, connection error, 5xx, or rate-limit threshold. | Bifrost, OpenZiti llm-gateway, LiteLLM, gateway projects like Kong/APISIX |
| Active background health checks | Periodically test disabled/degraded endpoints and restore them when checks pass. This is what enables traffic to switch back to the primary. | Ogem, OpenZiti llm-gateway, Kong, APISIX, Envoy AI Gateway |
| Cooldown and penalty decay | Instead of binary healthy/unhealthy state, temporarily penalize endpoints and let penalties decay. This reduces flapping. | MrFadiAi/free-llm-gateway, LiteLLM-style router designs |
| Sticky sessions | Keep a conversation, user, or request group on the same provider/model while healthy to avoid behavioral drift. | MrFadiAi/free-llm-gateway, FreeLLMAPI-inspired designs |
| Failure classification | Retry/fallback only on connectivity, timeout, 429, and 5xx errors; do not fallback on invalid request, auth, model-not-found, or policy violations unless explicitly configured. | LiteLLM, Portkey, Bifrost, Squirrel |
| Runtime route editing | Allow administrators to reorder fallback chains or disable endpoints without restart. | MrFadiAi/free-llm-gateway, Squirrel, One API/New API |
| Observability headers/logs | Add headers or logs such as routed provider, fallback attempts, health state, latency, and failure reason. | MrFadiAi/free-llm-gateway, Helicone, LiteLLM, Portkey |

## Suggested Bodhi design sketch

For Bodhi, I would implement the feature as a **route group** abstraction. Each route group maps a logical model alias, such as `bodhi-default-chat`, to an ordered list of concrete AI endpoints. Every endpoint should maintain independent runtime health state.

| Component | Responsibility | Suggested fields |
|---|---|---|
| Route group | User-facing logical model/API route | `name`, `strategy`, `fallback_order`, `sticky_key`, `max_attempts`, `timeout_ms` |
| Endpoint | Concrete upstream provider/model/API base URL | `provider`, `model`, `base_url`, `api_key_ref`, `priority`, `weight`, `enabled` |
| Health state | Runtime state derived from active and passive checks | `status`, `last_success_at`, `last_failure_at`, `consecutive_failures`, `cooldown_until`, `penalty_score` |
| Router | Chooses candidate endpoint for each request | ordered primary-first selection, skip unhealthy/cooldown endpoints, optional sticky lookup |
| Executor | Performs request and failure classification | timeout handling, stream handling, retryable vs terminal error classification |
| Health checker | Restores primary/failed endpoints | periodic probe, half-open trial, penalty decay, flapping guard |
| Observability | Makes behavior debuggable | routed provider, fallback attempts, failure reason, health transitions, latency metrics |

A simple but effective algorithm is: first honor stickiness if the sticky endpoint is healthy; otherwise iterate the ordered fallback list and choose the first endpoint that is enabled, not in cooldown, and not marked unhealthy. If an endpoint fails with a retryable error, update passive health state and continue to the next candidate. A background health checker periodically probes unhealthy endpoints. When the primary becomes healthy again, new non-sticky requests should return to it; existing sticky conversations can either remain on the fallback until expiry or migrate on the next request, depending on the desired product semantics.

## Notes and caveats

Some popular projects, especially **Kong**, **APISIX**, **Higress**, and **Envoy AI Gateway**, are broader API gateways with AI-gateway extensions. They may not expose “fallback model chain” as directly as LiteLLM or Bifrost, but they are still useful because health checks, retries, upstream pools, circuit breakers, observability, and plugin lifecycle are mature. Conversely, smaller repositories such as **Ogem**, **OpenZiti llm-gateway**, **Squirrel**, and **MrFadiAi/free-llm-gateway** may be less popular but closer to the exact feature semantics you described.

License compatibility should be reviewed before copying implementation details. In particular, **AGPL-licensed** projects such as **QuantumNous/new-api** and **linto-ai/llm-gateway** are useful for architectural study, but code reuse may impose obligations depending on Bodhi’s licensing and distribution model.

## Reference links

[1]: https://github.com/BerriAI/litellm
[2]: https://docs.litellm.ai/docs/proxy/reliability
[3]: https://github.com/Portkey-AI/gateway
[4]: https://github.com/maximhq/bifrost
[5]: https://github.com/tensorzero/tensorzero
[6]: https://github.com/tashfeenahmed/freellmapi
[7]: https://github.com/MrFadiAi/free-llm-gateway
[8]: https://github.com/yanolja/ogem
[9]: https://github.com/openziti/llm-gateway
[10]: https://github.com/mylxsw/llm-gateway
[11]: https://github.com/Helicone/ai-gateway
[12]: https://github.com/QuantumNous/new-api
[13]: https://github.com/songquanpeng/one-api
[14]: https://github.com/Kong/kong
[15]: https://developer.konghq.com/ai-gateway/load-balancing/
[16]: https://github.com/apache/apisix
[17]: https://github.com/higress-group/higress
[18]: https://github.com/envoyproxy/ai-gateway
[19]: https://github.com/APIParkLab/APIPark
[20]: https://github.com/Azure-Samples/AI-Gateway
[21]: https://github.com/aws-solutions-library-samples/guidance-for-multi-provider-generative-ai-gateway-on-aws
[22]: https://github.com/linto-ai/llm-gateway
