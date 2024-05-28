use super::server::{DEFAULT_HOST, DEFAULT_PORT_STR};
use clap::{ArgGroup, Parser, Subcommand};
use serde::Serialize;

#[derive(Debug, PartialEq, Parser)]
#[command(version)]
#[command(about = "Run GenerativeAI LLMs locally and serve them via OpenAI compatible API")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum Command {
  /// launch as native app
  App {},
  /// start the OpenAI compatible REST API server and Web UI
  Serve {
    /// Start with the given host, e.g. '0.0.0.0' to allow traffic from any ip on network
    #[clap(short='H', default_value = DEFAULT_HOST)]
    host: String,
    /// Start on the given port
    #[clap(short, default_value = DEFAULT_PORT_STR, value_parser = clap::value_parser!(u16).range(1..=65535))]
    port: u16,
  },
  /// Default: list the model aliases configured on local system
  #[clap(group = ArgGroup::new("variant"))]
  List {
    /// List pre-configured model aliases available to download and configure
    #[clap(long, short = 'r', group = "variant")]
    remote: bool,
    /// List the GGUF model files from Huggingface cache folder on local system
    #[clap(long, short = 'm', group = "variant")]
    models: bool,
  },
  /// Pull a gguf model from huggingface repository
  #[clap(group = ArgGroup::new("pull").required(true))]
  Pull {
    /// Download and configure the model using a pre-configured model alias.
    /// Run `bodhi list -r` to list all the pre-configured model aliases.
    #[clap(group = "pull")]
    alias: Option<String>,

    /// The hugging face repo to pull the model from, e.g. `bartowski/Meta-Llama-3-8B-Instruct-GGUF`
    #[clap(long, short = 'r', requires = "filename", group = "pull")]
    repo: Option<String>,

    /// The gguf model file to pull from the repo, e.g. `Meta-Llama-3-8B-Instruct-Q8_0.gguf`,
    /// or file pattern for sharded models `Meta-Llama-3-70B-Instruct.Q8_0-*.gguf`
    #[clap(long, short = 'f', requires = "repo")]
    filename: Option<String>,

    /// If the file already exists in $HF_HOME, force download it again
    #[clap(long = "force")]
    force: bool,
  },
  /// Run the given model in interactive mode.
  /// This command also downloads the model if not downloaded already.
  #[clap(group = ArgGroup::new("run").required(true))]
  #[clap(group = ArgGroup::new("using_alias").args(["alias"]).conflicts_with_all(["filename"]))]
  #[clap(group = ArgGroup::new("using_repo").args(["repo"]).requires_all(["filename"]))]
  Run {
    /// Download the model using model id.
    /// Run `bodhi list -r` to list all the pre-configured model ids.
    #[clap(group = "run")]
    alias: Option<String>,

    /// The hugging face repo to pull the model from, e.g. `bartowski/Meta-Llama-3-8B-Instruct-GGUF`
    #[clap(long, short = 'r', group = "run")]
    repo: Option<String>,

    /// The gguf model file to pull from the repo, e.g. `Meta-Llama-3-8B-Instruct-Q8_0.gguf`,
    /// or file pattern for sharded models `Meta-Llama-3-70B-Instruct.Q8_0-*.gguf`
    #[clap(long, short = 'f')]
    filename: Option<String>,
  },
}
/*
    #[clap(
      long,
      short = 't',
      requires = "repo",
      group = "template",
      help = r#"Configure the chat template using remote tokenizer_config.json
  Example: `--tokenizer_config meta-llama/Meta-Llama-3-70B-Instruct`
    "#
    )]
    tokenizer_config: Option<String>,

    /// Chat template to use for converting chat messages to LLM prompt.
    /// Ignored/not required if pulling model using <ID>.
    #[clap(long, short = 'c', requires = "repo", group = "template")]
    chat_template: Option<ChatTemplate>,
*/

#[derive(clap::ValueEnum, Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ChatTemplate {
  Llama3,
  Llama2,
  Llama2Legacy,
  Phi3,
  Gemma,
  Deepseek,
  CommandR,
  Openchat,
}

#[cfg(test)]
mod test {
  use super::*;
  use clap::CommandFactory;
  use rstest::rstest;

  #[test]
  fn test_cli_debug_assert() -> anyhow::Result<()> {
    Cli::command().debug_assert();
    Ok(())
  }

  #[test]
  fn test_cli_invalid() -> anyhow::Result<()> {
    let args = vec!["bodhi"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    Ok(())
  }

  #[test]
  fn test_cli_app() -> anyhow::Result<()> {
    let args = vec!["bodhi", "app"];
    let cli = Cli::try_parse_from(args)?;
    let expected = Command::App {};
    assert_eq!(expected, cli.command);
    Ok(())
  }

  #[test]
  fn test_cli_app_invalid() -> anyhow::Result<()> {
    let args = vec!["bodhi", "app", "--extra", "args"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    assert_eq!(
      r#"error: unexpected argument '--extra' found

Usage: bodhi app

For more information, try '--help'.
"#,
      cli.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "serve", "-H", "0.0.0.0", "-p", "8080"], "0.0.0.0", 8080)]
  #[case(vec!["bodhi", "serve", "-p", "8080"], "127.0.0.1", 8080)]
  #[case(vec!["bodhi", "serve", "-H", "0.0.0.0"], "0.0.0.0", 1135)]
  #[case(vec!["bodhi", "serve"], "127.0.0.1", 1135)]
  fn test_cli_serve(
    #[case] args: Vec<&str>,
    #[case] host: &str,
    #[case] port: u16,
  ) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args)?;
    let expected = Command::Serve {
      host: String::from(host),
      port,
    };
    assert_eq!(expected, cli.command);
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "serve", "-p", "65536"],
  r#"error: invalid value '65536' for '-p <PORT>': 65536 is not in 1..=65535

