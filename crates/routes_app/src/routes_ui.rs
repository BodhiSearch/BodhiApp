use axum::{
  body::Body,
  extract::{Path as UrlPath, State},
  http::{header::LOCATION, status::StatusCode, Response},
  response::Json,
  routing::{delete, get, post},
  Router,
};
use objs::ApiError;
use server_core::RouterState;
use services::db::Conversation;
use std::sync::Arc;

pub fn chats_router() -> Router<Arc<dyn RouterState>> {
  Router::new()
    .route("/chats", get(ui_chats_handler))
    .route("/chats", delete(ui_chats_delete_handler))
    .route("/chats/{id}", get(ui_chat_handler))
    .route("/chats/{id}", post(ui_chat_new_handler))
    .route("/chats/{id}", delete(ui_chat_delete_handler))
}

async fn ui_chats_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<Vec<Conversation>>, ApiError> {
  let convos = state
    .app_service()
    .db_service()
    .list_conversations()
    .await?;
  Ok(Json(convos))
}

async fn ui_chat_handler(
  State(state): State<Arc<dyn RouterState>>,
  UrlPath(id): UrlPath<String>,
) -> Result<Json<Conversation>, ApiError> {
  let convo = state
    .app_service()
    .db_service()
    .get_conversation_with_messages(&id)
    .await?;
  Ok(Json(convo))
}

async fn ui_chat_new_handler(
  State(state): State<Arc<dyn RouterState>>,
  UrlPath(id): UrlPath<String>,
  Json(mut conversation): Json<Conversation>,
) -> Result<Response<Body>, ApiError> {
  if !conversation.id.eq(&id) {
    conversation.id = id;
  }
  state
    .app_service()
    .db_service()
    .save_conversation(&mut conversation)
    .await?;
  let response = Response::builder()
    .status(StatusCode::CREATED)
    .header(LOCATION, format!("/chats/{}", conversation.id))
    .body(Body::empty())
    .unwrap();
  Ok(response)
}

async fn ui_chats_delete_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<(), ApiError> {
  state
    .app_service()
    .db_service()
    .delete_all_conversations()
    .await?;
  Ok(())
}

async fn ui_chat_delete_handler(
  State(state): State<Arc<dyn RouterState>>,
  UrlPath(id): UrlPath<String>,
) -> Result<(), ApiError> {
  state
    .app_service()
    .db_service()
    .delete_conversations(&id)
    .await?;
  Ok(())
}

#[cfg(test)]
mod test {
  use crate::chats_router;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
  };
  use objs::{test_utils::setup_l10n, FluentLocalizationService};
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContext,
  };
  use services::test_utils::AppServiceStubBuilder;
  use services::{
    db::{Conversation, ConversationBuilder, DbService, MessageBuilder},
    test_utils::{test_db_service, TestDbService},
  };
  use std::sync::Arc;
  use tower::ServiceExt;
  use uuid::Uuid;
  use validator::ValidateLength;

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_chat_routes_index(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let mut convo_1 = ConversationBuilder::default()
      .title("test title 1")
      .messages(vec![MessageBuilder::default()
        .role("user")
        .content("test content")
        .build()?])
      .build()?;
    let mut convo_2 = ConversationBuilder::default()
      .title("test title 2")
      .build()?;
    db_service.save_conversation(&mut convo_1).await?;
    db_service.save_conversation(&mut convo_2).await?;
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .build()?;
    let router_state =
      DefaultRouterState::new(Arc::new(MockSharedContext::new()), Arc::new(app_service));
    let router = chats_router().with_state(Arc::new(router_state));
    let response = router
      .oneshot(Request::get("/chats").body(Body::empty()).unwrap())
      .await?
      .json::<Value>()
      .await?;
    assert_eq!(2, response.as_array().length().unwrap());
    let expected_1: Value = serde_json::from_str(&format!(
      r#"{{"id":"{}","title":"test title 1","messages":[]}}"#,
      convo_1.id
    ))?;
    assert_eq!(expected_1, response[0]);
    let expected_2: Value = serde_json::from_str(&format!(
      r#"{{"id":"{}","title":"test title 2","messages":[]}}"#,
      convo_2.id
    ))?;
    assert_eq!(expected_2, response[1]);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_chat_routes_get(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let mut convo = ConversationBuilder::default().title("test title").build()?;
    let message_1 = MessageBuilder::default()
      .conversation_id(&convo.id)
      .role("user")
      .content("test content")
      .build()?;
    let message_2 = MessageBuilder::default()
      .conversation_id(&convo.id)
      .role("assistant")
      .content("test reply")
      .build()?;
    convo.messages.push(message_1);
    convo.messages.push(message_2);
    db_service.save_conversation(&mut convo).await?;
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .build()?;
    let router_state =
      DefaultRouterState::new(Arc::new(MockSharedContext::new()), Arc::new(app_service));
    let router = chats_router().with_state(Arc::new(router_state));
    let response = router
      .oneshot(
        Request::get(format!("/chats/{}", &convo.id))
          .body(Body::empty())
          .unwrap(),
      )
      .await?
      .json::<Value>()
      .await?;
    let expected = format!(
      r#"{{"id":"{}","title":"test title","messages":[{{"role":"user","content":"test content"}},{{"role":"assistant","content":"test reply"}}]}}"#,
      convo.id
    );
    let expected = serde_json::from_str::<Value>(&expected)?;
    assert_eq!(expected, response);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_chat_routes_delete(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let mut convo = ConversationBuilder::default().id(Uuid::new_v4()).build()?;
    let message_1 = MessageBuilder::default()
      .conversation_id(&convo.id)
      .build()?;
    let message_2 = MessageBuilder::default()
      .conversation_id(&convo.id)
      .build()?;
    convo.messages.push(message_1);
    convo.messages.push(message_2);
    db_service.save_conversation(&mut convo).await?;
    let app_service = AppServiceStubBuilder::default()
      .db_service(Arc::new(db_service))
      .build()?;
    let router_state =
      DefaultRouterState::new(Arc::new(MockSharedContext::new()), Arc::new(app_service));
    let router = chats_router().with_state(Arc::new(router_state));
    let response = router
      .clone()
      .oneshot(
        Request::delete(format!("/chats/{}", &convo.id))
          .body(Body::empty())
          .unwrap(),
      )
      .await?
      .text()
      .await?;
    assert_eq!("", response);
    let convos = router
      .oneshot(Request::get("/chats").body(Body::empty()).unwrap())
      .await?
      .json::<Vec<Conversation>>()
      .await?;
    assert!(convos.is_empty());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_chat_routes_delete_chats(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let mut convo = ConversationBuilder::default().build()?;
    let message_1 = MessageBuilder::default()
      .conversation_id(&convo.id)
      .build()?;
    let message_2 = MessageBuilder::default()
      .conversation_id(&convo.id)
      .build()?;
    convo.messages.push(message_1);
    convo.messages.push(message_2);
    db_service.save_conversation(&mut convo).await?;
    let db_service = Arc::new(db_service);
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .db_service(db_service.clone())
        .build()?,
    );
    let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::new()), app_service);
    let router = chats_router().with_state(Arc::new(router_state));
    let response = router
      .clone()
      .oneshot(Request::delete("/chats").body(Body::empty()).unwrap())
      .await?;
    assert_eq!(StatusCode::OK, response.status());
    let convos = db_service.list_conversations().await?;
    assert!(convos.is_empty());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_chat_routes_get_not_found(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .db_service(Arc::new(db_service))
        .build()?,
    );
    let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::new()), app_service);
    let router = chats_router().with_state(Arc::new(router_state));
    let response = router
      .clone()
      .oneshot(
        Request::get(format!("/chats/{}", Uuid::new_v4()))
          .body(Body::empty())
          .unwrap(),
      )
      .await?;
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": "no rows returned by a query that expected to return at least one row",
          "code": "sqlx_error",
          "type": "internal_server_error"
        }
      }},
      response
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_chat_routes_new_chat(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(db_service);
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .db_service(db_service.clone())
        .build()?,
    );
    let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::new()), app_service);
    let router = chats_router().with_state(Arc::new(router_state));
    let content = r#"{
