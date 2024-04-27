use crate::{hf::find_model, server::utils::ApiError, SharedContextRwExts};

use super::routes::RouterState;
use axum::response::{IntoResponse, Response};
use llama_server_bindings::GptParams;

pub(crate) async fn load_context(state: &mut RouterState, model: &str) -> Result<(), Response> {
  let Ok(gpt_params) = state.ctx.get_gpt_params().await else {
    return Err(
      ApiError::ServerError("error loading current context params".to_string()).into_response(),
    );
  };
  match gpt_params {
    Some(mut gpt_params) => {
      let Some(request_model) = find_model(model) else {
        // request model not exits, return error
        return Err(
          ApiError::BadRequest(format!("model does not exists: {model}")).into_response(),
        );
      };
      match &gpt_params.model {
        Some(ctx_model) => {
          if !ctx_model.eq(&request_model.path) {
            if let Err(err) = state.ctx.reload(Some(gpt_params)).await {
              return Err(
                ApiError::ServerError(format!("cannot load the new model: {err}")).into_response(),
              );
            }
          }
        }
        None => {
          // context does not have a model, load request model
          gpt_params.model = Some(request_model.path.clone());
          if let Err(err) = state.ctx.reload(Some(gpt_params)).await {
            return Err(
              ApiError::ServerError(format!("cannot load the new model: {err}")).into_response(),
            );
          }
        }
      }
    }
    None => {
      // if context does not have a model loaded, load the model from request
      let gpt_params = GptParams {
        model: Some(model.to_owned()),
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
