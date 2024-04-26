use anyhow::bail;
use llama_server_bindings::Callback;
use llama_server_bindings::{BodhiServerContext, GptParams};
use std::clone::Clone;
use std::ffi::c_void;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

pub struct SharedResource<T>(Arc<Mutex<Option<T>>>);

impl<T> Deref for SharedResource<T> {
  type Target = Mutex<Option<T>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> Clone for SharedResource<T> {
  fn clone(&self) -> Self {
    SharedResource(self.0.clone())
  }
}

pub type SharedContext = SharedResource<BodhiServerContext>;

impl SharedContext {
  pub fn new_shared(gpt_params: Option<GptParams>) -> anyhow::Result<Self> {
    let mut ctx = Self(Arc::new(Mutex::new(None)));
    ctx.reload(gpt_params)?;
    Ok(ctx)
  }

  pub fn completions(
    &self,
    input: &str,
    callback: Option<Callback>,
    userdata: *mut c_void,
  ) -> anyhow::Result<()> {
    let guard = self.lock().unwrap();
    let Some(ctx) = guard.as_ref() else {
      bail!("bodhiserver context is not set");
    };
    ctx.completions(input, callback, userdata)?;
    Ok(())
  }

  pub fn reload(&mut self, gpt_params: Option<GptParams>) -> anyhow::Result<()> {
    self.try_stop()?;
    if let Some(gpt_params) = gpt_params {
      let ctx = BodhiServerContext::new(gpt_params)?;
      let mut guard = self.lock().unwrap();
      *guard = Some(ctx);

      let ctx = guard.as_mut().unwrap();
      ctx.init()?;
      ctx.start_event_loop()?;
    }
    Ok(())
  }

  pub fn try_stop(&mut self) -> anyhow::Result<()> {
    let opt = self.lock().unwrap().take();
    let Some(mut ctx) = opt else { return Ok(()) };
    ctx.stop()?;
    drop(ctx);
    Ok(())
  }

  pub fn stop(&mut self) -> anyhow::Result<()> {
    let opt = self.lock().unwrap().take();
    let Some(mut ctx) = opt else {
      bail!("context is not initialized");
    };
    ctx.stop()?;
    drop(ctx);
    Ok(())
  }
}
