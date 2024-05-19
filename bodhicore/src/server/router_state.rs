use super::SharedContextRw;
use crate::{hf::find_model, server::SharedContextRwExts};
use anyhow::{anyhow, bail};
use llama_server_bindings::{Callback, GptParams};

#[derive(Debug, Clone)]
pub(crate) struct RouterState {
  pub(crate) ctx: SharedContextRw,
}

impl RouterState {
  pub(crate) fn new(ctx: SharedContextRw) -> Self {
    Self { ctx }
  }
}

impl RouterState {
  pub async fn completions(
    &self,
    model: &str,
    input: &str,
    chat_template: &str,
    callback: Option<Callback>,
    userdata: &String,
  ) -> anyhow::Result<()> {
    let Some(local_model) = find_model(model) else {
      bail!("model not found: '{}'", model)
    };
    let requested_model = local_model.model_path();
    let lock = self.ctx.read().await;
    let ctx = lock.as_ref();
    match ctx {
      Some(ctx) => {
        let gpt_params = ctx.gpt_params.clone();
        let loaded_model = gpt_params.model.clone();
        if loaded_model.eq(&requested_model) {
          ctx.completions(
            input,
            chat_template,
            callback,
            userdata as *const _ as *mut _,
          )
        } else {
          tracing::info!(
            loaded_model,
            requested_model,
            "requested model not loaded, loading model"
          );
          drop(lock);
          let new_gpt_params = GptParams {
            model: requested_model,
            ..gpt_params
          };
          self.ctx.reload(Some(new_gpt_params)).await?;
          let lock = self.ctx.read().await;
          let ctx = lock.as_ref().ok_or(anyhow!("context not present"))?;
          ctx.completions(
            input,
            chat_template,
            callback,
            userdata as *const _ as *mut _,
          )
        }
      }
      None => {
        let gpt_params = GptParams {
          model: requested_model,
          ..Default::default()
        };
        drop(lock);
        self.ctx.reload(Some(gpt_params)).await?;
        let lock = self.ctx.read().await;
        let ctx = lock.as_ref().ok_or(anyhow!("context not present"))?;
        ctx.completions(
          input,
          chat_template,
          callback,
          userdata as *const _ as *mut _,
        )
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::RouterState;
  use crate::{
    bindings::{disable_llama_log, llama_server_disable_logging},
    hf::HF_HOME,
    server::{SharedContextRw, SharedContextRwExts},
    test_utils::{init_test_tracing, test_callback},
  };
  use anyhow::anyhow;
  use anyhow_trace::anyhow_trace;
  use async_openai::types::CreateChatCompletionResponse;
  use llama_server_bindings::GptParams;
  use rstest::{fixture, rstest};
  use serde_json::json;
  use serial_test::serial;

  fn setup() {
    std::env::remove_var(HF_HOME);
    disable_llama_log();
    unsafe {
      llama_server_disable_logging();
    }
    init_test_tracing();
  }

  #[fixture]
  async fn state() -> RouterState {
    setup();
    let model_path = dirs::home_dir()
      .ok_or(anyhow!("unable to locate home dir"))
      .unwrap()
      .join(".cache/huggingface/hub/models--TheBloke--Llama-2-7B-Chat-GGUF/snapshots/08a5566d61d7cb6b420c3e4387a39e0078e1f2fe5f055f3a03887385304d4bfa/llama-2-7b-chat.Q4_K_M.gguf")
      .canonicalize()
      .unwrap()
      .to_str()
      .unwrap()
      .to_owned();
    let gpt_params = GptParams {
      model: model_path,
      ..Default::default()
    };
    let ctx = SharedContextRw::new_shared_rw(Some(gpt_params))
      .await
      .unwrap();
    RouterState::new(ctx)
  }

  #[fixture]
  async fn empty_state() -> RouterState {
    let ctx = SharedContextRw::new_shared_rw(None).await.unwrap();
    RouterState::new(ctx)
  }

  #[fixture]
  fn inputs() -> (String, String) {
    let model = String::from("TheBloke/Llama-2-7B-Chat-GGUF:llama-2-7b-chat.Q4_K_M.gguf");
    let request = serde_json::to_string(&json! {{
      "model": "TheBloke/Llama-2-7B-Chat-GGUF:llama-2-7b-chat.Q4_K_M.gguf",
      "seed": 42,
      "prompt": "<s>[INST] <<SYS>>\nyou are a helpful assistant\n<</SYS>>\n\nwhat day comes after Monday? [/INST]"
    }})
    .unwrap();
    (model, request)
  }

  #[rstest]
  #[case::loaded_state(state())]
  #[case::empty_state(empty_state())]
  #[awt]
  #[tokio::test]
  #[serial(router_state)]
  #[anyhow_trace]
  async fn test_router_state_read_from_same_model(
    inputs: (String, String),
    #[case]
    #[future]
    state: RouterState,
  ) -> anyhow::Result<()> {
    let (model, request) = inputs;
    let userdata = String::with_capacity(2048);
    state
      .completions(&model, &request, "", Some(test_callback), &userdata)
      .await?;
    let response: CreateChatCompletionResponse = serde_json::from_str(&userdata)?;
    let loaded_model = state
      .ctx
      .get_gpt_params()
      .await?
      .ok_or(anyhow!("gpt params not present"))?
      .model
      .clone();
    let (repo, file) = model.split_once(':').ok_or(anyhow!("failed to split"))?;
    let repo = format!("models--{}", repo.replace('/', "--"));
    assert!(loaded_model.contains(&repo));
    assert!(loaded_model.ends_with(file));
    assert_eq!(
      "  Great, I'm glad you asked! The day that comes after Monday is Tuesday! 😊",
      response
        .choices
        .first()
        .ok_or(anyhow!("choices not present"))?
        .message
        .content
        .as_ref()
        .ok_or(anyhow!("content not present"))?
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[serial(router_state)]
  #[anyhow_trace]
  async fn test_router_state_fails_if_model_not_found(
    #[future] state: RouterState,
  ) -> anyhow::Result<()> {
    let state = state.await;
    let model = "non-existing-model";
    let result = state.completions(model, "", "", None, &String::new()).await;
    assert!(result.is_err());
    assert_eq!(
      format!("model not found: '{}'", model),
      result.unwrap_err().to_string()
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[serial(router_state)]
  #[anyhow_trace]
  async fn test_router_state_load_new_model(#[future] state: RouterState) -> anyhow::Result<()> {
    let state = state.await;
    let model = "TheBloke/Llama-2-7B-Chat-GGUF:llama-2-7b-chat.Q8_0.gguf";
    let request = serde_json::to_string(&json! {{
      "model": model,
      "seed": 42,
      "prompt": "<s>[INST] <<SYS>>\nyou are a helpful assistant\n<</SYS>>\n\nwhat day comes after Monday? [/INST]"
    }})
    .unwrap();
    let userdata = String::with_capacity(2048);
    state
      .completions(model, &request, "", Some(test_callback), &userdata)
      .await?;
    let response: CreateChatCompletionResponse = serde_json::from_str(&userdata).unwrap();
    assert_eq!(model, response.model);
    let loaded_model = state
      .ctx
      .get_gpt_params()
      .await?
      .ok_or(anyhow!("gpt params not present"))?
      .model
      .clone();
    let (repo, file) = model.split_once(':').ok_or(anyhow!("failed to split"))?;
    let repo = format!("models--{}", repo.replace('/', "--"));
    assert!(loaded_model.contains(&repo));
    assert!(loaded_model.ends_with(file));
    assert_eq!(
      "  Great question! The day that comes after Monday is Tuesday! 😊",
      response
        .choices
        .first()
        .ok_or(anyhow!("choices not present"))?
        .message
        .content
        .as_ref()
        .ok_or(anyhow!("content not present"))?
    );
    Ok(())
  }
}
