use async_openai::types::CreateChatCompletionRequest;
use tokio::sync::mpsc::Sender;
use crate::server::RouterStateFn;

mockall::mock! {
  pub RouterState {
  }

  #[async_trait::async_trait]
  impl RouterStateFn for RouterState {
    async fn chat_completions(
      &self,
      request: CreateChatCompletionRequest,
      userdata: Sender<String>,
    ) -> crate::oai::Result<()>;
  }

  impl Clone for RouterState {
    fn clone(&self) -> Self;
  }
}
