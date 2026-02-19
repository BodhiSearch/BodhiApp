use crate::{
  BodhiOpenAPIDoc, GlobalErrorResponses, ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP, ENDPOINT_LOGOUT,
  ENDPOINT_MODELS, ENDPOINT_MODEL_FILES, ENDPOINT_MODEL_PULL, ENDPOINT_PING, ENDPOINT_TOKENS,
  ENDPOINT_USER_INFO,
};
use pretty_assertions::assert_eq;
use serde_json::json;
use utoipa::{
  openapi::{path::ParameterIn, OpenApi as OpenApiSpec, RefOr},
  Modify, OpenApi,
};

/// Helper function to get OpenAPI spec with GlobalErrorResponses modifier applied
/// This ensures tests validate against the same spec used in production
fn get_openapi_with_modifiers() -> OpenApiSpec {
  let mut spec = BodhiOpenAPIDoc::openapi();
  let modifier = GlobalErrorResponses;
  modifier.modify(&mut spec);
  spec
}

#[test]
fn test_openapi_basic_info() {
  let api_doc = BodhiOpenAPIDoc::openapi();

  // Test API Info
  let info = &api_doc.info;
  assert_eq!(info.title, "Bodhi App APIs");

  // Test Contact Info
  let contact = info.contact.as_ref().unwrap();
  assert_eq!(contact.name.as_deref().unwrap(), "Bodhi API Support");
  assert_eq!(
    contact.url.as_deref().unwrap(),
    "https://github.com/BodhiSearch/BodhiApp/issues"
  );
  assert_eq!(contact.email.as_deref().unwrap(), "support@getbodhi.app");

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
  let api_doc = get_openapi_with_modifiers();

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
  let api_doc = get_openapi_with_modifiers();

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
  let api_doc = get_openapi_with_modifiers();

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

  // Verify JSON response in 200 response
  let success_response = responses.responses.get("200").unwrap();
  if let RefOr::T(response) = success_response {
    assert!(response.content.contains_key("application/json"));
  }
}

#[test]
fn test_ping_endpoint() {
  let api_doc = BodhiOpenAPIDoc::openapi();

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
  let api_doc = get_openapi_with_modifiers();

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

  // Verify response schema references UserResponse
  let success_response = responses.responses.get("200").unwrap();
  if let RefOr::T(response) = success_response {
    assert!(response.content.get("application/json").is_some());
  }
}

#[test]
fn test_modelfiles_endpoint() {
  let api_doc = get_openapi_with_modifiers();

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
  let api_doc = get_openapi_with_modifiers();

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
  let api_doc = get_openapi_with_modifiers();

  // Verify endpoint
  let paths = &api_doc.paths;
  let aliases = paths
    .paths
    .get(ENDPOINT_MODELS)
    .expect("Model aliases endpoint not found");
  let get_op = aliases.get.as_ref().expect("GET operation not found");

  // Check operation details
  assert_eq!(get_op.tags.as_ref().unwrap()[0], "models");
  assert_eq!(get_op.operation_id.as_ref().unwrap(), "listAllModels");

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
    }
  }
}

#[test]
fn test_create_token_endpoint() {
  let api_doc = get_openapi_with_modifiers();

  // Verify endpoint
  let paths = &api_doc.paths;
  let tokens = paths
    .paths
    .get(ENDPOINT_TOKENS)
    .expect("Tokens endpoint not found");
  let post_op = tokens.post.as_ref().expect("POST operation not found");

  // Check operation details
  assert_eq!(post_op.tags.as_ref().unwrap()[0], "api-keys");
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
fn test_get_download_status_endpoint() {
  let api_doc = get_openapi_with_modifiers();
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
      assert_eq!(error.get("code").unwrap(), "db_error-item_not_found");
    } else {
      panic!("No example found for 404 status");
    }
  }

  // Check 500 response exists
  assert!(responses.responses.contains_key("500"));
}

