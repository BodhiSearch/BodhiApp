use bodhiserver::{build_server, port_from_env_vars, server::ServerHandle, shutdown_signal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static ENV_BODHISERVER_PORT: &str = "BODHISERVER_PORT";

#[tokio::main]
pub async fn main() {
  let result = launch_server().await;
  match result {
    Ok(_) => tracing::info!("Server shutdown successfully"),
    Err(err) => tracing::warn!(err = ?err, "Server shutdown with error"),
  }
}

async fn launch_server() -> anyhow::Result<()> {
  dotenv::dotenv().ok();
  tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
    .init();
  let port = port_from_env_vars(std::env::var(ENV_BODHISERVER_PORT));
  let ServerHandle { server, shutdown } = build_server(String::from("127.0.0.1"), port).await?;
  let server_join = tokio::spawn(async move {
    if let Err(err) = server.await {
      tracing::error!(err = ?err, "Server error");
    }
  });
  tokio::spawn(async move {
    shutdown_signal().await;
    shutdown.send(()).unwrap();
  });
  server_join.await?;
  Ok(())
}
