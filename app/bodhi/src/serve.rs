use crate::server::ServerParams;

#[derive(Debug, Clone)]
pub struct Serve {
  pub host: String,
  pub port: u16,
}

#[allow(clippy::from_over_into)]
impl Into<ServerParams> for Serve {
  fn into(self) -> ServerParams {
    ServerParams {
      host: self.host,
      port: self.port,
    }
  }
}