#[test]
fn test_list_tokens_endpoint() {
  let api_doc = get_openapi_with_modifiers();
  let paths = &api_doc.paths;

  // Verify endpoint
  let tokens_path = paths
    .paths
    .get(ENDPOINT_TOKENS)
    .expect("Tokens endpoint not found");

  let get_op = tokens_path.get.as_ref().expect("GET operation not found");

  // Check operation details
  assert_eq!(get_op.tags.as_ref().unwrap()[0], "api-keys");
  assert_eq!(get_op.operation_id.as_ref().unwrap(), "listApiTokens");

  // Check pagination parameters
  let params = get_op.parameters.as_ref().unwrap();
  assert!(params.iter().any(|p| p.name == "page"));
  assert!(params.iter().any(|p| p.name == "page_size"));
  assert!(params.iter().any(|p| p.name == "sort"));
  assert!(params.iter().any(|p| p.name == "sort_order"));

  // Check responses
  let responses = &get_op.responses;

  // Check 200 response
  let success = responses.responses.get("200").unwrap();
  if let RefOr::T(response) = success {
    let content = response.content.get("application/json").unwrap();
    if let Some(example) = &content.example {
      // Verify paginated response structure
      assert!(example.get("data").is_some());
      assert!(example.get("total").is_some());
      assert!(example.get("page").is_some());
      assert!(example.get("page_size").is_some());

      // Verify token data structure
      let data = example.get("data").unwrap().as_array().unwrap();
      let token = &data[0];
      assert!(token.get("id").is_some());
      assert!(token.get("user_id").is_some());
      assert!(token.get("name").is_some());
      assert!(token.get("token_id").is_some());
      assert!(token.get("status").is_some());
      assert!(token.get("created_at").is_some());
      assert!(token.get("updated_at").is_some());
    } else {
      panic!("No example found for 200 status");
    }
  }

  // Check 401 response - added by GlobalErrorResponses
  let unauthorized = responses.responses.get("401").unwrap();
  if let RefOr::T(response) = unauthorized {
    let content = response.content.get("application/json").unwrap();
    // Verify schema reference instead of example (GlobalErrorResponses uses schema refs)
    assert!(content.schema.is_some());
  }

  // Check 500 response exists
  assert!(responses.responses.contains_key("500"));
}

#[test]
fn test_update_token_endpoint() {
  let api_doc = get_openapi_with_modifiers();
  let paths = &api_doc.paths;

  // Verify endpoint
  let update_path = paths
    .paths
    .get("/bodhi/v1/tokens/{id}")
    .expect("Update token endpoint not found");

  let put_op = update_path.put.as_ref().expect("PUT operation not found");

  // Check operation details
  assert_eq!(put_op.tags.as_ref().unwrap()[0], "api-keys");
  assert_eq!(put_op.operation_id.as_ref().unwrap(), "updateApiToken");

  // Check path parameters
  let params = put_op.parameters.as_ref().unwrap();
  let id_param = params
    .iter()
    .find(|p| p.name == "id")
    .expect("ID parameter not found");
  assert_eq!(
    serde_json::to_string(&id_param.parameter_in).unwrap(),
    serde_json::to_string(&ParameterIn::Path).unwrap()
  );
  assert!(id_param.description.is_some());

  // Check request body
  let request_body = put_op.request_body.as_ref().unwrap();
  let content = request_body.content.get("application/json").unwrap();
  if let Some(example) = &content.example {
    assert!(example.get("name").is_some());
    assert!(example.get("status").is_some());
    assert_eq!(example.get("status").unwrap(), "inactive");
  } else {
    panic!("No example found for request body");
  }

  // Check responses
  let responses = &put_op.responses;

  // Check 200 response
  let success = responses.responses.get("200").unwrap();
  if let RefOr::T(response) = success {
    let content = response.content.get("application/json").unwrap();
    if let Some(example) = &content.example {
      // Verify token structure
      assert!(example.get("id").is_some());
      assert!(example.get("user_id").is_some());
      assert!(example.get("name").is_some());
      assert!(example.get("token_id").is_some());
      assert!(example.get("status").is_some());
      assert!(example.get("created_at").is_some());
      assert!(example.get("updated_at").is_some());

      // Verify updated values
      assert_eq!(example.get("name").unwrap(), "Updated Token Name");
      assert_eq!(example.get("status").unwrap(), "inactive");
    } else {
      panic!("No example found for 200 status");
    }
  }

  // Check 401 response - added by GlobalErrorResponses
  let unauthorized = responses.responses.get("401").unwrap();
  if let RefOr::T(response) = unauthorized {
    let content = response.content.get("application/json").unwrap();
    // Verify schema reference instead of example (GlobalErrorResponses uses schema refs)
    assert!(content.schema.is_some());
  }

  // Check 404 response
  let not_found = responses.responses.get("404").unwrap();
  if let RefOr::T(response) = not_found {
    let content = response.content.get("application/json").unwrap();
    if let Some(example) = &content.example {
      let error = example.get("error").unwrap();
      assert_eq!(error.get("type").unwrap(), "not_found_error");
      assert_eq!(error.get("code").unwrap(), "entity_error-not_found");
    } else {
      panic!("No example found for 404 status");
    }
  }

  // Check 500 response exists
  assert!(responses.responses.contains_key("500"));
}

