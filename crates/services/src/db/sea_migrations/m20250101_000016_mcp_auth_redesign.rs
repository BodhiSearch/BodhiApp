use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

// Existing tables referenced by FKs
#[derive(DeriveIden)]
enum McpServers {
  Table,
  Id,
}

#[derive(DeriveIden)]
enum Mcps {
  Table,
  Id,
  AuthUuid,
  AuthConfigId,
}

// mcp_oauth_tokens: dropped first, then recreated with new schema
#[derive(DeriveIden)]
enum McpOauthTokens {
  Table,
  Id,
  TenantId,
  McpId,
  AuthConfigId,
  UserId,
  EncryptedAccessToken,
  AccessTokenSalt,
  AccessTokenNonce,
  EncryptedRefreshToken,
  RefreshTokenSalt,
  RefreshTokenNonce,
  ScopesGranted,
  ExpiresAt,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum McpOauthConfigs {
  Table,
}

#[derive(DeriveIden)]
enum McpAuthHeaders {
  Table,
}

// New tables
#[derive(DeriveIden)]
enum McpAuthConfigs {
  Table,
  Id,
  TenantId,
  McpServerId,
  ConfigType,
  Name,
  CreatedBy,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum McpAuthConfigParams {
  Table,
  Id,
  TenantId,
  AuthConfigId,
  ParamType,
  ParamKey,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum McpOauthConfigDetails {
  Table,
  AuthConfigId,
  TenantId,
  RegistrationType,
  ClientId,
  EncryptedClientSecret,
  ClientSecretSalt,
  ClientSecretNonce,
  AuthorizationEndpoint,
  TokenEndpoint,
  RegistrationEndpoint,
  EncryptedRegistrationAccessToken,
  RegistrationAccessTokenSalt,
  RegistrationAccessTokenNonce,
  ClientIdIssuedAt,
  TokenEndpointAuthMethod,
  Scopes,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum McpAuthParams {
  Table,
  Id,
  TenantId,
  McpId,
  ParamType,
  ParamKey,
  EncryptedValue,
  ValueSalt,
  ValueNonce,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // =========================================================
    // 1. Drop old tables (FK order: tokens -> configs -> headers)
    // =========================================================
    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      for table in &["mcp_oauth_tokens", "mcp_oauth_configs", "mcp_auth_headers"] {
        conn
          .execute_unprepared(&format!(
            "DROP POLICY IF EXISTS tenant_isolation ON {table};"
          ))
          .await?;
      }
    }

    manager
      .drop_table(
        Table::drop()
          .table(McpOauthTokens::Table)
          .if_exists()
          .to_owned(),
      )
      .await?;
    manager
      .drop_table(
        Table::drop()
          .table(McpOauthConfigs::Table)
          .if_exists()
          .to_owned(),
      )
      .await?;
    manager
      .drop_table(
        Table::drop()
          .table(McpAuthHeaders::Table)
          .if_exists()
          .to_owned(),
      )
      .await?;

    // =========================================================
    // 2. Create mcp_auth_configs base table
    // =========================================================
    manager
      .create_table(
        Table::create()
          .table(McpAuthConfigs::Table)
          .col(string(McpAuthConfigs::Id).primary_key())
          .col(string(McpAuthConfigs::TenantId))
          .col(string(McpAuthConfigs::McpServerId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_mcp_auth_configs_mcp_server_id")
              .from(McpAuthConfigs::Table, McpAuthConfigs::McpServerId)
              .to(McpServers::Table, McpServers::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(McpAuthConfigs::ConfigType))
          .col(string(McpAuthConfigs::Name))
          .col(string(McpAuthConfigs::CreatedBy))
          .col(timestamp_with_time_zone(McpAuthConfigs::CreatedAt))
          .col(timestamp_with_time_zone(McpAuthConfigs::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_auth_configs_mcp_server_id")
          .table(McpAuthConfigs::Table)
          .col(McpAuthConfigs::McpServerId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_auth_configs_tenant_id")
          .table(McpAuthConfigs::Table)
          .col(McpAuthConfigs::TenantId)
          .to_owned(),
      )
      .await?;

    // =========================================================
    // 3. Create mcp_auth_config_params table
    // =========================================================
    manager
      .create_table(
        Table::create()
          .table(McpAuthConfigParams::Table)
          .col(string(McpAuthConfigParams::Id).primary_key())
          .col(string(McpAuthConfigParams::TenantId))
          .col(string(McpAuthConfigParams::AuthConfigId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_mcp_auth_config_params_auth_config_id")
              .from(
                McpAuthConfigParams::Table,
                McpAuthConfigParams::AuthConfigId,
              )
              .to(McpAuthConfigs::Table, McpAuthConfigs::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(McpAuthConfigParams::ParamType))
          .col(string(McpAuthConfigParams::ParamKey))
          .col(timestamp_with_time_zone(McpAuthConfigParams::CreatedAt))
          .col(timestamp_with_time_zone(McpAuthConfigParams::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_auth_config_params_auth_config_id")
          .table(McpAuthConfigParams::Table)
          .col(McpAuthConfigParams::AuthConfigId)
          .to_owned(),
      )
      .await?;

    // Unique constraint: (tenant_id, auth_config_id, param_type, param_key)
    manager
      .create_index(
        Index::create()
          .name("idx_mcp_auth_config_params_unique")
          .table(McpAuthConfigParams::Table)
          .col(McpAuthConfigParams::TenantId)
          .col(McpAuthConfigParams::AuthConfigId)
          .col(McpAuthConfigParams::ParamType)
          .col(McpAuthConfigParams::ParamKey)
          .unique()
          .to_owned(),
      )
      .await?;

    // =========================================================
    // 4. Create mcp_oauth_config_details table (1:1 with mcp_auth_configs)
    // =========================================================
    manager
      .create_table(
        Table::create()
          .table(McpOauthConfigDetails::Table)
          .col(string(McpOauthConfigDetails::AuthConfigId).primary_key())
          .foreign_key(
            ForeignKey::create()
              .name("fk_mcp_oauth_config_details_auth_config_id")
              .from(
                McpOauthConfigDetails::Table,
                McpOauthConfigDetails::AuthConfigId,
              )
              .to(McpAuthConfigs::Table, McpAuthConfigs::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(McpOauthConfigDetails::TenantId))
          .col(string(McpOauthConfigDetails::RegistrationType).default("pre_registered"))
          .col(string(McpOauthConfigDetails::ClientId))
          .col(string_null(McpOauthConfigDetails::EncryptedClientSecret))
          .col(string_null(McpOauthConfigDetails::ClientSecretSalt))
          .col(string_null(McpOauthConfigDetails::ClientSecretNonce))
          .col(string(McpOauthConfigDetails::AuthorizationEndpoint))
          .col(string(McpOauthConfigDetails::TokenEndpoint))
          .col(string_null(McpOauthConfigDetails::RegistrationEndpoint))
          .col(string_null(
            McpOauthConfigDetails::EncryptedRegistrationAccessToken,
          ))
          .col(string_null(
            McpOauthConfigDetails::RegistrationAccessTokenSalt,
          ))
          .col(string_null(
            McpOauthConfigDetails::RegistrationAccessTokenNonce,
          ))
          .col(timestamp_with_time_zone_null(
            McpOauthConfigDetails::ClientIdIssuedAt,
          ))
          .col(string_null(McpOauthConfigDetails::TokenEndpointAuthMethod))
          .col(string_null(McpOauthConfigDetails::Scopes))
          .col(timestamp_with_time_zone(McpOauthConfigDetails::CreatedAt))
          .col(timestamp_with_time_zone(McpOauthConfigDetails::UpdatedAt))
          .to_owned(),
      )
      .await?;

    // =========================================================
    // 5. Create mcp_auth_params table (instance-level header/query auth)
    // =========================================================
    manager
      .create_table(
        Table::create()
          .table(McpAuthParams::Table)
          .col(string(McpAuthParams::Id).primary_key())
          .col(string(McpAuthParams::TenantId))
          .col(string(McpAuthParams::McpId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_mcp_auth_params_mcp_id")
              .from(McpAuthParams::Table, McpAuthParams::McpId)
              .to(Mcps::Table, Mcps::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(McpAuthParams::ParamType))
          .col(string(McpAuthParams::ParamKey))
          .col(string(McpAuthParams::EncryptedValue))
          .col(string(McpAuthParams::ValueSalt))
          .col(string(McpAuthParams::ValueNonce))
          .col(timestamp_with_time_zone(McpAuthParams::CreatedAt))
          .col(timestamp_with_time_zone(McpAuthParams::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_auth_params_mcp_id")
          .table(McpAuthParams::Table)
          .col(McpAuthParams::McpId)
          .to_owned(),
      )
      .await?;

    // Unique constraint: (tenant_id, mcp_id, param_type, param_key)
    manager
      .create_index(
        Index::create()
          .name("idx_mcp_auth_params_unique")
          .table(McpAuthParams::Table)
          .col(McpAuthParams::TenantId)
          .col(McpAuthParams::McpId)
          .col(McpAuthParams::ParamType)
          .col(McpAuthParams::ParamKey)
          .unique()
          .to_owned(),
      )
      .await?;

    // =========================================================
    // 6. Recreate mcp_oauth_tokens table (with access_token, nullable mcp_id)
    // =========================================================
    manager
      .create_table(
        Table::create()
          .table(McpOauthTokens::Table)
          .col(string(McpOauthTokens::Id).primary_key())
          .col(string(McpOauthTokens::TenantId))
          .col(string_null(McpOauthTokens::McpId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_mcp_oauth_tokens_mcp_id")
              .from(McpOauthTokens::Table, McpOauthTokens::McpId)
              .to(Mcps::Table, Mcps::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(McpOauthTokens::AuthConfigId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_mcp_oauth_tokens_auth_config_id")
              .from(McpOauthTokens::Table, McpOauthTokens::AuthConfigId)
              .to(McpAuthConfigs::Table, McpAuthConfigs::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(McpOauthTokens::UserId))
          .col(string(McpOauthTokens::EncryptedAccessToken))
          .col(string(McpOauthTokens::AccessTokenSalt))
          .col(string(McpOauthTokens::AccessTokenNonce))
          .col(string_null(McpOauthTokens::EncryptedRefreshToken))
          .col(string_null(McpOauthTokens::RefreshTokenSalt))
          .col(string_null(McpOauthTokens::RefreshTokenNonce))
          .col(string_null(McpOauthTokens::ScopesGranted))
          .col(timestamp_with_time_zone_null(McpOauthTokens::ExpiresAt))
          .col(timestamp_with_time_zone(McpOauthTokens::CreatedAt))
          .col(timestamp_with_time_zone(McpOauthTokens::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_oauth_tokens_mcp_id")
          .table(McpOauthTokens::Table)
          .col(McpOauthTokens::McpId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_oauth_tokens_auth_config_id")
          .table(McpOauthTokens::Table)
          .col(McpOauthTokens::AuthConfigId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_oauth_tokens_tenant_id")
          .table(McpOauthTokens::Table)
          .col(McpOauthTokens::TenantId)
          .to_owned(),
      )
      .await?;

    // =========================================================
    // 7. Alter mcps table: drop auth_uuid, add auth_config_id
    // =========================================================
    // SQLite doesn't support multiple alter options in one statement
    manager
      .alter_table(
        Table::alter()
          .table(Mcps::Table)
          .drop_column(Mcps::AuthUuid)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(Mcps::Table)
          .add_column(string_null(Mcps::AuthConfigId))
          .to_owned(),
      )
      .await?;

    // SQLite doesn't support ADD FOREIGN KEY via ALTER TABLE, so only add FK for Postgres
    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared(
          "ALTER TABLE mcps ADD CONSTRAINT fk_mcps_auth_config_id \
           FOREIGN KEY (auth_config_id) REFERENCES mcp_auth_configs(id) \
           ON DELETE SET NULL ON UPDATE CASCADE",
        )
        .await?;
    }

    // =========================================================
    // 8. Case-insensitive unique indexes
    // =========================================================
    let db = manager.get_connection();
    let backend = db.get_database_backend();
    match backend {
      sea_orm::DatabaseBackend::Sqlite => {
        db.execute_unprepared(
          "CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_auth_configs_server_name_unique \
           ON mcp_auth_configs(mcp_server_id COLLATE NOCASE, name COLLATE NOCASE)",
        )
        .await?;
      }
      sea_orm::DatabaseBackend::Postgres => {
        db.execute_unprepared(
          "CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_auth_configs_server_name_unique \
           ON mcp_auth_configs(LOWER(mcp_server_id), LOWER(name))",
        )
        .await?;
      }
      _ => {}
    }

    // =========================================================
    // 9. RLS policies for PostgreSQL
    // =========================================================
    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      for table in &[
        "mcp_auth_configs",
        "mcp_auth_config_params",
        "mcp_oauth_config_details",
        "mcp_auth_params",
        "mcp_oauth_tokens",
      ] {
        conn
          .execute_unprepared(&format!("ALTER TABLE {table} ENABLE ROW LEVEL SECURITY;"))
          .await?;
        conn
          .execute_unprepared(&format!("ALTER TABLE {table} FORCE ROW LEVEL SECURITY;"))
          .await?;
        conn
          .execute_unprepared(&format!(
            "CREATE POLICY tenant_isolation ON {table}
               FOR ALL
               USING (tenant_id = (SELECT current_tenant_id()))
               WITH CHECK (tenant_id = (SELECT current_tenant_id()));"
          ))
          .await?;
      }
    }

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // =========================================================
    // Reverse: Drop new tables, recreate old tables
    // =========================================================
    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      for table in &[
        "mcp_oauth_tokens",
        "mcp_auth_params",
        "mcp_oauth_config_details",
        "mcp_auth_config_params",
        "mcp_auth_configs",
      ] {
        conn
          .execute_unprepared(&format!(
            "DROP POLICY IF EXISTS tenant_isolation ON {table};"
          ))
          .await?;
      }
    }

    // Drop FK on mcps.auth_config_id first, then drop column
    manager
      .alter_table(
        Table::alter()
          .table(Mcps::Table)
          .drop_column(Mcps::AuthConfigId)
          .to_owned(),
      )
      .await?;

    // Add back auth_uuid column
    manager
      .alter_table(
        Table::alter()
          .table(Mcps::Table)
          .add_column(string_null(Mcps::AuthUuid))
          .to_owned(),
      )
      .await?;

    // Drop new tables in correct FK order
    manager
      .drop_table(
        Table::drop()
          .table(McpOauthTokens::Table)
          .if_exists()
          .to_owned(),
      )
      .await?;
    manager
      .drop_table(
        Table::drop()
          .table(McpAuthParams::Table)
          .if_exists()
          .to_owned(),
      )
      .await?;
    manager
      .drop_table(
        Table::drop()
          .table(McpOauthConfigDetails::Table)
          .if_exists()
          .to_owned(),
      )
      .await?;
    manager
      .drop_table(
        Table::drop()
          .table(McpAuthConfigParams::Table)
          .if_exists()
          .to_owned(),
      )
      .await?;
    manager
      .drop_table(
        Table::drop()
          .table(McpAuthConfigs::Table)
          .if_exists()
          .to_owned(),
      )
      .await?;

    // Recreate old tables (simplified - just structure, no data migration)
    // mcp_auth_headers
    manager
      .create_table(
        Table::create()
          .table(McpAuthHeaders::Table)
          .col(ColumnDef::new(Alias::new("id")).string().primary_key())
          .col(ColumnDef::new(Alias::new("tenant_id")).string().not_null())
          .col(
            ColumnDef::new(Alias::new("name"))
              .string()
              .not_null()
              .default("Header"),
          )
          .col(
            ColumnDef::new(Alias::new("mcp_server_id"))
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(Alias::new("header_key")).string().not_null())
          .col(
            ColumnDef::new(Alias::new("encrypted_header_value"))
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("header_value_salt"))
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("header_value_nonce"))
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("created_at"))
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("updated_at"))
              .timestamp_with_time_zone()
              .not_null(),
          )
          .to_owned(),
      )
      .await?;

    // mcp_oauth_configs (old structure)
    manager
      .create_table(
        Table::create()
          .table(McpOauthConfigs::Table)
          .col(ColumnDef::new(Alias::new("id")).string().primary_key())
          .col(ColumnDef::new(Alias::new("tenant_id")).string().not_null())
          .col(
            ColumnDef::new(Alias::new("name"))
              .string()
              .not_null()
              .default("OAuth"),
          )
          .col(
            ColumnDef::new(Alias::new("mcp_server_id"))
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("registration_type"))
              .string()
              .not_null()
              .default("pre_registered"),
          )
          .col(ColumnDef::new(Alias::new("client_id")).string().not_null())
          .col(ColumnDef::new(Alias::new("encrypted_client_secret")).string())
          .col(ColumnDef::new(Alias::new("client_secret_salt")).string())
          .col(ColumnDef::new(Alias::new("client_secret_nonce")).string())
          .col(
            ColumnDef::new(Alias::new("authorization_endpoint"))
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("token_endpoint"))
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(Alias::new("registration_endpoint")).string())
          .col(ColumnDef::new(Alias::new("encrypted_registration_access_token")).string())
          .col(ColumnDef::new(Alias::new("registration_access_token_salt")).string())
          .col(ColumnDef::new(Alias::new("registration_access_token_nonce")).string())
          .col(ColumnDef::new(Alias::new("client_id_issued_at")).timestamp_with_time_zone())
          .col(ColumnDef::new(Alias::new("token_endpoint_auth_method")).string())
          .col(ColumnDef::new(Alias::new("scopes")).string())
          .col(
            ColumnDef::new(Alias::new("created_at"))
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("updated_at"))
              .timestamp_with_time_zone()
              .not_null(),
          )
          .to_owned(),
      )
      .await?;

    // mcp_oauth_tokens (old structure)
    manager
      .create_table(
        Table::create()
          .table(McpOauthTokens::Table)
          .col(ColumnDef::new(Alias::new("id")).string().primary_key())
          .col(ColumnDef::new(Alias::new("tenant_id")).string().not_null())
          .col(
            ColumnDef::new(Alias::new("mcp_oauth_config_id"))
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("encrypted_access_token"))
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("access_token_salt"))
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("access_token_nonce"))
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(Alias::new("encrypted_refresh_token")).string())
          .col(ColumnDef::new(Alias::new("refresh_token_salt")).string())
          .col(ColumnDef::new(Alias::new("refresh_token_nonce")).string())
          .col(ColumnDef::new(Alias::new("scopes_granted")).string())
          .col(ColumnDef::new(Alias::new("expires_at")).timestamp_with_time_zone())
          .col(ColumnDef::new(Alias::new("user_id")).string().not_null())
          .col(
            ColumnDef::new(Alias::new("created_at"))
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(Alias::new("updated_at"))
              .timestamp_with_time_zone()
              .not_null(),
          )
          .to_owned(),
      )
      .await?;

    Ok(())
  }
}
