mod route_setup;
pub use route_setup::*;

#[cfg(test)]
#[path = "test_setup_auth.rs"]
mod test_setup_auth;
