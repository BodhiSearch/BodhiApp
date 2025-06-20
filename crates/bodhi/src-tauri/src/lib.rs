#[cfg(feature = "native")]
mod native_init;
#[cfg(not(feature = "native"))]
mod server_init;

#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

pub mod app;
mod common;
pub mod env;
mod error;
mod ui;
