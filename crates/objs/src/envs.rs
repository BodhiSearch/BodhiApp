#[derive(Debug, Clone, PartialEq, Default, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum EnvType {
  Production,
  #[default]
  Development,
}
