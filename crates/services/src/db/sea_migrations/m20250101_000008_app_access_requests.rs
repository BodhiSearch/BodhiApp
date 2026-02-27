use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AppAccessRequests {
  Table,
  Id,
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

    let db = manager.get_connection();
    db.execute_unprepared(
      "CREATE UNIQUE INDEX IF NOT EXISTS idx_access_request_scope_unique \
       ON app_access_requests(access_request_scope) \
       WHERE access_request_scope IS NOT NULL",
    )
    .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(AppAccessRequests::Table).to_owned())
      .await
  }
}
