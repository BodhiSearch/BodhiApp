use crate::{
  bodhi_err_exts::BodhiErrorExt,
  error::{LlamaCppError, Result},
  objs::CommonParams,
};
use llamacpp_sys::{
  bindings::{bodhi_error, bodhi_server_context},
  BodhiServer, Callback, DynamicBodhiServer,
};
use std::{
  ffi::{c_char, c_void, CString},
  path::Path,
  sync::RwLock,
};
use tracing::warn;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ServerContext: Send + Sync + std::fmt::Debug {
  fn load_library(&self, path: &Path) -> Result<()>;

  fn create_context(&self, common_params: &CommonParams) -> Result<()>;

  fn init(&self) -> Result<()>;

  fn get_common_params(&self) -> CommonParams;

  fn start_event_loop(&self) -> Result<()>;

  fn completions(
    &self,
    input: &str,
    chat_template: &str,
    callback: Callback,
    userdata: *mut c_void,
  ) -> Result<()>;

  fn stop(&mut self) -> Result<()>;

  fn disable_logging(&self) -> Result<()>;
}

#[derive(Debug)]
pub struct BodhiServerContext {
  bodhi_server: Box<dyn BodhiServer>,
  common_params: CommonParams,
  bodhi_ctx_ptr: RwLock<Option<*mut bodhi_server_context>>,
}

impl BodhiServerContext {
  fn new(bodhi_server: Box<dyn BodhiServer>) -> Self {
    Self {
      bodhi_server,
      common_params: CommonParams::default(),
      bodhi_ctx_ptr: RwLock::new(None),
    }
  }

  fn with_ctx<F, R>(&self, f: F) -> Result<R>
  where
    F: FnOnce(*mut bodhi_server_context) -> Result<R>,
  {
    let read_guard = self
      .bodhi_ctx_ptr
      .read()
      .map_err(|_| LlamaCppError::LockError)?;
    match *read_guard {
      Some(ctx_ptr) => f(ctx_ptr),
      None => Err(LlamaCppError::ContextNotInitialized),
    }
  }
}

impl Default for BodhiServerContext {
  fn default() -> Self {
    Self::new(Box::new(DynamicBodhiServer::default()))
  }
}

unsafe impl Send for BodhiServerContext {}
unsafe impl Sync for BodhiServerContext {}

impl ServerContext for BodhiServerContext {
  fn load_library(&self, path: &Path) -> Result<()> {
    self.bodhi_server.load_library(path)?;
    Ok(())
  }

  fn create_context(&self, common_params: &CommonParams) -> Result<()> {
    if self.bodhi_ctx_ptr.read().unwrap().is_some() {
      return Err(LlamaCppError::ContextAlreadyInitialized);
    }
    let err = bodhi_error::default();
    let params_vec = common_params.as_args();
    let argv: Vec<CString> = params_vec
      .iter()
      .map(|s| CString::new(s.as_str()).unwrap())
      .collect();
    let mut c_param_ptrs: Vec<*mut c_char> =
      argv.iter().map(|s| s.as_ptr() as *mut c_char).collect();
    let ctx_params = self.bodhi_server.bodhi_server_common_params_new(
      c_param_ptrs.as_mut_ptr(),
      c_param_ptrs.len(),
      err.as_ptr(),
    )?;
    err.map_err(LlamaCppError::BodhiParamsNew)?;
    let bodhi_ctx_ptr = self
      .bodhi_server
      .bodhi_server_context_new(ctx_params, err.as_ptr())?;
    err.map_err(LlamaCppError::BodhiContextInit)?;
    let mut ctx_ptr = self.bodhi_ctx_ptr.write().unwrap();
    ctx_ptr.replace(bodhi_ctx_ptr);
    Ok(())
  }

  fn get_common_params(&self) -> CommonParams {
    self.common_params.clone()
  }

  fn init(&self) -> Result<()> {
    let err = bodhi_error::default();
    self.with_ctx(|ctx_ptr| {
      self.bodhi_server.bodhi_server_init(ctx_ptr, err.as_ptr())?;
      err.map_err(LlamaCppError::BodhiContextInit)
    })
  }

  fn start_event_loop(&self) -> Result<()> {
    let err = bodhi_error::default();
    self.with_ctx(|ctx_ptr| {
      self
        .bodhi_server
        .bodhi_server_start_event_loop(ctx_ptr, err.as_ptr())?;
      err.map_err(LlamaCppError::BodhiServerStart)
    })
  }

  #[allow(clippy::not_unsafe_ptr_arg_deref)]
  fn completions(
    &self,
    input: &str,
    chat_template: &str,
    callback: Callback,
    userdata: *mut c_void,
  ) -> Result<()> {
    let input_cstr = CString::new(input)?;
    let chat_template_cstr = CString::new(chat_template)?;
    let err = bodhi_error::default();
    self.with_ctx(|ctx_ptr| {
      self.bodhi_server.bodhi_server_chat_completions(
        ctx_ptr,
        input_cstr.as_ptr(),
        chat_template_cstr.as_ptr(),
        err.as_ptr(),
        callback,
        userdata,
      )?;
      err.map_err(LlamaCppError::BodhiServerChatCompletion)
    })
  }

  fn stop(&mut self) -> Result<()> {
    let err = bodhi_error::default();
    let result = if let Ok(mut ctx) = self.bodhi_ctx_ptr.write() {
      if let Some(ctx_ptr) = ctx.take() {
        let free_result = self
          .bodhi_server
          .bodhi_server_context_free(ctx_ptr, err.as_ptr());
        if free_result.is_err() || err.is_err() {
          Err(LlamaCppError::BodhiServerStop(err.to_string()))
        } else {
          Ok(())
        }
      } else {
        Ok(())
      }
    } else {
      Err(LlamaCppError::LockError)
    };

    result
  }

  fn disable_logging(&self) -> Result<()> {
    self.bodhi_server.llama_server_disable_logging()?;
    Ok(())
  }
}

impl Drop for BodhiServerContext {
  fn drop(&mut self) {
    if let Ok(mut ctx) = self.bodhi_ctx_ptr.write() {
      if let Some(ctx_ptr) = ctx.take() {
        let err = bodhi_error::default();
        let _ = self
          .bodhi_server
          .bodhi_server_context_free(ctx_ptr, err.as_ptr());
        if err.is_err() {
          warn!(
            err = err.to_string(),
            "error freeing bodhi_common_params pointer"
          );
        }
        // Set the ctx_ptr to nullptr after freeing
        *ctx = None;
      }
    }
  }
}
