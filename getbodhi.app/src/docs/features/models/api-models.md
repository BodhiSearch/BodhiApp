---
title: 'API Models'
description: 'Configure and use API models alongside local models'
order: 210
---

# API Models

## Overview

Bodhi App supports hybrid AI architecture - use powerful API models from leading providers alongside your local GGUF models in a unified interface.

**Key Features**:

- Five API formats: OpenAI Completions, OpenAI Responses, Anthropic, Anthropic OAuth, and Google Gemini
- Multi-provider support — connect to OpenAI, Anthropic, Google, OpenRouter, HuggingFace, or any compatible endpoint
- Native proxy endpoints for each format (e.g. `/v1/chat/completions`, `/v1/responses`, `/anthropic/v1/messages`, `/v1beta/models/...`)
- Optional API key with encrypted storage (AES-GCM)
- Extra headers and extra body fields for advanced per-provider customization
- Model discovery from provider APIs
- Prefix-based routing for namespace separation
- Forward All with Prefix mode — forward every model from a provider via prefix routing without selecting individual models
- Test connection before saving
- Seamless switching between local and remote models

## Why Use API Models?

**Advantages**:

- Access latest frontier models (GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Flash, and others via cloud)
- No local GPU required
- Instant availability (no downloads)
- Automatic model updates from providers
- Pay-per-use pricing

**When to Use Local vs Remote**:

- **Local**: Privacy-sensitive data, offline access, no usage costs
- **Remote**: Latest capabilities, specialized tasks, no hardware limits

## Supported API Formats and Providers

Bodhi App supports five API formats. Each format determines how requests are routed, which proxy endpoints are available, and what authentication method is used.

| API Format         | UI Display Name               | Auth Method                          | Proxy Endpoint(s)                        |
| ------------------ | ----------------------------- | ------------------------------------ | ---------------------------------------- |
| `openai`           | OpenAI - Completions          | API key (Bearer)                     | `/v1/chat/completions`, `/v1/embeddings` |
| `openai_responses` | OpenAI - Responses            | API key (Bearer)                     | `/v1/responses` (full CRUD)              |
| `anthropic`        | Anthropic                     | API key (`x-api-key` or Bearer)      | `/anthropic/v1/messages`, `/v1/messages` |
| `anthropic_oauth`  | Anthropic (Claude Code OAuth) | Bearer token + extra headers         | `/anthropic/v1/messages`, `/v1/messages` |
| `gemini`           | Google Gemini                 | API key (`x-goog-api-key` or Bearer) | `/v1beta/models/{model}:{action}`        |

**How providers work**: A provider is defined by the combination of:

- **API Format**: Which API protocol Bodhi App uses to talk to the upstream service
- **Base URL**: The upstream endpoint for that provider

**Available models**: When you configure a provider, Bodhi App fetches all currently available models from that provider. Only the models you add are accessible for chat or API calls.

### Getting Started with Common Providers

