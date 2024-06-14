use std::io::{self, Stdout, Write};

#[cfg_attr(test, mockall::automock)]
pub trait StdoutWriter {
  fn write(&mut self, str: &str) -> io::Result<usize>;
}

pub struct DefaultStdoutWriter {
  stdout: Stdout,
}

impl StdoutWriter for DefaultStdoutWriter {
  fn write(&mut self, str: &str) -> io::Result<usize> {
    self.stdout.write(str.as_bytes())
  }
}

impl Default for DefaultStdoutWriter {
  fn default() -> Self {
    Self {
      stdout: std::io::stdout(),
    }
  }
}
