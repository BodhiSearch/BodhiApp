use super::{DEFAULT_HOST, DEFAULT_PORT_STR};
use crate::ServerParams;
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use llama_server_bindings::GptParams;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version)]
#[command(about = "Run GenerativeAI LLMs locally and serve them via OpenAI compatible API")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
  /// start the server
  Serve {
    #[clap(short='H', default_value = DEFAULT_HOST)]
    host: Option<String>,
    #[clap(short, default_value = DEFAULT_PORT_STR)]
    port: Option<u16>,
    #[clap(short = 'm')]
    model: Option<PathBuf>,
  },
}

impl Command {
  pub fn to_params(self) -> anyhow::Result<(ServerParams, GptParams)> {
    match self {
      Command::Serve { model, host, port } => {
        let mut gpt_params = GptParams::default();
        if let Some(model) = model {
          let model = model
            .to_str()
            .ok_or_else(|| anyhow!("failed to convert path to string"))?
            .to_owned();
          gpt_params.model = Some(model);
        }
        let mut server_params = ServerParams::default();
        if let Some(host) = host {
          server_params.host = host;
        }
        if let Some(port) = port {
          server_params.port = port;
        }
        Ok((server_params, gpt_params))
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::Command;
  use crate::ServerParams;
  use llama_server_bindings::GptParams;
  use std::path::PathBuf;

  #[test]
  pub fn test_to_params() -> anyhow::Result<()> {
    let command = Command::Serve {
      host: Some(String::from("0.0.0.0")),
      port: Some(8080),
      model: Some(PathBuf::from("models/llama-2-7B.gguf")),
    };
    let (server_params, gpt_params) = command.to_params()?;
    let expected = GptParams {
      seed: None,
      n_predict: None,
      n_ctx: None,
      model: Some("models/llama-2-7B.gguf".to_string()),
      embedding: None,
    };
    assert_eq!(expected, gpt_params);
    let expected = ServerParams {
      host: String::from("0.0.0.0"),
      port: 8080,
    };
    assert_eq!(expected, server_params);
    Ok(())
  }
}
