use crate::db::{
  AccessRepository, DbError, SqliteDbService, UserAccessRequest, UserAccessRequestStatus,
};
use chrono::{DateTime, Utc};
use sqlx::query_as;
use std::str::FromStr;

#[async_trait::async_trait]
impl AccessRepository for SqliteDbService {
  async fn insert_pending_request(
    &self,
    username: String,
    user_id: String,
  ) -> Result<UserAccessRequest, DbError> {
    let now = self.time_service.utc_now();
    let result = query_as::<
      _,
      (
        i64,
        String,
        String,
        Option<String>,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      "INSERT INTO access_requests (username, user_id, created_at, updated_at, status)
         VALUES (?, ?, ?, ?, ?)
         RETURNING id, username, user_id, reviewer, status, created_at, updated_at",
    )
    .bind(&username)
    .bind(&user_id)
    .bind(now)
    .bind(now)
    .bind(UserAccessRequestStatus::Pending.to_string())
    .fetch_one(&self.pool)
    .await?;

    Ok(UserAccessRequest {
      id: result.0,
      username: result.1,
      user_id: result.2,
      reviewer: result.3,
      status: UserAccessRequestStatus::from_str(&result.4)?,
      created_at: result.5,
      updated_at: result.6,
    })
  }

  async fn get_pending_request(
    &self,
    user_id: String,
  ) -> Result<Option<UserAccessRequest>, DbError> {
    let result = query_as::<
      _,
      (
        i64,
        String,
        String,
        Option<String>,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      "SELECT id, username, user_id, reviewer, status, created_at, updated_at
         FROM access_requests
         WHERE user_id = ? AND status = ?",
    )
    .bind(&user_id)
    .bind(UserAccessRequestStatus::Pending.to_string())
    .fetch_optional(&self.pool)
    .await?;

    let result = result
      .map(
        |(id, username, user_id, reviewer, status, created_at, updated_at)| {
          let Ok(status) = UserAccessRequestStatus::from_str(&status) else {
            tracing::warn!("unknown request status: {} for id: {}", status, id);
            return None;
          };
          let result = UserAccessRequest {
            id,
            username,
            user_id,
            reviewer,
            status,
            created_at,
            updated_at,
          };
          Some(result)
        },
      )
      .unwrap_or(None);
    Ok(result)
  }

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError> {
    let offset = (page - 1) * per_page;
    // Get total count of pending requests
    let total_count: (i64,) = query_as("SELECT COUNT(*) FROM access_requests WHERE status = ?")
      .bind(UserAccessRequestStatus::Pending.to_string())
      .fetch_one(&self.pool)
      .await?;
    let results = query_as::<
      _,
      (
        i64,
        String,
        String,
        Option<String>,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      "SELECT id, username, user_id, reviewer, status, created_at, updated_at
         FROM access_requests
         WHERE status = ?
         ORDER BY created_at ASC
         LIMIT ? OFFSET ?",
    )
    .bind(UserAccessRequestStatus::Pending.to_string())
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&self.pool)
    .await?;

    let results = results
      .into_iter()
      .filter_map(
        |(id, username, user_id, reviewer, status, created_at, updated_at)| {
          let Ok(status) = UserAccessRequestStatus::from_str(&status) else {
            tracing::warn!("unknown request status: {} for id: {}", status, id);
            return None;
          };
          let result = UserAccessRequest {
            id,
            username,
            user_id,
            reviewer,
            status,
            created_at,
            updated_at,
          };
          Some(result)
        },
      )
      .collect::<Vec<UserAccessRequest>>();
    Ok((results, total_count.0 as usize))
  }

  async fn list_all_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError> {
    let offset = (page - 1) * per_page;
    // Get total count of all requests
    let total_count: (i64,) = query_as("SELECT COUNT(*) FROM access_requests")
      .fetch_one(&self.pool)
      .await?;
    let results = query_as::<
      _,
      (
        i64,
        String,
        String,
        Option<String>,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      "SELECT id, username, user_id, reviewer, status, created_at, updated_at
         FROM access_requests
         ORDER BY created_at ASC
         LIMIT ? OFFSET ?",
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&self.pool)
    .await?;

    let results = results
      .into_iter()
      .filter_map(
        |(id, username, user_id, reviewer, status, created_at, updated_at)| {
          let status = UserAccessRequestStatus::from_str(&status).ok()?;
          let result = UserAccessRequest {
            id,
            username,
            user_id,
            reviewer,
            status,
            created_at,
            updated_at,
          };
          Some(result)
        },
      )
      .collect::<Vec<UserAccessRequest>>();
    Ok((results, total_count.0 as usize))
  }

  async fn update_request_status(
    &self,
    id: i64,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    sqlx::query(
      "UPDATE access_requests
         SET status = ?, updated_at = ?, reviewer = ?
         WHERE id = ?",
    )
    .bind(status.to_string())
    .bind(now)
    .bind(&reviewer)
    .bind(id)
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  async fn get_request_by_id(&self, id: i64) -> Result<Option<UserAccessRequest>, DbError> {
    let result = query_as::<
      _,
      (
        i64,
        String,
        String,
        Option<String>,
        String,
        DateTime<Utc>,
        DateTime<Utc>,
      ),
    >(
      "SELECT id, username, user_id, reviewer, status, created_at, updated_at
         FROM access_requests
         WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    if let Some((id, username, user_id, reviewer, status, created_at, updated_at)) = result {
      let status = UserAccessRequestStatus::from_str(&status).map_err(DbError::StrumParse)?;
      Ok(Some(UserAccessRequest {
        id,
        username,
        user_id,
        reviewer,
        status,
        created_at,
        updated_at,
      }))
    } else {
      Ok(None)
    }
  }
}
