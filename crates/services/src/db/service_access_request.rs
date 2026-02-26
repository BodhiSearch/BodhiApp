use crate::db::{AccessRequestRepository, AppAccessRequestRow, DbError, SqliteDbService};
use sqlx::query_as;

#[async_trait::async_trait]
impl AccessRequestRepository for SqliteDbService {
  async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError> {
    let result = query_as::<_, AppAccessRequestRow>(
      "INSERT INTO app_access_requests
        (id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
         approved, user_id, requested_role, approved_role, access_request_scope, error_message,
         expires_at, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
       RETURNING id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
                 approved, user_id, requested_role, approved_role, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(&row.id)
    .bind(&row.app_client_id)
    .bind(&row.app_name)
    .bind(&row.app_description)
    .bind(&row.flow_type)
    .bind(&row.redirect_uri)
    .bind(&row.status)
    .bind(&row.requested)
    .bind(&row.approved)
    .bind(&row.user_id)
    .bind(&row.requested_role)
    .bind(&row.approved_role)
    .bind(&row.access_request_scope)
    .bind(&row.error_message)
    .bind(row.expires_at)
    .bind(row.created_at)
    .bind(row.updated_at)
    .fetch_one(&self.pool)
    .await?;

    Ok(result)
  }

  async fn get(&self, id: &str) -> Result<Option<AppAccessRequestRow>, DbError> {
    let result = query_as::<_, AppAccessRequestRow>(
      "SELECT id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
              approved, user_id, requested_role, approved_role, access_request_scope, error_message,
              expires_at, created_at, updated_at
       FROM app_access_requests WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result)
  }

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    approved: &str,
    approved_role: &str,
    access_request_scope: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = query_as::<_, AppAccessRequestRow>(
      "UPDATE app_access_requests
       SET status = 'approved', user_id = ?, approved = ?,
           approved_role = ?, access_request_scope = ?, updated_at = ?
       WHERE id = ?
       RETURNING id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
                 approved, user_id, requested_role, approved_role, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(user_id)
    .bind(approved)
    .bind(approved_role)
    .bind(access_request_scope)
    .bind(now)
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(result)
  }

  async fn update_denial(&self, id: &str, user_id: &str) -> Result<AppAccessRequestRow, DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = query_as::<_, AppAccessRequestRow>(
      "UPDATE app_access_requests
       SET status = 'denied', user_id = ?, updated_at = ?
       WHERE id = ?
       RETURNING id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
                 approved, user_id, requested_role, approved_role, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(user_id)
    .bind(now)
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(result)
  }

  async fn update_failure(
    &self,
    id: &str,
    error_message: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = query_as::<_, AppAccessRequestRow>(
      "UPDATE app_access_requests
       SET status = 'failed', error_message = ?, updated_at = ?
       WHERE id = ?
       RETURNING id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
                 approved, user_id, requested_role, approved_role, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(error_message)
    .bind(now)
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(result)
  }

  async fn get_by_access_request_scope(
    &self,
    scope: &str,
  ) -> Result<Option<AppAccessRequestRow>, DbError> {
    let result = query_as::<_, AppAccessRequestRow>(
      "SELECT id, app_client_id, app_name, app_description, flow_type,
              redirect_uri, status, requested, approved, user_id,
              requested_role, approved_role, access_request_scope, error_message,
              expires_at, created_at, updated_at
       FROM app_access_requests
       WHERE access_request_scope = ?",
    )
    .bind(scope)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result)
  }
}
