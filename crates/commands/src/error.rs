#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CmdIntoError {
  #[error("Command '{input}' cannot be converted into command '{output}'")]
  Convert { input: String, output: String },
  #[error("Command '{input}' cannot be converted into command '{output}', error: '{error}'")]
  BadRequest {
    input: String,
    output: String,
    error: String,
  },
}
