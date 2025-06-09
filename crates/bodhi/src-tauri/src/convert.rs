use server_app::ServeCommand;

use crate::error::BodhiError;

pub fn build_serve_command(host: String, port: u16) -> Result<ServeCommand, BodhiError> {
  Ok(ServeCommand::ByParams { host, port })
}

#[cfg(test)]
mod tests {
  use crate::convert::build_serve_command;
  use rstest::rstest;
  use server_app::ServeCommand;

  #[rstest]
  #[case("localhost", 1135, ServeCommand::ByParams {
    host: "localhost".to_string(),
    port: 1135,
  })]
  fn test_build_serve_command(
    #[case] host: String,
    #[case] port: u16,
    #[case] expected: ServeCommand,
  ) -> anyhow::Result<()> {
    let result = build_serve_command(host, port)?;
    assert_eq!(expected, result);
    Ok(())
  }
}
