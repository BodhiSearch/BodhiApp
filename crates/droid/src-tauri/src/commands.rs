use crate::error::Result;

#[tauri::command]
pub fn download() -> Result<String> {
  Ok("123".to_string())
}
