mod router_state;
mod routes;
mod routes_chat;
mod routes_models;
mod routes_ui;
#[allow(clippy::module_inception)]
mod server;
mod shutdown;
mod utils;
pub use crate::server::routes::build_routes;
pub use crate::server::server::*;
pub use crate::server::shutdown::shutdown_signal;
pub use crate::server::utils::{BODHI_HOME, DEFAULT_HOST, DEFAULT_PORT_STR};
