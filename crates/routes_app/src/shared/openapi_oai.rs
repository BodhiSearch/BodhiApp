use crate::oai::{
  __path_chat_completions_handler, __path_embeddings_handler, __path_oai_model_handler,
  __path_oai_models_handler, __path_responses_cancel_handler, __path_responses_create_handler,
  __path_responses_delete_handler, __path_responses_get_handler,
  __path_responses_input_items_handler,
};
use crate::ollama::{
  __path_ollama_model_chat_handler, __path_ollama_model_show_handler, __path_ollama_models_handler,
};
use crate::{API_TAG_OLLAMA, API_TAG_OPENAI, API_TAG_RESPONSES};
use async_openai::error::{ApiError as OaiErrorBody, WrappedError as OaiWrappedError};
use async_openai::types::{
  chat::{
    ChatChoice, ChatChoiceStream, ChatCompletionRequestMessage, ChatCompletionResponseMessage,
    CompletionUsage, CreateChatCompletionRequest, CreateChatCompletionResponse,
    CreateChatCompletionStreamResponse,
  },
  embeddings::{
    CreateEmbeddingRequest, CreateEmbeddingResponse, Embedding, EmbeddingInput, EmbeddingUsage,
  },
  models::{ListModelResponse, Model},
  responses::{CreateResponse, DeleteResponse as OaiDeleteResponse, Response as OaiResponse},
};
use utoipa::OpenApi;

/// OpenAPI documentation for OpenAI- and Ollama-compatible LLM endpoints exposed by Bodhi App.
///
/// This spec is intentionally separate from `BodhiOpenAPIDoc` (which documents the
/// BodhiApp management API) so that LLM client SDKs and OpenAI-compatible tooling
/// can target a focused, namespace-isolated specification.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Bodhi App - OpenAI Compatible APIs",
        version = env!("CARGO_PKG_VERSION"),
        contact(
            name = "Bodhi API Support",
            url = "https://github.com/BodhiSearch/BodhiApp/issues",
            email = "support@getbodhi.app"
        ),
        description = r#"OpenAI Chat Completions, Embeddings, Responses API and Ollama-compatible endpoints exposed by Bodhi App.

Use the standard OpenAI or Ollama SDKs against these routes. Authentication options:

- **API Token** — `Authorization: Bearer bodhiapp_<token>` (created via the Bodhi web UI)
- **OAuth Token Exchange** — `Authorization: Bearer <oauth_exchanged_token>`
- **Browser Session** — login via `/bodhi/v1/auth/initiate`

For BodhiApp management endpoints (auth, settings, model aliases, MCPs, tokens, etc.) see the separate `openapi.json` spec.
"#
    ),
    servers(
        (url = "http://localhost:1135", description = "Local running instance"),
    ),
    tags(
        (name = API_TAG_OPENAI, description = "OpenAI-compatible API endpoints"),
        (name = API_TAG_RESPONSES, description = "OpenAI Responses API proxy endpoints"),
        (name = API_TAG_OLLAMA, description = "Ollama-compatible API endpoints"),
    ),
    components(
        schemas(
            // OpenAI-native error envelope used in 4xx/5xx responses
            // (emitted as `openai.WrappedError` + `openai.ApiError` in the spec)
            OaiWrappedError,
            OaiErrorBody,
            // openai
            ListModelResponse,
            Model,
            CreateChatCompletionRequest,
            CreateChatCompletionResponse,
            CreateChatCompletionStreamResponse,
            ChatCompletionRequestMessage,
            ChatCompletionResponseMessage,
            ChatChoice,
            ChatChoiceStream,
            CompletionUsage,
            CreateEmbeddingRequest,
            CreateEmbeddingResponse,
            Embedding,
            EmbeddingInput,
            EmbeddingUsage,
            // responses api
            CreateResponse,
            OaiResponse,
            OaiDeleteResponse,
        ),
        responses( ),
    ),
    paths(
        // OpenAI endpoints
        oai_models_handler,
        oai_model_handler,
        chat_completions_handler,
        embeddings_handler,

        // Responses API endpoints
        responses_create_handler,
        responses_get_handler,
        responses_delete_handler,
        responses_input_items_handler,
        responses_cancel_handler,

        // Ollama endpoints
        ollama_models_handler,
        ollama_model_show_handler,
        ollama_model_chat_handler,
    )
)]
pub struct BodhiOAIOpenAPIDoc;
