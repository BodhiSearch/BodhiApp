mod routes_alias;

#[cfg(test)]
#[path = "test_aliases_auth.rs"]
mod test_aliases_auth;

pub use routes_alias::*;
