use axum::{body::Body, response::Response};
use http_body_util::BodyExt;

pub async fn parse<T: serde::de::DeserializeOwned>(response: Response<Body>) -> T {
  let bytes = response.into_body().collect().await.unwrap().to_bytes();
  let str = String::from_utf8_lossy(&bytes);
  serde_json::from_str(&str).unwrap()
}

pub async fn parse_txt(response: Response<Body>) -> String {
  let bytes = response.into_body().collect().await.unwrap().to_bytes();
  let str = String::from_utf8_lossy(&bytes);
  str.to_string()
}
