use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AppAccessRequests {
  Table,
  Id,
  TenantId,
  AppClientId,
  AppName,
  AppDescription,
  FlowType,
  RedirectUri,
  Status,
  Requested,
  Approved,
  UserId,
  RequestedRole,
  ApprovedRole,
  AccessRequestScope,
  ErrorMessage,
  ExpiresAt,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(AppAccessRequests::Table)
          .col(string(AppAccessRequests::Id).primary_key())
          .col(string_null(AppAccessRequests::TenantId))
          .col(string(AppAccessRequests::AppClientId))
          .col(string_null(AppAccessRequests::AppName))
          .col(string_null(AppAccessRequests::AppDescription))
          .col(string(AppAccessRequests::FlowType))
          .col(string_null(AppAccessRequests::RedirectUri))
          .col(string(AppAccessRequests::Status).default("draft"))
          .col(string(AppAccessRequests::Requested))
          .col(string_null(AppAccessRequests::Approved))
          .col(string_null(AppAccessRequests::UserId))
          .col(string(AppAccessRequests::RequestedRole))
          .col(string_null(AppAccessRequests::ApprovedRole))
          .col(string_null(AppAccessRequests::AccessRequestScope))
          .col(string_null(AppAccessRequests::ErrorMessage))
          .col(timestamp_with_time_zone(AppAccessRequests::ExpiresAt))
          .col(timestamp_with_time_zone(AppAccessRequests::CreatedAt))
          .col(timestamp_with_time_zone(AppAccessRequests::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_app_access_requests_status")
          .table(AppAccessRequests::Table)
          .col(AppAccessRequests::Status)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_app_access_requests_app_client")
          .table(AppAccessRequests::Table)
          .col(AppAccessRequests::AppClientId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_app_access_requests_tenant_id")
          .table(AppAccessRequests::Table)
          .col(AppAccessRequests::TenantId)
          .to_owned(),
      )
      .await?;

    let db = manager.get_connection();
    db.execute_unprepared(
      "CREATE UNIQUE INDEX IF NOT EXISTS idx_access_request_scope_tenant_unique \
       ON app_access_requests(tenant_id, access_request_scope) \
       WHERE access_request_scope IS NOT NULL",
    )
    .await?;

    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("ALTER TABLE app_access_requests ENABLE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE app_access_requests FORCE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY app_access_requests_read ON app_access_requests
             FOR SELECT USING (true);",
        )
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY app_access_requests_insert ON app_access_requests
             FOR INSERT
             WITH CHECK (tenant_id IS NULL OR tenant_id = (SELECT current_tenant_id()));",
        )
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY app_access_requests_update ON app_access_requests
             FOR UPDATE
             USING (tenant_id IS NULL OR tenant_id = (SELECT current_tenant_id()))
             WITH CHECK (tenant_id IS NULL OR tenant_id = (SELECT current_tenant_id()));",
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
          "DROP POLICY IF EXISTS app_access_requests_update ON app_access_requests;",
        )
        .await?;
      conn
        .execute_unprepared(
          "DROP POLICY IF EXISTS app_access_requests_insert ON app_access_requests;",
        )
        .await?;
      conn
        .execute_unprepared(
          "DROP POLICY IF EXISTS app_access_requests_read ON app_access_requests;",
        )
        .await?;
      conn
        .execute_unprepared("ALTER TABLE app_access_requests DISABLE ROW LEVEL SECURITY;")
        .await?;
    }
    manager
      .drop_table(Table::drop().table(AppAccessRequests::Table).to_owned())
      .await
  }
}
