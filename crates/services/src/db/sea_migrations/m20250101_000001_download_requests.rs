use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum DownloadRequests {
  Table,
  Id,
  TenantId,
  Repo,
  Filename,
  Status,
  Error,
  TotalBytes,
  DownloadedBytes,
  StartedAt,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(DownloadRequests::Table)
          .col(string(DownloadRequests::Id).primary_key())
          .col(string(DownloadRequests::TenantId))
          .col(string(DownloadRequests::Repo))
          .col(string(DownloadRequests::Filename))
          .col(string(DownloadRequests::Status))
          .col(string_null(DownloadRequests::Error))
          .col(big_integer_null(DownloadRequests::TotalBytes))
          .col(big_integer(DownloadRequests::DownloadedBytes).default(0))
          .col(timestamp_with_time_zone_null(DownloadRequests::StartedAt))
          .col(timestamp_with_time_zone(DownloadRequests::CreatedAt))
          .col(timestamp_with_time_zone(DownloadRequests::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_download_requests_status")
          .table(DownloadRequests::Table)
          .col(DownloadRequests::Status)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_download_requests_tenant_id")
          .table(DownloadRequests::Table)
          .col(DownloadRequests::TenantId)
          .to_owned(),
      )
      .await?;

    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("ALTER TABLE download_requests ENABLE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE download_requests FORCE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY tenant_isolation ON download_requests
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
        .execute_unprepared("DROP POLICY IF EXISTS tenant_isolation ON download_requests;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE download_requests DISABLE ROW LEVEL SECURITY;")
        .await?;
    }
    manager
      .drop_table(Table::drop().table(DownloadRequests::Table).to_owned())
      .await?;
    Ok(())
  }
}
