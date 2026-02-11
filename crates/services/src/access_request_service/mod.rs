pub mod error;
pub mod service;

pub use error::AccessRequestError;
pub use service::{AccessRequestService, DefaultAccessRequestService};

#[cfg(any(test, feature = "test-utils"))]
pub use service::MockAccessRequestService;
