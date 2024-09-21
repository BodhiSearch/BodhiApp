#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod auth_middleware;
mod bindings;
mod direct_sse;
mod error;
mod fwd_sse;
mod interactive;
mod oai;
mod obj_exts;
mod router_state;
mod routes;
mod routes_chat;
mod routes_dev;
mod routes_login;
mod routes_models;
mod routes_oai_models;
mod routes_ollama;
mod routes_proxy;
mod routes_setup;
mod routes_ui;
mod run;
mod serve;
mod server;
mod shared_rw;
mod shutdown;
mod tokenizer_config;
mod utils;

pub(crate) use auth_middleware::*;
pub use bindings::*;
pub(crate) use direct_sse::*;
pub use error::*;
pub(crate) use router_state::*;
pub(crate) use routes::*;
pub use run::{RunCommand, RunCommandError};
pub use serve::*;
pub use server::*;
pub use shared_rw::*;
pub(crate) use shutdown::*;
pub(crate) use utils::*;
