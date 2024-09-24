use clap::{ArgGroup, Parser, Subcommand};
use objs::{ChatTemplateId, GptContextParams, OAIRequestParams, GGUF_EXTENSION, REGEX_REPO};
use services::{DEFAULT_HOST, DEFAULT_PORT_STR};
use strum::Display;

#[derive(Debug, PartialEq, Parser)]
#[command(name = "bodhi")]
#[command(version)]
#[command(about = "Run GenerativeAI LLMs locally and serve them via OpenAI compatible API")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Debug, PartialEq, Subcommand, Display)]
#[strum(serialize_all = "lowercase")]
#[allow(clippy::large_enum_variant)]
pub enum Command {
  /// list down the environment variables for current runtime
  Envs {},
  /// launch as native app
  App {
    /// open the browser with chat interface
    #[clap(long)]
    ui: bool,
  },
  /// start the OpenAI compatible REST API server and Web UI
  Serve {
    /// Start with the given host, e.g. '0.0.0.0' to allow traffic from any ip on network
    #[clap(short='H', default_value = DEFAULT_HOST)]
    host: String,
    /// Start on the given port
    #[clap(short, default_value = DEFAULT_PORT_STR, value_parser = clap::value_parser!(u16).range(1..=65535))]
    port: u16,
  },
  /// list the model aliases on local
  #[clap(group = ArgGroup::new("variant"))]
  List {
    /// List pre-configured model aliases available to download and configure in a single quickstart command
    #[clap(long, short = 'r', group = "variant")]
    remote: bool,
    /// List the compatible GGUF model files from $HF_HOME folder on local system
    #[clap(long, short = 'm', group = "variant")]
    models: bool,
  },
  /// Pull a compatible GGUF model from huggingface.co repository
  #[clap(group = ArgGroup::new("pull").required(true))]
  Pull {
    /// Download and configure the model using a pre-configured model alias,
    /// run `bodhi list -r` to list all the pre-configured model aliases
    #[clap(group = "pull")]
    alias: Option<String>,

    /// The hugging face repo to pull the model from, e.g. `TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF`
    #[clap(long, short = 'r', requires = "filename", group = "pull", value_parser = repo_parser)]
    repo: Option<String>,

    /// The GGUF model file to pull from the repo, e.g. `tinyllama-1.1b-chat-v1.0.Q4_0.gguf`,
    #[clap(long, short = 'f', requires = "repo", value_parser = gguf_filename_parser)]
    filename: Option<String>,

    /// The snapshot or branch name of the repo to pull the model from, e.g. `main`, `abcdsha`
    #[clap(long, short = 's', requires = "repo")]
    snapshot: Option<String>,
  },

  /// Create or update a model alias
  #[clap(group = ArgGroup::new("template").required(true))]
  Create {
    /// Unique name of the model alias. E.g. llama3:8b-instruct, model alias should not be present,
    /// run `bodhi list` to list the existing model aliases
    alias: String,

    /// The hugging face repo to pull the model from, e.g. `TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF`
    #[clap(long, short = 'r', value_parser = repo_parser)]
    repo: String,

    /// The gguf model file to pull from the repo, e.g. `tinyllama-1.1b-chat-v1.0.Q4_0.gguf`,
    #[clap(long, short = 'f', requires = "repo", value_parser = gguf_filename_parser)]
    filename: String,

    /// The snapshot or branch name of the repo to pull the model from, e.g. `main`, `abcdsha`
    #[clap(long, short = 's', requires = "repo")]
    snapshot: Option<String>,

    /// In-built chat template mapping to use to convert chat messages to LLM prompt
    #[clap(long, group = "template")]
    chat_template: Option<ChatTemplateId>,

    /// Repo containing tokenizer_config.json file to convert chat messages to LLM prompt. e.g. `TinyLlama/TinyLlama-1.1B-Chat-v1.0`
    #[clap(long, group = "template", value_parser = repo_parser)]
    tokenizer_config: Option<String>,

    /// Optional meta information, family of the model
    #[clap(long)]
    family: Option<String>,

    /// Update the existing alias if it already exists
    #[clap(long)]
    update: bool,

    #[clap(flatten, next_help_heading = "OpenAI Compatible Request defaults")]
    oai_request_params: OAIRequestParams,

    #[clap(flatten, next_help_heading = "Model Context defaults")]
    context_params: GptContextParams,
  },
  /// Run the given model alias in interactive mode.
  Run {
    /// Model alias to run, run `bodhi list` to list the existing model aliases
    alias: String,
  },
  /// Display the given alias configuration
  Show {
    /// Model alias to show, run `bodhi list` to list the existing model aliases
    alias: String,
  },
  /// Make a copy of given ALIAS using the NEW-ALIAS id
  Cp {
    /// Source alias to copy from, run `bodhi list` to list the existing model aliases
    alias: String,
    /// New destination alias name, should not be already present
    new_alias: String,
  },
  /// Edit the given alias yaml in external editor $EDITOR
  Edit {
    /// Model alias to edit, run `bodhi list` to list the existing model aliases
    alias: String,
  },
  /// Delete the given alias configuration
  Rm {
    /// Model alias to delete, run `bodhi list` to list the existing model aliases
    alias: String,
  },
}

