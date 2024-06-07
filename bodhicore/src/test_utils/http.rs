use axum::{
  body::Body,
  http::{request::Builder, Request},
  response::Response,
};
use http_body_util::BodyExt;
use reqwest::header::CONTENT_TYPE;
use serde::{de::DeserializeOwned, Deserialize};
use std::io::Cursor;

pub trait ResponseTestExt {
  async fn json<T>(self) -> anyhow::Result<T>
  where
    T: DeserializeOwned;

  async fn json_obj<T>(self) -> anyhow::Result<T>
  where
    T: for<'a> Deserialize<'a>;

  async fn text(self) -> anyhow::Result<String>;

  async fn sse<T>(self) -> anyhow::Result<Vec<T>>
  where
    T: DeserializeOwned;
}

impl ResponseTestExt for Response {
  async fn json<T>(self) -> anyhow::Result<T>
  where
    T: DeserializeOwned,
  {
    let bytes = self.into_body().collect().await.unwrap().to_bytes();
    let str = String::from_utf8_lossy(&bytes);
    let reader = Cursor::new(str.into_owned());
    let result = serde_json::from_reader::<_, T>(reader)?;
    Ok(result)
  }

  async fn json_obj<T>(self) -> anyhow::Result<T>
  where
    T: for<'de> Deserialize<'de>,
  {
    let bytes = self.into_body().collect().await.unwrap().to_bytes();
    let str = String::from_utf8_lossy(&bytes).into_owned();
    let result = serde_json::from_str(&str)?;
    Ok(result)
  }

  async fn text(self) -> anyhow::Result<String> {
    let bytes = self.into_body().collect().await.unwrap().to_bytes();
    let str = String::from_utf8_lossy(&bytes);
    Ok(str.into_owned())
  }

  async fn sse<T>(self) -> anyhow::Result<Vec<T>>
  where
    T: DeserializeOwned,
  {
    let text = self.text().await?;
    let lines = text.lines().peekable();
    let mut result = Vec::<T>::new();
    for line in lines {
      if line.is_empty() {
        continue;
      }
      let (_, value) = line.split_once(':').unwrap();
      let value = value.trim();
      let value = serde_json::from_reader::<_, T>(Cursor::new(value.to_owned()))?;
      result.push(value);
    }
    Ok(result)
  }
}

pub trait RequestTestExt {
  fn json<T: serde::Serialize>(self, value: T) -> Result<Request<Body>, anyhow::Error>;

  fn json_str(self, value: &str) -> Result<Request<Body>, anyhow::Error>;
}

impl RequestTestExt for Builder {
  fn json<T: serde::Serialize>(
    self,
    value: T,
  ) -> std::result::Result<Request<Body>, anyhow::Error> {
    let this = self.header(CONTENT_TYPE, "application/json");
    let content = serde_json::to_string(&value)?;
    let result = this.body(Body::from(content))?;
    Ok(result)
  }

  fn json_str(self, value: &str) -> Result<Request<Body>, anyhow::Error> {
    let this = self.header(CONTENT_TYPE, "application/json");
    let result = this.body(Body::from(value.to_string()))?;
    Ok(result)
  }
}
