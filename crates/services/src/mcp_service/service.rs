use super::{McpError, McpServerError};
use crate::db::{DbService, McpRow, McpServerRow, McpWithServerRow, TimeService};
use chrono::DateTime;
use mcp_client::McpClient;
use objs::{Mcp, McpExecutionRequest, McpExecutionResponse, McpServer, McpServerInfo, McpTool};
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait McpService: Debug + Send + Sync {
  // ---- MCP Server admin operations ----

  async fn create_mcp_server(
    &self,
    name: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
    created_by: &str,
  ) -> Result<McpServer, McpServerError>;

  async fn update_mcp_server(
    &self,
    id: &str,
    name: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
    updated_by: &str,
  ) -> Result<McpServer, McpServerError>;

  async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServer>, McpServerError>;

  async fn list_mcp_servers(&self, enabled: Option<bool>)
    -> Result<Vec<McpServer>, McpServerError>;

  async fn count_mcps_for_server(&self, server_id: &str) -> Result<(i64, i64), McpServerError>;

  // ---- MCP user instance operations ----

  async fn list(&self, user_id: &str) -> Result<Vec<Mcp>, McpError>;

  async fn get(&self, user_id: &str, id: &str) -> Result<Option<Mcp>, McpError>;

  async fn create(
    &self,
    user_id: &str,
    name: &str,
    slug: &str,
    mcp_server_id: &str,
    description: Option<String>,
    enabled: bool,
    tools_cache: Option<Vec<McpTool>>,
    tools_filter: Option<Vec<String>>,
  ) -> Result<Mcp, McpError>;

  async fn update(
    &self,
    user_id: &str,
    id: &str,
    name: &str,
    slug: &str,
    description: Option<String>,
    enabled: bool,
    tools_filter: Option<Vec<String>>,
    tools_cache: Option<Vec<McpTool>>,
  ) -> Result<Mcp, McpError>;

  async fn delete(&self, user_id: &str, id: &str) -> Result<(), McpError>;

  async fn fetch_tools(&self, user_id: &str, id: &str) -> Result<Vec<McpTool>, McpError>;

  async fn fetch_tools_for_server(&self, server_id: &str) -> Result<Vec<McpTool>, McpError>;

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

  fn mcp_server_row_to_model(&self, row: McpServerRow) -> McpServer {
    McpServer {
      id: row.id,
      url: row.url,
      name: row.name,
      description: row.description,
      enabled: row.enabled,
      created_by: row.created_by,
      updated_by: row.updated_by,
      created_at: DateTime::from_timestamp(row.created_at, 0).unwrap(),
      updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap(),
    }
  }

  fn mcp_with_server_to_model(&self, row: McpWithServerRow) -> Mcp {
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
      mcp_server: McpServerInfo {
        id: row.mcp_server_id,
        url: row.server_url,
        name: row.server_name,
        enabled: row.server_enabled,
      },
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

  fn mcp_row_to_model(&self, row: McpRow, server: &McpServerRow) -> Mcp {
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
      mcp_server: McpServerInfo {
        id: server.id.clone(),
        url: server.url.clone(),
        name: server.name.clone(),
        enabled: server.enabled,
      },
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

  async fn get_mcp_with_server(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<(McpRow, McpServerRow)>, McpError> {
    let row = self.db_service.get_mcp(user_id, id).await?;
    match row {
      Some(mcp_row) => {
        let server = self
          .db_service
          .get_mcp_server(&mcp_row.mcp_server_id)
          .await?;
        match server {
          Some(s) => Ok(Some((mcp_row, s))),
          None => Ok(None),
        }
      }
      None => Ok(None),
    }
  }
}

#[async_trait::async_trait]
impl McpService for DefaultMcpService {
  // ---- MCP Server admin operations ----

  async fn create_mcp_server(
    &self,
    name: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
    created_by: &str,
  ) -> Result<McpServer, McpServerError> {
    let trimmed_url = url.trim();

    objs::validate_mcp_server_name(name).map_err(|_| {
      if name.is_empty() {
        McpServerError::NameRequired
      } else {
        McpServerError::NameTooLong
      }
    })?;

    objs::validate_mcp_server_url(trimmed_url).map_err(|e| {
      if trimmed_url.is_empty() {
        McpServerError::UrlRequired
      } else if trimmed_url.len() > objs::MAX_MCP_SERVER_URL_LEN {
        McpServerError::UrlTooLong
      } else {
        McpServerError::UrlInvalid(e)
      }
    })?;

    if let Some(ref desc) = description {
      objs::validate_mcp_server_description(desc)
        .map_err(|_| McpServerError::DescriptionTooLong)?;
    }

    if let Some(existing) = self.db_service.get_mcp_server_by_url(trimmed_url).await? {
      return Err(McpServerError::UrlAlreadyExists(existing.url));
    }

    let now = self.time_service.utc_now().timestamp();
    let row = McpServerRow {
      id: Uuid::new_v4().to_string(),
      url: trimmed_url.to_string(),
      name: name.to_string(),
      description,
      enabled,
      created_by: created_by.to_string(),
      updated_by: created_by.to_string(),
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp_server(&row).await?;
    Ok(self.mcp_server_row_to_model(result))
  }

  async fn update_mcp_server(
    &self,
    id: &str,
    name: &str,
    url: &str,
    description: Option<String>,
    enabled: bool,
    updated_by: &str,
  ) -> Result<McpServer, McpServerError> {
    let trimmed_url = url.trim();

    objs::validate_mcp_server_name(name).map_err(|_| {
      if name.is_empty() {
        McpServerError::NameRequired
      } else {
        McpServerError::NameTooLong
      }
    })?;

    objs::validate_mcp_server_url(trimmed_url).map_err(|e| {
      if trimmed_url.is_empty() {
        McpServerError::UrlRequired
      } else if trimmed_url.len() > objs::MAX_MCP_SERVER_URL_LEN {
        McpServerError::UrlTooLong
      } else {
        McpServerError::UrlInvalid(e)
      }
    })?;

    if let Some(ref desc) = description {
      objs::validate_mcp_server_description(desc)
        .map_err(|_| McpServerError::DescriptionTooLong)?;
    }

    let existing = self
      .db_service
      .get_mcp_server(id)
      .await?
      .ok_or_else(|| McpServerError::McpServerNotFound(id.to_string()))?;

    // Check URL uniqueness excluding current server (case-insensitive)
    if let Some(dup) = self.db_service.get_mcp_server_by_url(trimmed_url).await? {
      if dup.id != existing.id {
        return Err(McpServerError::UrlAlreadyExists(dup.url));
      }
    }

    // If URL changed, cascade: clear tools_cache and tools_filter on linked MCPs
    let url_changed = existing.url.to_lowercase() != trimmed_url.to_lowercase();
    if url_changed {
      self.db_service.clear_mcp_tools_by_server_id(id).await?;
    }

    let now = self.time_service.utc_now().timestamp();
    let row = McpServerRow {
      id: existing.id,
      url: trimmed_url.to_string(),
      name: name.to_string(),
      description,
      enabled,
      created_by: existing.created_by,
      updated_by: updated_by.to_string(),
      created_at: existing.created_at,
      updated_at: now,
    };

    let result = self.db_service.update_mcp_server(&row).await?;
    Ok(self.mcp_server_row_to_model(result))
  }

  async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServer>, McpServerError> {
    let row = self.db_service.get_mcp_server(id).await?;
    Ok(row.map(|r| self.mcp_server_row_to_model(r)))
  }

  async fn list_mcp_servers(
    &self,
    enabled: Option<bool>,
  ) -> Result<Vec<McpServer>, McpServerError> {
    let rows = self.db_service.list_mcp_servers(enabled).await?;
    Ok(
      rows
        .into_iter()
        .map(|r| self.mcp_server_row_to_model(r))
        .collect(),
    )
  }

  async fn count_mcps_for_server(&self, server_id: &str) -> Result<(i64, i64), McpServerError> {
    Ok(self.db_service.count_mcps_by_server_id(server_id).await?)
  }

  // ---- MCP user instance operations ----

  async fn list(&self, user_id: &str) -> Result<Vec<Mcp>, McpError> {
    let rows = self.db_service.list_mcps_with_server(user_id).await?;
    Ok(
      rows
        .into_iter()
        .map(|r| self.mcp_with_server_to_model(r))
        .collect(),
    )
  }

  async fn get(&self, user_id: &str, id: &str) -> Result<Option<Mcp>, McpError> {
    match self.get_mcp_with_server(user_id, id).await? {
      Some((row, server)) => Ok(Some(self.mcp_row_to_model(row, &server))),
      None => Ok(None),
    }
  }

  async fn create(
    &self,
    user_id: &str,
    name: &str,
    slug: &str,
    mcp_server_id: &str,
    description: Option<String>,
    enabled: bool,
    tools_cache: Option<Vec<McpTool>>,
    tools_filter: Option<Vec<String>>,
  ) -> Result<Mcp, McpError> {
    if name.is_empty() {
      return Err(McpError::NameRequired);
    }

    objs::validate_mcp_slug(slug).map_err(McpError::InvalidSlug)?;

    if let Some(ref desc) = description {
      objs::validate_mcp_description(desc).map_err(McpError::InvalidDescription)?;
    }

    let mcp_server = self
      .db_service
      .get_mcp_server(mcp_server_id)
      .await?
      .ok_or_else(|| McpError::McpServerNotFound(mcp_server_id.to_string()))?;

    if !mcp_server.enabled {
      return Err(McpError::McpDisabled);
    }

    if self
      .db_service
      .get_mcp_by_slug(user_id, slug)
      .await?
      .is_some()
    {
      return Err(McpError::SlugExists(slug.to_string()));
    }

    let tools_cache_json = tools_cache
      .as_ref()
      .map(|tc| serde_json::to_string(tc).expect("Vec<McpTool> serialization cannot fail"));
    let tools_filter_json = tools_filter
      .as_ref()
      .map(|tf| serde_json::to_string(tf).expect("Vec<String> serialization cannot fail"));

    let now = self.time_service.utc_now().timestamp();
    let row = McpRow {
      id: Uuid::new_v4().to_string(),
      user_id: user_id.to_string(),
      mcp_server_id: mcp_server.id.clone(),
      name: name.to_string(),
      slug: slug.to_string(),
      description,
      enabled,
      tools_cache: tools_cache_json,
      tools_filter: tools_filter_json,
      created_at: now,
      updated_at: now,
    };

    let result = self.db_service.create_mcp(&row).await?;
    Ok(self.mcp_row_to_model(result, &mcp_server))
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
    tools_cache: Option<Vec<McpTool>>,
  ) -> Result<Mcp, McpError> {
    if name.is_empty() {
      return Err(McpError::NameRequired);
    }

    objs::validate_mcp_slug(slug).map_err(McpError::InvalidSlug)?;

    if let Some(ref desc) = description {
      objs::validate_mcp_description(desc).map_err(McpError::InvalidDescription)?;
    }

    let (existing, server) = self
      .get_mcp_with_server(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

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

    let resolved_cache = if let Some(cache) = tools_cache {
      Some(serde_json::to_string(&cache).expect("Vec<McpTool> serialization cannot fail"))
    } else {
      existing.tools_cache
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
      tools_cache: resolved_cache,
      tools_filter: resolved_filter,
      created_at: existing.created_at,
      updated_at: now,
    };

    let result = self.db_service.update_mcp(&row).await?;
    Ok(self.mcp_row_to_model(result, &server))
  }

  async fn delete(&self, user_id: &str, id: &str) -> Result<(), McpError> {
    let _ = self
      .get_mcp_with_server(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    self.db_service.delete_mcp(user_id, id).await?;
    Ok(())
  }

  async fn fetch_tools(&self, user_id: &str, id: &str) -> Result<Vec<McpTool>, McpError> {
    let (existing, server) = self
      .get_mcp_with_server(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    if !server.enabled {
      return Err(McpError::McpDisabled);
    }

    let tools = self.mcp_client.fetch_tools(&server.url).await?;

    let tools_cache_json = serde_json::to_string(&tools).unwrap_or_default();
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

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

  async fn fetch_tools_for_server(&self, server_id: &str) -> Result<Vec<McpTool>, McpError> {
    let server = self
      .db_service
      .get_mcp_server(server_id)
      .await?
      .ok_or_else(|| McpError::McpServerNotFound(server_id.to_string()))?;

    if !server.enabled {
      return Err(McpError::McpDisabled);
    }

    let tools = self.mcp_client.fetch_tools(&server.url).await?;
    Ok(tools)
  }

  async fn execute(
    &self,
    user_id: &str,
    id: &str,
    tool_name: &str,
    request: McpExecutionRequest,
  ) -> Result<McpExecutionResponse, McpError> {
    let (existing, server) = self
      .get_mcp_with_server(user_id, id)
      .await?
      .ok_or_else(|| McpError::McpNotFound(id.to_string()))?;

    if !server.enabled {
      return Err(McpError::McpDisabled);
    }

    if !existing.enabled {
      return Err(McpError::McpDisabled);
    }

    let tools_filter: Vec<String> = existing
      .tools_filter
      .as_ref()
      .and_then(|tf| serde_json::from_str(tf).ok())
      .unwrap_or_default();

    if !tools_filter.iter().any(|t| t == tool_name) {
      return Err(McpError::ToolNotAllowed(tool_name.to_string()));
    }

    match self
      .mcp_client
      .call_tool(&server.url, tool_name, request.params)
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
