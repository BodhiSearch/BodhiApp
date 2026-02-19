use crate::db::{AccessRequestRepository, AppAccessRequestRow, DbError, SqliteDbService};
use sqlx::query_as;

#[async_trait::async_trait]
impl AccessRequestRepository for SqliteDbService {
  async fn create(&self, row: &AppAccessRequestRow) -> Result<AppAccessRequestRow, DbError> {
    let result = query_as::<_, (String, String, Option<String>, Option<String>, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "INSERT INTO app_access_requests
        (id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
         approved, user_id, resource_scope, access_request_scope, error_message,
         expires_at, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
       RETURNING id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
                 approved, user_id, resource_scope, access_request_scope, error_message,
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
    .bind(&row.resource_scope)
    .bind(&row.access_request_scope)
    .bind(&row.error_message)
    .bind(row.expires_at)
    .bind(row.created_at)
    .bind(row.updated_at)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppAccessRequestRow {
      id: result.0,
      app_client_id: result.1,
      app_name: result.2,
      app_description: result.3,
      flow_type: result.4,
      redirect_uri: result.5,
      status: result.6,
      requested: result.7,
      approved: result.8,
      user_id: result.9,
      resource_scope: result.10,
      access_request_scope: result.11,
      error_message: result.12,
      expires_at: result.13,
      created_at: result.14,
      updated_at: result.15,
    })
  }

  async fn get(&self, id: &str) -> Result<Option<AppAccessRequestRow>, DbError> {
    let result = query_as::<_, (String, String, Option<String>, Option<String>, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "SELECT id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
              approved, user_id, resource_scope, access_request_scope, error_message,
              expires_at, created_at, updated_at
       FROM app_access_requests WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(|r| AppAccessRequestRow {
      id: r.0,
      app_client_id: r.1,
      app_name: r.2,
      app_description: r.3,
      flow_type: r.4,
      redirect_uri: r.5,
      status: r.6,
      requested: r.7,
      approved: r.8,
      user_id: r.9,
      resource_scope: r.10,
      access_request_scope: r.11,
      error_message: r.12,
      expires_at: r.13,
      created_at: r.14,
      updated_at: r.15,
    }))
  }

  async fn update_approval(
    &self,
    id: &str,
    user_id: &str,
    approved: &str,
    resource_scope: &str,
    access_request_scope: Option<String>,
  ) -> Result<AppAccessRequestRow, DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = query_as::<_, (String, String, Option<String>, Option<String>, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "UPDATE app_access_requests
       SET status = 'approved', user_id = ?, approved = ?,
           resource_scope = ?, access_request_scope = ?, updated_at = ?
       WHERE id = ?
       RETURNING id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
                 approved, user_id, resource_scope, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(user_id)
    .bind(approved)
    .bind(resource_scope)
    .bind(access_request_scope)
    .bind(now)
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppAccessRequestRow {
      id: result.0,
      app_client_id: result.1,
      app_name: result.2,
      app_description: result.3,
      flow_type: result.4,
      redirect_uri: result.5,
      status: result.6,
      requested: result.7,
      approved: result.8,
      user_id: result.9,
      resource_scope: result.10,
      access_request_scope: result.11,
      error_message: result.12,
      expires_at: result.13,
      created_at: result.14,
      updated_at: result.15,
    })
  }

  async fn update_denial(&self, id: &str, user_id: &str) -> Result<AppAccessRequestRow, DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = query_as::<_, (String, String, Option<String>, Option<String>, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "UPDATE app_access_requests
       SET status = 'denied', user_id = ?, updated_at = ?
       WHERE id = ?
       RETURNING id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
                 approved, user_id, resource_scope, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(user_id)
    .bind(now)
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppAccessRequestRow {
      id: result.0,
      app_client_id: result.1,
      app_name: result.2,
      app_description: result.3,
      flow_type: result.4,
      redirect_uri: result.5,
      status: result.6,
      requested: result.7,
      approved: result.8,
      user_id: result.9,
      resource_scope: result.10,
      access_request_scope: result.11,
      error_message: result.12,
      expires_at: result.13,
      created_at: result.14,
      updated_at: result.15,
    })
  }

  async fn update_failure(
    &self,
    id: &str,
    error_message: &str,
  ) -> Result<AppAccessRequestRow, DbError> {
    let now = self.time_service.utc_now().timestamp();
    let result = query_as::<_, (String, String, Option<String>, Option<String>, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, i64, i64, i64)>(
      "UPDATE app_access_requests
       SET status = 'failed', error_message = ?, updated_at = ?
       WHERE id = ?
       RETURNING id, app_client_id, app_name, app_description, flow_type, redirect_uri, status, requested,
                 approved, user_id, resource_scope, access_request_scope, error_message,
                 expires_at, created_at, updated_at"
    )
    .bind(error_message)
    .bind(now)
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

    Ok(AppAccessRequestRow {
      id: result.0,
      app_client_id: result.1,
      app_name: result.2,
      app_description: result.3,
      flow_type: result.4,
      redirect_uri: result.5,
      status: result.6,
      requested: result.7,
      approved: result.8,
      user_id: result.9,
      resource_scope: result.10,
      access_request_scope: result.11,
      error_message: result.12,
      expires_at: result.13,
      created_at: result.14,
      updated_at: result.15,
    })
  }

  async fn get_by_access_request_scope(
    &self,
    scope: &str,
  ) -> Result<Option<AppAccessRequestRow>, DbError> {
    let result = query_as::<
      _,
      (
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        Option<String>,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        i64,
        i64,
        i64,
      ),
    >(
      "SELECT id, app_client_id, app_name, app_description, flow_type,
              redirect_uri, status, requested, approved, user_id,
              resource_scope, access_request_scope, error_message,
              expires_at, created_at, updated_at
       FROM app_access_requests
       WHERE access_request_scope = ?",
    )
    .bind(scope)
    .fetch_optional(&self.pool)
    .await?;

    Ok(result.map(|r| AppAccessRequestRow {
      id: r.0,
      app_client_id: r.1,
      app_name: r.2,
      app_description: r.3,
      flow_type: r.4,
      redirect_uri: r.5,
      status: r.6,
      requested: r.7,
      approved: r.8,
      user_id: r.9,
      resource_scope: r.10,
      access_request_scope: r.11,
      error_message: r.12,
      expires_at: r.13,
      created_at: r.14,
      updated_at: r.15,
    }))
  }
}
