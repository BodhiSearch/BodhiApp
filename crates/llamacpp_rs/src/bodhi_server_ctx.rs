use crate::objs::CommonParams;
use std::ffi::{c_char, c_void};

pub type Callback =
  unsafe extern "C" fn(contents: *const c_char, size: usize, userdata: *mut c_void) -> usize;

pub trait ServerContext: Send + Sync + std::fmt::Debug {
  fn init(&self) -> crate::error::Result<()>;

  fn get_common_params(&self) -> CommonParams;

  fn start_event_loop(&self) -> crate::error::Result<()>;

  fn completions(
    &self,
    input: &str,
    chat_template: &str,
    callback: Option<Callback>,
    userdata: *mut c_void,
  ) -> crate::error::Result<()>;

  fn stop(&mut self) -> crate::error::Result<()>;
}
