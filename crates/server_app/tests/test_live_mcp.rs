mod utils;

use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use serde_json::{json, Value};
use utils::{create_test_session_for_live_server, start_test_live_server};

const MCP_TEST_URL: &str = "https://mcp.deepwiki.com/mcp";

/// Full CRUD flow for MCP servers and instances through the live HTTP server.
///
/// Test steps:
///   1. Admin creates an MCP server via POST /mcp_servers
///   2. Admin gets MCP server by ID via GET /mcp_servers/{id}
///   3. Admin updates MCP server via PUT /mcp_servers/{id}
///   4. User creates an MCP instance via POST /mcps (with mcp_server_id)
///   5. User lists MCPs via GET /mcps -> asserts instance present with nested mcp_server
///   6. User gets MCP by ID via GET /mcps/{id} -> asserts nested mcp_server
///   7. User updates MCP via PUT /mcps/{id} -> asserts updated fields
///   8. User lists MCP servers via GET /mcp_servers -> asserts server with counts
///   9. User deletes MCP via DELETE /mcps/{id} -> asserts gone
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_crud_flow() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let (admin_cookie, _admin_user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  // Step 1: Create MCP server (admin)
  let resp = client
    .post(format!("{}/bodhi/v1/mcp_servers", server.base_url))
    .header("Cookie", &admin_cookie)
    .json(&json!({
      "url": MCP_TEST_URL,
      "name": "DeepWiki",
      "description": "DeepWiki MCP server",
      "enabled": true
    }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::CREATED,
    resp.status(),
    "Admin should be able to create an MCP server"
  );
  let server_resp: Value = resp.json().await?;
  let mcp_server_id = server_resp["id"].as_str().unwrap().to_string();
  assert_eq!(MCP_TEST_URL, server_resp["url"].as_str().unwrap());
  assert_eq!("DeepWiki", server_resp["name"].as_str().unwrap());
  assert_eq!(true, server_resp["enabled"].as_bool().unwrap());
  assert_eq!(0, server_resp["enabled_mcp_count"].as_i64().unwrap());
  assert_eq!(0, server_resp["disabled_mcp_count"].as_i64().unwrap());

  // Step 2: Get MCP server by ID
  let resp = client
    .get(format!(
      "{}/bodhi/v1/mcp_servers/{}",
      server.base_url, mcp_server_id
    ))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let fetched_server: Value = resp.json().await?;
  assert_eq!(mcp_server_id, fetched_server["id"].as_str().unwrap());
  assert_eq!("DeepWiki", fetched_server["name"].as_str().unwrap());

  // Step 3: Update MCP server name/description
  let resp = client
    .put(format!(
      "{}/bodhi/v1/mcp_servers/{}",
      server.base_url, mcp_server_id
    ))
    .header("Cookie", &admin_cookie)
    .json(&json!({
      "url": MCP_TEST_URL,
      "name": "DeepWiki Updated",
      "description": "Updated MCP server",
      "enabled": true
    }))
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let updated_server: Value = resp.json().await?;
  assert_eq!("DeepWiki Updated", updated_server["name"].as_str().unwrap());

  // Step 4: Create MCP instance (uses mcp_server_id)
  let resp = client
    .post(format!("{}/bodhi/v1/mcps", server.base_url))
    .header("Cookie", &admin_cookie)
    .json(&json!({
      "name": "DeepWiki MCP",
      "slug": "deepwiki",
      "mcp_server_id": mcp_server_id,
      "description": "DeepWiki documentation tool",
      "enabled": true
    }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::CREATED,
    resp.status(),
    "Should be able to create an MCP instance"
  );
  let mcp: Value = resp.json().await?;
  let mcp_id = mcp["id"].as_str().unwrap().to_string();
  assert_eq!("DeepWiki MCP", mcp["name"].as_str().unwrap());
  assert_eq!("deepwiki", mcp["slug"].as_str().unwrap());
  assert_eq!(MCP_TEST_URL, mcp["mcp_server"]["url"].as_str().unwrap());
  assert_eq!(mcp_server_id, mcp["mcp_server"]["id"].as_str().unwrap());

  // Step 5: List MCPs -> nested mcp_server
  let resp = client
    .get(format!("{}/bodhi/v1/mcps", server.base_url))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let list: Value = resp.json().await?;
  let items = list["mcps"].as_array().unwrap();
  assert_eq!(1, items.len());
  assert_eq!(mcp_id, items[0]["id"].as_str().unwrap());
  assert_eq!(
    MCP_TEST_URL,
    items[0]["mcp_server"]["url"].as_str().unwrap()
  );

  // Step 6: Get MCP by ID -> nested mcp_server
  let resp = client
    .get(format!("{}/bodhi/v1/mcps/{}", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let fetched: Value = resp.json().await?;
  assert_eq!(mcp_id, fetched["id"].as_str().unwrap());
  assert_eq!(mcp_server_id, fetched["mcp_server"]["id"].as_str().unwrap());

  // Step 7: Update MCP
  let resp = client
    .put(format!("{}/bodhi/v1/mcps/{}", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
    .json(&json!({
      "name": "DeepWiki Updated",
      "slug": "deepwiki-v2",
      "description": "Updated description",
      "enabled": false
    }))
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let updated: Value = resp.json().await?;
  assert_eq!("DeepWiki Updated", updated["name"].as_str().unwrap());
  assert_eq!("deepwiki-v2", updated["slug"].as_str().unwrap());
  assert_eq!(false, updated["enabled"].as_bool().unwrap());

  // Step 8: List MCP servers with counts
  let resp = client
    .get(format!("{}/bodhi/v1/mcp_servers", server.base_url))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let servers: Value = resp.json().await?;
  let server_items = servers["mcp_servers"].as_array().unwrap();
  assert_eq!(1, server_items.len());
  assert_eq!(MCP_TEST_URL, server_items[0]["url"].as_str().unwrap());
  assert_eq!(0, server_items[0]["enabled_mcp_count"].as_i64().unwrap());
  assert_eq!(1, server_items[0]["disabled_mcp_count"].as_i64().unwrap());

  // Step 9: Delete MCP
  let resp = client
    .delete(format!("{}/bodhi/v1/mcps/{}", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::NO_CONTENT, resp.status());

  // Verify deletion
  let resp = client
    .get(format!("{}/bodhi/v1/mcps/{}", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, resp.status());

  server.handle.shutdown().await?;
  Ok(())
}

/// Full flow: create server -> create MCP -> fetch tools -> execute tool via real deepwiki MCP.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_tool_execution_flow() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let (admin_cookie, _admin_user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  // Step 1: Create MCP server
  let resp = client
    .post(format!("{}/bodhi/v1/mcp_servers", server.base_url))
    .header("Cookie", &admin_cookie)
    .json(&json!({
      "url": MCP_TEST_URL,
      "name": "DeepWiki",
      "description": "DeepWiki MCP server",
      "enabled": true
    }))
    .send()
    .await?;
  assert_eq!(StatusCode::CREATED, resp.status());
  let server_resp: Value = resp.json().await?;
  let mcp_server_id = server_resp["id"].as_str().unwrap().to_string();

  // Step 2: Create MCP instance
  let resp = client
    .post(format!("{}/bodhi/v1/mcps", server.base_url))
    .header("Cookie", &admin_cookie)
    .json(&json!({
      "name": "DeepWiki MCP",
      "slug": "deepwiki",
      "mcp_server_id": mcp_server_id,
      "enabled": true
    }))
    .send()
    .await?;
  assert_eq!(StatusCode::CREATED, resp.status());
  let mcp: Value = resp.json().await?;
  let mcp_id = mcp["id"].as_str().unwrap().to_string();

  // Step 3: Fetch (refresh) tools from real deepwiki MCP server
  let resp = client
    .post(format!(
      "{}/bodhi/v1/mcps/{}/tools/refresh",
      server.base_url, mcp_id
    ))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let tools_resp: Value = resp.json().await?;
  let tools = tools_resp["tools"].as_array().unwrap();
  assert!(
    !tools.is_empty(),
    "deepwiki MCP should return at least one tool"
  );

  let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
  assert!(
    tool_names.contains(&"read_wiki_structure"),
    "Should contain read_wiki_structure, got: {:?}",
    tool_names
  );

  // Step 4: Execute read_wiki_structure tool
  let resp = client
    .post(format!(
      "{}/bodhi/v1/mcps/{}/tools/read_wiki_structure/execute",
      server.base_url, mcp_id
    ))
    .header("Cookie", &admin_cookie)
    .json(&json!({ "params": { "repo_name": "jina-ai/reader" } }))
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let exec_resp: Value = resp.json().await?;
  assert!(
    exec_resp["result"].is_array()
      || exec_resp["result"].is_object()
      || exec_resp["result"].is_string(),
    "Tool execution should return a result, got: {}",
    exec_resp
  );
  assert!(
    exec_resp["error"].is_null(),
    "Tool execution should not return an error"
  );

  server.handle.shutdown().await?;
  Ok(())
}

/// Non-admin user cannot create MCP servers.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_non_admin_cannot_create_mcp_server() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let (user_cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let resp = client
    .post(format!("{}/bodhi/v1/mcp_servers", server.base_url))
    .header("Cookie", &user_cookie)
    .json(&json!({
      "url": MCP_TEST_URL,
      "name": "DeepWiki",
      "enabled": true
    }))
    .send()
    .await?;

  assert_eq!(
    StatusCode::FORBIDDEN,
    resp.status(),
    "Non-admin should not be able to create MCP servers"
  );

  server.handle.shutdown().await?;
  Ok(())
}
