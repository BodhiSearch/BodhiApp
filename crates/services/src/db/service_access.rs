use crate::db::{
  entities::access_request, AccessRepository, DbError, DefaultDbService, UserAccessRequest,
  UserAccessRequestStatus,
};
use sea_orm::prelude::*;
use sea_orm::{QueryOrder, QuerySelect, Set};

#[async_trait::async_trait]
impl AccessRepository for DefaultDbService {
  async fn insert_pending_request(
    &self,
    username: String,
    user_id: String,
  ) -> Result<UserAccessRequest, DbError> {
    let now = self.time_service.utc_now();
    let id = ulid::Ulid::new().to_string();

    let model = access_request::ActiveModel {
      id: Set(id.clone()),
      username: Set(username),
      user_id: Set(user_id),
      reviewer: Set(None),
      status: Set(UserAccessRequestStatus::Pending),
      created_at: Set(now),
      updated_at: Set(now),
    };

    access_request::Entity::insert(model)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    self
      .get_request_by_id(&id)
      .await?
      .ok_or_else(|| DbError::from(sea_orm::DbErr::RecordNotInserted))
  }

  async fn get_pending_request(
    &self,
    user_id: String,
  ) -> Result<Option<UserAccessRequest>, DbError> {
    let result = access_request::Entity::find()
      .filter(access_request::Column::UserId.eq(user_id))
      .filter(access_request::Column::Status.eq(UserAccessRequestStatus::Pending.to_string()))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result)
  }

  async fn list_pending_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError> {
    let offset = ((page - 1) * per_page) as u64;

    let total = access_request::Entity::find()
      .filter(access_request::Column::Status.eq(UserAccessRequestStatus::Pending.to_string()))
      .count(&self.db)
      .await
      .map_err(DbError::from)? as usize;

    let results = access_request::Entity::find()
      .filter(access_request::Column::Status.eq(UserAccessRequestStatus::Pending.to_string()))
      .order_by_asc(access_request::Column::CreatedAt)
      .offset(offset)
      .limit(per_page as u64)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok((results, total))
  }

  async fn list_all_requests(
    &self,
    page: u32,
    per_page: u32,
  ) -> Result<(Vec<UserAccessRequest>, usize), DbError> {
    let offset = ((page - 1) * per_page) as u64;

    let total = access_request::Entity::find()
      .count(&self.db)
      .await
      .map_err(DbError::from)? as usize;

    let results = access_request::Entity::find()
      .order_by_asc(access_request::Column::CreatedAt)
      .offset(offset)
      .limit(per_page as u64)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok((results, total))
  }

  async fn update_request_status(
    &self,
    id: &str,
    status: UserAccessRequestStatus,
    reviewer: String,
  ) -> Result<(), DbError> {
    let now = self.time_service.utc_now();

    let mut active: access_request::ActiveModel = Default::default();
    active.id = Set(id.to_string());
    active.status = Set(status);
    active.reviewer = Set(Some(reviewer));
    active.updated_at = Set(now);

    access_request::Entity::update(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(())
  }

  async fn get_request_by_id(&self, id: &str) -> Result<Option<UserAccessRequest>, DbError> {
    let result = access_request::Entity::find_by_id(id.to_string())
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result)
  }
}
