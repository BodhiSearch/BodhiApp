use objs::ObjError;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
  #[error("{0}")]
  BadRequest(String),
  #[error("Command '{0}' cannot be converted into command '{1}'")]
  ConvertCommand(String, String),
  #[error(transparent)]
  ObjError(#[from] ObjError),
}