#[test]
fn test_oai_models_endpoint() {
  let api_doc = get_openapi_with_modifiers();
  let paths = &api_doc.paths;

  // Verify endpoint
  let models_path = paths
    .paths
    .get("/v1/models")
    .expect("OpenAI models endpoint not found");

  let get_op = models_path.get.as_ref().expect("GET operation not found");

  // Check operation details
  assert_eq!(get_op.tags.as_ref().unwrap()[0], "openai");
  assert_eq!(get_op.operation_id.as_ref().unwrap(), "listModels");

  // Check responses
  let responses = &get_op.responses;

  // Check 200 response
  let success = responses.responses.get("200").unwrap();
  if let RefOr::T(response) = success {
    let content = response.content.get("application/json").unwrap();
    if let Some(example) = &content.example {
      // Verify response structure
      assert_eq!(example.get("object").unwrap(), "list");
      let data = example.get("data").unwrap().as_array().unwrap();

      // Check first model in the list
      let model = &data[0];
      assert!(model.get("id").is_some());
      assert_eq!(model.get("object").unwrap(), "model");
      assert!(model.get("created").is_some());
      assert!(model.get("owned_by").is_some());

      // Verify example values
      assert_eq!(model.get("id").unwrap(), "llama2:chat");
      assert_eq!(model.get("owned_by").unwrap(), "bodhi");
    } else {
      panic!("No example found for 200 status");
    }
  }

  // Check 401 response - added by GlobalErrorResponses
  let unauthorized = responses.responses.get("401").unwrap();
  if let RefOr::T(response) = unauthorized {
    let content = response.content.get("application/json").unwrap();
    // Verify schema reference instead of example (GlobalErrorResponses uses schema refs)
    assert!(content.schema.is_some());
  }

  // Check 500 response exists
  assert!(responses.responses.contains_key("500"));
}

/// Test that runtime OpenAPI spec matches the generated openapi.json file
#[test]
fn test_all_endpoints_match_spec() {
  let runtime_spec = BodhiOpenAPIDoc::openapi();
  let runtime_value = serde_json::to_value(&runtime_spec).unwrap();

  // Load the generated openapi.json file
  let spec_content = include_str!("../../../../openapi.json");
  let generated_spec: serde_json::Value = serde_json::from_str(spec_content).unwrap();

  // Compare key sections to ensure they're in sync
  assert_eq!(
    runtime_value["info"]["title"], generated_spec["info"]["title"],
    "API title mismatch between runtime and generated spec"
  );

  // Compare paths - ensure all paths exist in both specs
  let runtime_paths = runtime_value["paths"].as_object().unwrap();
  let generated_paths = generated_spec["paths"].as_object().unwrap();

  for (path, _) in runtime_paths {
    assert!(
      generated_paths.contains_key(path),
      "Path '{}' exists in runtime spec but missing from generated openapi.json",
      path
    );
  }

  for (path, _) in generated_paths {
    assert!(
      runtime_paths.contains_key(path),
      "Path '{}' exists in generated openapi.json but missing from runtime spec",
      path
    );
  }

  // Compare number of endpoints
  assert_eq!(
    runtime_paths.len(),
    generated_paths.len(),
    "Number of paths mismatch: runtime={}, generated={}",
    runtime_paths.len(),
    generated_paths.len()
  );
}
