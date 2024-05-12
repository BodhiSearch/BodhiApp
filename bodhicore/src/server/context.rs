use super::routes::RouterState;
use crate::{hf::find_model, server::utils::ApiError, server::SharedContextRwExts};
use axum::response::{IntoResponse, Response};
use llama_server_bindings::GptParams;

pub(crate) async fn load_context(state: &mut RouterState, model: &str) -> Result<(), Response> {
  let Some(request_model) = find_model(model) else {
    // request model not exits, return error
    return Err(ApiError::BadRequest(format!("model does not exists: {model}")).into_response());
  };
  let Ok(gpt_params) = state.ctx.get_gpt_params().await else {
    return Err(
      ApiError::ServerError("error loading current context params".to_string()).into_response(),
    );
  };
  let model_path = request_model.model_path();
  match gpt_params {
    // context is loaded
    Some(gpt_params) => {
      match &gpt_params.model {
        Some(ctx_model) => {
          if !ctx_model.eq(&model_path) {
            let new_gpt_params = GptParams {
              model: Some(model_path),
              ..gpt_params
            };
            if let Err(err) = state.ctx.reload(Some(new_gpt_params)).await {
              return Err(
                ApiError::ServerError(format!("cannot load the new model: {err}")).into_response(),
              );
            }
          }
        }
        None => {
          // context does not have a model, load request model
          let new_gpt_params = GptParams {
            model: Some(model_path),
            ..gpt_params
          };
          if let Err(err) = state.ctx.reload(Some(new_gpt_params)).await {
            return Err(
              ApiError::ServerError(format!("cannot load the new model: {err}")).into_response(),
            );
          }
        }
      }
    }
    None => {
      // if context is not loaded
      let gpt_params = GptParams {
        model: Some(model_path),
        ..Default::default()
      };
      if let Err(err) = state.ctx.reload(Some(gpt_params)).await {
        return Err(
          ApiError::ServerError(format!("failed to reload context with model: {err}"))
            .into_response(),
        );
      }
    }
  }
  Ok(())
}
