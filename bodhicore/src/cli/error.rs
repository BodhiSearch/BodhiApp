use super::Command;
use crate::objs::ObjError;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::large_enum_variant)]
pub enum CliError {
  #[error("{0}")]
  BadRequest(String),
  #[error("Command '{0}' cannot be converted into command '{1}'")]
  ConvertCommand(Command, String),
  #[error(transparent)]
  ObjError(#[from] ObjError),
}
