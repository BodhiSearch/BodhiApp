use super::{DEFAULT_HOST, DEFAULT_PORT_STR};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version)]
#[command(about = "Run GenerativeAI LLMs locally and serve them via OpenAI compatible API")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  /// start the server
  Serve {
    #[clap(short='H', default_value = DEFAULT_HOST)]
    host: Option<String>,
    #[clap(short, default_value = DEFAULT_PORT_STR)]
    port: Option<u16>,
    #[clap(short = 'm')]
    model: PathBuf,
  },
}
