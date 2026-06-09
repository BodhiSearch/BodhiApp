mod gemini_api_schemas;
mod routes_gemini;

pub use gemini_api_schemas::*;
pub use routes_gemini::*;

pub const ENDPOINT_GEMINI_MODELS: &str = "/v1beta/models";
/// Single path pattern for BOTH GET (model lookup) and POST (action dispatch).
/// Wildcard `{*model_path}` so prefixed aliases (e.g. `gem/gemini-flash-latest`)
/// don't 404 — Axum's `{id}` only matches a single path segment.
pub const ENDPOINT_GEMINI_MODEL: &str = "/v1beta/models/{*model_path}";
