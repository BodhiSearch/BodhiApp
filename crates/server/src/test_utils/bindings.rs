use llama_server_bindings::{bindings::llama_server_disable_logging, disable_llama_log};

#[allow(unused)]
pub fn disable_test_logging() {
  disable_llama_log();
  unsafe {
    llama_server_disable_logging();
  }
}
