# AI Gateway: Model Routing Strategy Analysis

**Date**: September 4, 2025
**Author**: Gemini Assistant
**Status**: Final Recommendation

## 1. Objective

This document outlines the analysis and final recommendation for the model routing strategy to be used in the BodhiApp AI Gateway. The goal is to select a scalable, maintainable, and user-friendly approach for directing requests to the correct AI provider as we expand beyond the initial OpenAI implementation.

---

## 2. Routing Strategies Analyzed

Two primary strategies were considered for how the gateway identifies the target provider for a given request.

*   **Strategy A: Implicit Resolution**: The user requests a model by its name alone (e.g., `"claude-3-opus"`). The gateway then searches all configured providers to find one that offers this model.
*   **Strategy B: Explicit Prefix**: The user prefixes the model name with a unique provider identifier (e.g., `"anthropic/claude-3-opus"`). The gateway uses this prefix for direct, unambiguous routing.

---

## 3. Industry Precedent

Research into popular open-source and commercial AI gateways reveals a clear industry standard:

*   **LiteLLM**: Uses the explicit `provider/model_name` format (e.g., `litellm.completion("anthropic/claude-3-haiku...")`).
*   **Portkey**: Also uses an explicit format, `@{provider_slug}/{model_name}` (e.g., `model="@openai-prod/gpt-4o"`).

Both leading tools have converged on the explicit prefix strategy for its clarity and scalability.

---

## 4. Comparison & Analysis

| Aspect | Implicit Resolution (e.g., `claude-3-opus`) | Explicit Prefix (e.g., `anthropic/claude-3-opus`) |
| :--- | :--- | :--- |
| **Clarity** | ❌ **Ambiguous**. If multiple providers offer a model with the same name (e.g., `gpt-4`), routing becomes non-deterministic or relies on arbitrary rules like creation date. | ✅ **Unambiguous**. The user's intent is perfectly clear. `openai/gpt-4` is distinct from a potential `azure/gpt-4`. |
| **Scalability** | ❌ **Poor**. The router must query every configured provider for every request to check for model availability. This adds significant latency and complexity as more providers are added. | ✅ **Excellent**. The router's logic is simple and fast: parse the provider from the string and route directly to that provider's configuration. |
| **User Control** | ❌ **Low**. The user has no direct way to specify *which* provider's version of a model they want to use in case of overlaps. | ✅ **High**. The user has full, explicit control over the provider and model selection. |
| **Maintainability** | ❌ **Complex**. The routing code becomes a series of complex lookups and conflict-resolution rules that are difficult to maintain and debug. | ✅ **Simple**. The code is cleaner, easier to understand, and more straightforward to test. |

---

## 5. Final Recommendation

I strongly recommend adopting the **explicit prefix (`provider/model_name`)** strategy as the primary method for model routing in the BodhiApp AI Gateway.

This approach is the most robust, scalable, and user-friendly solution. It aligns with industry best practices and prevents a class of future ambiguity-related bugs.

To ensure a smooth transition, a **hybrid approach** should be implemented in the `DefaultModelRouter`:

1.  **Primary Logic**: If a model string contains a `/`, the router will parse it as `provider/model` for direct routing.
2.  **Fallback for Backward Compatibility**: If no `/` is present, the router will use the existing lookup behavior, iterating through all configured providers to find a match. This ensures that current user configurations and workflows continue to function without disruption.

This strategy provides a clear and scalable path forward for all future provider integrations, starting with Anthropic.
