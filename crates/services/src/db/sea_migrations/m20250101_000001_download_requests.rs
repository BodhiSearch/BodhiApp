use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum DownloadRequests {
  Table,
  Id,
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

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(DownloadRequests::Table).to_owned())
      .await
  }
}
