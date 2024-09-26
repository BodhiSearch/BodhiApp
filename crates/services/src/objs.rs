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

#[derive(
  Debug, Serialize, Deserialize, PartialEq, strum::Display, Clone, Default, strum::EnumString,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum AppStatus {
  #[default]
  Setup,
  Ready,
  ResourceAdmin,
}
