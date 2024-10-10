#![allow(unused_imports)]

mod bodhi;
mod error;
mod hf;
mod io;
mod l10n;
mod objs;

pub use bodhi::*;
pub use error::*;
pub use hf::*;
pub use io::*;
pub use l10n::*;
pub use objs::*;

pub static SNAPSHOT: &str = "5007652f7a641fe7170e0bad4f63839419bd9213";

#[ctor::ctor]
fn init_tracing() {
  use tracing_subscriber::fmt::format::FmtSpan;
  tracing_subscriber::fmt()
    .with_test_writer()
    .with_span_events(FmtSpan::FULL)
    .init();
}
