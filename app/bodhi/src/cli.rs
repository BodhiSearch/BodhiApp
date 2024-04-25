use super::{DEFAULT_HOST, DEFAULT_PORT_STR};
use crate::{pull::Pull, ServerParams};
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
  /// Pull a gguf model from huggingface repository
  Pull {
    /// The hugging face repo to pull the model from, e.g. `bartowski/Meta-Llama-3-8B-Instruct-GGUF`
    #[clap()]
    repo: String,
    /// The gguf model file to pull from the repo, e.g. `Meta-Llama-3-8B-Instruct-Q8_0.gguf`
    #[clap()]
    file: String,
    /// If the file already exists in $HF_HOME, force download it again
    #[clap(short = 'f', long = "force")]
    force: bool,
  },
}

impl Command {
  pub fn into_serve_params(self) -> anyhow::Result<(ServerParams, GptParams)> {
    if let Command::Serve { model, host, port } = self {
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
    } else {
      panic!("should not be called for non Command::Serve commands")
    }
  }

  pub fn into_pull_param(self) -> anyhow::Result<Pull> {
    if let Command::Pull { repo, file, force } = self {
      Ok(Pull { repo, file, force })
    } else {
      panic!("should not be called for non Command::Pull commands")
    }
  }
}

#[cfg(test)]
mod test {
  use super::Command;
  use crate::{pull::Pull, ServerParams};
  use llama_server_bindings::GptParams;
  use std::path::PathBuf;

  #[test]
  pub fn test_to_serve_params() -> anyhow::Result<()> {
    let command = Command::Serve {
      host: Some(String::from("0.0.0.0")),
      port: Some(8080),
      model: Some(PathBuf::from("models/llama-2-7B.gguf")),
    };
    let (server_params, gpt_params) = command.into_serve_params()?;
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

  #[test]
  fn test_into_pull_params() -> anyhow::Result<()> {
    let command = Command::Pull {
      repo: "bartowski/Meta-Llama-3-8B-Instruct-GGUF".to_string(),
      file: "Meta-Llama-3-8B-Instruct-Q8_0.gguf".to_string(),
      force: false,
    };
    let params = command.into_pull_param()?;
    assert_eq!(
      Pull {
        repo: "bartowski/Meta-Llama-3-8B-Instruct-GGUF".to_string(),
        file: "Meta-Llama-3-8B-Instruct-Q8_0.gguf".to_string(),
        force: false,
      },
      params
    );
    Ok(())
  }
}
