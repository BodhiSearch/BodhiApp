#[cfg(feature = "native")]
use crate::native;

use crate::error::AppExecuteError;
use clap::{Parser, Subcommand};
use objs::ApiError;
use server_app::ServeCommand;
use services::AppService;
use std::sync::Arc;

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
  /// Launch the native application with system tray
  App {
    /// Show the UI window on startup
    #[arg(long)]
    ui: bool,
  },
  /// Start the server in deployment mode
  Serve {
    /// Host address to bind to
    #[arg(short = 'H', long, default_value = services::DEFAULT_HOST)]
    host: String,
    /// Port number to bind to
    #[arg(short, long, default_value_t = services::DEFAULT_PORT)]
    port: u16,
  },
}

pub async fn start(app_service: Arc<dyn AppService>) -> Result<(), ApiError> {
  // Parse command line arguments using clap
  let cli = Cli::parse();
  match cli.command {
    #[allow(unused_variables)]
    Some(Commands::App { ui }) => {
      // Launch native app with optional UI flag
      if app_service.setting_service().is_native() {
        if cfg!(feature = "native") {
          #[cfg(feature = "native")]
          native::NativeCommand::new(app_service, ui)
            .aexecute(Some(crate::ui::router()))
            .await?;
        } else {
          Err(AppExecuteError::Unreachable(
            r#"setting_service.is_native() returned true, but cfg!(feature = "native") is false"#
              .to_string(),
          ))?;
        }
      } else {
        Err(AppExecuteError::NativeNotSupported)?;
      }
    }
    Some(Commands::Serve { host, port }) => {
      // Server deployment mode
      let serve_command = ServeCommand::ByParams { host, port };
      serve_command
        .aexecute(app_service, Some(crate::ui::router()))
        .await?;
    }
    None => {
      // No subcommand - launch native app if supported (default behavior)
      if app_service.setting_service().is_native() {
        if cfg!(feature = "native") {
          #[cfg(feature = "native")]
          native::NativeCommand::new(app_service, true)
            .aexecute(Some(crate::ui::router()))
            .await?;
        } else {
          Err(AppExecuteError::Unreachable(
            r#"setting_service.is_native() returned true, but cfg!(feature = "native") is false"#
              .to_string(),
          ))?;
        }
      } else {
        Err(AppExecuteError::NativeNotSupported)?;
      }
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::{CommandFactory, Parser};
  use rstest::rstest;

  #[test]
  fn test_cli_debug_assert() -> anyhow::Result<()> {
    Cli::command().debug_assert();
    Ok(())
  }

  #[test]
  fn test_cli_no_subcommand() -> anyhow::Result<()> {
    let args = vec!["bodhi"];
    let cli = Cli::try_parse_from(args)?;
    assert!(cli.command.is_none());
    Ok(())
  }

  #[test]
  fn test_cli_app_basic() -> anyhow::Result<()> {
    let args = vec!["bodhi", "app"];
    let cli = Cli::try_parse_from(args)?;
    match cli.command {
      Some(Commands::App { ui }) => {
        assert!(!ui);
      }
      _ => panic!("Expected App command"),
    }
    Ok(())
  }

  #[test]
  fn test_cli_app_with_ui() -> anyhow::Result<()> {
    let args = vec!["bodhi", "app", "--ui"];
    let cli = Cli::try_parse_from(args)?;
    match cli.command {
      Some(Commands::App { ui }) => {
        assert!(ui);
      }
      _ => panic!("Expected App command with ui=true"),
    }
    Ok(())
  }

  #[test]
  fn test_cli_app_invalid_args() -> anyhow::Result<()> {
    let args = vec!["bodhi", "app", "--extra", "args"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    let error_msg = cli.unwrap_err().to_string();
    assert!(error_msg.contains("unexpected argument '--extra'"));
    Ok(())
  }

  #[rstest]
  #[case(vec!["bodhi", "serve", "-H", "0.0.0.0", "-p", "8080"], "0.0.0.0", 8080)]
  #[case(vec!["bodhi", "serve", "--host", "127.0.0.1", "--port", "3000"], "127.0.0.1", 3000)]
  #[case(vec!["bodhi", "serve", "-p", "8080"], services::DEFAULT_HOST, 8080)]
  #[case(vec!["bodhi", "serve", "-H", "0.0.0.0"], "0.0.0.0", services::DEFAULT_PORT)]
  #[case(vec!["bodhi", "serve"], services::DEFAULT_HOST, services::DEFAULT_PORT)]
  fn test_cli_serve_valid(
    #[case] args: Vec<&str>,
    #[case] expected_host: &str,
    #[case] expected_port: u16,
  ) -> anyhow::Result<()> {
    let cli = Cli::try_parse_from(args)?;
    match cli.command {
      Some(Commands::Serve { host, port }) => {
        assert_eq!(expected_host, host);
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
        assert_eq!(0, port);
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

  #[test]
  fn test_cli_unknown_command() -> anyhow::Result<()> {
    let args = vec!["bodhi", "unknown"];
    let cli = Cli::try_parse_from(args);
    assert!(cli.is_err());
    let error_msg = cli.unwrap_err().to_string();
    assert!(error_msg.contains("unrecognized subcommand"));
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