fn repo_parser(repo: &str) -> Result<String, String> {
  if REGEX_REPO.is_match(repo) {
    Ok(repo.to_string())
  } else {
    Err("does not match huggingface repo format - `owner/repo`".to_string())
  }
}

fn gguf_filename_parser(filename: &str) -> Result<String, String> {
  if filename.ends_with(GGUF_EXTENSION) {
    Ok(filename.to_string())
  } else {
    Err("only GGUF file extension supported".to_string())
  }
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
mod test {
  use crate::{Cli, Command};
  use clap::{CommandFactory, Parser};
  use objs::{ChatTemplateId, GptContextParams, OAIRequestParams};
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
    let expected = Command::App { ui: false };
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

Usage: bodhi app [OPTIONS]

For more information, try '--help'.
"#,
      cli.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "serve", "-H", "0.0.0.0", "-p", "8080"], "0.0.0.0", 8080)]
  #[case(vec!["bodhi", "serve", "-p", "8080"], "localhost", 8080)]
  #[case(vec!["bodhi", "serve", "-H", "0.0.0.0"], "0.0.0.0", 1135)]
  #[case(vec!["bodhi", "serve"], "localhost", 1135)]
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
  #[case(vec!["bodhi", "run", "llama3:instruct"], "llama3:instruct")]
  fn test_cli_run(#[case] args: Vec<&str>, #[case] alias: String) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args)?;
    let expected = Command::Run { alias };
    assert_eq!(expected, cli.command);
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "pull", "llama3:instruct"], Some(String::from("llama3:instruct")), None, None,None)]
  #[case(vec!["bodhi", "pull",
      "-r", "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
      "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf",
    ],
    None,
    Some(String::from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF")),
    Some(String::from("Meta-Llama-3-8B-Instruct.Q8_0.gguf")), None)]
  #[case(vec!["bodhi", "pull",
      "-r", "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
      "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf",
      "-s", "main"
    ],
    None,
    Some(String::from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF")),
    Some(String::from("Meta-Llama-3-8B-Instruct.Q8_0.gguf")), Some("main".to_string()))]
  #[case(vec![ "bodhi", "pull",
      "-r", "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
      "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf",
      "-s", "191239b3e26b2882fb562ffccdd1cf0f65402adb",
    ],
    None,
    Some(String::from("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF")),
    Some(String::from("Meta-Llama-3-8B-Instruct.Q8_0.gguf")), Some("191239b3e26b2882fb562ffccdd1cf0f65402adb".to_string()))]
  fn test_cli_pull_valid(
    #[case] args: Vec<&str>,
    #[case] alias: Option<String>,
    #[case] repo: Option<String>,
    #[case] filename: Option<String>,
    #[case] snapshot: Option<String>,
  ) -> anyhow::Result<()> {
    let actual = Cli::try_parse_from(args)?.command;
    let expected = Command::Pull {
      alias,
      repo,
      filename,
      snapshot,
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
  #[case(
    vec!["bodhi", "pull", "-r", "meta-llama$Meta-Llama-3-8B", "-f", "Meta-Llama-3-8B-Instruct.Q8_0.gguf"],
r#"error: invalid value 'meta-llama$Meta-Llama-3-8B' for '--repo <REPO>': does not match huggingface repo format - `owner/repo`

For more information, try '--help'.
"#)]
  #[case(
    vec!["bodhi", "pull", "-r", "meta-llama/Meta-Llama-3-8B", "-f", "Meta-Llama-3-8B-Instruct.Q8_0.safetensor"],
r#"error: invalid value 'Meta-Llama-3-8B-Instruct.Q8_0.safetensor' for '--filename <FILENAME>': only GGUF file extension supported

For more information, try '--help'.
"#)]
  fn test_cli_pull_invalid(#[case] args: Vec<&str>, #[case] err_msg: &str) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    assert_eq!(err_msg, cli.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[case(vec![
    "bodhi", "create",
    "testalias:instruct",
    "--repo", "MyFactory/testalias-gguf",
    "--filename", "testalias.Q8_0.gguf",
    "--family", "testalias",
    "--chat-template", "llama3"
  ],
    "testalias:instruct",
    "MyFactory/testalias-gguf",
    "testalias.Q8_0.gguf",
    None,
    "testalias",
    ChatTemplateId::Llama3,
    false,
    OAIRequestParams::default(),
    GptContextParams::default(),
  )]
  #[case(vec![
    "bodhi", "create",
    "testalias:instruct",
    "--repo", "MyFactory/testalias-gguf",
    "--filename", "testalias.Q8_0.gguf",
    "--family", "testalias",
    "--chat-template", "llama3",
    "--frequency-penalty", "0.8",
    "--max-tokens", "512",
    "--presence-penalty", "1.1",
    "--seed", "42",
    "--stop", "\n",
    "--stop", "\n\n",
    "--temperature", "0.8",
    "--top-p", "0.9",
    "--user", "testuser",
    "--n-threads", "6",
    "--n-ctx", "1024",
    "--n-parallel", "4",
    "--n-predict", "512",
    "--n-keep", "4",
  ],
    "testalias:instruct".to_string(),
    "MyFactory/testalias-gguf".to_string(),
    "testalias.Q8_0.gguf".to_string(),
    None,
    "testalias".to_string(),
    ChatTemplateId::Llama3,
    false,
    OAIRequestParams {
      frequency_penalty: Some(0.8),
      max_tokens: Some(512),
      presence_penalty: Some(1.1),
      seed: Some(42),
      stop: vec!["\n".to_string(), "\n\n".to_string()],
      temperature: Some(0.8),
      top_p: Some(0.9),
      user: Some("testuser".to_string())
    },
    GptContextParams {
      n_seed: None,
      n_threads:Some(6),
      n_ctx: Some(1024),
      n_parallel: Some(4),
      n_predict: Some(512),
      n_keep: Some(4),
    }
  ,
  )]
  #[case(vec![
    "bodhi", "create",
    "testalias:instruct",
    "--repo", "MyFactory/testalias-gguf",
    "--filename", "testalias.Q8_0.gguf",
    "--snapshot", "abcdsha1",
    "--family", "testalias",
    "--chat-template", "llama3",
    "--update"
  ],
    "testalias:instruct",
    "MyFactory/testalias-gguf",
    "testalias.Q8_0.gguf",
    Some("abcdsha1".to_string()),
    "testalias",
    ChatTemplateId::Llama3,
    true,
    OAIRequestParams::default(),
    GptContextParams::default(),
  )]
  fn test_cli_create_valid(
    #[case] args: Vec<&str>,
    #[case] alias: String,
    #[case] repo: String,
    #[case] filename: String,
    #[case] snapshot: Option<String>,
    #[case] family: String,
    #[case] chat_template: ChatTemplateId,
    #[case] update: bool,
    #[case] oai_request_params: OAIRequestParams,
    #[case] context_params: GptContextParams,
  ) -> anyhow::Result<()> {
    let actual = Cli::try_parse_from(args)?.command;
    let expected = Command::Create {
      alias,
      repo,
      filename,
      snapshot,
      chat_template: Some(chat_template),
      tokenizer_config: None,
      family: Some(family),
      update,
      oai_request_params,
      context_params,
    };
    assert_eq!(expected, actual);
    Ok(())
  }

  #[rstest]
  #[case(vec![
    "bodhi", "create",
    "testalias:instruct",
    "--repo", "MyFactory/testalias-gguf",
    "--filename", "testalias.Q8_0.gguf",
    "--chat-template", "llama3",
    "--tokenizer-config", "MyFactory/testalias-gguf",
  ], r#"error: the argument '--chat-template <CHAT_TEMPLATE>' cannot be used with '--tokenizer-config <TOKENIZER_CONFIG>'

