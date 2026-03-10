use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum UserAliases {
  Table,
  Id,
  TenantId,
  UserId,
  Alias,
  Repo,
  Filename,
  Snapshot,
  RequestParams,
  ContextParams,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(UserAliases::Table)
          .col(string(UserAliases::Id).primary_key())
          .col(string(UserAliases::TenantId))
          .col(string(UserAliases::UserId))
          .col(string(UserAliases::Alias))
          .col(string(UserAliases::Repo))
          .col(string(UserAliases::Filename))
          .col(string(UserAliases::Snapshot))
          .col(json_binary(UserAliases::RequestParams))
          .col(json_binary(UserAliases::ContextParams))
          .col(timestamp_with_time_zone(UserAliases::CreatedAt))
          .col(timestamp_with_time_zone(UserAliases::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_user_aliases_tenant_id")
          .table(UserAliases::Table)
          .col(UserAliases::TenantId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_user_aliases_user_id")
          .table(UserAliases::Table)
          .col(UserAliases::UserId)
          .to_owned(),
      )
      .await?;

    // Composite unique index: (tenant_id, user_id, alias)
    manager
      .create_index(
        Index::create()
          .name("idx_user_aliases_tenant_alias")
          .table(UserAliases::Table)
          .col(UserAliases::TenantId)
          .col(UserAliases::UserId)
          .col(UserAliases::Alias)
          .unique()
          .to_owned(),
      )
      .await?;

    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("ALTER TABLE user_aliases ENABLE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE user_aliases FORCE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY tenant_isolation ON user_aliases
             FOR ALL
             USING (tenant_id = (SELECT current_tenant_id()))
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
        .execute_unprepared("DROP POLICY IF EXISTS tenant_isolation ON user_aliases;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE user_aliases DISABLE ROW LEVEL SECURITY;")
        .await?;
    }
    manager
      .drop_table(Table::drop().table(UserAliases::Table).to_owned())
      .await
  }
}
