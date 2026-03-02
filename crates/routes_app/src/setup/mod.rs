mod error;
mod routes_setup;
mod setup_api_schemas;

#[cfg(test)]
#[path = "test_setup.rs"]
mod test_setup;

#[cfg(test)]
#[path = "test_setup_auth.rs"]
mod test_setup_auth;

pub use error::*;
pub use routes_setup::*;
pub use setup_api_schemas::*;
