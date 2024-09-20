/// .
///
/// # Safety
///
/// .
pub unsafe fn llama_server_disable_logging() {
  llama_server_bindings::bindings::llama_server_disable_logging()
}

pub fn disable_llama_log() {
  llama_server_bindings::disable_llama_log()
}
