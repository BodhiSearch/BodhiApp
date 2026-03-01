use rstest::fixture;

#[fixture]
#[once]
pub fn enable_tracing() -> () {
  use tracing_subscriber::fmt::format::FmtSpan;

  tracing_subscriber::fmt()
    .with_test_writer()
    .with_span_events(FmtSpan::FULL)
    .with_env_filter("tower_sessions=off,tower_sessions_core=off")
    .init();
}
