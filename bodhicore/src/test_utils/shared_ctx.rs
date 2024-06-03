use crate::{objs::*, SharedContextRwFn};
use async_openai::types::CreateChatCompletionRequest;
use llama_server_bindings::{Callback, GptParams};
use std::ffi::c_void;
use tokio::sync::mpsc::Sender;

mockall::mock! {
  pub SharedContext {}

  impl Clone for SharedContext {
    fn clone(&self) -> Self;
  }

  impl std::fmt::Debug for SharedContext {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
  }

  unsafe impl Sync for SharedContext {}

  unsafe impl Send for SharedContext {}

  #[async_trait::async_trait]
  impl SharedContextRwFn for SharedContext {
    async fn reload(&self, gpt_params: Option<GptParams>) -> crate::shared_rw::Result<()>;

    async fn try_stop(&self) -> crate::shared_rw::Result<()>;

    async fn has_model(&self) -> bool;

    async fn get_gpt_params(&self) -> crate::shared_rw::Result<Option<GptParams>>;

    async fn chat_completions(
      &self,
      request: CreateChatCompletionRequest,
      model_file: HubFile,
      tokenizer_file: HubFile,
      userdata: Sender<String>,
    ) -> crate::shared_rw::Result<()>;
  }
}

mockall::mock! {
  pub BodhiServerContext {
    pub fn new(gpt_params: GptParams) -> anyhow::Result<Self>;

    pub fn init(&self) -> anyhow::Result<()>;

    pub fn get_gpt_params(&self) -> GptParams;

    pub fn start_event_loop(&self) -> anyhow::Result<()>;

    pub fn completions(
      &self,
      input: &str,
      chat_template: &str,
      callback: Option<Callback>,
      userdata: *mut c_void,
    ) -> anyhow::Result<()>;

    pub fn stop(&mut self) -> anyhow::Result<()>;
  }

  impl std::fmt::Debug for BodhiServerContext {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
  }
}
