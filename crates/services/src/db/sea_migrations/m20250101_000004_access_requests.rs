use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum UserAccessRequests {
  Table,
  Id,
  TenantId,
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
          .table(UserAccessRequests::Table)
          .col(string(UserAccessRequests::Id).primary_key())
          .col(string(UserAccessRequests::TenantId))
          .col(string(UserAccessRequests::Username))
          .col(string(UserAccessRequests::UserId))
          .col(string_null(UserAccessRequests::Reviewer))
          .col(string(UserAccessRequests::Status).default("pending"))
          .col(timestamp_with_time_zone(UserAccessRequests::CreatedAt))
          .col(timestamp_with_time_zone(UserAccessRequests::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_user_access_requests_user_id")
          .table(UserAccessRequests::Table)
          .col(UserAccessRequests::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_user_access_requests_status")
          .table(UserAccessRequests::Table)
          .col(UserAccessRequests::Status)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_user_access_requests_tenant_id")
          .table(UserAccessRequests::Table)
          .col(UserAccessRequests::TenantId)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(UserAccessRequests::Table).to_owned())
      .await
  }
}
