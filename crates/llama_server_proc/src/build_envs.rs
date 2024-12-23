pub static BUILD_TARGET: &str = env!("BUILD_TARGET");

lazy_static::lazy_static! {
  pub static ref BUILD_VARIANTS: Vec<String> = {
    env!("BUILD_VARIANTS").split(',').map(String::from).collect()
  };
}
pub static DEFAULT_VARIANT: &str = env!("DEFAULT_VARIANT");
pub static EXEC_NAME: &str = env!("EXEC_NAME");
