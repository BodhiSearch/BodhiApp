pub fn enable_tracing() {
  use tracing_subscriber::fmt::format::FmtSpan;

  tracing_subscriber::fmt()
    .with_test_writer()
    .with_span_events(FmtSpan::FULL)
    .init();
}
