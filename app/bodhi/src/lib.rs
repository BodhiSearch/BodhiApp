mod app;
mod error;
mod native;

pub use app::{main_internal, setup_logs};
pub use error::AppError;
pub(crate) use error::Result;

static PROD_DB: &str = "bodhi.sqlite";
