mod error;
mod inference_service;
mod noop;

pub use error::InferenceError;
#[cfg(any(test, feature = "test-utils"))]
pub use inference_service::MockInferenceService;
pub use inference_service::{InferenceService, LlmEndpoint};
pub use noop::NoopInferenceService;
