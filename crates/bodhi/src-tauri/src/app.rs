// Function signature for delegation to feature-specific modules
// Implementation will be provided by conditional compilation
#[cfg(feature = "native")]
pub use crate::native_init::initialize_and_execute;
#[cfg(not(feature = "native"))]
pub use crate::server_init::initialize_and_execute;

use clap::{Parser, Subcommand};

pub use lib_bodhiserver::AppCommand;

#[derive(Parser, Debug)]
#[command(name = "bodhi")]
#[command(about = "Bodhi App - Your personal, private, open-source AI Stack")]
#[command(version)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
  #[cfg(not(feature = "native"))]
  /// Start the server in deployment mode
  Serve {
    /// Host address to bind to
    #[arg(short = 'H', long)]
    host: Option<String>,
    /// Port number to bind to
    #[arg(short, long)]
    port: Option<u16>,
  },
}

// CLI-first entry point
pub fn main(args: &[String]) {
  let cli = Cli::parse_from(args);
  let command = match cli.command {
    #[cfg(not(feature = "native"))]
    Some(Commands::Serve { host, port }) => AppCommand::Serve { host, port },
    None => AppCommand::Default,
    #[allow(unreachable_patterns)]
    Some(_) => AppCommand::Default,
  };

  if let Err(err) = initialize_and_execute(command) {
    tracing::error!("fatal error: {err}\nexiting application");
    std::process::exit(1);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::{CommandFactory, Parser};

  #[test]
  fn test_cli_debug_assert() -> anyhow::Result<()> {
    Cli::command().debug_assert();
    Ok(())
  }

  #[test]
  fn test_cli_help() -> anyhow::Result<()> {
    let args = vec!["bodhi", "--help"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    let error_msg = cli.unwrap_err().to_string();
    assert!(error_msg.contains("Bodhi App") || error_msg.contains("AI Model Management"));
    Ok(())
  }

  #[test]
  fn test_cli_version() -> anyhow::Result<()> {
    let args = vec!["bodhi", "--version"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err()); // clap exits with error code for --version
    Ok(())
  }
}

#[cfg(not(feature = "native"))]
#[cfg(test)]
mod server_test {
  use crate::app::{Cli, Commands};
  use clap::Parser;
  use rstest::rstest;

  #[rstest]
  fn test_cli_non_native() -> anyhow::Result<()> {
    let args = vec!["bodhi"];
    let cli = Cli::try_parse_from(args)?;
    assert!(cli.command.is_none());
    Ok(())
  }

  #[test]
  fn test_cli_unknown_command() -> anyhow::Result<()> {
    let args = vec!["bodhi", "unknown"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    let error_msg = cli.unwrap_err().to_string();
    assert!(error_msg.contains("unrecognized subcommand"));
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "serve", "-H", "0.0.0.0"], Some("0.0.0.0"), None)]
  #[case(vec!["bodhi", "serve", "-p", "8080"], None, Some(8080))]
  #[case(vec!["bodhi", "serve", "-H", "0.0.0.0", "-p", "8080"], Some("0.0.0.0"), Some(8080))]
  #[case(vec!["bodhi", "serve", "--host", "127.0.0.1", "--port", "3000"], Some("127.0.0.1"), Some(3000))]
  #[case(vec!["bodhi", "serve"], None, None)]
  fn test_cli_serve_valid(
    #[case] args: Vec<&str>,
    #[case] expected_host: Option<&str>,
    #[case] expected_port: Option<u16>,
  ) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args)?;
    match cli.command {
      Some(Commands::Serve { host, port }) => {
        assert_eq!(expected_host, host.as_deref());
        assert_eq!(expected_port, port);
      }
      _ => panic!("Expected Serve command"),
    }
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "serve", "-p", "65536"], "65536 is not in 0..=65535")]
  #[case(vec!["bodhi", "serve", "--port", "abc"], "invalid value 'abc'")]
  fn test_cli_serve_invalid_port(
    #[case] args: Vec<&str>,
    #[case] expected_error_fragment: &str,
  ) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    let error_msg = cli.unwrap_err().to_string();
    assert!(
      error_msg.contains(expected_error_fragment),
      "Error message '{}' should contain '{}'",
      error_msg,
      expected_error_fragment
    );
    Ok(())
  }

  #[test]
  fn test_cli_serve_port_zero_allowed() -> anyhow::Result<()> {
    // Port 0 is actually allowed by clap's u16 parser
    let args = vec!["bodhi", "serve", "-p", "0"];
    let cli = Cli::try_parse_from(args)?;
    match cli.command {
      Some(Commands::Serve { host: _, port }) => {
        assert_eq!(Some(0), port);
      }
      _ => panic!("Expected Serve command"),
    }
    Ok(())
  }

  #[test]
  fn test_cli_serve_missing_host_value() -> anyhow::Result<()> {
    let args = vec!["bodhi", "serve", "--host"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    let error_msg = cli.unwrap_err().to_string();
    assert!(error_msg.contains("a value is required"));
    Ok(())
  }

  #[test]
  fn test_cli_serve_missing_port_value() -> anyhow::Result<()> {
    let args = vec!["bodhi", "serve", "--port"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    let error_msg = cli.unwrap_err().to_string();
    assert!(error_msg.contains("a value is required"));
    Ok(())
  }
}

#[cfg(feature = "native")]
#[cfg(test)]
mod native_test {
  use crate::app::Cli;
  use clap::Parser;
  use rstest::rstest;

  #[rstest]
  fn test_cli_native() -> anyhow::Result<()> {
    let args = vec!["bodhi"];
    let cli = Cli::try_parse_from(args)?;
    assert!(matches!(cli.command, None));
    Ok(())
  }

  #[test]
  fn test_cli_unknown_command() -> anyhow::Result<()> {
    let args = vec!["bodhi", "unknown"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    let error_msg = cli.unwrap_err().to_string();
    assert!(
      error_msg.contains("unexpected argument") || error_msg.contains("unrecognized subcommand")
    );
    Ok(())
  }

  #[rstest]
  fn test_cli_native_invalid_subcommand() -> anyhow::Result<()> {
    let args = vec!["bodhi", "serve"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    let error_msg = cli.unwrap_err().to_string();
    assert_eq!("error: unexpected argument 'serve' found\n\nUsage: bodhi\n\nFor more information, try '--help'.\n", error_msg);
    Ok(())
  }
}
