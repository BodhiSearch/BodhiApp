mod routes;
mod routes_chat;
mod routes_models;
mod routes_ui;
#[allow(clippy::module_inception)]
mod server;
mod shared_rw;
mod shutdown;
mod utils;
pub use crate::server::routes::build_routes;
pub use crate::server::routes_ui::{Chat, ChatPreview, Message};
pub use crate::server::server::*;
pub use crate::server::shared_rw::{SharedContextRw, SharedContextRwExts};
pub use crate::server::shutdown::shutdown_signal;
pub use crate::server::utils::{
  port_from_env_vars, BODHI_HOME, DEFAULT_HOST, DEFAULT_PORT, DEFAULT_PORT_STR,
};
