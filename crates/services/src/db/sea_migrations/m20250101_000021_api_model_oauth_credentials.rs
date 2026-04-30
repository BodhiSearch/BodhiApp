use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ApiModelAliases {
  Table,
  Id,
}

// NOTE: Avoid camel humps in enum names so DeriveIden produces snake_case correctly.
#[derive(DeriveIden)]
enum ApiModelOauthCredentials {
  Table,
  ApiAliasId,
  TenantId,
  UserId,
  EnvelopeVersion,
  Provider,
  EncryptedAccessToken,
  AccessSalt,
  AccessNonce,
  EncryptedRefreshToken,
  RefreshSalt,
  RefreshNonce,
  ExpiresAt,
  AuthIn,
  AuthKey,
  AuthScheme,
  OauthAuthorizeUrl,
  OauthTokenUrl,
  OauthRevokeUrl,
  OauthClientId,
  OauthClientSecret,
  ApiBaseUrl,
  ApiChatUrl,
  ApiModelsUrl,
  HeadersJson,
  BodyJson,
  ExtraJson,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(ApiModelOauthCredentials::Table)
          .col(string(ApiModelOauthCredentials::ApiAliasId).primary_key())
          .foreign_key(
            ForeignKey::create()
              .name("fk_api_model_oauth_creds_alias_id")
              .from(
                ApiModelOauthCredentials::Table,
                ApiModelOauthCredentials::ApiAliasId,
              )
              .to(ApiModelAliases::Table, ApiModelAliases::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(ApiModelOauthCredentials::TenantId))
          .col(string(ApiModelOauthCredentials::UserId))
          .col(string(ApiModelOauthCredentials::EnvelopeVersion))
          .col(string(ApiModelOauthCredentials::Provider))
          .col(string(ApiModelOauthCredentials::EncryptedAccessToken))
          .col(string(ApiModelOauthCredentials::AccessSalt))
          .col(string(ApiModelOauthCredentials::AccessNonce))
          .col(string(ApiModelOauthCredentials::EncryptedRefreshToken))
          .col(string(ApiModelOauthCredentials::RefreshSalt))
          .col(string(ApiModelOauthCredentials::RefreshNonce))
          .col(timestamp_with_time_zone(
            ApiModelOauthCredentials::ExpiresAt,
          ))
          .col(string(ApiModelOauthCredentials::AuthIn))
          .col(string(ApiModelOauthCredentials::AuthKey))
          .col(string(ApiModelOauthCredentials::AuthScheme))
          .col(string(ApiModelOauthCredentials::OauthAuthorizeUrl))
          .col(string(ApiModelOauthCredentials::OauthTokenUrl))
          .col(string_null(ApiModelOauthCredentials::OauthRevokeUrl))
          .col(string(ApiModelOauthCredentials::OauthClientId))
          .col(string_null(ApiModelOauthCredentials::OauthClientSecret))
          .col(string(ApiModelOauthCredentials::ApiBaseUrl))
          .col(string(ApiModelOauthCredentials::ApiChatUrl))
          .col(string_null(ApiModelOauthCredentials::ApiModelsUrl))
          .col(json_binary(ApiModelOauthCredentials::HeadersJson).default("{}"))
          .col(json_binary(ApiModelOauthCredentials::BodyJson).default("{}"))
          .col(json_binary_null(ApiModelOauthCredentials::ExtraJson))
          .col(timestamp_with_time_zone(
            ApiModelOauthCredentials::CreatedAt,
          ))
          .col(timestamp_with_time_zone(
            ApiModelOauthCredentials::UpdatedAt,
          ))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_api_model_oauth_creds_provider")
          .table(ApiModelOauthCredentials::Table)
          .col(ApiModelOauthCredentials::Provider)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_api_model_oauth_creds_tenant_id")
          .table(ApiModelOauthCredentials::Table)
          .col(ApiModelOauthCredentials::TenantId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_api_model_oauth_creds_expires_at")
          .table(ApiModelOauthCredentials::Table)
          .col(ApiModelOauthCredentials::ExpiresAt)
          .to_owned(),
      )
      .await?;

    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("ALTER TABLE api_model_oauth_credentials ENABLE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE api_model_oauth_credentials FORCE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY tenant_isolation ON api_model_oauth_credentials \
           FOR ALL \
           USING (tenant_id = (SELECT current_tenant_id())) \
           WITH CHECK (tenant_id = (SELECT current_tenant_id()));",
        )
        .await?;
    }

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared(
          "DROP POLICY IF EXISTS tenant_isolation ON api_model_oauth_credentials;",
        )
        .await?;
      conn
        .execute_unprepared("ALTER TABLE api_model_oauth_credentials DISABLE ROW LEVEL SECURITY;")
        .await?;
    }
    manager
      .drop_table(
        Table::drop()
          .table(ApiModelOauthCredentials::Table)
          .to_owned(),
      )
      .await
  }
}
