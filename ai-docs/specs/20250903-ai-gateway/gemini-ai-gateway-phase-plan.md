# Plan: Extending the AI Gateway for Anthropic Integration

**Objective**: This document provides a strategic analysis and a detailed implementation plan for adding Anthropic as a new provider to the BodhiApp AI Gateway, establishing a scalable pattern for future provider integrations.

---

## 1. Analysis: Model Routing Strategy

A critical architectural decision is how the gateway identifies which provider to route a request to. Based on your question and industry research, we have two primary strategies:

*   **Strategy A: Implicit Resolution (Current Method)**: The user requests a model by its name (e.g., `"claude-3-opus"`), and the gateway searches all configured providers to find one that offers this model.
*   **Strategy B: Explicit Prefix (Recommended)**: The user prefixes the model name with the provider ID (e.g., `"anthropic/claude-3-opus"`). The gateway uses the prefix for direct routing.

My research into open-source gateways like LiteLLM confirms that the **explicit prefix** strategy is the industry standard and best practice for its clarity and scalability.

### Comparison of Strategies

| Aspect | Implicit Resolution (e.g., `claude-3-opus`) | Explicit Prefix (e.g., `anthropic/claude-3-opus`) |
| :--- | :--- | :--- |
| **Clarity** | ❌ Ambiguous. If multiple providers offer a model named `gpt-4`, the routing is non-deterministic or relies on arbitrary rules (e.g., creation date). | ✅ **Unambiguous**. The user's intent is perfectly clear. `openai/gpt-4` is distinct from `azure/gpt-4`. |
| **Scalability** | ❌ **Poor**. The router must query every single configured provider for every request to check if it can serve the model. This adds significant latency and complexity as providers are added. | ✅ **Excellent**. The router's logic is simple: parse the provider from the string and route directly to that provider's configuration. |
| **User Control** | ❌ **Low**. The user has no direct way to specify *which* provider's version of a model they want to use. | ✅ **High**. The user has full, explicit control over the provider and model selection. |
| **Maintainability** | ❌ **Complex**. The routing logic becomes a series of complex lookups and conflict resolution rules. | ✅ **Simple**. The code is cleaner, easier to understand, and easier to test. |

### Recommendation: Adopt Explicit Prefixes with a Fallback

I strongly recommend adopting the **explicit prefix (`provider/model_name`)** strategy as the primary method for model routing. It is a more robust, scalable, and user-friendly solution for a feature-rich gateway.

To ensure a smooth transition and maintain backward compatibility, I propose a **hybrid approach**:

1.  **If a model string contains a `/`**, the router will parse it as `provider/model` for direct routing.
2.  **If a model string does *not* contain a `/`**, the router will use the **existing lookup behavior** as a fallback. This ensures current integrations continue to work seamlessly.

This approach provides a clear path forward without breaking existing functionality.

---

## 2. Phased Plan for Anthropic Integration

This plan implements Anthropic support using the recommended explicit prefix strategy.

### Phase 1: Backend Core Logic (3-4 weeks)

**Goal**: Update the core routing and service layers to support Anthropic and the new model naming convention.

#### 1.1. Refactor the Model Router
*   **Business Value**: Implement the scalable and unambiguous routing strategy.
*   **Technical Implementation**:
    *   Modify `DefaultModelRouter::route_request` in `crates/server_core/src/model_router.rs`.
    *   The new logic will first check if the `model` string contains a `/`.
    *   If it does, parse the `provider_id` and `model_name`.
    *   Use `db_service.get_api_model_alias(&provider_id)` to fetch the correct provider configuration directly.
    *   If no `/` is present, execute the existing logic as a fallback.
*   **Affected Crates**: `crates/server_core`

#### 1.2. Implement Anthropic API Transformer
*   **Business Value**: Enable communication with Anthropic's API by translating requests and responses.
*   **Technical Implementation**:
    *   Anthropic's API differs from OpenAI's (e.g., `system` prompt, `max_tokens_to_sample`). Create a new module within `services`, such as `services/src/ai_api_service/anthropic.rs`.
    *   Implement `transform_request_to_anthropic` and `transform_response_from_anthropic` functions.
    *   Update `AiApiService::forward_chat_completion` to accept a `provider` argument and dispatch to the appropriate transformer based on the provider name.
*   **Affected Crates**: `services`

### Phase 2: Frontend & API Integration (1-2 weeks)

**Goal**: Expose Anthropic as an option in the UI and ensure the frontend communicates using the new model format.

#### 2.1. Update API Model Form
*   **Business Value**: Allow users to securely add their Anthropic API keys.
*   **Technical Implementation**:
    *   In `crates/bodhi/src/schemas/apiModel.ts` (or equivalent schema definition location), add an "Anthropic" preset to `PROVIDER_PRESETS`.
    *   The `ApiModelForm.tsx` component should now list "Anthropic" in the provider dropdown, pre-filling the appropriate `base_url`.
*   **Affected Crates**: `crates/bodhi/src/app/ui/api-models`

#### 2.2. Update Chat Interface
*   **Business Value**: Allow users to select and use configured Anthropic models for chat.
*   **Technical Implementation**:
    *   Modify the chat model selector component (e.g., `AliasSelector.tsx`).
    *   When a user selects a model from a configured API provider, the frontend must construct the prefixed string (e.g., `"anthropic/claude-3-opus"`) to send to the backend `/v1/chat/completions` endpoint.
    *   The UI should continue to display the model name in a user-friendly format, such as `Claude 3 Opus (Anthropic)`.
*   **Affected Crates**: `crates/bodhi/src/app/ui/chat`

### Phase 3: Testing & Validation (1 week)

**Goal**: Ensure the integration is robust, reliable, and bug-free.

#### 3.1. Unit & Integration Tests
*   **Technical Implementation**:
    *   Write unit tests for the new Anthropic request/response transformers in `services`.
    *   Update integration tests for `DefaultModelRouter` to cover all routing scenarios: direct-prefixed, fallback, and invalid-prefixed.
*   **Affected Crates**: `services`, `crates/server_core`

#### 3.2. End-to-End (E2E) Testing
*   **Technical Implementation**:
    *   Update the Playwright test suite in `crates/lib_bodhiserver_napi/tests-js/playwright/`.
    *   Add a new test case that:
        1.  Creates an Anthropic API model configuration via the UI.
        2.  Navigates to the chat page.
        3.  Selects an Anthropic model (e.g., `claude-3-haiku`).
        4.  Sends a message and verifies that a valid response is received.
*   **Affected Crates**: `crates/lib_bodhiserver_napi`
