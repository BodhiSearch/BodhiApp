# Tech Debt Backlog

Deferred work items intentionally scoped out of their originating effort, to be addressed later.

## Ollama `/api/chat` is not covered by the inference grant middleware
- **Source**: Tokens screen-v2 / App Token grants review — Batch 3 (2026-06-30).
- **What**: `model_inference_grant_middleware` (`routes_app/src/middleware/model_grant.rs`) uniformly
  enforces per-model grants on the OpenAI (`/v1/chat/completions`, `/v1/embeddings`), OpenAI-Responses
  (`/v1/responses`), Anthropic (`/v1/messages`, `/anthropic/v1/messages`), and Gemini
  (`/v1beta/models/{model}:{action}`) inference surfaces. `ollama_model_chat_handler` (`/api/chat`) is
  an inference path that is **not** classified by the middleware and has no in-handler check either —
  a scoped token / external app can run inference on any model via the Ollama endpoint.
- **Why deferred**: out of the named scope for this effort (deliberate decision, 2026-06-30).
- **Fix**: add `"/api/chat" => InferenceFormat::OpenAi` (model in body) to `classify()` and attach the
  middleware to the Ollama route group; or confirm Ollama is intentionally session-only.

## Relocate access-request handlers to a dedicated `access_requests` module + normalize endpoint paths
- **Source**: Tokens screen-v2 / App Token grants review — finding F32 (`docs/claude-plans/202606/screen-v2/tokens-review/`).
- **Date logged**: 2026-06-30
- **What**: `access-requests` is a domain model, but its handlers currently live under
  `routes_app/src/apps/`. Move them to a dedicated `routes_app/src/access_requests/` module and
  normalize the endpoint path set to the domain-first shape:
  ```
  ENDPOINT_ACCESS_REQUESTS_REVIEW   = "/bodhi/v1/access-requests/{id}/review"
  ENDPOINT_ACCESS_REQUESTS_APPROVE  = "/bodhi/v1/access-requests/{id}/approve"
  ENDPOINT_ACCESS_REQUESTS_DENY     = "/bodhi/v1/access-requests/{id}/deny"
  ENDPOINT_ACCESS_REQUESTS_APPS     = "/bodhi/v1/access-requests/apps"
  ENDPOINT_ACCESS_REQUESTS_REVOKE   = "/bodhi/v1/access-requests/{id}/revoke"
  ```
- **Why deferred**: expands scope beyond the grants review; touches routing, OpenAPI, ts-client,
  and frontend wiring.
- **Blocks these review fixes** (discovered during Batch 1 impl, 2026-06-30): the *app* access-request
  endpoints currently share the `/bodhi/v1/access-requests/{id}/...` namespace with the *user*
  (Manager/Admin) access-request domain (`users/routes_users_access_request.rs`), separated only by
  HTTP method (app approve = `PUT`, user approve = `POST`, both at `…/{id}/approve`; user list =
  `GET /bodhi/v1/access-requests`). Until the app endpoints are relocated to their own namespace
  (e.g. `/bodhi/v1/app-access-requests/*`), these three review fixes are **infeasible** and are
  deferred here:
  - **F9** — app approve `PUT`→`POST` (collides with user-domain `POST …/approve` at the same path).
  - **F32** — drop `/apps` from the list endpoint (`/bodhi/v1/access-requests` is taken by the user
    domain's `listAllAccessRequests`).
  - **F33** — de-pluralize operationId `approveAppsAccessRequest` (a bare `approveAccessRequest`
    duplicates the user domain's operationId). Do this rename as part of the relocation.
- **Note / reconcile on landing**: the review's in-scope decision drops the `/apps` qualifier on
  the *user's* list endpoint (use `/bodhi/v1/access-requests/` to list a user's access-requests,
  since user access-requests now live under `/users/*` and the `/apps` disambiguation is
  redundant). When this debt item is picked up, reconcile `ENDPOINT_ACCESS_REQUESTS_APPS` with that
  decision. `/bodhi/v1/apps/*` remains a reserved prefix for endpoints accessed directly by 3rd-party
  apps — that placement is acceptable and is **not** a reason to move on its own.
- **Reference**: see `decision.md` (F32) in the tokens-review folder for the full rationale.
</content>
