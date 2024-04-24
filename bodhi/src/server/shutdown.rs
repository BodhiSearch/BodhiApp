use tokio::signal::{self, unix::SignalKind};

pub async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
    tracing::info!("received Ctrl+C, stopping server");
  };

  #[cfg(unix)]
  let terminate = async {
    signal::unix::signal(SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
    tracing::info!("received SIGTERM, stopping server");
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();
  tokio::select! {
      _ = ctrl_c => {},
      _ = terminate => {},
  }
}
