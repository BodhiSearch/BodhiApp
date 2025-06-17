#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod app_initializer;
mod config;

// Re-export main types for JavaScript consumption
pub use app_initializer::{BodhiApp, NapiAppState};
pub use config::AppConfig;
