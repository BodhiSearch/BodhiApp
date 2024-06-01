use async_openai::types::CreateChatCompletionRequest;
use llama_server_bindings::{BodhiServerContext, Callback, GptParams};
use mockall::automock;
use std::future::Future;
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::sync::watch::error;
use tokio::sync::RwLock;

use crate::objs::{Alias, LocalModelFile};

#[derive(Debug)]
pub struct SharedContextRw {
  // TODO: remove pub access
  pub ctx: RwLock<Option<BodhiServerContext>>,
}

#[derive(Debug, Error)]
pub enum ContextError {
  #[error("{0}")]
  LlamaCpp(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ContextError>;

#[async_trait::async_trait]
pub trait SharedContextRwFn: std::fmt::Debug + Send + Sync {
  async fn reload(&self, gpt_params: Option<GptParams>) -> Result<()>;

  async fn try_stop(&self) -> Result<()>;

  async fn has_model(&self) -> bool;

  async fn get_gpt_params(&self) -> Result<Option<GptParams>>;

  #[allow(clippy::ptr_arg)]
  async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
    alias: Alias,
    model_file: LocalModelFile,
    tokenizer_file: LocalModelFile,
    callback: Option<Callback>,
    userdata: &String,
  ) -> Result<()>;
}

impl SharedContextRw {
  pub async fn new_shared_rw(gpt_params: Option<GptParams>) -> Result<Self>
  where
    Self: Sized,
  {
    let ctx = SharedContextRw {
      ctx: RwLock::new(None),
    };
    ctx.reload(gpt_params).await?;
    Ok(ctx)
  }
}

#[async_trait::async_trait]
impl SharedContextRwFn for SharedContextRw {
  async fn has_model(&self) -> bool {
    let lock = self.ctx.read().await;
    lock.as_ref().is_some()
  }

  async fn reload(&self, gpt_params: Option<GptParams>) -> crate::shared_rw::Result<()> {
    let mut lock = self.ctx.write().await;
    try_stop_with(&mut lock)?;
    let Some(gpt_params) = gpt_params else {
      return Ok(());
    };
    let ctx = BodhiServerContext::new(gpt_params)?;
    *lock = Some(ctx);
    let Some(ctx) = lock.as_ref() else {
      unreachable!("just injected ctx in rwlock");
    };
    ctx.init()?;
    ctx.start_event_loop()?;
    // TODO - if stopping server immediately after starting, gets stuck in
    // `waiting for event_thread to complete`
    // sleep for .5 sec to avoid this scenario
    tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
    Ok(())
  }

  async fn try_stop(&self) -> crate::shared_rw::Result<()> {
    let mut lock = self.ctx.write().await;
    try_stop_with(&mut lock)?;
    Ok(())
  }

  async fn get_gpt_params(&self) -> crate::shared_rw::Result<Option<GptParams>> {
    let lock = self.ctx.read().await;
    if let Some(opt) = lock.as_ref() {
      Ok(Some(opt.gpt_params.clone()))
    } else {
      Ok(None)
    }
  }

  async fn chat_completions(
    &self,
    request: CreateChatCompletionRequest,
    alias: Alias,
    model_file: LocalModelFile,
    tokenizer_file: LocalModelFile,
    callback: Option<Callback>,
    userdata: &String,
  ) -> crate::shared_rw::Result<()> {
    todo!()
  }
}

fn try_stop_with(
  lock: &mut tokio::sync::RwLockWriteGuard<'_, Option<BodhiServerContext>>,
) -> Result<()> {
  let opt = lock.take();
  if let Some(mut ctx) = opt {
    ctx
      .stop()
      .map_err(|err: anyhow::Error| ContextError::LlamaCpp(err))?;
    drop(ctx);
  };
  Ok(())
}

#[cfg(test)]
mod test {
  use crate::shared_rw::{SharedContextRw, SharedContextRwFn};
  use anyhow::anyhow;
  use async_openai::types::CreateChatCompletionResponse;
  use llama_server_bindings::{
    bindings::llama_server_disable_logging, disable_llama_log, GptParams,
  };
  use rstest::{fixture, rstest};
  use serde_json::json;
  use std::{
    ffi::{c_char, c_void},
    slice,
  };

