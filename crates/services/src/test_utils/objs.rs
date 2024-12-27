use crate::AppRegInfo;
use jsonwebtoken::Algorithm;
use rstest::fixture;

#[fixture]
pub fn app_reg_info() -> AppRegInfo {
  AppRegInfo {
    public_key: "test-public-key".to_string(),
    alg: Algorithm::RS256,
    kid: "test-kid".to_string(),
    issuer: "test-issuer".to_string(),
    client_id: "test-client".to_string(),
    client_secret: "test-secret".to_string(),
  }
}
