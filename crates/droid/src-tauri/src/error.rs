use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum AppError {}

pub type Result<T> = std::result::Result<T, AppError>;
