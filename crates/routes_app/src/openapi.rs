use crate::{AppInfo, __path_app_info_handler};
use objs::OpenAIApiError;
use services::AppStatus;
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

/// API documentation configuration
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Bodhi App APIs",
        version = env!("CARGO_PKG_VERSION"),
        description = "API documentation for Bodhi App",
        contact(
            name = "Bodhi Support",
            url = "https://github.com/BodhiSearch/BodhiApp"
        ),
    ),
    servers(
        (url = "http://localhost:1135", description = "Local running instance"),
    ),
    tags(
        (name = "system", description = "System information and operations"),
    ),
    components(
        schemas(
            OpenAIApiError,
            AppInfo,
            AppStatus,
        ),
        responses( ),
    ),
    paths(app_info_handler)
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
  use crate::{ApiDoc, ENDPOINT_APP_INFO};
  use pretty_assertions::assert_eq;
  use utoipa::{openapi::RefOr, OpenApi};

  #[test]
  fn test_openapi_basic_info() {
    let api_doc = ApiDoc::openapi();

    // Test API Info
    let info = &api_doc.info;
    assert_eq!(info.title, "Bodhi App APIs");
    assert_eq!(info.version, "0.1.0");
    assert_eq!(
      info.description.as_deref().unwrap(),
      "API documentation for Bodhi App"
    );

    // Test Contact Info
    let contact = info.contact.as_ref().unwrap();
    assert_eq!(contact.name.as_deref().unwrap(), "Bodhi Support");
    assert_eq!(
      contact.url.as_deref().unwrap(),
      "https://github.com/BodhiSearch/BodhiApp"
    );

    // Test Servers
    let servers = api_doc.servers.as_ref().unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].url, "http://localhost:1135");
    assert_eq!(
      servers[0].description.as_deref().unwrap(),
      "Local running instance"
    );
  }

  #[test]
  fn test_app_info_endpoint() {
    let api_doc = ApiDoc::openapi();

    // Verify tags
    let tags = api_doc.tags.as_ref().unwrap();
    assert!(tags.iter().any(|t| t.name == "system"));

    // Verify endpoint
    let paths = &api_doc.paths;
    let app_info = paths
      .paths
      .get(ENDPOINT_APP_INFO)
      .expect("App info endpoint not found");
    let get_op = app_info.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "system");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "getAppInfo");

    // Check responses
    let responses = &get_op.responses;
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("500"));

    // Verify response schema references AppInfo
    let success_response = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success_response {
      assert!(response.content.get("application/json").is_some());
    }
  }
}
