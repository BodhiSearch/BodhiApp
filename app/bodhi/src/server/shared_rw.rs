use std::sync::Arc;

use llama_server_bindings::{BodhiServerContext, GptParams};
use tokio::sync::RwLock;

pub type SharedContextRw = Arc<RwLock<Option<BodhiServerContext>>>;

pub trait SharedContextRwExts {
  async fn new_shared_rw(gpt_params: Option<GptParams>) -> anyhow::Result<Self>
  where
    Self: Sized;

  async fn has_model(&self) -> anyhow::Result<bool>
  where
    Self: Sized;
}

impl SharedContextRwExts for SharedContextRw {
  async fn new_shared_rw(gpt_params: Option<GptParams>) -> anyhow::Result<Self>
  where
    Self: Sized,
  {
    let ctx = Arc::new(RwLock::new(None));
    if gpt_params.is_none() {
      return Ok(ctx);
    }
    todo!()
  }

  async fn has_model(&self) -> anyhow::Result<bool>
  where
    Self: Sized,
  {
    let lock = RwLock::read(self).await;
    Ok(lock.as_ref().is_some())
  }
}

#[cfg(test)]
mod test {
  use super::SharedContextRwExts;
  use crate::server::shared_rw::SharedContextRw;

  #[tokio::test]
  async fn test_new_shared_rw() -> anyhow::Result<()> {
    let ctx = SharedContextRw::new_shared_rw(None).await?;
    assert!(!ctx.has_model().await?);
    Ok(())
  }
}
