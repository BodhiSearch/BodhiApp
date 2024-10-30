use tokio::signal;

pub async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
    tracing::info!("received Ctrl+C, stopping server");
  };

  #[cfg(unix)]
  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
    tracing::info!("received SIGTERM, stopping server");
  };

  #[cfg(not(unix))]
  let terminate = async {
    signal::windows::ctrl_break()
      .expect("failed to install Ctrl+Break handler")
      .recv()
      .await;
    tracing::info!("received Ctrl+Break, stopping server");
  };

  tokio::select! {
    _ = ctrl_c => {},
    _ = terminate => {},
  }
}
