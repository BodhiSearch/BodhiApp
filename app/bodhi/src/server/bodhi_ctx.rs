use anyhow::bail;
use llama_server_bindings::{BodhiServerContext, GptParams};

pub(crate) struct BodhiContextWrapper {
  pub(crate) ctx: Option<BodhiServerContext>,
}

impl BodhiContextWrapper {
  pub(crate) fn new(gpt_params: &GptParams) -> anyhow::Result<Self> {
    let mut wrapper = Self { ctx: None };
    wrapper.reload(gpt_params)?;
    return Ok(wrapper);
  }

  pub(crate) fn reload(&mut self, gpt_params: &GptParams) -> anyhow::Result<()> {
    if self.ctx.is_some() {
      self.stop()?;
    }
    let ctx = BodhiServerContext::new(&gpt_params)?;
    ctx.init()?;
    ctx.start_event_loop()?;
    self.ctx = Some(ctx);
    Ok(())
  }

  pub(crate) fn stop(&mut self) -> anyhow::Result<()> {
    let Some(ctx) = self.ctx.as_mut() else {
      bail!("context is not initialized");
    };
    ctx.stop()?;
    drop(self.ctx.take());
    Ok(())
  }
}
