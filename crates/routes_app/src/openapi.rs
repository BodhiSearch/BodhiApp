use crate::{
  AppInfo, NewDownloadRequest, SetupRequest, SetupResponse, UserInfo, __path_app_info_handler,
  __path_create_pull_request_handler, __path_list_downloads_handler,
  __path_list_local_modelfiles_handler, __path_logout_handler, __path_ping_handler,
  __path_setup_handler, __path_user_info_handler,
};
use objs::OpenAIApiError;
use services::{db::DownloadRequest, AppStatus};
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
        (name = "setup", description = "Application setup and initialization"),
        (name = "auth", description = "Authentication and session management"),
        (name = "models", description = "Model files and aliases"),
    ),
    components(
        schemas(
            OpenAIApiError,
            AppInfo,
            AppStatus,
            SetupRequest,
            SetupResponse,
            UserInfo,
            NewDownloadRequest,
            DownloadRequest,
        ),
        responses( ),
    ),
    paths(
        app_info_handler,
        setup_handler,
        logout_handler,
        ping_handler,
        user_info_handler,
        list_local_modelfiles_handler,
        list_downloads_handler,
        create_pull_request_handler,
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
  use crate::{
    ApiDoc, ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP, ENDPOINT_LOGOUT, ENDPOINT_MODEL_FILES,
    ENDPOINT_MODEL_PULL, ENDPOINT_PING, ENDPOINT_USER_INFO,
  };
  use pretty_assertions::assert_eq;
  use serde_json::json;
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

  #[test]
  fn test_setup_endpoint() {
    let api_doc = ApiDoc::openapi();

    // Verify tags
    let tags = api_doc.tags.as_ref().unwrap();
    assert!(tags.iter().any(|t| t.name == "setup"));

    // Verify endpoint
    let paths = &api_doc.paths;
    let setup = paths
      .paths
      .get(ENDPOINT_APP_SETUP)
      .expect("Setup endpoint not found");
    let post_op = setup.post.as_ref().expect("POST operation not found");

    // Check operation details
    assert_eq!(post_op.tags.as_ref().unwrap()[0], "setup");
    assert_eq!(post_op.operation_id.as_ref().unwrap(), "setupApp");

    // Check responses
    let responses = &post_op.responses;
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("400"));
    assert!(responses.responses.contains_key("500"));
  }

  #[test]
  fn test_logout_endpoint() {
    let api_doc = ApiDoc::openapi();

    // Verify tags
    let tags = api_doc.tags.as_ref().unwrap();
    assert!(tags.iter().any(|t| t.name == "auth"));

    // Verify endpoint
    let paths = &api_doc.paths;
    let logout = paths
      .paths
      .get(ENDPOINT_LOGOUT)
      .expect("Logout endpoint not found");
    let post_op = logout.post.as_ref().expect("POST operation not found");

    // Check operation details
    assert_eq!(post_op.tags.as_ref().unwrap()[0], "auth");
    assert_eq!(post_op.operation_id.as_ref().unwrap(), "logoutUser");

    // Check responses
    let responses = &post_op.responses;
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("500"));

    // Verify headers in 200 response
    let success_response = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success_response {
      assert!(response.headers.contains_key("Location"));
    }
  }

  #[test]
  fn test_ping_endpoint() {
    let api_doc = ApiDoc::openapi();

    // Verify endpoint
    let paths = &api_doc.paths;
    let ping = paths
      .paths
      .get(ENDPOINT_PING)
      .expect("Ping endpoint not found");
    let get_op = ping.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "system");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "pingServer");

    // Check response
    let responses = &get_op.responses;
    let success = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        assert_eq!(example, &json!({"message": "pong"}));
      } else {
        panic!("No example found for 200 status");
      }
    } else {
      panic!("No response found for 200 status");
    }
  }

  #[test]
  fn test_user_info_endpoint() {
    let api_doc = ApiDoc::openapi();

    // Verify endpoint
    let paths = &api_doc.paths;
    let user_info = paths
      .paths
      .get(ENDPOINT_USER_INFO)
      .expect("User info endpoint not found");
    let get_op = user_info.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "auth");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "getCurrentUser");

    // Check responses
    let responses = &get_op.responses;
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("500"));

    // Verify response schema references UserInfo
    let success_response = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success_response {
      assert!(response.content.get("application/json").is_some());
    }
  }

  #[test]
  fn test_modelfiles_endpoint() {
    let api_doc = ApiDoc::openapi();

    // Verify tags
    let tags = api_doc.tags.as_ref().unwrap();
    assert!(tags.iter().any(|t| t.name == "models"));

    // Verify endpoint
    let paths = &api_doc.paths;
    let modelfiles = paths
      .paths
      .get(ENDPOINT_MODEL_FILES)
      .expect("Modelfiles endpoint not found");
    let get_op = modelfiles.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "listModelFiles");

    // Check query parameters
    let params = get_op.parameters.as_ref().unwrap();
    assert!(params.iter().any(|p| p.name == "page"));
    assert!(params.iter().any(|p| p.name == "page_size"));
    assert!(params.iter().any(|p| p.name == "sort"));
    assert!(params.iter().any(|p| p.name == "sort_order"));

    // Check responses
    let responses = &get_op.responses;
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("500"));

    // Verify response schema references PaginatedResponse<LocalModelResponse>
    let success_response = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success_response {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        // Verify example has correct structure
        assert!(example.get("data").is_some());
        assert!(example.get("total").is_some());
        assert!(example.get("page").is_some());
        assert!(example.get("page_size").is_some());
      } else {
        panic!("No example found for 200 status");
      }
    } else {
      panic!("No response found for 200 status");
    }
  }

  #[test]
  fn test_download_endpoints() {
    let api_doc = ApiDoc::openapi();

    // Verify tags
    let tags = api_doc.tags.as_ref().unwrap();
    assert!(tags.iter().any(|t| t.name == "models"));

    let paths = &api_doc.paths;

    // Test GET /modelfiles/pull endpoint
    let downloads = paths
      .paths
      .get(ENDPOINT_MODEL_PULL)
      .expect("Downloads endpoint not found");

    // Check GET operation
    let get_op = downloads.get.as_ref().expect("GET operation not found");
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "listDownloads");

    // Check query parameters
    let params = get_op.parameters.as_ref().unwrap();
    assert!(params.iter().any(|p| p.name == "page"));
    assert!(params.iter().any(|p| p.name == "page_size"));
    assert!(params.iter().any(|p| p.name == "sort"));
    assert!(params.iter().any(|p| p.name == "sort_order"));

    // Check GET responses
    let get_responses = &get_op.responses;
    let get_200 = get_responses.responses.get("200").unwrap();
    if let RefOr::T(response) = get_200 {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        assert!(example.get("data").is_some());
        assert!(example.get("total").is_some());
        assert!(example.get("page").is_some());
        assert!(example.get("page_size").is_some());
      } else {
        panic!("No example found for GET 200 status");
      }
    }

    // Check POST operation
    let post_op = downloads.post.as_ref().expect("POST operation not found");
    assert_eq!(post_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(post_op.operation_id.as_ref().unwrap(), "pullModelFile");

    // Verify request body schema
    assert!(post_op.request_body.is_some());

    // Check POST responses
    let post_responses = &post_op.responses;
    assert!(post_responses.responses.contains_key("200"));
    assert!(post_responses.responses.contains_key("500"));

    // Verify response schema references DownloadRequest
    let success_response = post_responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success_response {
      let content = response.content.get("application/json").unwrap();
      assert!(content.schema.is_some());
    } else {
      panic!("No response found for POST 200 status");
    }
  }
}