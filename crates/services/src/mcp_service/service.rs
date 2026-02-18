use super::McpError;
use crate::db::{DbService, McpRow, TimeService};
use chrono::DateTime;
use mcp_client::McpClient;
use objs::{Mcp, McpExecutionRequest, McpExecutionResponse, McpServer, McpTool};
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait McpService: Debug + Send + Sync {
  /// List all MCP instances for a user
  async fn list(&self, user_id: &str) -> Result<Vec<Mcp>, McpError>;

  /// Get a specific MCP instance by ID
  async fn get(&self, user_id: &str, id: &str) -> Result<Option<Mcp>, McpError>;

  /// Create a new MCP instance (resolves url to mcp_server_id internally)
  async fn create(
    &self,
    user_id: &str,
    name: &str,
    slug: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
  ) -> Result<Mcp, McpError>;

  /// Update an existing MCP instance
  async fn update(
    &self,
    user_id: &str,
    id: &str,
    name: &str,
    slug: &str,
    description: Option<String>,
    enabled: bool,
    tools_filter: Option<Vec<String>>,
  ) -> Result<Mcp, McpError>;

  /// Delete an MCP instance
  async fn delete(&self, user_id: &str, id: &str) -> Result<(), McpError>;

  /// Check if a URL is in the allowlist and enabled
  async fn is_url_enabled(&self, url: &str) -> Result<bool, McpError>;

  /// Enable/disable an MCP server URL in the admin allowlist
  async fn set_mcp_server_enabled(
    &self,
    url: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<McpServer, McpError>;

  /// List all MCP server URLs in the allowlist
  async fn list_mcp_servers(&self) -> Result<Vec<McpServer>, McpError>;

  /// Get MCP server by URL
  async fn get_mcp_server_by_url(&self, url: &str) -> Result<Option<McpServer>, McpError>;

  /// Fetch tools from MCP server, cache them, seed the filter
  async fn fetch_tools(&self, user_id: &str, id: &str) -> Result<Vec<McpTool>, McpError>;

  /// Execute a tool on an MCP instance
  async fn execute(
    &self,
    user_id: &str,
    id: &str,
    tool_name: &str,
    request: McpExecutionRequest,
  ) -> Result<McpExecutionResponse, McpError>;
}

#[derive(Debug)]
pub struct DefaultMcpService {
  db_service: Arc<dyn DbService>,
  mcp_client: Arc<dyn McpClient>,
  time_service: Arc<dyn TimeService>,
}

impl DefaultMcpService {
  pub fn new(
    db_service: Arc<dyn DbService>,
    mcp_client: Arc<dyn McpClient>,
    time_service: Arc<dyn TimeService>,
  ) -> Self {
    Self {
      db_service,
      mcp_client,
      time_service,
    }
  }

  fn mcp_server_row_to_model(&self, row: crate::db::McpServerRow) -> McpServer {
    McpServer {
      id: row.id,
      url: row.url,
      enabled: row.enabled,
      updated_by: row.updated_by,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  fn mcp_row_to_model(&self, row: McpRow, url: String) -> Mcp {
    let tools_cache: Option<Vec<McpTool>> = row
      .tools_cache
      .as_ref()
      .and_then(|tc| serde_json::from_str(tc).ok());
    let tools_filter: Option<Vec<String>> = row
      .tools_filter
      .as_ref()
      .and_then(|tf| serde_json::from_str(tf).ok());

    Mcp {
      id: row.id,
      mcp_server_id: row.mcp_server_id,
      url,
      slug: row.slug,
      name: row.name,
      description: row.description,
      enabled: row.enabled,
      tools_cache,
      tools_filter,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  async fn get_mcp_with_url(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<(McpRow, String)>, McpError> {
    let row = self.db_service.get_mcp(user_id, id).await?;
    match row {
      Some(mcp_row) => {
        let server = self
          .db_service
          .get_mcp_server(&mcp_row.mcp_server_id)
          .await?;
        match server {
          Some(s) => Ok(Some((mcp_row, s.url))),
          None => Ok(None),
        }
      }
      None => Ok(None),
    }
  }
}

#[async_trait::async_trait]
impl McpService for DefaultMcpService {
  async fn list(&self, user_id: &str) -> Result<Vec<Mcp>, McpError> {
    let rows = self.db_service.list_mcps(user_id).await?;
    let mut mcps = Vec::with_capacity(rows.len());
    for row in rows {
      let server = self.db_service.get_mcp_server(&row.mcp_server_id).await?;
      let url = server.map(|s| s.url).unwrap_or_default();
      mcps.push(self.mcp_row_to_model(row, url));
    }
    Ok(mcps)
  }

  async fn get(&self, user_id: &str, id: &str) -> Result<Option<Mcp>, McpError> {
    match self.get_mcp_with_url(user_id, id).await? {
      Some((row, url)) => Ok(Some(self.mcp_row_to_model(row, url))),
      None => Ok(None),
    }
  }

  async fn create(
    &self,
    user_id: &str,
    name: &str,
    slug: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
  ) -> Result<Mcp, McpError> {
    if name.is_empty() {
      return Err(McpError::NameRequired);
    }

    objs::validate_mcp_slug(slug).map_err(McpError::InvalidSlug)?;

    if let Some(ref desc) = description {
      objs::validate_mcp_description(desc).map_err(McpError::InvalidDescription)?;
    }

    // Look up mcp_server by exact URL match
    let mcp_server = self
      .db_service
      .get_mcp_server_by_url(url)
      .await?
      .ok_or(McpError::McpUrlNotAllowed)?;

    if !mcp_server.enabled {
      return Err(McpError::McpDisabled);
    }

    // Check slug uniqueness (case-insensitive)
    if self
      .db_service
      .get_mcp_by_slug(user_id, slug)
      .await?
      .is_some()
    {
      return Err(McpError::SlugExists(slug.to_string()));
    }

    let now = self.time_service.utc_now().timestamp();
    let row = McpRow {
      id: Uuid::new_v4().to_string(),
      user_id: user_id.to_string(),
      mcp_server_id: mcp_server.id.clone(),
      name: name.to_string(),
      slug: slug.to_string(),
      description,
      enabled,
      tools_cache: None,
      tools_filter: None,
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp(&row).await?;
    Ok(self.mcp_row_to_model(result, mcp_server.url))
  }

  async fn update(
    &self,
    user_id: &str,
    id: &str,
    name: &str,
    slug: &str,
    description: Option<String>,
    enabled: bool,
    tools_filter: Option<Vec<String>>,
  ) -> Result<Mcp, McpError> {
    if name.is_empty() {
      return Err(McpError::NameRequired);
    }

    objs::validate_mcp_slug(slug).map_err(McpError::InvalidSlug)?;

    if let Some(ref desc) = description {
      objs::validate_mcp_description(desc).map_err(McpError::InvalidDescription)?;
    }

    let (existing, url) = self
      .get_mcp_with_url(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    // Check slug uniqueness if changed
    if slug.to_lowercase() != existing.slug.to_lowercase()
      && self
        .db_service
        .get_mcp_by_slug(user_id, slug)
        .await?
        .is_some()
    {
      return Err(McpError::SlugExists(slug.to_string()));
    }

    let resolved_filter = if let Some(filter) = tools_filter {
      Some(serde_json::to_string(&filter).expect("Vec<String> serialization cannot fail"))
    } else {
      existing.tools_filter
    };

    let now = self.time_service.utc_now().timestamp();
    let row = McpRow {
      id: id.to_string(),
      user_id: user_id.to_string(),
      mcp_server_id: existing.mcp_server_id,
      name: name.to_string(),
      slug: slug.to_string(),
      description,
      enabled,
      tools_cache: existing.tools_cache,
      tools_filter: resolved_filter,
      created_at: existing.created_at,
      updated_at: now,
    };

    let result = self.db_service.update_mcp(&row).await?;
    Ok(self.mcp_row_to_model(result, url))
  }

  async fn delete(&self, user_id: &str, id: &str) -> Result<(), McpError> {
    let _ = self
      .get_mcp_with_url(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    self.db_service.delete_mcp(user_id, id).await?;
    Ok(())
  }

  async fn is_url_enabled(&self, url: &str) -> Result<bool, McpError> {
    match self.db_service.get_mcp_server_by_url(url).await? {
      Some(server) => Ok(server.enabled),
      None => Ok(false),
    }
  }

  async fn set_mcp_server_enabled(
    &self,
    url: &str,
    enabled: bool,
    updated_by: &str,
  ) -> Result<McpServer, McpError> {
    let row = self
      .db_service
      .set_mcp_server_enabled(url, enabled, updated_by)
      .await?;
    Ok(self.mcp_server_row_to_model(row))
  }

  async fn list_mcp_servers(&self) -> Result<Vec<McpServer>, McpError> {
    let rows = self.db_service.list_mcp_servers().await?;
    Ok(
      rows
        .into_iter()
        .map(|r| self.mcp_server_row_to_model(r))
        .collect(),
    )
  }

  async fn get_mcp_server_by_url(&self, url: &str) -> Result<Option<McpServer>, McpError> {
    let row = self.db_service.get_mcp_server_by_url(url).await?;
    Ok(row.map(|r| self.mcp_server_row_to_model(r)))
  }

  async fn fetch_tools(&self, user_id: &str, id: &str) -> Result<Vec<McpTool>, McpError> {
    let (existing, url) = self
      .get_mcp_with_url(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    // Verify the MCP server is still enabled
    let server = self
      .db_service
      .get_mcp_server(&existing.mcp_server_id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    if !server.enabled {
      return Err(McpError::McpDisabled);
    }

    // Fetch tools from the MCP server
    let tools = self.mcp_client.fetch_tools(&url).await?;

    // Cache tools and seed filter with all tool names
    let tools_cache_json = serde_json::to_string(&tools).unwrap_or_default();
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

    // Only seed filter if currently empty/null
    let tools_filter_json = if existing.tools_filter.is_none() {
      Some(serde_json::to_string(&tool_names).unwrap_or_default())
    } else {
      existing.tools_filter
    };

    let now = self.time_service.utc_now().timestamp();
    let updated_row = McpRow {
      tools_cache: Some(tools_cache_json),
      tools_filter: tools_filter_json,
      updated_at: now,
      ..existing
    };

    self.db_service.update_mcp(&updated_row).await?;
    Ok(tools)
  }

  async fn execute(
    &self,
    user_id: &str,
    id: &str,
    tool_name: &str,
    request: McpExecutionRequest,
  ) -> Result<McpExecutionResponse, McpError> {
    let (existing, url) = self
      .get_mcp_with_url(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    // Verify the MCP server is still enabled
    let server = self
      .db_service
      .get_mcp_server(&existing.mcp_server_id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    if !server.enabled {
      return Err(McpError::McpDisabled);
    }

    if !existing.enabled {
      return Err(McpError::McpDisabled);
    }

    // Check tool is in filter whitelist
    let tools_filter: Vec<String> = existing
      .tools_filter
      .as_ref()
      .and_then(|tf| serde_json::from_str(tf).ok())
      .unwrap_or_default();

    if !tools_filter.iter().any(|t| t == tool_name) {
      return Err(McpError::ToolNotAllowed(tool_name.to_string()));
    }

    // Execute via mcp_client
    match self
      .mcp_client
      .call_tool(&url, tool_name, request.params)
      .await
    {
      Ok(result) => Ok(McpExecutionResponse {
        result: Some(result),
        error: None,
      }),
      Err(e) => Ok(McpExecutionResponse {
        result: None,
        error: Some(e.to_string()),
      }),
    }
  }
}
