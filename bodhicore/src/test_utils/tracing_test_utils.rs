use tracing_subscriber::{fmt, EnvFilter};

pub fn init_test_tracing() {
  let filter = EnvFilter::from_default_env(); // Use RUST_LOG environment variable
  let subscriber = fmt::Subscriber::builder()
    .with_env_filter(filter) // Set the filter to use the RUST_LOG environment variable
    .finish();
  let _ = tracing::subscriber::set_global_default(subscriber);
}
