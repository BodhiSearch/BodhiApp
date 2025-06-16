use crate::AppRegInfo;
use rstest::fixture;

#[fixture]
pub fn app_reg_info() -> AppRegInfo {
  AppRegInfo {
    client_id: "test-client".to_string(),
    client_secret: "test-secret".to_string(),
  }
}
