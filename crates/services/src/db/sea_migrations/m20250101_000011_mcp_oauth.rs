use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum McpServers {
  Table,
  Id,
}

// NOTE: Use `McpOauthConfigs` (not `McpOAuthConfigs`) so DeriveIden produces
// `mcp_oauth_configs` instead of `mcp_o_auth_configs`.
#[derive(DeriveIden)]
enum McpOauthConfigs {
  Table,
  Id,
  Name,
  McpServerId,
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
  CreatedBy,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum McpOauthTokens {
  Table,
  Id,
  McpOauthConfigId,
  EncryptedAccessToken,
  AccessTokenSalt,
  AccessTokenNonce,
  EncryptedRefreshToken,
  RefreshTokenSalt,
  RefreshTokenNonce,
  ScopesGranted,
  ExpiresAt,
  CreatedBy,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(McpOauthConfigs::Table)
          .col(string(McpOauthConfigs::Id).primary_key())
          .col(string(McpOauthConfigs::Name).default("OAuth"))
          .col(string(McpOauthConfigs::McpServerId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_mcp_oauth_configs_mcp_server_id")
              .from(McpOauthConfigs::Table, McpOauthConfigs::McpServerId)
              .to(McpServers::Table, McpServers::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(McpOauthConfigs::RegistrationType).default("pre_registered"))
          .col(string(McpOauthConfigs::ClientId))
          .col(string_null(McpOauthConfigs::EncryptedClientSecret))
          .col(string_null(McpOauthConfigs::ClientSecretSalt))
          .col(string_null(McpOauthConfigs::ClientSecretNonce))
          .col(string(McpOauthConfigs::AuthorizationEndpoint))
          .col(string(McpOauthConfigs::TokenEndpoint))
          .col(string_null(McpOauthConfigs::RegistrationEndpoint))
          .col(string_null(
            McpOauthConfigs::EncryptedRegistrationAccessToken,
          ))
          .col(string_null(McpOauthConfigs::RegistrationAccessTokenSalt))
          .col(string_null(McpOauthConfigs::RegistrationAccessTokenNonce))
          .col(timestamp_with_time_zone_null(
            McpOauthConfigs::ClientIdIssuedAt,
          ))
          .col(string_null(McpOauthConfigs::TokenEndpointAuthMethod))
          .col(string_null(McpOauthConfigs::Scopes))
          .col(string(McpOauthConfigs::CreatedBy))
          .col(timestamp_with_time_zone(McpOauthConfigs::CreatedAt))
          .col(timestamp_with_time_zone(McpOauthConfigs::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(McpOauthTokens::Table)
          .col(string(McpOauthTokens::Id).primary_key())
          .col(string(McpOauthTokens::McpOauthConfigId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_mcp_oauth_tokens_mcp_oauth_config_id")
              .from(McpOauthTokens::Table, McpOauthTokens::McpOauthConfigId)
              .to(McpOauthConfigs::Table, McpOauthConfigs::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(McpOauthTokens::EncryptedAccessToken))
          .col(string(McpOauthTokens::AccessTokenSalt))
          .col(string(McpOauthTokens::AccessTokenNonce))
          .col(string_null(McpOauthTokens::EncryptedRefreshToken))
          .col(string_null(McpOauthTokens::RefreshTokenSalt))
          .col(string_null(McpOauthTokens::RefreshTokenNonce))
          .col(string_null(McpOauthTokens::ScopesGranted))
          .col(timestamp_with_time_zone_null(McpOauthTokens::ExpiresAt))
          .col(string(McpOauthTokens::CreatedBy))
          .col(timestamp_with_time_zone(McpOauthTokens::CreatedAt))
          .col(timestamp_with_time_zone(McpOauthTokens::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_oauth_configs_mcp_server_id")
          .table(McpOauthConfigs::Table)
          .col(McpOauthConfigs::McpServerId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_oauth_tokens_mcp_oauth_config_id")
          .table(McpOauthTokens::Table)
          .col(McpOauthTokens::McpOauthConfigId)
          .to_owned(),
      )
      .await?;

    let db = manager.get_connection();
    let backend = db.get_database_backend();
    match backend {
      sea_orm::DatabaseBackend::Sqlite => {
        db.execute_unprepared(
          "CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_oauth_configs_server_name_unique \
           ON mcp_oauth_configs(mcp_server_id COLLATE NOCASE, name COLLATE NOCASE)",
        )
        .await?;
      }
      sea_orm::DatabaseBackend::Postgres => {
        db.execute_unprepared(
          "CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_oauth_configs_server_name_unique \
           ON mcp_oauth_configs(LOWER(mcp_server_id), LOWER(name))",
        )
        .await?;
      }
      _ => {}
    }

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(McpOauthTokens::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(McpOauthConfigs::Table).to_owned())
      .await
  }
}
