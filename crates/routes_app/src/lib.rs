#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod api_models_dto;
mod error;
mod objs;
mod openapi;
mod routes_api_models;
mod routes_api_token;
mod routes_create;
mod routes_dev;
mod routes_login;
mod routes_models;
mod routes_pull;
mod routes_settings;
mod routes_setup;
mod routes_user;
mod utils;

pub use api_models_dto::*;
pub use error::*;
pub use objs::*;
pub use openapi::*;
pub use routes_api_models::*;
pub use routes_api_token::*;
pub use routes_create::*;
pub use routes_dev::*;
pub use routes_login::*;
pub use routes_models::*;
pub use routes_pull::*;
pub use routes_settings::*;
pub use routes_setup::*;
pub use routes_user::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
