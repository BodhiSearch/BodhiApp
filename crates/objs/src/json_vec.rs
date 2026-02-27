use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, ToSchema)]
pub struct JsonVec(Vec<String>);

impl JsonVec {
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn push(&mut self, val: String) {
    self.0.push(val);
  }
}

impl Deref for JsonVec {
  type Target = Vec<String>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for JsonVec {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl Default for JsonVec {
  fn default() -> Self {
    Self(Vec::new())
  }
}

impl From<Vec<String>> for JsonVec {
  fn from(v: Vec<String>) -> Self {
    Self(v)
  }
}

impl From<JsonVec> for Vec<String> {
  fn from(v: JsonVec) -> Self {
    v.0
  }
}

impl FromIterator<String> for JsonVec {
  fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
    Self(iter.into_iter().collect())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_json_vec_roundtrip() -> anyhow::Result<()> {
    let original = JsonVec(vec!["gpt-4".to_string(), "gpt-3.5".to_string()]);
    let serialized = serde_json::to_string(&original)?;
    let deserialized: JsonVec = serde_json::from_str(&serialized)?;
    assert_eq!(original, deserialized);
    Ok(())
  }

  #[test]
  fn test_json_vec_deref() {
    let jv = JsonVec(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(2, jv.len());
    assert!(jv.contains(&"a".to_string()));
  }

  #[test]
  fn test_json_vec_default() {
    let jv = JsonVec::default();
    assert!(jv.is_empty());
  }

  #[test]
  fn test_json_vec_from_vec() {
    let v = vec!["x".to_string()];
    let jv: JsonVec = v.into();
    assert_eq!(1, jv.len());
  }

  #[test]
  fn test_json_vec_into_vec() {
    let jv = JsonVec(vec!["y".to_string()]);
    let v: Vec<String> = jv.into();
    assert_eq!(vec!["y".to_string()], v);
  }
}
