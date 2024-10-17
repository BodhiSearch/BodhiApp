use llama_server_bindings::{Callback, CommonParams};
use std::ffi::c_void;

mockall::mock! {
  pub ServerContext {
    pub fn new(gpt_params: CommonParams) -> llama_server_bindings::Result<Self>;

    pub fn init(&self) -> llama_server_bindings::Result<()>;

    pub fn get_common_params(&self) -> CommonParams;

    pub fn start_event_loop(&self) -> llama_server_bindings::Result<()>;

    pub fn completions(
      &self,
      input: &str,
      chat_template: &str,
      callback: Option<Callback>,
      userdata: *mut c_void,
    ) -> llama_server_bindings::Result<()>;

    pub fn stop(&mut self) -> llama_server_bindings::Result<()>;
  }

  impl std::fmt::Debug for ServerContext {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
  }
}
