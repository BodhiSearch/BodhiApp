use bodhiserver::{build_server, ServerHandle};

#[tokio::test]
pub async fn test_build_server_ping() -> anyhow::Result<()> {
  let host = String::from("127.0.0.1");
  let port = rand::random::<u16>();
  let ServerHandle { server, shutdown } = build_server(host.clone(), port).await?;
  #[allow(clippy::redundant_async_block)]
  let join = tokio::spawn(async move { server.await });
  let response = reqwest::get(format!("http://{}:{}/ping", host, port))
    .await?
    .text()
    .await?;
  assert_eq!(response, "pong");
  shutdown.send(()).unwrap();
  (join.await?)?;
  Ok(())
}
