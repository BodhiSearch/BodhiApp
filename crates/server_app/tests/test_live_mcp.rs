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
///   1. Admin enables an MCP server URL via PUT /mcp_servers
///   2. User creates an MCP instance via POST /mcps
///   3. User lists MCPs via GET /mcps -> asserts instance present
///   4. User gets MCP by ID via GET /mcps/{id} -> asserts fields
///   5. User updates MCP via PUT /mcps/{id} -> asserts updated fields
///   6. User lists MCP servers via GET /mcp_servers -> asserts enabled URL
///   7. User deletes MCP via DELETE /mcps/{id} -> asserts gone
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_crud_flow() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  // Create admin session (resource_admin role needed for PUT/DELETE /mcp_servers)
  let (admin_cookie, _admin_user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  // Step 1: Enable MCP server URL (admin)
  let resp = client
    .put(format!("{}/bodhi/v1/mcp_servers", server.base_url))
    .header("Cookie", &admin_cookie)
    .json(&json!({ "url": MCP_TEST_URL, "enabled": true }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::OK,
    resp.status(),
    "Admin should be able to enable an MCP server URL"
  );
  let server_resp: Value = resp.json().await?;
  assert_eq!(MCP_TEST_URL, server_resp["url"].as_str().unwrap());
  assert_eq!(true, server_resp["enabled"].as_bool().unwrap());
  assert!(
    server_resp["id"].as_str().is_some(),
    "mcp_server should have an id"
  );

  // Step 2: Create MCP instance (uses same admin session which also has user access)
  let resp = client
    .post(format!("{}/bodhi/v1/mcps", server.base_url))
    .header("Cookie", &admin_cookie)
    .json(&json!({
      "name": "DeepWiki MCP",
      "slug": "deepwiki",
      "url": MCP_TEST_URL,
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
  assert_eq!(MCP_TEST_URL, mcp["url"].as_str().unwrap());
  assert_eq!(
    "DeepWiki documentation tool",
    mcp["description"].as_str().unwrap()
  );
  assert_eq!(true, mcp["enabled"].as_bool().unwrap());

  // Step 3: List MCPs
  let resp = client
    .get(format!("{}/bodhi/v1/mcps", server.base_url))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let list: Value = resp.json().await?;
  let items = list["mcps"].as_array().unwrap();
  assert_eq!(1, items.len(), "Should have exactly 1 MCP instance");
  assert_eq!(mcp_id, items[0]["id"].as_str().unwrap());

  // Step 4: Get MCP by ID
  let resp = client
    .get(format!("{}/bodhi/v1/mcps/{}", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let fetched: Value = resp.json().await?;
  assert_eq!(mcp_id, fetched["id"].as_str().unwrap());
  assert_eq!("DeepWiki MCP", fetched["name"].as_str().unwrap());
  assert_eq!("deepwiki", fetched["slug"].as_str().unwrap());

  // Step 5: Update MCP
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
  assert_eq!(
    StatusCode::OK,
    resp.status(),
    "Should be able to update an MCP instance"
  );
  let updated: Value = resp.json().await?;
  assert_eq!("DeepWiki Updated", updated["name"].as_str().unwrap());
  assert_eq!("deepwiki-v2", updated["slug"].as_str().unwrap());
  assert_eq!(
    "Updated description",
    updated["description"].as_str().unwrap()
  );
  assert_eq!(false, updated["enabled"].as_bool().unwrap());

  // Step 6: List MCP servers (all users can read)
  let resp = client
    .get(format!("{}/bodhi/v1/mcp_servers", server.base_url))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let servers: Value = resp.json().await?;
  let server_items = servers["mcp_servers"].as_array().unwrap();
  assert_eq!(
    1,
    server_items.len(),
    "Should have exactly 1 MCP server in allowlist"
  );
  assert_eq!(MCP_TEST_URL, server_items[0]["url"].as_str().unwrap());

  // Step 7: Delete MCP
  let resp = client
    .delete(format!("{}/bodhi/v1/mcps/{}", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(
    StatusCode::NO_CONTENT,
    resp.status(),
    "Should be able to delete an MCP instance"
  );

  // Verify deletion
  let resp = client
    .get(format!("{}/bodhi/v1/mcps/{}", server.base_url, mcp_id))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(
    StatusCode::NOT_FOUND,
    resp.status(),
    "Deleted MCP should return 404"
  );

  server.handle.shutdown().await?;
  Ok(())
}

/// Full flow: enable URL -> create MCP -> fetch tools -> execute tool via real deepwiki MCP.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_mcp_tool_execution_flow() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  let (admin_cookie, _admin_user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_admin"]).await?;

  // Step 1: Enable MCP server URL
  let resp = client
    .put(format!("{}/bodhi/v1/mcp_servers", server.base_url))
    .header("Cookie", &admin_cookie)
    .json(&json!({ "url": MCP_TEST_URL, "enabled": true }))
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());

  // Step 2: Create MCP instance
  let resp = client
    .post(format!("{}/bodhi/v1/mcps", server.base_url))
    .header("Cookie", &admin_cookie)
    .json(&json!({
      "name": "DeepWiki MCP",
      "slug": "deepwiki",
      "url": MCP_TEST_URL,
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

  // Verify known tools are present
  let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
  assert!(
    tool_names.contains(&"read_wiki_structure"),
    "Should contain read_wiki_structure, got: {:?}",
    tool_names
  );

  // Step 4: List cached tools (should return same tools from cache)
  let resp = client
    .get(format!(
      "{}/bodhi/v1/mcps/{}/tools",
      server.base_url, mcp_id
    ))
    .header("Cookie", &admin_cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let cached_tools: Value = resp.json().await?;
  let cached = cached_tools["tools"].as_array().unwrap();
  assert_eq!(
    tools.len(),
    cached.len(),
    "Cached tools should match refreshed tools"
  );

  // Step 5: Execute read_wiki_structure tool
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

/// Non-admin user cannot enable/disable MCP servers.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_non_admin_cannot_enable_mcp_server() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();

  // Create a regular user session (no admin role)
  let (user_cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let resp = client
    .put(format!("{}/bodhi/v1/mcp_servers", server.base_url))
    .header("Cookie", &user_cookie)
    .json(&json!({ "url": MCP_TEST_URL, "enabled": true }))
    .send()
    .await?;

  assert_eq!(
    StatusCode::FORBIDDEN,
    resp.status(),
    "Non-admin should not be able to enable MCP servers"
  );

  server.handle.shutdown().await?;
  Ok(())
}
