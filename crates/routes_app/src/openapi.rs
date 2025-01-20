use crate::{
  AliasResponse, ApiTokenResponse, AppInfo, CreateApiTokenRequest, NewDownloadRequest,
  SetupRequest, SetupResponse, UserInfo, __path_app_info_handler,
  __path_create_pull_request_handler, __path_create_token_handler,
  __path_get_download_status_handler, __path_list_chat_templates_handler,
  __path_list_downloads_handler, __path_list_local_aliases_handler,
  __path_list_local_modelfiles_handler, __path_logout_handler, __path_ping_handler,
  __path_pull_by_alias_handler, __path_setup_handler, __path_user_info_handler,
};
use objs::{ChatTemplateId, ChatTemplateType, OpenAIApiError, Repo};
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
make_ui_endpoint!(ENDPOINT_MODEL_PULL_BY_ALIAS, "modelfiles/pull/:alias");
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
            AliasResponse,
            ChatTemplateType,
            ChatTemplateId,
            Repo,
            CreateApiTokenRequest,
            ApiTokenResponse,
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
        pull_by_alias_handler,
        list_local_aliases_handler,
        list_chat_templates_handler,
        create_token_handler,
        get_download_status_handler
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
  use crate::{
    ApiDoc, ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP, ENDPOINT_CHAT_TEMPLATES, ENDPOINT_LOGOUT,
    ENDPOINT_MODELS, ENDPOINT_MODEL_FILES, ENDPOINT_MODEL_PULL, ENDPOINT_PING, ENDPOINT_TOKENS,
    ENDPOINT_USER_INFO,
  };
  use pretty_assertions::assert_eq;
  use serde_json::json;
  use utoipa::{
    openapi::{path::ParameterIn, RefOr},
    OpenApi,
  };

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

  #[test]
  fn test_model_aliases_endpoint() {
    let api_doc = ApiDoc::openapi();

    // Verify endpoint
    let paths = &api_doc.paths;
    let aliases = paths
      .paths
      .get(ENDPOINT_MODELS)
      .expect("Model aliases endpoint not found");
    let get_op = aliases.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "listModelAliases");

    // Check query parameters
    let params = get_op.parameters.as_ref().unwrap();
    assert!(params.iter().any(|p| p.name == "page"));
    assert!(params.iter().any(|p| p.name == "page_size"));
    assert!(params.iter().any(|p| p.name == "sort"));
    assert!(params.iter().any(|p| p.name == "sort_order"));

    // Check responses
    let responses = &get_op.responses;
    let success = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        // Verify example has correct structure
        assert!(example.get("data").is_some());
        assert!(example.get("total").is_some());
        assert!(example.get("page").is_some());
        assert!(example.get("page_size").is_some());

        // Verify alias response structure
        let data = example.get("data").unwrap().as_array().unwrap();
        let alias = &data[0];
        assert!(alias.get("alias").is_some());
        assert!(alias.get("repo").is_some());
        assert!(alias.get("filename").is_some());
        assert!(alias.get("source").is_some());
        assert!(alias.get("chat_template").is_some());
        assert!(alias.get("model_params").is_some());
        assert!(alias.get("request_params").is_some());
        assert!(alias.get("context_params").is_some());
      }
    }
  }

  #[test]
  fn test_chat_templates_endpoint() {
    let api_doc = ApiDoc::openapi();

    // Verify endpoint
    let paths = &api_doc.paths;
    let templates = paths
      .paths
      .get(ENDPOINT_CHAT_TEMPLATES)
      .expect("Chat templates endpoint not found");
    let get_op = templates.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "listChatTemplates");

    // Check responses
    let responses = &get_op.responses;
    let success = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        let templates = example.as_array().unwrap();
        // Verify we have both types of templates
        assert!(templates.iter().any(|t| t.get("id").is_some()));
        assert!(templates.iter().any(|t| t.get("repo").is_some()));
      }
    }
  }

  #[test]
  fn test_create_token_endpoint() {
    let api_doc = ApiDoc::openapi();

    // Verify endpoint
    let paths = &api_doc.paths;
    let tokens = paths
      .paths
      .get(ENDPOINT_TOKENS)
      .expect("Tokens endpoint not found");
    let post_op = tokens.post.as_ref().expect("POST operation not found");

    // Check operation details
    assert_eq!(post_op.tags.as_ref().unwrap()[0], "auth");
    assert_eq!(post_op.operation_id.as_ref().unwrap(), "createApiToken");

    // Verify request body schema
    let request_body = post_op.request_body.as_ref().unwrap();
    let content = request_body.content.get("application/json").unwrap();
    if let Some(example) = &content.example {
      assert!(example.get("name").is_some());
    }

    // Check responses
    let responses = &post_op.responses;
    assert!(responses.responses.contains_key("201"));
    assert!(responses.responses.contains_key("400"));
    assert!(responses.responses.contains_key("500"));

    // Verify response schema
    let success_response = responses.responses.get("201").unwrap();
    if let RefOr::T(response) = success_response {
      let content = response.content.get("application/json").unwrap();
      assert!(content.schema.is_some());
    }
  }

  #[test]
  fn test_pull_by_alias_endpoint() {
    let api_doc = ApiDoc::openapi();
    // Verify endpoint
    let paths = &api_doc.paths;
    let pull_alias = paths
      .paths
      .get("/bodhi/v1/modelfiles/pull/{alias}")
      .expect("Pull by alias endpoint not found");

    // Check POST operation
    let post_op = pull_alias.post.as_ref().expect("POST operation not found");
    assert_eq!(post_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(post_op.operation_id.as_ref().unwrap(), "pullModelByAlias");

    // Check path parameters
    let params = post_op.parameters.as_ref().unwrap();
    let alias_param = params
      .iter()
      .find(|p| p.name == "alias")
      .expect("Alias parameter not found");
    assert_eq!(
      serde_json::to_string(&alias_param.parameter_in).unwrap(),
      serde_json::to_string(&ParameterIn::Path).unwrap()
    );

    // Check responses
    let responses = &post_op.responses;
    assert!(responses.responses.contains_key("201"));
    assert!(responses.responses.contains_key("200"));
    assert!(responses.responses.contains_key("404"));
    assert!(responses.responses.contains_key("400"));
    assert!(responses.responses.contains_key("500"));

    // Verify response schema references DownloadRequest
    let created_response = responses.responses.get("201").unwrap();
    if let RefOr::T(response) = created_response {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        assert!(example.get("id").is_some());
        assert!(example.get("repo").is_some());
        assert!(example.get("filename").is_some());
        assert!(example.get("status").is_some());
      }
    }
  }

  #[test]
  fn test_get_download_status_endpoint() {
    let api_doc = ApiDoc::openapi();
    let paths = &api_doc.paths;

    // Verify endpoint
    let status_path = paths
      .paths
      .get("/bodhi/v1/modelfiles/pull/{id}")
      .expect("Download status endpoint not found");

    let get_op = status_path.get.as_ref().expect("GET operation not found");

    // Check operation details
    assert_eq!(get_op.tags.as_ref().unwrap()[0], "models");
    assert_eq!(get_op.operation_id.as_ref().unwrap(), "getDownloadStatus");

    // Check path parameters
    let params = get_op.parameters.as_ref().unwrap();
    let id_param = params
      .iter()
      .find(|p| p.name == "id")
      .expect("ID parameter not found");
    assert_eq!(
      serde_json::to_string(&id_param.parameter_in).unwrap(),
      serde_json::to_string(&ParameterIn::Path).unwrap()
    );
    assert!(id_param.description.is_some());

    // Check responses
    let responses = &get_op.responses;

    // Check 200 response
    let success = responses.responses.get("200").unwrap();
    if let RefOr::T(response) = success {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        // Verify example has correct structure
        assert!(example.get("id").is_some());
        assert!(example.get("repo").is_some());
        assert!(example.get("filename").is_some());
        assert!(example.get("status").is_some());
        assert!(example.get("created_at").is_some());
        assert!(example.get("updated_at").is_some());

        // Verify status is "completed" in example
        assert_eq!(example.get("status").unwrap(), "completed");
      } else {
        panic!("No example found for 200 status");
      }
    }

    // Check 404 response
    let not_found = responses.responses.get("404").unwrap();
    if let RefOr::T(response) = not_found {
      let content = response.content.get("application/json").unwrap();
      if let Some(example) = &content.example {
        let error = example.get("error").unwrap();
        assert_eq!(error.get("type").unwrap(), "not_found_error");
        assert_eq!(error.get("code").unwrap(), "item_not_found");
      } else {
        panic!("No example found for 404 status");
      }
    }

    // Check 500 response exists
    assert!(responses.responses.contains_key("500"));
  }
}