  #[fixture]
  fn model_file() -> String {
    let user_home = dirs::home_dir()
      .ok_or_else(|| anyhow!("failed to get users home dir"))
      .unwrap();
    let model_file = user_home.join(".cache/huggingface/hub/models--TheBloke--Llama-2-7B-Chat-GGUF/snapshots/08a5566d61d7cb6b420c3e4387a39e0078e1f2fe5f055f3a03887385304d4bfa/llama-2-7b-chat.Q4_K_M.gguf");
    assert!(model_file.exists());
    model_file.to_string_lossy().into_owned()
  }

  #[ignore]
  #[tokio::test]
  async fn test_shared_rw_new() -> anyhow::Result<()> {
    let ctx = SharedContextRw::new_shared_rw(None).await?;
    assert!(!ctx.has_model().await);
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_new_with_params(model_file: String) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let gpt_params = GptParams {
      model: model_file,
      ..GptParams::default()
    };
    let ctx = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    assert!(ctx.has_model().await);
    ctx.try_stop().await?;
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_reload(model_file: String) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let gpt_params = GptParams {
      model: model_file.clone(),
      ..GptParams::default()
    };
    let ctx = SharedContextRw::new_shared_rw(Some(gpt_params.clone())).await?;
    let model_params = ctx.get_gpt_params().await?.unwrap();
    assert_eq!(model_file, model_params.model);
    ctx.reload(None).await?;
    assert!(ctx.get_gpt_params().await?.is_none());
    ctx.reload(Some(gpt_params)).await?;
    let model_params = ctx.get_gpt_params().await?.unwrap();
    assert_eq!(model_file, model_params.model);
    Ok(())
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_try_stop(model_file: String) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let gpt_params = GptParams {
      model: model_file,
      ..GptParams::default()
    };
    let mut ctx = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    ctx.try_stop().await?;
    assert!(!ctx.has_model().await);
    Ok(())
  }

  pub unsafe extern "C" fn test_callback(
    contents: *const c_char,
    size: usize,
    userdata: *mut c_void,
  ) -> usize {
    let slice = unsafe { slice::from_raw_parts(contents as *const u8, size) };
    let input_str = match std::str::from_utf8(slice) {
      Ok(s) => s,
      Err(_) => return 0,
    };

    let user_data_str = unsafe { &mut *(userdata as *mut String) };
    user_data_str.push_str(input_str);
    size
  }

  #[fixture]
  pub fn chat_request() -> String {
    let request = json!({
      "seed": 42,
      "messages": [
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "What day comes after Monday?"},
      ],
    });
    serde_json::to_string(&request).expect("should serialize chat completion request to string")
  }

  #[ignore]
  #[rstest]
  #[tokio::test]
  async fn test_shared_rw_completions(
    model_file: String,
    chat_request: String,
  ) -> anyhow::Result<()> {
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    let gpt_params = GptParams {
      seed: Some(42),
      model: model_file,
      ..GptParams::default()
    };
    let ctx = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
    let userdata = String::with_capacity(1024);
    let lock = ctx.ctx.read().await;
    let inner = lock.as_ref().expect("should have context loaded");
    inner.completions(
      &chat_request,
      "",
      Some(test_callback),
      &userdata as *const _ as *mut _,
    )?;
    let response: CreateChatCompletionResponse =
      serde_json::from_str(&userdata).expect("parse as chat completion response json");
    assert_eq!(
      "The day that comes after Monday is Tuesday.",
      response.choices[0]
        .message
        .content
        .as_ref()
        .expect("content does not exists")
    );
    Ok(())
  }

  #[rstest]
  fn test_shared_rw_loaded_model_same_as_alias() -> anyhow::Result<()> {
    todo!()
  }
}
