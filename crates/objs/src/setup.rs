use jsonwebtoken::Algorithm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, derive_builder::Builder)]
pub struct AppRegInfo {
  pub public_key: String,
  pub alg: Algorithm,
  pub kid: String,
  pub issuer: String,
  pub client_id: String,
  pub client_secret: String,
}
