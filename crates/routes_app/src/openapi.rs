use crate::__path_app_info_handler;
use utoipa::OpenApi;

macro_rules! make_ui_endpoint {
  ($name:ident, $path:expr) => {
    pub const $name: &str = concat!("/bodhi/v1/", $path);
  };
}

pub const ENDPOINT_PING: &str = "/ping";
// sent by frontend to redirect to oauth server login
pub const ENDPOINT_LOGIN: &str = "/app/login";
// redirected by oauth server for auth code exchange
pub const ENDPOINT_LOGIN_CALLBACK: &str = "/app/login/callback";

make_ui_endpoint!(ENDPOINT_LOGOUT, "logout");
make_ui_endpoint!(ENDPOINT_APP_INFO, "info");
make_ui_endpoint!(ENDPOINT_APP_SETUP, "setup");
make_ui_endpoint!(ENDPOINT_USER_INFO, "user");
make_ui_endpoint!(ENDPOINT_MODEL_FILES, "modelfiles");
make_ui_endpoint!(ENDPOINT_MODEL_PULL, "modelfiles/pull");
make_ui_endpoint!(ENDPOINT_MODELS, "models");
make_ui_endpoint!(ENDPOINT_CHAT_TEMPLATES, "chat_templates");
make_ui_endpoint!(ENDPOINT_TOKENS, "tokens");

pub const ENDPOINT_OAI_MODELS: &str = "/v1/models";
pub const ENDPOINT_OAI_CHAT_COMPLETIONS: &str = "/v1/chat/completions";
pub const ENDPOINT_OLLAMA_TAGS: &str = "/api/tags";
pub const ENDPOINT_OLLAMA_SHOW: &str = "/api/show";
pub const ENDPOINT_OLLAMA_CHAT: &str = "/api/chat";

// dev-only debugging info endpoint
pub const ENDPOINT_DEV_SECRETS: &str = "/dev/secrets";

#[derive(OpenApi)]
#[openapi(paths(app_info_handler))]
pub struct ApiDoc;
