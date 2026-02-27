use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AccessRequests {
  Table,
  Id,
  Username,
  UserId,
  Reviewer,
  Status,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(AccessRequests::Table)
          .col(string(AccessRequests::Id).primary_key())
          .col(string(AccessRequests::Username))
          .col(string(AccessRequests::UserId))
          .col(string_null(AccessRequests::Reviewer))
          .col(string(AccessRequests::Status).default("pending"))
          .col(timestamp_with_time_zone(AccessRequests::CreatedAt))
          .col(timestamp_with_time_zone(AccessRequests::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_access_requests_user_id")
          .table(AccessRequests::Table)
          .col(AccessRequests::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_access_requests_status")
          .table(AccessRequests::Table)
          .col(AccessRequests::Status)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(AccessRequests::Table).to_owned())
      .await
  }
}