"title": "What is the capital of France?",
"createdAt": 1713590479639,
"id": "NEWID07",
"messages": [
  {
    "role": "user",
    "content": "What is the capital of France?"
  },
  {
    "content": "The capital of France is Paris.",
    "role": "assistant"
  }
]}"#;
    let response = router
      .clone()
      .oneshot(Request::post("/chats/NEWID07").json_str(content).unwrap())
      .await?;
    assert_eq!(StatusCode::CREATED, response.status());
    let location = response
      .headers()
      .get("Location")
      .unwrap()
      .to_str()
      .unwrap();
    assert!(location.starts_with("/chats/NEWID07"));
    let chat_id = location.strip_prefix("/chats/").unwrap();
    let from_db = db_service.get_conversation_with_messages(chat_id).await?;
    assert_eq!(1713590479, from_db.created_at.timestamp());
    assert_eq!(2, from_db.messages.len());
    assert_eq!("What is the capital of France?", from_db.title);
    assert_eq!(
      "What is the capital of France?",
      from_db.messages.first().unwrap().content.as_ref().unwrap()
    );
    assert_eq!("user", from_db.messages.first().unwrap().role);
    assert_eq!(
      "The capital of France is Paris.",
      from_db.messages.get(1).unwrap().content.as_ref().unwrap()
    );
    assert_eq!("assistant", from_db.messages.get(1).unwrap().role);
    let text = response.text().await?;
    assert_eq!("", text);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_chat_routes_update_chat(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(db_service);
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .db_service(db_service.clone())
        .build()?,
    );
    let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::new()), app_service);
    let router = chats_router().with_state(Arc::new(router_state));
    let content = r#"{
      "title": "What is the capital of France?",
      "createdAt": 1713590479639,
      "id": "NEWID07",
      "messages": [
        {
          "role": "user",
          "content": "What is the capital of France?"
        }
      ]}"#;
    let content: Conversation = serde_json::from_str(content).unwrap();
    let response = router
      .clone()
      .oneshot(Request::post("/chats/NEWID07").json(content).unwrap())
      .await?;
    assert_eq!(StatusCode::CREATED, response.status());

    let content = r#"{
      "title": "What is the capital of France?",
      "createdAt": 1713590479639,
      "id": "NEWID07",
      "messages": [
        {
          "role": "user",
          "content": "What is the capital of France?"
        },
        {
          "content": "The capital of France is Paris.",
          "role": "assistant"
        }
      ]}"#;
    let response = router
      .clone()
      .oneshot(Request::post("/chats/NEWID07").json_str(content).unwrap())
      .await?;
    // assert_eq!(StatusCode::CREATED, response.status());
    assert_eq!("", response.text().await?);
    let from_db = db_service.get_conversation_with_messages("NEWID07").await?;
    assert_eq!(2, from_db.messages.len());
    assert_eq!(
      "What is the capital of France?",
      from_db.messages.first().unwrap().content.as_ref().unwrap()
    );
    assert_eq!(
      "The capital of France is Paris.",
      from_db.messages.get(1).unwrap().content.as_ref().unwrap()
    );
    Ok(())
  }
}
