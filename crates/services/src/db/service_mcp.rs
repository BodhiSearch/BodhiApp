use crate::db::{
  encryption::decrypt_api_key, DbError, McpAuthHeaderRow, McpOAuthConfigRow, McpOAuthTokenRow,
  McpRepository, McpRow, McpServerRow, McpWithServerRow, SqliteDbService,
};

#[async_trait::async_trait]
impl McpRepository for SqliteDbService {
  async fn create_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError> {
    let enabled_int = if row.enabled { 1 } else { 0 };

    sqlx::query(
      r#"
      INSERT INTO mcp_servers (id, url, name, description, enabled, created_by, updated_by, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&row.id)
    .bind(&row.url)
    .bind(&row.name)
    .bind(&row.description)
    .bind(enabled_int)
    .bind(&row.created_by)
    .bind(&row.updated_by)
    .bind(row.created_at)
    .bind(row.updated_at)
    .execute(&self.pool)
    .await?;

    Ok(row.clone())
  }

  async fn update_mcp_server(&self, row: &McpServerRow) -> Result<McpServerRow, DbError> {
    let enabled_int = if row.enabled { 1 } else { 0 };

    sqlx::query(
      r#"
      UPDATE mcp_servers
      SET url = ?, name = ?, description = ?, enabled = ?, updated_by = ?, updated_at = ?
      WHERE id = ?
      "#,
    )
    .bind(&row.url)
    .bind(&row.name)
    .bind(&row.description)
    .bind(enabled_int)
    .bind(&row.updated_by)
    .bind(row.updated_at)
    .bind(&row.id)
    .execute(&self.pool)
    .await?;

    Ok(row.clone())
  }

  async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServerRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, Option<String>, i64, String, String, i64, i64)>(
      "SELECT id, url, name, description, enabled, created_by, updated_by, created_at, updated_at FROM mcp_servers WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(id, url, name, description, enabled, created_by, updated_by, created_at, updated_at)| {
        McpServerRow {
          id,
          url,
          name,
          description,
          enabled: enabled != 0,
          created_by,
          updated_by,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn get_mcp_server_by_url(&self, url: &str) -> Result<Option<McpServerRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, Option<String>, i64, String, String, i64, i64)>(
      "SELECT id, url, name, description, enabled, created_by, updated_by, created_at, updated_at FROM mcp_servers WHERE url = ? COLLATE NOCASE",
    )
    .bind(url)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(id, url, name, description, enabled, created_by, updated_by, created_at, updated_at)| {
        McpServerRow {
          id,
          url,
          name,
          description,
          enabled: enabled != 0,
          created_by,
          updated_by,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn list_mcp_servers(&self, enabled: Option<bool>) -> Result<Vec<McpServerRow>, DbError> {
    let (sql, bind_enabled) = match enabled {
      Some(e) => (
        "SELECT id, url, name, description, enabled, created_by, updated_by, created_at, updated_at FROM mcp_servers WHERE enabled = ?",
        Some(if e { 1i64 } else { 0i64 }),
      ),
      None => (
        "SELECT id, url, name, description, enabled, created_by, updated_by, created_at, updated_at FROM mcp_servers",
        None,
      ),
    };

    let results = if let Some(en) = bind_enabled {
      sqlx::query_as::<
        _,
        (
          String,
          String,
          String,
          Option<String>,
          i64,
          String,
          String,
          i64,
          i64,
        ),
      >(sql)
      .bind(en)
      .fetch_all(&self.pool)
      .await?
    } else {
      sqlx::query_as::<
        _,
        (
          String,
          String,
          String,
          Option<String>,
          i64,
          String,
          String,
          i64,
          i64,
        ),
      >(sql)
      .fetch_all(&self.pool)
      .await?
    };

    Ok(
      results
        .into_iter()
        .map(
          |(
            id,
            url,
            name,
            description,
            enabled,
            created_by,
            updated_by,
            created_at,
            updated_at,
          )| McpServerRow {
            id,
            url,
            name,
            description,
            enabled: enabled != 0,
            created_by,
            updated_by,
            created_at,
            updated_at,
          },
        )
        .collect(),
    )
  }

  async fn count_mcps_by_server_id(&self, server_id: &str) -> Result<(i64, i64), DbError> {
    let result = sqlx::query_as::<_, (i64, i64)>(
      r#"
      SELECT
        COALESCE(SUM(CASE WHEN enabled = 1 THEN 1 ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN enabled = 0 THEN 1 ELSE 0 END), 0)
      FROM mcps WHERE mcp_server_id = ?
      "#,
    )
    .bind(server_id)
    .fetch_one(&self.pool)
    .await?;

    Ok(result)
  }

  async fn clear_mcp_tools_by_server_id(&self, server_id: &str) -> Result<u64, DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = sqlx::query(
      "UPDATE mcps SET tools_cache = NULL, tools_filter = NULL, updated_at = ? WHERE mcp_server_id = ?",
    )
    .bind(now)
    .bind(server_id)
    .execute(&self.pool)
    .await?;

    Ok(result.rows_affected())
  }

  async fn create_mcp(&self, row: &McpRow) -> Result<McpRow, DbError> {
    let enabled = if row.enabled { 1 } else { 0 };

    sqlx::query(
      r#"
      INSERT INTO mcps (id, created_by, mcp_server_id, name, slug, description, enabled,
                         tools_cache, tools_filter, auth_type, auth_uuid,
                         created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&row.id)
    .bind(&row.created_by)
    .bind(&row.mcp_server_id)
    .bind(&row.name)
    .bind(&row.slug)
    .bind(&row.description)
    .bind(enabled)
    .bind(&row.tools_cache)
    .bind(&row.tools_filter)
    .bind(&row.auth_type)
    .bind(&row.auth_uuid)
    .bind(row.created_at)
    .bind(row.updated_at)
    .execute(&self.pool)
    .await?;

    Ok(row.clone())
  }

  async fn get_mcp(&self, user_id: &str, id: &str) -> Result<Option<McpRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, i64, Option<String>, Option<String>, String, Option<String>, i64, i64)>(
      "SELECT id, created_by, mcp_server_id, name, slug, description, enabled, tools_cache, tools_filter, auth_type, auth_uuid, created_at, updated_at FROM mcps WHERE created_by = ? AND id = ?",
    )
    .bind(user_id)
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        created_by,
        mcp_server_id,
        name,
        slug,
        description,
        enabled,
        tools_cache,
        tools_filter,
        auth_type,
        auth_uuid,
        created_at,
        updated_at,
      )| {
        McpRow {
          id,
          created_by,
          mcp_server_id,
          name,
          slug,
          description,
          enabled: enabled != 0,
          tools_cache,
          tools_filter,
          auth_type,
          auth_uuid,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn get_mcp_by_slug(&self, user_id: &str, slug: &str) -> Result<Option<McpRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, i64, Option<String>, Option<String>, String, Option<String>, i64, i64)>(
      "SELECT id, created_by, mcp_server_id, name, slug, description, enabled, tools_cache, tools_filter, auth_type, auth_uuid, created_at, updated_at FROM mcps WHERE created_by = ? AND slug = ? COLLATE NOCASE",
    )
    .bind(user_id)
    .bind(slug)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        created_by,
        mcp_server_id,
        name,
        slug,
        description,
        enabled,
        tools_cache,
        tools_filter,
        auth_type,
        auth_uuid,
        created_at,
        updated_at,
      )| {
        McpRow {
          id,
          created_by,
          mcp_server_id,
          name,
          slug,
          description,
          enabled: enabled != 0,
          tools_cache,
          tools_filter,
          auth_type,
          auth_uuid,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn list_mcps_with_server(&self, user_id: &str) -> Result<Vec<McpWithServerRow>, DbError> {
    let rows = sqlx::query(
      r#"
      SELECT m.id, m.created_by, m.mcp_server_id, m.name, m.slug, m.description, m.enabled,
             m.tools_cache, m.tools_filter,
             m.auth_type, m.auth_uuid,
             m.created_at, m.updated_at,
             s.url AS server_url, s.name AS server_name, s.enabled AS server_enabled
      FROM mcps m
      INNER JOIN mcp_servers s ON m.mcp_server_id = s.id
      WHERE m.created_by = ?
      "#,
    )
    .bind(user_id)
    .fetch_all(&self.pool)
    .await?;

    use sqlx::Row;
    Ok(
      rows
        .into_iter()
        .map(|row| McpWithServerRow {
          id: row.get("id"),
          created_by: row.get("created_by"),
          mcp_server_id: row.get("mcp_server_id"),
          name: row.get("name"),
          slug: row.get("slug"),
          description: row.get("description"),
          enabled: row.get::<i64, _>("enabled") != 0,
          tools_cache: row.get("tools_cache"),
          tools_filter: row.get("tools_filter"),
          auth_type: row.get("auth_type"),
          auth_uuid: row.get("auth_uuid"),
          created_at: row.get("created_at"),
          updated_at: row.get("updated_at"),
          server_url: row.get("server_url"),
          server_name: row.get("server_name"),
          server_enabled: row.get::<i64, _>("server_enabled") != 0,
        })
        .collect(),
    )
  }

  async fn update_mcp(&self, row: &McpRow) -> Result<McpRow, DbError> {
    let enabled = if row.enabled { 1 } else { 0 };

    sqlx::query(
      r#"
      UPDATE mcps
      SET name = ?, slug = ?, description = ?, enabled = ?, tools_cache = ?, tools_filter = ?,
          auth_type = ?, auth_uuid = ?,
          updated_at = ?
      WHERE created_by = ? AND id = ?
      "#,
    )
    .bind(&row.name)
    .bind(&row.slug)
    .bind(&row.description)
    .bind(enabled)
    .bind(&row.tools_cache)
    .bind(&row.tools_filter)
    .bind(&row.auth_type)
    .bind(&row.auth_uuid)
    .bind(row.updated_at)
    .bind(&row.created_by)
    .bind(&row.id)
    .execute(&self.pool)
    .await?;

    Ok(row.clone())
  }

  async fn delete_mcp(&self, user_id: &str, id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM mcps WHERE created_by = ? AND id = ?")
      .bind(user_id)
      .bind(id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  // ---- MCP Auth Header operations ----

  async fn create_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError> {
    sqlx::query(
      r#"
      INSERT INTO mcp_auth_headers (id, name, mcp_server_id, header_key, encrypted_header_value, header_value_salt, header_value_nonce,
                                     created_by, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&row.id)
    .bind(&row.name)
    .bind(&row.mcp_server_id)
    .bind(&row.header_key)
    .bind(&row.encrypted_header_value)
    .bind(&row.header_value_salt)
    .bind(&row.header_value_nonce)
    .bind(&row.created_by)
    .bind(row.created_at)
    .bind(row.updated_at)
    .execute(&self.pool)
    .await?;

    Ok(row.clone())
  }

  async fn get_mcp_auth_header(&self, id: &str) -> Result<Option<McpAuthHeaderRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String, String, String, String, String, i64, i64)>(
      "SELECT id, name, mcp_server_id, header_key, encrypted_header_value, header_value_salt, header_value_nonce, created_by, created_at, updated_at FROM mcp_auth_headers WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        name,
        mcp_server_id,
        header_key,
        encrypted_header_value,
        header_value_salt,
        header_value_nonce,
        created_by,
        created_at,
        updated_at,
      )| {
        McpAuthHeaderRow {
          id,
          name,
          mcp_server_id,
          header_key,
          encrypted_header_value,
          header_value_salt,
          header_value_nonce,
          created_by,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn update_mcp_auth_header(
    &self,
    row: &McpAuthHeaderRow,
  ) -> Result<McpAuthHeaderRow, DbError> {
    sqlx::query(
      r#"
      UPDATE mcp_auth_headers
      SET name = ?, header_key = ?, encrypted_header_value = ?, header_value_salt = ?, header_value_nonce = ?,
          updated_at = ?
      WHERE id = ?
      "#,
    )
    .bind(&row.name)
    .bind(&row.header_key)
    .bind(&row.encrypted_header_value)
    .bind(&row.header_value_salt)
    .bind(&row.header_value_nonce)
    .bind(row.updated_at)
    .bind(&row.id)
    .execute(&self.pool)
    .await?;

    Ok(row.clone())
  }

  async fn delete_mcp_auth_header(&self, id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM mcp_auth_headers WHERE id = ?")
      .bind(id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn list_mcp_auth_headers_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpAuthHeaderRow>, DbError> {
    let results = sqlx::query_as::<_, (String, String, String, String, String, String, String, String, i64, i64)>(
      "SELECT id, name, mcp_server_id, header_key, encrypted_header_value, header_value_salt, header_value_nonce, created_by, created_at, updated_at FROM mcp_auth_headers WHERE mcp_server_id = ? ORDER BY created_at DESC",
    )
    .bind(mcp_server_id)
    .fetch_all(&self.pool)
    .await?;

    Ok(
      results
        .into_iter()
        .map(
          |(
            id,
            name,
            mcp_server_id,
            header_key,
            encrypted_header_value,
            header_value_salt,
            header_value_nonce,
            created_by,
            created_at,
            updated_at,
          )| {
            McpAuthHeaderRow {
              id,
              name,
              mcp_server_id,
              header_key,
              encrypted_header_value,
              header_value_salt,
              header_value_nonce,
              created_by,
              created_at,
              updated_at,
            }
          },
        )
        .collect(),
    )
  }

  async fn get_decrypted_auth_header(&self, id: &str) -> Result<Option<(String, String)>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String)>(
      "SELECT header_key, encrypted_header_value, header_value_salt, header_value_nonce FROM mcp_auth_headers WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((header_key, encrypted, salt, nonce)) => {
        let value = decrypt_api_key(&self.encryption_key, &encrypted, &salt, &nonce)
          .map_err(|e| DbError::EncryptionError(e.to_string()))?;
        Ok(Some((header_key, value)))
      }
      None => Ok(None),
    }
  }

  // ---- MCP OAuth Config operations ----

  async fn create_mcp_oauth_config(
    &self,
    row: &McpOAuthConfigRow,
  ) -> Result<McpOAuthConfigRow, DbError> {
    sqlx::query(
      r#"
      INSERT INTO mcp_oauth_configs (id, name, mcp_server_id, registration_type, client_id,
                                      encrypted_client_secret, client_secret_salt, client_secret_nonce,
                                      authorization_endpoint, token_endpoint,
                                      registration_endpoint,
                                      encrypted_registration_access_token, registration_access_token_salt, registration_access_token_nonce,
                                      client_id_issued_at, token_endpoint_auth_method,
                                      scopes, created_by, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&row.id)
    .bind(&row.name)
    .bind(&row.mcp_server_id)
    .bind(&row.registration_type)
    .bind(&row.client_id)
    .bind(&row.encrypted_client_secret)
    .bind(&row.client_secret_salt)
    .bind(&row.client_secret_nonce)
    .bind(&row.authorization_endpoint)
    .bind(&row.token_endpoint)
    .bind(&row.registration_endpoint)
    .bind(&row.encrypted_registration_access_token)
    .bind(&row.registration_access_token_salt)
    .bind(&row.registration_access_token_nonce)
    .bind(row.client_id_issued_at)
    .bind(&row.token_endpoint_auth_method)
    .bind(&row.scopes)
    .bind(&row.created_by)
    .bind(row.created_at)
    .bind(row.updated_at)
    .execute(&self.pool)
    .await?;

    Ok(row.clone())
  }

  async fn get_mcp_oauth_config(&self, id: &str) -> Result<Option<McpOAuthConfigRow>, DbError> {
    let row = sqlx::query(
      r#"SELECT id, name, mcp_server_id, registration_type, client_id,
              encrypted_client_secret, client_secret_salt, client_secret_nonce,
              authorization_endpoint, token_endpoint,
              registration_endpoint,
              encrypted_registration_access_token, registration_access_token_salt, registration_access_token_nonce,
              client_id_issued_at, token_endpoint_auth_method,
              scopes, created_by, created_at, updated_at
       FROM mcp_oauth_configs WHERE id = ?"#,
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    use sqlx::Row;
    Ok(row.map(|r| McpOAuthConfigRow {
      id: r.get("id"),
      name: r.get("name"),
      mcp_server_id: r.get("mcp_server_id"),
      registration_type: r.get("registration_type"),
      client_id: r.get("client_id"),
      encrypted_client_secret: r.get("encrypted_client_secret"),
      client_secret_salt: r.get("client_secret_salt"),
      client_secret_nonce: r.get("client_secret_nonce"),
      authorization_endpoint: r.get("authorization_endpoint"),
      token_endpoint: r.get("token_endpoint"),
      registration_endpoint: r.get("registration_endpoint"),
      encrypted_registration_access_token: r.get("encrypted_registration_access_token"),
      registration_access_token_salt: r.get("registration_access_token_salt"),
      registration_access_token_nonce: r.get("registration_access_token_nonce"),
      client_id_issued_at: r.get("client_id_issued_at"),
      token_endpoint_auth_method: r.get("token_endpoint_auth_method"),
      scopes: r.get("scopes"),
      created_by: r.get("created_by"),
      created_at: r.get("created_at"),
      updated_at: r.get("updated_at"),
    }))
  }

  async fn list_mcp_oauth_configs_by_server(
    &self,
    mcp_server_id: &str,
  ) -> Result<Vec<McpOAuthConfigRow>, DbError> {
    let rows = sqlx::query(
      r#"SELECT id, name, mcp_server_id, registration_type, client_id,
              encrypted_client_secret, client_secret_salt, client_secret_nonce,
              authorization_endpoint, token_endpoint,
              registration_endpoint,
              encrypted_registration_access_token, registration_access_token_salt, registration_access_token_nonce,
              client_id_issued_at, token_endpoint_auth_method,
              scopes, created_by, created_at, updated_at
       FROM mcp_oauth_configs WHERE mcp_server_id = ? ORDER BY created_at DESC"#,
    )
    .bind(mcp_server_id)
    .fetch_all(&self.pool)
    .await?;

    use sqlx::Row;
    Ok(
      rows
        .into_iter()
        .map(|r| McpOAuthConfigRow {
          id: r.get("id"),
          name: r.get("name"),
          mcp_server_id: r.get("mcp_server_id"),
          registration_type: r.get("registration_type"),
          client_id: r.get("client_id"),
          encrypted_client_secret: r.get("encrypted_client_secret"),
          client_secret_salt: r.get("client_secret_salt"),
          client_secret_nonce: r.get("client_secret_nonce"),
          authorization_endpoint: r.get("authorization_endpoint"),
          token_endpoint: r.get("token_endpoint"),
          registration_endpoint: r.get("registration_endpoint"),
          encrypted_registration_access_token: r.get("encrypted_registration_access_token"),
          registration_access_token_salt: r.get("registration_access_token_salt"),
          registration_access_token_nonce: r.get("registration_access_token_nonce"),
          client_id_issued_at: r.get("client_id_issued_at"),
          token_endpoint_auth_method: r.get("token_endpoint_auth_method"),
          scopes: r.get("scopes"),
          created_by: r.get("created_by"),
          created_at: r.get("created_at"),
          updated_at: r.get("updated_at"),
        })
        .collect(),
    )
  }

  async fn delete_mcp_oauth_config(&self, id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM mcp_oauth_configs WHERE id = ?")
      .bind(id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn delete_oauth_config_cascade(&self, config_id: &str) -> Result<(), DbError> {
    let mut tx = self.pool.begin().await?;

    sqlx::query("DELETE FROM mcp_oauth_tokens WHERE mcp_oauth_config_id = ?")
      .bind(config_id)
      .execute(&mut *tx)
      .await?;

    sqlx::query("DELETE FROM mcp_oauth_configs WHERE id = ?")
      .bind(config_id)
      .execute(&mut *tx)
      .await?;

    tx.commit().await?;
    Ok(())
  }

  async fn get_decrypted_client_secret(
    &self,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    let row = self.get_mcp_oauth_config(id).await?;
    match row {
      Some(r) => {
        if let (Some(ref enc), Some(ref salt), Some(ref nonce)) = (
          &r.encrypted_client_secret,
          &r.client_secret_salt,
          &r.client_secret_nonce,
        ) {
          let secret = decrypt_api_key(&self.encryption_key, enc, salt, nonce)
            .map_err(|e| DbError::EncryptionError(e.to_string()))?;
          Ok(Some((r.client_id, secret)))
        } else {
          // Config exists but has no client secret stored
          Ok(None)
        }
      }
      // Config not found - return error instead of None to disambiguate
      None => Err(DbError::ItemNotFound {
        id: id.to_string(),
        item_type: "mcp_oauth_config".to_string(),
      }),
    }
  }

  // ---- MCP OAuth Token operations ----

  async fn create_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenRow,
  ) -> Result<McpOAuthTokenRow, DbError> {
    sqlx::query(
      r#"
      INSERT INTO mcp_oauth_tokens (id, mcp_oauth_config_id,
                                     encrypted_access_token, access_token_salt, access_token_nonce,
                                     encrypted_refresh_token, refresh_token_salt, refresh_token_nonce,
                                     scopes_granted, expires_at,
                                     created_by, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(&row.id)
    .bind(&row.mcp_oauth_config_id)
    .bind(&row.encrypted_access_token)
    .bind(&row.access_token_salt)
    .bind(&row.access_token_nonce)
    .bind(&row.encrypted_refresh_token)
    .bind(&row.refresh_token_salt)
    .bind(&row.refresh_token_nonce)
    .bind(&row.scopes_granted)
    .bind(row.expires_at)
    .bind(&row.created_by)
    .bind(row.created_at)
    .bind(row.updated_at)
    .execute(&self.pool)
    .await?;

    Ok(row.clone())
  }

  async fn get_mcp_oauth_token(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<McpOAuthTokenRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<i64>, String, i64, i64)>(
      "SELECT id, mcp_oauth_config_id, encrypted_access_token, access_token_salt, access_token_nonce, encrypted_refresh_token, refresh_token_salt, refresh_token_nonce, scopes_granted, expires_at, created_by, created_at, updated_at FROM mcp_oauth_tokens WHERE id = ? AND created_by = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        mcp_oauth_config_id,
        encrypted_access_token,
        access_token_salt,
        access_token_nonce,
        encrypted_refresh_token,
        refresh_token_salt,
        refresh_token_nonce,
        scopes_granted,
        expires_at,
        created_by,
        created_at,
        updated_at,
      )| {
        McpOAuthTokenRow {
          id,
          mcp_oauth_config_id,
          encrypted_access_token,
          access_token_salt,
          access_token_nonce,
          encrypted_refresh_token,
          refresh_token_salt,
          refresh_token_nonce,
          scopes_granted,
          expires_at,
          created_by,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn get_latest_oauth_token_by_config(
    &self,
    config_id: &str,
  ) -> Result<Option<McpOAuthTokenRow>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<i64>, String, i64, i64)>(
      "SELECT id, mcp_oauth_config_id, encrypted_access_token, access_token_salt, access_token_nonce, encrypted_refresh_token, refresh_token_salt, refresh_token_nonce, scopes_granted, expires_at, created_by, created_at, updated_at FROM mcp_oauth_tokens WHERE mcp_oauth_config_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(config_id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(
      |(
        id,
        mcp_oauth_config_id,
        encrypted_access_token,
        access_token_salt,
        access_token_nonce,
        encrypted_refresh_token,
        refresh_token_salt,
        refresh_token_nonce,
        scopes_granted,
        expires_at,
        created_by,
        created_at,
        updated_at,
      )| {
        McpOAuthTokenRow {
          id,
          mcp_oauth_config_id,
          encrypted_access_token,
          access_token_salt,
          access_token_nonce,
          encrypted_refresh_token,
          refresh_token_salt,
          refresh_token_nonce,
          scopes_granted,
          expires_at,
          created_by,
          created_at,
          updated_at,
        }
      },
    ))
  }

  async fn update_mcp_oauth_token(
    &self,
    row: &McpOAuthTokenRow,
  ) -> Result<McpOAuthTokenRow, DbError> {
    sqlx::query(
      r#"
      UPDATE mcp_oauth_tokens
      SET encrypted_access_token = ?, access_token_salt = ?, access_token_nonce = ?,
          encrypted_refresh_token = ?, refresh_token_salt = ?, refresh_token_nonce = ?,
          scopes_granted = ?, expires_at = ?,
          updated_at = ?
      WHERE id = ?
      "#,
    )
    .bind(&row.encrypted_access_token)
    .bind(&row.access_token_salt)
    .bind(&row.access_token_nonce)
    .bind(&row.encrypted_refresh_token)
    .bind(&row.refresh_token_salt)
    .bind(&row.refresh_token_nonce)
    .bind(&row.scopes_granted)
    .bind(row.expires_at)
    .bind(row.updated_at)
    .bind(&row.id)
    .execute(&self.pool)
    .await?;

    Ok(row.clone())
  }

  async fn delete_mcp_oauth_token(&self, user_id: &str, id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM mcp_oauth_tokens WHERE id = ? AND created_by = ?")
      .bind(id)
      .bind(user_id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn delete_oauth_tokens_by_config(&self, config_id: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM mcp_oauth_tokens WHERE mcp_oauth_config_id = ?")
      .bind(config_id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn delete_oauth_tokens_by_config_and_user(
    &self,
    config_id: &str,
    user_id: &str,
  ) -> Result<(), DbError> {
    sqlx::query("DELETE FROM mcp_oauth_tokens WHERE mcp_oauth_config_id = ? AND created_by = ?")
      .bind(config_id)
      .bind(user_id)
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  async fn get_decrypted_oauth_bearer(
    &self,
    id: &str,
  ) -> Result<Option<(String, String)>, DbError> {
    let result = sqlx::query_as::<_, (String, String, String)>(
      "SELECT encrypted_access_token, access_token_salt, access_token_nonce FROM mcp_oauth_tokens WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    match result {
      Some((encrypted, salt, nonce)) => {
        let token = decrypt_api_key(&self.encryption_key, &encrypted, &salt, &nonce)
          .map_err(|e| DbError::EncryptionError(e.to_string()))?;
        Ok(Some((
          "Authorization".to_string(),
          format!("Bearer {}", token),
        )))
      }
      None => Ok(None),
    }
  }
}
