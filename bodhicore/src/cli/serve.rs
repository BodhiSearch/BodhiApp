use crate::server::ServerParams;
use derive_new::new;

#[derive(Debug, Clone, new)]
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

#[cfg(test)]
mod test {
  use crate::{server::ServerParams, Serve};

  #[test]
  fn test_serve_into() -> anyhow::Result<()> {
    let params = Serve::new("localhost".to_string(), 1135);
    let server_params: ServerParams = params.into();
    let expected = ServerParams::new("localhost".to_string(), 1135);
    assert_eq!(expected, server_params);
    Ok(())
  }
}