For more information, try '--help'.
"#)]
  #[case(vec!["bodhi", "serve", "-p", "0"],
  r#"error: invalid value '0' for '-p <PORT>': 0 is not in 1..=65535

For more information, try '--help'.
"#)]
  fn test_cli_serve_invalid(#[case] args: Vec<&str>, #[case] err_msg: &str) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    assert_eq!(err_msg, cli.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "list"], false, false)]
  #[case(vec!["bodhi", "list", "-r"], true, false)]
  #[case(vec!["bodhi", "list", "-m"], false, true)]
  fn test_cli_list(
    #[case] args: Vec<&str>,
    #[case] remote: bool,
    #[case] models: bool,
  ) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args)?;
    let expected = Command::List { remote, models };
    assert_eq!(expected, cli.command);
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "list", "-r", "-m"], r#"error: the argument '--remote' cannot be used with '--models'

Usage: bodhi list --remote

For more information, try '--help'.
"#)]
  fn test_cli_list_invalid(#[case] args: Vec<&str>, #[case] err_msg: String) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    assert_eq!(err_msg, cli.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "run", "llama3"], Some(String::from("llama3")), None, None)]
  #[case(vec!["bodhi", "run", "-r", "meta-llama/Meta-Llama-3-8B", "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf"], None, Some(String::from("meta-llama/Meta-Llama-3-8B")), Some(String::from("Meta-Llama-3-8B-Instruct.Q8_0.gguf")))]
  fn test_cli_run(
    #[case] args: Vec<&str>,
    #[case] alias: Option<String>,
    #[case] repo: Option<String>,
    #[case] filename: Option<String>,
  ) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args)?;
    let expected = Command::Run { alias, repo, filename };
    assert_eq!(expected, cli.command);
    Ok(())
  }

  #[rstest]
  #[case(
    vec!["bodhi", "run", "llama3", "-r", "meta-llama/Meta-Llama-3-8B", "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf"],
r#"error: the argument '[ALIAS]' cannot be used with:
  --repo <REPO>
  --filename <FILENAME>

Usage: bodhi run <ALIAS|--repo <REPO>>

For more information, try '--help'.
"#)]
  #[case(
    vec!["bodhi", "run", "llama3", "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf"],
r#"error: the argument '[ALIAS]' cannot be used with '--filename <FILENAME>'

Usage: bodhi run <ALIAS|--repo <REPO>>

For more information, try '--help'.
"#)]
  #[case(
    vec!["bodhi", "run", "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf"],
r#"error: the following required arguments were not provided:
  <ALIAS|--repo <REPO>>

Usage: bodhi run --filename <FILENAME> <ALIAS|--repo <REPO>>

For more information, try '--help'.
"#)]
  #[case(
    vec!["bodhi", "run", "-r", "meta-llama/Meta-Llama-3-8B"],
r#"error: the following required arguments were not provided:
  --filename <FILENAME>

Usage: bodhi run --filename <FILENAME> <ALIAS|--repo <REPO>>

For more information, try '--help'.
"#)]
  fn test_cli_run_invalid(#[case] args: Vec<&str>, #[case] err_msg: &str) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    assert_eq!(err_msg, cli.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "pull", "llama3:instruct"], Some(String::from("llama3:instruct")), None, None, false)]
  #[case(vec!["bodhi",
      "pull",
      "-r", "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
      "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf",
    ],
    None,
    Some(String::from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF")),
    Some(String::from("Meta-Llama-3-8B-Instruct.Q8_0.gguf")),
    false
  )]
  #[case(vec![ "bodhi", "pull",
      "-r", "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
      "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf",
    ],
    None,
    Some(String::from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF")),
    Some(String::from("Meta-Llama-3-8B-Instruct.Q8_0.gguf")),
    false
  )]
  #[case(vec![ "bodhi", "pull",
      "-r", "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
      "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf"
  ],
    None,
    Some(String::from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF")),
    Some(String::from("Meta-Llama-3-8B-Instruct.Q8_0.gguf")),
    false
  )]
  fn test_cli_pull_valid(
    #[case] args: Vec<&str>,
    #[case] alias: Option<String>,
    #[case] repo: Option<String>,
    #[case] filename: Option<String>,
    #[case] force: bool,
  ) -> anyhow::Result<()> {
    let actual = Cli::try_parse_from(args)?.command;
    let expected = Command::Pull {
      alias,
      repo,
      filename,
      force,
    };
    assert_eq!(expected, actual);
    Ok(())
  }

  #[rstest]
  #[case(
    vec!["bodhi", "pull", "llama3:instruct", "-r", "meta-llama/Meta-Llama-3-8B", "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf"],
r#"error: the argument '[ALIAS]' cannot be used with '--repo <REPO>'

Usage: bodhi pull --filename <FILENAME> <ALIAS|--repo <REPO>>

For more information, try '--help'.
"#)]
  fn test_cli_pull_invalid(#[case] args: Vec<&str>, #[case] err_msg: &str) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    assert_eq!(err_msg, cli.unwrap_err().to_string());
    Ok(())
  }
}
