use super::{DEFAULT_HOST, DEFAULT_PORT_STR};
use crate::pull::Pull;
use clap::{Parser, Subcommand};
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
  /// start the OpenAI compatible REST API server and Web UI
  Serve {
    /// Start with the given host, e.g. '0.0.0.0' to allow traffic from any ip on network
    #[clap(short='H', default_value = DEFAULT_HOST)]
    host: String,
    /// Start on the given port
    #[clap(short, default_value = DEFAULT_PORT_STR)]
    port: u16,
    /// Load the GGUF model from the given path
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
  /// List all the models available on this machine
  List {},
}

impl Command {
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
  use crate::pull::Pull;

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
