use super::server::{DEFAULT_HOST, DEFAULT_PORT_STR};
use clap::{ArgGroup, Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
#[command(about = "Run GenerativeAI LLMs locally and serve them via OpenAI compatible API")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
  /// launch as native app
  App {},
  /// start the OpenAI compatible REST API server and Web UI
  Serve {
    /// Start with the given host, e.g. '0.0.0.0' to allow traffic from any ip on network
    #[clap(short='H', default_value = DEFAULT_HOST)]
    host: String,
    /// Start on the given port
    #[clap(short, default_value = DEFAULT_PORT_STR)]
    port: u16,
  },
  /// Pull a gguf model from huggingface repository
  #[clap(group = ArgGroup::new("pull").required(true))]
  Pull {
    /// Download the model using model id.
    /// Run `bodhi list -r` to list all the pre-configured model ids.
    #[clap(group = "pull")]
    id: Option<String>,

    /// The hugging face repo to pull the model from, e.g. `bartowski/Meta-Llama-3-8B-Instruct-GGUF`
    #[clap(long, short = 'r', requires = "file", group = "pull")]
    repo: Option<String>,

    /// The gguf model file to pull from the repo, e.g. `Meta-Llama-3-8B-Instruct-Q8_0.gguf`,
    /// or file pattern for sharded models `Meta-Llama-3-70B-Instruct.Q8_0-*.gguf`
    #[clap(long, short = 'f', requires = "repo")]
    file: Option<String>,

    #[clap(long, short = 'c', requires = "repo", help=r#"Configure the downlaoded model using a remote, local or inline tokenizer_config.json
    - for remote - `--config https://huggingface.co/meta-llama/Meta-Llama-3-70B-Instruct/blob/main/tokenizer_config.json`
    - for local - `--config '/Users/foobar/Downloads/tokenizer_config.json'` // does not resolve env_vars like $HOME, `~`
    - for inline - `--config '{"chat_template": "{% for message in messages %}<|{{ message[\'role\'] }}|> {{ message[\'content\'] }}\n{% endfor %}"}'`
    "#)]
    config: Option<String>,

    /// If the file already exists in $HF_HOME, force download it again
    #[clap(long = "force")]
    force: bool,
  },
  /// List all the models downloaded locally,
  /// and pre-configured models available for download
  List {
    /// List pre-configured models available for download from remote
    #[clap(long, short = 'r')]
    remote: bool,
  },
  /// Run the given model in interactive mode.
  /// This command also downloads the model if not downloaded already.
  #[clap(group = ArgGroup::new("run").required(true))]
  Run {
    /// Download the model using model id.
    /// Run `bodhi list -r` to list all the pre-configured model ids.
    #[clap(group = "run")]
    id: Option<String>,

    /// The hugging face repo to pull the model from, e.g. `bartowski/Meta-Llama-3-8B-Instruct-GGUF`
    #[clap(long, short = 'r', requires = "file", group = "run")]
    repo: Option<String>,

    /// The gguf model file to pull from the repo, e.g. `Meta-Llama-3-8B-Instruct-Q8_0.gguf`,
    /// or file pattern for sharded models `Meta-Llama-3-70B-Instruct.Q8_0-*.gguf`
    #[clap(long, short = 'f', requires = "repo")]
    file: Option<String>,
  },
}
