use crate::server::ServerParams;
use llama_server_bindings::GptParams;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Serve {
  pub host: String,
  pub port: u16,
  pub model: Option<PathBuf>,
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

#[allow(clippy::from_over_into)]
impl Into<Option<GptParams>> for Serve {
  fn into(self) -> Option<GptParams> {
    match self.model {
      Some(model) => {
        let gpt_params = GptParams {
          model: Some(model.to_string_lossy().into_owned()),
          ..GptParams::default()
        };
        Some(gpt_params)
      }
      None => None,
    }
  }
}
