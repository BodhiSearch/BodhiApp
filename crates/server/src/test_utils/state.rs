use crate::RouterStateFn;
use async_openai::types::CreateChatCompletionRequest;
use services::AppServiceFn;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

mockall::mock! {
  pub RouterState {
  }

  #[async_trait::async_trait]
  impl RouterStateFn for RouterState {
    fn app_service(&self) -> Arc<dyn AppServiceFn> ;

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
