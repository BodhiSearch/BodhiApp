use crate::db::{entities::api_token, ApiToken, DbError, DefaultDbService, TokenRepository};
use sea_orm::prelude::*;
use sea_orm::{QueryOrder, QuerySelect, Set};

#[async_trait::async_trait]
impl TokenRepository for DefaultDbService {
  async fn create_api_token(&self, token: &mut ApiToken) -> Result<(), DbError> {
    let now = self.time_service.utc_now();
    token.created_at = now;
    token.updated_at = now;

    let model = api_token::ActiveModel {
      id: Set(token.id.clone()),
      user_id: Set(token.user_id.clone()),
      name: Set(token.name.clone()),
      token_prefix: Set(token.token_prefix.clone()),
      token_hash: Set(token.token_hash.clone()),
      scopes: Set(token.scopes.clone()),
      status: Set(token.status.clone()),
      created_at: Set(token.created_at),
      updated_at: Set(token.updated_at),
    };

    api_token::Entity::insert(model)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(())
  }

  async fn list_api_tokens(
    &self,
    user_id: &str,
    page: usize,
    per_page: usize,
  ) -> Result<(Vec<ApiToken>, usize), DbError> {
    let page = page.max(1);
    let offset = ((page - 1) * per_page) as u64;

    let total = api_token::Entity::find()
      .filter(api_token::Column::UserId.eq(user_id))
      .count(&self.db)
      .await
      .map_err(DbError::from)? as usize;

    let results = api_token::Entity::find()
      .filter(api_token::Column::UserId.eq(user_id))
      .order_by_desc(api_token::Column::CreatedAt)
      .offset(offset)
      .limit(per_page as u64)
      .all(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok((results, total))
  }

  async fn get_api_token_by_id(
    &self,
    user_id: &str,
    id: &str,
  ) -> Result<Option<ApiToken>, DbError> {
    let result = api_token::Entity::find_by_id(id.to_string())
      .filter(api_token::Column::UserId.eq(user_id))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result)
  }

  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<ApiToken>, DbError> {
    let result = api_token::Entity::find()
      .filter(api_token::Column::TokenPrefix.eq(prefix))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(result)
  }

  async fn update_api_token(&self, user_id: &str, token: &mut ApiToken) -> Result<(), DbError> {
    token.updated_at = self.time_service.utc_now();

    let existing = api_token::Entity::find_by_id(token.id.clone())
      .filter(api_token::Column::UserId.eq(user_id))
      .one(&self.db)
      .await
      .map_err(DbError::from)?;

    if existing.is_none() {
      return Err(DbError::from(sea_orm::DbErr::RecordNotFound(
        token.id.clone(),
      )));
    }

    let mut active: api_token::ActiveModel = Default::default();
    active.id = Set(token.id.clone());
    active.name = Set(token.name.clone());
    active.status = Set(token.status.clone());
    active.updated_at = Set(token.updated_at);

    api_token::Entity::update(active)
      .exec(&self.db)
      .await
      .map_err(DbError::from)?;

    Ok(())
  }
}
