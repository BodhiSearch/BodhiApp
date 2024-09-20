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
mod shared_rw;
mod tokenizer_config;
mod utils;

pub(crate) use auth_middleware::*;
pub(crate) use bindings::*;
pub(crate) use direct_sse::*;
pub(crate) use error::*;
pub(crate) use fwd_sse::*;
pub(crate) use interactive::*;
pub(crate) use oai::*;
pub(crate) use router_state::*;
pub(crate) use routes::*;
pub(crate) use routes_chat::*;
pub(crate) use routes_dev::*;
pub(crate) use routes_login::*;
pub(crate) use routes_models::*;
pub(crate) use routes_oai_models::*;
pub(crate) use routes_ollama::*;
pub(crate) use routes_proxy::*;
pub(crate) use routes_setup::*;
pub(crate) use routes_ui::*;
pub(crate) use shared_rw::*;
pub(crate) use tokenizer_config::*;
pub(crate) use utils::*;

// pub mod run;
// pub mod serve;
// #[allow(clippy::module_inception)]
// mod server;
// mod shutdown;
// mod utils;
// pub use crate::server::router_state::{RouterState, RouterStateFn};
// pub use crate::server::routes::build_routes;
// pub use crate::server::server::*;
// pub use crate::server::shutdown::shutdown_signal;
// pub use crate::server::utils::AxumRequestExt;
// pub use auth_middleware::*;
// pub use error::*;