Usage: bodhi create --repo <REPO> --filename <FILENAME> <--chat-template <CHAT_TEMPLATE>|--tokenizer-config <TOKENIZER_CONFIG>> <ALIAS>

For more information, try '--help'.
"#)]
  #[case(vec![
    "bodhi", "create",
    "testalias:instruct",
    "--repo", "MyFactory/testalias-gguf",
    "--filename", "testalias.Q8_0.gguf",
    "--chat-template", "llama3",
    "--tokenizer-config", "My:Factory/testalias-gguf",
  ], r#"error: invalid value 'My:Factory/testalias-gguf' for '--tokenizer-config <TOKENIZER_CONFIG>': does not match huggingface repo format - `owner/repo`

For more information, try '--help'.
"#)]
  #[case(vec![
    "bodhi", "create",
    "testalias:instruct",
    "--repo", "MyFactory/testalias-gguf",
    "--filename", "testalias.Q8_0.safetensor",
    "--chat-template", "llama3",
    "--tokenizer-config", "MyFactory/testalias-gguf",
  ], r#"error: invalid value 'testalias.Q8_0.safetensor' for '--filename <FILENAME>': only GGUF file extension supported

For more information, try '--help'.
"#)]
  #[case(vec![
    "bodhi", "create",
    "testalias:instruct",
    "--repo", "MyFactory$testalias-gguf",
    "--filename", "testalias.Q8_0.gguf",
    "--chat-template", "llama3",
    "--tokenizer-config", "MyFactory/testalias-gguf",
  ], r#"error: invalid value 'MyFactory$testalias-gguf' for '--repo <REPO>': does not match huggingface repo format - `owner/repo`

For more information, try '--help'.
"#)]
  fn test_cli_create_invalid(
    #[case] args: Vec<&str>,
    #[case] message: String,
  ) -> anyhow::Result<()> {
    let actual = Cli::try_parse_from(args);
    assert!(actual.is_err());
    assert_eq!(message, actual.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[case(Command::App {ui: false}, "app")]
  #[case(Command::Serve {host: Default::default(), port: 0}, "serve")]
  #[case(Command::List {remote: false, models: false}, "list")]
  #[case(Command::Pull { alias: None, repo: None, filename: None, snapshot: None }, "pull")]
  #[case(Command::Create {
      alias: Default::default(),
      repo: Default::default(),
      filename: Default::default(),
      snapshot: None,
      chat_template: None,
      tokenizer_config: None,
      family: None,
      update: false,
      oai_request_params: OAIRequestParams::default(),
      context_params: GptContextParams::default(),
    }, "create")]
  #[case(Command::Run {alias: Default::default()}, "run")]
  fn test_cli_to_string(#[case] cmd: Command, #[case] expected: String) -> anyhow::Result<()> {
    assert_eq!(expected, cmd.to_string());
    Ok(())
  }
}
