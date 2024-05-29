use super::server::{DEFAULT_HOST, DEFAULT_PORT_STR};
use crate::objs::{ChatTemplateId, GGUF_EXTENSION, REGEX_REPO};
use clap::{ArgGroup, Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Parser)]
#[command(version)]
#[command(about = "Run GenerativeAI LLMs locally and serve them via OpenAI compatible API")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Debug, PartialEq, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
  /// launch as native app
  App {},
  /// initialize the configs folder
  Init {},
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
    #[clap(long, short = 'r', requires = "filename", group = "pull", value_parser = repo_parser)]
    repo: Option<String>,

    /// The gguf model file to pull from the repo, e.g. `Meta-Llama-3-8B-Instruct-Q8_0.gguf`,
    #[clap(long, short = 'f', requires = "repo", value_parser = gguf_filename_parser)]
    filename: Option<String>,

    /// If the file already exists in $HF_HOME, force download it again
    #[clap(long = "force")]
    force: bool,
  },

  /// Create a new model alias
  #[clap(group = ArgGroup::new("template").required(true))]
  Create {
    /// Unique name of the model alias. E.g. llama3:8b-instruct
    alias: String,

    /// The hugging face repo to pull the model from, e.g. `bartowski/Meta-Llama-3-8B-Instruct-GGUF`
    #[clap(long, short = 'r', value_parser = repo_parser)]
    repo: String,

    /// The gguf model file to pull from the repo, e.g. `Meta-Llama-3-8B-Instruct-Q8_0.gguf`,
    #[clap(long, short = 'f', value_parser = gguf_filename_parser)]
    filename: String,

    /// In-built chat template to use to convert chat messages to LLM prompt
    #[clap(long, group = "template")]
    chat_template: Option<ChatTemplateId>,

    /// Tokenizer config to convert chat messages to LLM prompt
    #[clap(long, group = "template", value_parser = repo_parser)]
    tokenizer_config: Option<String>,

    /// Optional meta information. Family of the model.
    #[clap(long)]
    family: Option<String>,

    /// Features supported by the model.
    // #[clap(long)]
    // feature: Vec<ModelFeature>,

    /// If the file already exists in $HF_HOME, force download it again
    #[clap(long)]
    force: bool,

    #[clap(flatten, next_help_heading = "OpenAI Compatible Request defaults")]
    oai_request_params: OAIRequestParams,

    #[clap(flatten, next_help_heading = "Model Context defaults")]
    context_params: GptContextParams,
  },
  /// Run the given model alias in interactive mode.
  Run {
    /// Model alias to run. Run `bodhi list` to list the configured model aliases.
    alias: String,
  },
}

#[derive(clap::ValueEnum, Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ModelFeature {
  Chat,
}

fn validate_range_neg_to_pos_2(s: &str) -> Result<f32, String> {
  let lower = -2.0;
  let upper = 2.0;
  validate_range(s, lower, upper)
}

fn validate_range_0_to_1(s: &str) -> Result<f32, String> {
  let lower = 0.0;
  let upper = 1.0;
  validate_range(s, lower, upper)
}

fn validate_range(s: &str, lower: f32, upper: f32) -> Result<f32, String> {
  match s.parse::<f32>() {
    Ok(val) if (lower..=upper).contains(&val) => Ok(val),
    Ok(_) => Err(String::from(
      "The value must be between -2 and 2 inclusive.",
    )),
    Err(_) => Err(String::from(
      "The value must be a valid floating point number.",
    )),
  }
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default, Args)]
pub struct OAIRequestParams {
  #[clap(long, value_parser = validate_range_neg_to_pos_2, help=r#"Number between -2.0 and 2.0. 
Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
default: 0.0 (disabled)"#)]
  pub frequency_penalty: Option<f32>,

  #[arg(
    long,
    help = r#"The maximum number of tokens that can be generated in the completion.
The token count of your prompt plus `max_tokens` cannot exceed the model's context length.
default: -1"#
  )]
  pub max_tokens: Option<u32>,

  #[arg(long, value_parser = validate_range_neg_to_pos_2, help=r#"Number between -2.0 and 2.0.
Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
default: 0.0 (disabled)"#)]
  pub presence_penalty: Option<f32>,

  #[arg(
    long,
    help = r#"An object specifying the format that the model must output.
Setting to `json_object` enables JSON mode, which guarantees the message the model generates is valid JSON.
default: text"#
  )]
  pub response_format: Option<ResponseFormat>,

  #[arg(long, value_parser = clap::value_parser!(i64).range(i64::MIN..=i64::MAX),
  help=r#"If specified, our system will make a best effort to sample deterministically, such that repeated requests with the same `seed` and parameters should return the same result."#)]
  pub seed: Option<i64>,

  #[arg(
    long,
    number_of_values = 4,
    help = r#"Up to 4 sequences where the API will stop generating further tokens."#
  )]
  pub stop: Option<Vec<String>>,

  #[arg(long, value_parser = validate_range_neg_to_pos_2, help=r#"Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
default: 0.0 (disabled)"#)]
  pub temperature: Option<f32>,

  #[arg(long, value_parser = validate_range_0_to_1, help=r#"An alternative to sampling with temperature, called nucleus sampling.
The model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered.
Alter this or `temperature` but not both.
default: 1.0 (disabled)"#)]
  pub top_p: Option<f32>,

  #[arg(
    long,
    help = r#"A unique identifier representing your end-user, which can help to monitor and detect abuse."#
  )]
  pub user: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
pub enum ResponseFormat {
  Text,
  #[clap(name = "json_object")]
  JsonObject,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default, Args)]
pub struct GptContextParams {
  /// default: num_cpu()
  #[arg(
    long,
    help = r#"number of threads to use during computation
default: num_cpus()"#
  )]
  pub threads: Option<u32>,

  /// default: 1
  #[arg(long)]
  pub ctx_size: Option<u32>,

  /// default: 1
  #[arg(long)]
  pub parallel: Option<u32>,

  /// default: -1
  #[arg(long)]
  pub n_predict: Option<u32>,
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
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
  #[case(vec!["bodhi", "run", "llama3:instruct"], "llama3:instruct")]
  fn test_cli_run(#[case] args: Vec<&str>, #[case] alias: String) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args)?;
    let expected = Command::Run { alias };
    assert_eq!(expected, cli.command);
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
    "testalias",
    ChatTemplateId::Llama3,
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
    "--temperature", "0.8",
    "--threads", "6",
  ],
    "testalias:instruct",
    "MyFactory/testalias-gguf",
    "testalias.Q8_0.gguf",
    "testalias",
    ChatTemplateId::Llama3,
    OAIRequestParams {temperature: Some(0.8), ..OAIRequestParams::default()},
    GptContextParams {threads: Some(6), ..GptContextParams::default()},
  )]
  fn test_cli_create_valid(
    #[case] args: Vec<&str>,
    #[case] alias: String,
    #[case] repo: String,
    #[case] filename: String,
    #[case] family: String,
    #[case] chat_template: ChatTemplateId,
    #[case] oai_request_params: OAIRequestParams,
    #[case] context_params: GptContextParams,
  ) -> anyhow::Result<()> {
    let actual = Cli::try_parse_from(args)?.command;
    let expected = Command::Create {
      alias,
      repo,
      filename,
      chat_template: Some(chat_template),
      tokenizer_config: None,
      family: Some(family),
      force: false,
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
}