1. **OpenAI**
   - **API Format**: OpenAI - Completions
   - **Base URL**: `https://api.openai.com/v1`
   - Get API key from [OpenAI Platform](https://platform.openai.com/api-keys)

2. **OpenAI Responses API**
   - **API Format**: OpenAI - Responses
   - **Base URL**: `https://api.openai.com/v1`
   - Same API key as OpenAI; use this format when you want to call `/v1/responses` instead of `/v1/chat/completions`

3. **Anthropic**
   - **API Format**: Anthropic
   - **Base URL**: `https://api.anthropic.com/v1`
   - Get API key from [Anthropic Console](https://console.anthropic.com/settings/keys)

4. **Anthropic via Claude Code OAuth**
   - **API Format**: Anthropic (Claude Code OAuth)
   - **Base URL**: `https://api.anthropic.com/v1`
   - Uses the OAuth Bearer token from Claude Code (token starts with `sk-ant-oat01-`)
   - Default extra headers and body fields are pre-filled automatically (see [Anthropic OAuth section](#anthropic-claude-code-oauth))

5. **Google Gemini**
   - **API Format**: Google Gemini
   - **Base URL**: `https://generativelanguage.googleapis.com/v1beta`
   - Get API key from [Google AI Studio](https://aistudio.google.com/app/apikey)

6. **OpenRouter**
   - **API Format**: OpenAI - Completions (compatible)
   - **Base URL**: `https://openrouter.ai/api/v1`
   - Get API key from OpenRouter dashboard

7. **HuggingFace Inference API**
   - **API Format**: OpenAI - Completions (compatible)
   - **Base URL**: `https://router.huggingface.co/v1`
   - Get API key from HuggingFace settings

8. **Custom OpenAI-compatible provider**
   - **API Format**: OpenAI - Completions (compatible)
   - **Base URL**: Provider-specific endpoint
   - Any provider that implements the OpenAI chat completions format

## Creating an API Model

<img
  src="/doc-images/api-models.jpg"
  alt="API Models Configuration Form"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

### Step 1: Navigate to API Models Page

**From Models Page**:

1. Go to Models page (<a href="/ui/models/" target="_blank" rel="noopener noreferrer">/ui/models/</a>)
2. Click "New API Model" button

**From Setup Wizard** (optional step):

1. During setup, on API Models step
2. Click "Configure API Model"

### Step 2: Select API Format and Configure Provider

Configure which provider you want to connect to by selecting the API format and entering the base URL.

1. **Select API Format**: Click the "API Format" dropdown and choose the format matching your provider
   - **OpenAI - Completions**: Standard OpenAI chat completions and embeddings
   - **OpenAI - Responses**: OpenAI Responses API (stateful, multi-turn responses)
   - **Anthropic**: Anthropic Messages API with `x-api-key` authentication
   - **Anthropic (Claude Code OAuth)**: Anthropic Messages API with OAuth Bearer token
   - **Google Gemini**: Google Gemini generate/stream/embed content API

2. **Enter Base URL**: The provider is determined by the base URL you enter
   - For **OpenAI** (Completions or Responses): The form auto-fills `https://api.openai.com/v1`
   - For **Anthropic** (API key or OAuth): The form auto-fills `https://api.anthropic.com/v1`
   - For **Gemini**: The form auto-fills `https://generativelanguage.googleapis.com/v1beta`
   - For **OpenRouter**: Enter `https://openrouter.ai/api/v1`
   - For **HuggingFace**: Enter `https://router.huggingface.co/v1`
   - For **other OpenAI-compatible providers**: Enter their specific base URL

**Base URL Requirements**:

- Must be a complete endpoint URL
- For OpenAI format, it should end with the path to which `/chat/completions` can be appended
- Example: For OpenAI, the base URL is `https://api.openai.com/v1`, so chat completions endpoint becomes `https://api.openai.com/v1/chat/completions`
- For Anthropic format, the base URL should end at the version prefix (`/v1`); Bodhi App appends `/messages` automatically
- For Gemini format, the base URL should be the `v1beta` base; Bodhi App routes to the correct action path

**Technical Specifications**: For API implementation details, see [API Reference](/docs/developer/openapi-reference).

### Step 3: Enter API Key

API key requirements vary by format. Use the "Use API Key" checkbox to toggle key usage.

**With API key**:

1. Check "Use API Key"
2. Enter API key in password field (masked for security)
3. Key is encrypted with AES-GCM before database storage
4. Key is decrypted from the database and sent to the provider when forwarding requests

**Without API key**:

1. Uncheck "Use API Key"
2. Requests are sent without an Authorization header
3. Useful for local or self-hosted providers that do not require authentication

**How the API key is sent depends on the format**:

- **OpenAI / OpenAI Responses**: sent as `Authorization: Bearer <key>`
- **Anthropic**: the form auto-detects; Bodhi App accepts both `x-api-key: <key>` and `Authorization: Bearer <key>` from callers (the middleware normalizes it)
- **Anthropic OAuth**: the token (starts with `sk-ant-oat01-`) is sent as `Authorization: Bearer <token>`; do not use a standard API key for this format
- **Gemini**: Bodhi App accepts `x-goog-api-key: <key>` or `Authorization: Bearer <key>` from callers (the middleware normalizes it)

You can add, change, or remove the API key at any time by editing the API model.

### Step 3b: Extra Headers and Extra Body (Advanced, Optional)

The **Extra Headers** and **Extra Body** fields let you inject additional data into every request forwarded to the upstream provider. Both fields accept a JSON object.

**Extra Headers** — merged into the HTTP request headers sent upstream:

```json
{
  "anthropic-version": "2023-06-01",
  "anthropic-beta": "claude-code-20250219,oauth-2025-04-20"
}
```

**Extra Body** — deep-merged into the JSON request body forwarded upstream:

```json
{
  "max_tokens": 4096,
  "system": [{ "type": "text", "text": "You are a helpful assistant." }]
}
```

**Restrictions**:

- Authorization headers (`authorization`, `x-api-key`, `x-goog-api-key`) are not allowed in Extra Headers; use the API key field instead
- Both fields must be valid JSON objects (not arrays or primitives)
- Fields are optional; leave blank if not needed

**When to use**:

- **Anthropic OAuth**: the pre-filled defaults include the required `anthropic-version`, `anthropic-beta`, and `user-agent` headers plus the `max_tokens` and `system` body fields that Claude Code OAuth expects
- **Custom providers**: inject provider-specific headers or body fields that your provider requires but that Bodhi App does not set by default

When you select the **Anthropic (Claude Code OAuth)** format, Bodhi App pre-fills these fields with the correct defaults. You can modify them if your use case requires different values.

### Step 4: Configure Model Prefix (Optional)

Model prefixes enable namespace separation and explicit provider routing when using multiple providers.

**What is Prefix Routing?**
Provider resolution happens based on the `model` field in the request. Prefixes allow you to explicitly control which provider handles the request, especially when multiple providers offer the same models.

**Real-World Example**:
Both OpenAI and OpenRouter can provide access to GPT-4. Using prefixes lets you choose which provider to use:

- **Without prefix** (OpenAI direct):
  - Model ID: `gpt-4`
  - Routes to: OpenAI provider (direct API)

- **With prefix** `openai/` (OpenAI explicit):
  - Configure OpenAI with prefix: `openai/`
  - Model ID: `openai/gpt-4`
  - Routes to: OpenAI provider (useful when you have multiple OpenAI configs)

**Steps**:

1. Toggle "Enable Prefix" switch
2. Enter prefix using provider identifier (e.g., `openai/`, `openrouter/`, `hf/`)
3. Prefix should be alphanumeric with a few allowed symbols, no spaces (maximum 32 characters)
4. All models from this provider will use the prefix

**When to Use Prefixes**:

- Multiple providers offer access to the same models (e.g., both OpenAI and OpenRouter provide GPT-4)
- You want explicit control over which provider handles the request
- You need clear provider identification in the model list
- Organizing models by source/provider

**When NOT to Use Prefixes**:

- You only configure one provider
- Each provider offers unique models with no name conflicts

**Trade-offs**:

- With prefix enabled, you must include it in model IDs when using Bodhi App's API
- Example: `openai/gpt-4` instead of `gpt-4`

### Forward All with Prefix

When a prefix is configured, you can enable **Forward All with Prefix** mode. Instead of selecting individual models from the provider, this mode forwards ALL requests whose model ID starts with the configured prefix to the provider.

**How it works**:

1. Configure a prefix (e.g., `fwd/`)
2. Select the "Forward All" radio button (this disables individual model selection)
3. Any request with a model ID like `fwd/gpt-4o`, `fwd/gpt-4o-mini`, etc. is automatically routed to the provider
4. The prefix is stripped before forwarding — `fwd/gpt-4o` becomes `gpt-4o` at the provider

**When to use Forward All**:

- You want access to every model a provider offers without selecting them one by one
- The provider frequently adds new models and you want automatic access
- You are using a provider like OpenRouter that aggregates many models

**Prefix uniqueness**: Each prefix must be unique across all API models. Attempting to create a second model with the same prefix will fail with an error.

### Step 5: Fetch Available Models

Discover which models are available from your API provider.

1. Click "Fetch Models" button
2. Bodhi App queries the provider's model listing endpoint (format-specific: `/models` for OpenAI, `/models` for Anthropic, `/models` for Gemini)
3. Model list appears in dropdown
4. Models are sorted by: Local models first, then grouped by provider, alphabetically within each group

**If Fetch Fails**:

- Verify API key is correct
- Check base URL is accurate
- Ensure network connectivity
- Review error message for specific issue

### Step 6: Select Models

Choose which models to make available in Bodhi App.

1. Click "Models" dropdown (multi-select)
2. Select one or more models
3. Selected models shown as chips
4. **Model Limit**: Maximum of 50 models per API provider

**Model Selection Tips**:

- Select models for different use cases (fast, capable, specialized)
- Models can be added or removed after API model creation using the Edit function

### Step 7: Test Connection (Recommended)

Validate configuration before saving to catch errors early.

1. Click "Test Connection" button
2. Bodhi App sends a format-appropriate test request to the provider
3. Success or error message displayed
4. For new models, the test uses the API key entered in the form. For existing API models, it uses the stored API key. The test makes a small request appropriate to the selected API format with retry logic.

**Format-specific test behavior**:

- **OpenAI Completions / Responses / Anthropic / Anthropic OAuth / Gemini**: Each format uses its own native API call for the connection test, so the test validates the full format configuration including any extra headers or body fields.

**Test Success**: Configuration is valid, proceed to save

**Test Failure Common Reasons**:

- Network failure is the most common connection failure reason
- API key invalidity might also cause failure
- Verify API key is correct
- Check base URL is accurate
- Ensure network connectivity

### Step 8: Save Configuration

1. Click "Create" or "Save" button
2. Configuration saved to database (API key encrypted)
3. Redirected to Models page
4. API models appear in unified model list

## Using Remote Models in Chat

Once configured, remote models appear alongside local models in chat.

**Steps**:

1. Open Chat page (<a href="/ui/chat/" target="_blank" rel="noopener noreferrer">/ui/chat/</a>)
2. Open Settings panel (right sidebar)
3. Click "Model" dropdown
4. Type in the model name selected in API model form
5. Chat with remote model as usual

**Visual Indicators**:

- Remote models are grouped by API provider in the model selector
- Models are grouped by the API provider and have the prefix if configured for easy identification

## Managing API Models

### Editing API Models

Update configuration, change API key, or modify model selection.

**Steps**:

1. Go to Models page (<a href="/ui/models/" target="_blank" rel="noopener noreferrer">/ui/models/</a>)
2. Locate API model in table
3. Click "Edit" button in Actions column
4. Modify fields as needed
5. API key shows placeholder (for security)
6. To update the API key, enter a new one in the field
7. Re-fetch models if needed
8. Test connection
9. Save changes

**Editing Capabilities**:

- Update base URL
- Add, change, or remove API key
- Modify prefix
- Toggle between Forward All mode and selected models mode
- Add or remove models (models can be added/removed after API model creation)

### Deleting API Models

Remove API model configuration when no longer needed.

**Steps**:

1. Go to Models page
2. Locate API model
3. Click "Delete" button
4. Confirm deletion
5. API model removed

**Effects**:

- Configuration deleted from database
- API key removed from storage
- Models no longer available in chat
- Existing chats using this model are preserved but cannot be continued (continuation may return an error)

### Viewing API Models

API models appear in unified Models page alongside local aliases.

**Display**:

- For API Model, action column shows top 3 available models from provider, clicking on it takes you to Chat UI with model preselected
- When multiple models from one provider: models are grouped by the API provider for easy selection
- API Format column shows API format
- File/Endpoint column shows Base URL configured
- Source badge: API models show "☁️ API" badge, local models show "model" badge

## Hybrid Usage Patterns

### Cost Optimization

Combine local and remote models based on task:

**Free (Local)**:

- Drafting and ideation
- Simple questions
- Privacy-sensitive queries
- High-volume tasks
- Offline scenarios

**Paid (Remote)**:

- Final output generation
- Complex reasoning
- Specialized tasks (coding, analysis)
- Latest capabilities
- Maximum quality requirements

### Privacy Strategy

**Local for Sensitive Data**:

- Personal information
- Proprietary code
- Confidential documents
- Medical or financial data
- Any data you cannot share externally

**Remote for Public Knowledge**:

- General questions
- Public information queries
- Learning and exploration
- Creative writing
- Standard coding tasks

### Performance Strategy

**Local for Speed Control**:

- You control inference speed
- Predictable performance
- No network latency
- Works offline

**Remote for Maximum Performance**:

- Latest hardware acceleration
- Faster inference for large models
- No local resource constraints
- Continuous improvements

## Troubleshooting

### API Key Invalid

**Symptoms**: "Invalid API key" error on test or usage

**Solutions**:

- Verify API key copied correctly (no extra spaces)
- Check key hasn't been revoked in provider dashboard
- Ensure key has required permissions
- Obtain API keys from your provider's dashboard - each provider has specific requirements, consult their documentation
- Generate new key if necessary

### Model Fetch Fails

**Symptoms**: Cannot retrieve model list from provider

**Solutions**:

- Verify API key is valid
- Check base URL is correct (e.g. `/v1` for OpenAI/Anthropic, `/v1beta` for Gemini)
- Ensure network connectivity
- Try testing connection first
- Review error message for specific issue
- For Anthropic OAuth, verify the OAuth token is current and starts with `sk-ant-oat01-`
- For Gemini, verify the API key is from Google AI Studio (not a service account key)

### Connection Test Fails

**Symptoms**: Test connection returns error

**Common Errors**:

- **"Invalid API key"**: The API key is incorrect. Verify your API key is copied correctly from the provider
- **"Network error"**: Unable to connect to the provider. Check your internet connection
- **"Connection failed"**: Cannot reach the provider. Verify the base URL is correct
- **"Model not found"**: The selected model is not available from this provider
- **"Rate limit exceeded"**: Too many requests. Wait a moment before trying again

**Solutions**:

- Verify each configuration field
- Check provider service status
- Review error message details
- Ensure API key permissions are sufficient

For additional troubleshooting, see the [Troubleshooting](/docs/troubleshooting) guide.

### Model Not Appearing in Chat

**Symptoms**: API model configured but not in chat dropdown

**Solutions**:

- Refresh page
- Verify models were selected during configuration
- Re-save API model configuration
- Check that models are properly listed on the Models page

### Chat Fails with API Model

**Symptoms**: Selected API model but chat returns error

**Solutions**:

- Verify API key is still valid
- Check provider account has credits/quota
- Test connection in API Models page
- Check error message for specific issue
- Ensure the model is still available from the provider

### Unexpected Costs

**Prevention**:

- Monitor usage in provider dashboards
- Set spending limits in provider accounts
- Use local models for high-volume tasks
- Review model pricing before selection
- Track costs per provider monthly

**Token Usage Tracking**: Check your provider's dashboard for token usage and billing information.

## Security Best Practices

### API Key Management

**Storage**:

- Bodhi App encrypts keys with AES-GCM
- Never commit keys to version control
- Don't share keys in plaintext
- Use environment variables for scripts

**API Key Updates**:

- No automatic key rotation
- Update keys manually anytime via the Edit API Model page
- Enter new key and click "Test Connection" to verify
- Changes take effect immediately

**Key Rotation Process**:

- Create new key in provider dashboard
- Update in Bodhi App using Edit API Model
- Test with new key to verify it works
- Delete old key from provider after confirming new key works

**Permissions**:

- Grant minimum required permissions
- Obtain API keys from your provider's dashboard - each provider has specific requirements, consult their documentation
- Review key permissions regularly
- Revoke unused keys

### Access Control

**Bodhi App Permissions**:

- Only authorized users can configure API models
- API keys are encrypted and cannot be viewed after creation
- Keys are securely stored and never displayed in plain text

### Monitoring

**Track Usage**:

- Review provider usage dashboards
- Set up spending alerts in provider accounts
- Monitor for unexpected spikes in your provider's dashboard

**Token Usage Tracking**: Currently not available in Bodhi App. Check your provider's dashboard for detailed usage information.

## API Format Details

### OpenAI - Completions

The standard OpenAI chat completions format. Works with any OpenAI-compatible API.

**Proxy endpoints exposed by Bodhi App**:

- `POST /v1/chat/completions` — chat completion (streaming supported)
- `POST /v1/embeddings` — embedding generation
- `GET /v1/models`, `GET /v1/models/{id}` — model listing

**Authentication**: API key sent as `Authorization: Bearer <key>`.

**Model fetch**: Bodhi App calls the provider's `/models` endpoint to retrieve available models.

**Providers**: OpenAI, OpenRouter, HuggingFace Inference API, or any OpenAI-compatible endpoint.

**Example request through Bodhi App**:

```bash
curl http://localhost:1135/v1/chat/completions \
  -H "Authorization: Bearer <bodhi-api-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

### OpenAI - Responses API

A pass-through proxy for the OpenAI Responses API. Use this format when your workflow requires stateful, multi-turn response objects instead of stateless chat completions.

**Proxy endpoints exposed by Bodhi App**:

- `POST /v1/responses` — create a response
- `GET /v1/responses/{response_id}` — retrieve a response
- `DELETE /v1/responses/{response_id}` — delete a response
- `GET /v1/responses/{response_id}/input_items` — list input items
- `POST /v1/responses/{response_id}/cancel` — cancel a response

All five operations are pure pass-through to the upstream OpenAI Responses endpoint.

**Authentication**: API key sent as `Authorization: Bearer <key>`.

**Important**: Chat completions (`/v1/chat/completions`) and embeddings (`/v1/embeddings`) reject models configured with the `openai_responses` format. Use those endpoints only with models configured as OpenAI - Completions.

**Example request through Bodhi App**:

```bash
curl http://localhost:1135/v1/responses \
  -H "Authorization: Bearer <bodhi-api-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "input": "Tell me a joke."
  }'
```

### Anthropic

Proxies to the Anthropic Messages API using a standard Anthropic API key.

**Proxy endpoints exposed by Bodhi App**:

- `POST /anthropic/v1/messages` — create a message (primary path)
- `POST /v1/messages` — alternative path (same handler)
- `GET /anthropic/v1/models` — list Anthropic models (served from local cache)
- `GET /anthropic/v1/models/{model_id}` — get model details (served from local cache)

**Authentication**: The `x-api-key` header or `Authorization: Bearer` are both accepted by Bodhi App's auth middleware; only one is needed. The key is forwarded to Anthropic as `x-api-key`.

**Model fetch**: Bodhi App calls Anthropic's models endpoint and caches the full model metadata (capabilities, context window, etc.).

**Example request through Bodhi App**:

```bash
curl http://localhost:1135/anthropic/v1/messages \
  -H "x-api-key: <bodhi-api-token>" \
  -H "anthropic-version: 2023-06-01" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 1024,
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

**Anthropic-prefixed headers**: Any request headers whose name starts with `anthropic-` (e.g. `anthropic-version`, `anthropic-beta`) are forwarded verbatim to the upstream Anthropic API.

### Anthropic (Claude Code OAuth)

The same Anthropic Messages API, but authenticated with an OAuth Bearer token issued by Claude Code. This format is designed for users who already use Claude Code and want to reuse its OAuth credentials in Bodhi App.

**Proxy endpoints**: Same as Anthropic (`/anthropic/v1/messages`, `/v1/messages`, model listing endpoints).

**Authentication**: Paste your Claude Code OAuth token (begins with `sk-ant-oat01-`) into the API key field. The token is sent as `Authorization: Bearer <token>`.

**Pre-filled defaults**: When you select this format, the Extra Headers and Extra Body fields are automatically populated with the values Claude Code OAuth requires:

- Extra Headers: `anthropic-version`, `anthropic-beta` (includes `claude-code-20250219,oauth-2025-04-20`), `user-agent`
- Extra Body: `max_tokens` (4096), `system` (sets the Claude Code system prompt)

You can adjust these defaults before saving.

**Obtaining the token**: The OAuth token is managed by Claude Code on your machine. The token is not generated within Bodhi App; refer to Claude Code documentation for how to retrieve or refresh it.

**Example request through Bodhi App**:

```bash
curl http://localhost:1135/anthropic/v1/messages \
  -H "Authorization: Bearer <bodhi-api-token>" \
  -H "anthropic-version: 2023-06-01" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 4096,
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

### Google Gemini

Proxies to the Google Gemini API using the `v1beta` endpoint. Supports text generation, streaming, and embeddings.

**Proxy endpoints exposed by Bodhi App**:

- `GET /v1beta/models` — list Gemini models (served from local cache)
- `GET /v1beta/models/{model_id}` — get model metadata (served from local cache)
- `POST /v1beta/models/{model}:generateContent` — generate content
- `POST /v1beta/models/{model}:streamGenerateContent` — stream generated content
- `POST /v1beta/models/{model}:embedContent` — embed content

**Authentication**: The `x-goog-api-key` header or `Authorization: Bearer` are both accepted by Bodhi App's auth middleware. The key is forwarded to Google as the API key.

**Telemetry headers**: Request headers whose name starts with `x-goog-` (e.g. `x-goog-api-client`, `x-goog-request-params`) are forwarded verbatim so Google SDK telemetry reaches Google.

**Query parameters**: Query parameters (e.g. `?alt=sse` for SSE streaming) are forwarded verbatim to the upstream.

**Model prefix support**: When a prefix is configured (e.g. `gemini/`), the prefix is stripped before forwarding to Google. Model names in the `name` field are returned as `models/{prefix}{model_id}` for round-tripping.

**Example request through Bodhi App**:

```bash
curl "http://localhost:1135/v1beta/models/gemini-1.5-pro:generateContent" \
  -H "x-goog-api-key: <bodhi-api-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "contents": [{"parts": [{"text": "Hello"}]}]
  }'
```

## Provider-Specific Notes

### OpenAI

**Available Models**:

- All current OpenAI models are fetched dynamically when you add the provider
- Models include GPT-4, o1, o3, and other offerings
- Refer to OpenAI's pricing page for latest model availability

**Considerations**:

- Rate limits vary by tier
- Function calling supported
- Check [OpenAI Pricing](https://openai.com/pricing) for current rates
- Use **OpenAI - Completions** for chat; use **OpenAI - Responses** if your tooling targets the Responses API

### OpenRouter

**Available Models**:

- All current OpenRouter models are fetched dynamically when you add the provider
- OpenRouter provides access to multiple AI providers through a single API

**Considerations**:

- Aggregates models from multiple providers (including Anthropic, Google, Meta, and others)
- Pricing varies by model — check OpenRouter dashboard
- Good option for accessing multiple providers with one API key
- Use the prefix feature to namespace models if you have multiple providers configured

### HuggingFace Inference API

**Available Models**:

- All current HuggingFace Inference API models are fetched dynamically when you add the provider
- Wide variety of open-source models available

**Considerations**:

- Open-source model ecosystem
- Pricing and availability vary by model
- Check HuggingFace documentation for model-specific details

### Anthropic

**Available Models**:

- Fetched from Anthropic's models API and cached locally with full metadata (capabilities, context window)
- Includes Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku, and newer models

**Considerations**:

- API key available from [Anthropic Console](https://console.anthropic.com/settings/keys)
- Both `/anthropic/v1/messages` and `/v1/messages` paths work identically
- Prefix routing works the same as other formats

### Google Gemini

**Available Models**:

- Fetched from Google's Gemini API and cached locally
- Includes Gemini 2.5 Flash, Gemini 1.5 Pro, Gemini 1.5 Flash, and others

**Considerations**:

- API key available from [Google AI Studio](https://aistudio.google.com/app/apikey)
- Streaming uses `?alt=sse` query parameter, forwarded automatically
- Model IDs in Bodhi App use the bare model name (e.g. `gemini-1.5-pro`); Bodhi App handles the `models/` path prefix internally

### Other Providers

**Custom Providers**: Any OpenAI-compatible API can be configured using the OpenAI - Completions format. For technical API specifications, see [API Reference](/docs/developer/openapi-reference).

## Related Documentation

- [Models](/docs/features/models/model-alias) - Local GGUF models and unified Models page
- [Chat Interface](/docs/features/chat/chat-ui) - Using models in chat
- [API Tokens](/docs/features/auth/api-tokens) - Programmatic access
- [Setup Wizard](/docs/install#post-installation-setup) - First-time setup
