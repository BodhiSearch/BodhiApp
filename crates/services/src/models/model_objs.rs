use crate::shared_objs::{is_default, to_safe_filename};
use derive_builder::UninitializedFieldError;
use errmeta::{AppError, ErrorType};
use once_cell::sync::Lazy;
use regex::Regex;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
  fmt::Display,
  ops::{Deref, DerefMut},
  str::FromStr,
};
use utoipa::ToSchema;
use validator::{Validate, ValidationErrors};

// =============================================================================
// BuilderError
// =============================================================================

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
#[non_exhaustive]
pub enum BuilderError {
  #[error("Configuration incomplete: missing {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  UninitializedField(&'static str),
  #[error("Configuration invalid: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ValidationError(String),
}

impl From<UninitializedFieldError> for BuilderError {
  fn from(s: UninitializedFieldError) -> Self {
    Self::UninitializedField(s.field_name())
  }
}

impl From<String> for BuilderError {
  fn from(s: String) -> Self {
    Self::ValidationError(s)
  }
}

// =============================================================================
// ModelValidationError (new — split from ObjValidationError)
// =============================================================================

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ModelValidationError {
  #[error("{0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ValidationErrors(#[from] ValidationErrors),

  #[error("Invalid repository format '{0}'. Expected 'username/repo'.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  FilePatternMismatch(String),

  #[error("Prefix is required when forwarding all requests.")]
  #[error_meta(error_type = ErrorType::BadRequest, code = "model_validation_error-forward_all_requires_prefix")]
  ForwardAllRequiresPrefix,
}

// =============================================================================
// JsonVec
// =============================================================================

#[derive(
  Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, ToSchema, Default,
)]
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

// =============================================================================
// Repo
// =============================================================================

pub static TOKENIZER_CONFIG_JSON: &str = "tokenizer_config.json";
pub static GGUF: &str = "gguf";
pub static GGUF_EXTENSION: &str = ".gguf";
pub static REGEX_VALID_REPO: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_.-]+$").unwrap());

#[derive(Debug, Clone, PartialEq, Default, PartialOrd, Eq, Ord, Hash, Validate, ToSchema)]
pub struct Repo {
  #[validate(regex(
    path = *REGEX_VALID_REPO,
    message = "repo contains invalid characters"
  ))]
  user: String,
  #[validate(regex(
    path = *REGEX_VALID_REPO,
    message = "repo contains invalid characters",
  ))]
  name: String,
}

impl Repo {
  pub fn new<T: Into<String>>(user: T, repo_name: T) -> Self {
    Self {
      user: user.into(),
      name: repo_name.into(),
    }
  }

  pub fn path(&self) -> String {
    hf_hub::Repo::model(self.to_string()).folder_name()
  }

  pub fn namespace(&self) -> &str {
    &self.user
  }

  pub fn repo_name(&self) -> &str {
    &self.name
  }
}

impl FromStr for Repo {
  type Err = ModelValidationError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    let (user, repo_name) = value
      .split_once('/')
      .ok_or_else(|| ModelValidationError::FilePatternMismatch(value.to_string()))?;
    let repo = Repo::new(user, repo_name);
    repo.validate()?;
    Ok(repo)
  }
}

impl TryFrom<String> for Repo {
  type Error = ModelValidationError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Repo::from_str(&value)
  }
}

impl TryFrom<&str> for Repo {
  type Error = ModelValidationError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    Repo::from_str(value)
  }
}

impl Display for Repo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}/{}", self.user, self.name)
  }
}

impl<'de> Deserialize<'de> for Repo {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let repo = Repo::from_str(&s).map_err(|err| serde::de::Error::custom(err.to_string()))?;
    Ok(repo)
  }
}

impl Serialize for Repo {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    if self.user.is_empty() || self.name.is_empty() {
      return serializer.serialize_str("");
    }
    serializer.serialize_str(&self.to_string())
  }
}

// =============================================================================
// HubFile
// =============================================================================

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, derive_new::new)]
#[cfg_attr(any(test, feature = "test-utils"), derive(derive_builder::Builder))]
pub struct HubFile {
  pub hf_cache: std::path::PathBuf,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  pub size: Option<u64>,
}

impl HubFile {
  pub fn path(&self) -> std::path::PathBuf {
    let mut path = self.hf_cache.clone();
    path.push(self.repo.path());
    path.push("snapshots");
    path.push(&self.snapshot);
    path.push(&self.filename);
    path
  }
}

impl TryFrom<std::path::PathBuf> for HubFile {
  type Error = ModelValidationError;

  fn try_from(mut value: std::path::PathBuf) -> Result<Self, Self::Error> {
    let path_str = value.display().to_string();
    let size = std::fs::metadata(&value)
      .ok()
      .map(|metadata| metadata.len());
    // Get filename
    let filename = value
      .file_name()
      .and_then(|f| f.to_str())
      .ok_or_else(|| ModelValidationError::FilePatternMismatch(path_str.clone()))?
      .to_string();

    // Get snapshot hash
    value.pop(); // move to parent
    let snapshot = value
      .file_name()
      .and_then(|f| f.to_str())
      .ok_or_else(|| ModelValidationError::FilePatternMismatch(path_str.clone()))?
      .to_string();

    // Verify "snapshots" directory
    value.pop();
    if value.file_name().and_then(|f| f.to_str()) != Some("snapshots") {
      return Err(ModelValidationError::FilePatternMismatch(path_str));
    }
    value.pop();

    // Extract repo info from models--username--repo_name format
    let repo_dir = value
      .file_name()
      .and_then(|f| f.to_str())
      .ok_or_else(|| ModelValidationError::FilePatternMismatch(path_str.clone()))?
      .to_string();

    // Store repo parts before moving value
    let repo_parts: Vec<&str> = repo_dir.split("--").collect();
    if repo_parts.len() != 3 || repo_parts[0] != "models" {
      return Err(ModelValidationError::FilePatternMismatch(path_str));
    }

    // Get hf_cache (parent directory of the repo directory)
    value.pop();
    let hf_cache = value;

    // Construct repo from username/repo_name
    let repo = Repo::try_from(format!("{}/{}", repo_parts[1], repo_parts[2]))?;

    Ok(HubFile {
      hf_cache,
      repo,
      filename,
      snapshot,
      size,
    })
  }
}

impl std::fmt::Display for HubFile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "HubFile {{ repo: {}, filename: {}, snapshot: {} }}",
      self.repo, self.filename, self.snapshot
    )
  }
}

// =============================================================================
// DownloadStatus
// =============================================================================

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  strum::EnumString,
  strum::Display,
  PartialEq,
  ToSchema,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DownloadStatus {
  Pending,
  Completed,
  Error,
}

// =============================================================================
// OAIRequestParams
// =============================================================================

#[derive(
  Deserialize,
  Serialize,
  Debug,
  Clone,
  PartialEq,
  Default,
  PartialOrd,
  clap::Args,
  derive_builder::Builder,
  ToSchema,
  sea_orm::FromJsonQueryResult,
)]
#[builder(
  default,
  setter(into, strip_option),
  build_fn(error = BuilderError))]
pub struct OAIRequestParams {
  #[clap(long, value_parser = validate_range_neg_to_pos_2, help=r#"Number between -2.0 and 2.0.
Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
default: 0.0 (disabled)"#)]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub frequency_penalty: Option<f32>,

  #[arg(
    long,
    help = r#"The maximum number of tokens that can be generated in the completion.
The token count of your prompt plus `max_tokens` cannot exceed the model's context length.
default: -1 (disabled)"#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_tokens: Option<u32>,

  #[arg(long, value_parser = validate_range_neg_to_pos_2, help=r#"Number between -2.0 and 2.0.
Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
default: 0.0 (disabled)"#)]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub presence_penalty: Option<f32>,

  #[arg(long, value_parser = clap::value_parser!(i64).range(i64::MIN..=i64::MAX),
  help=r#"If specified, our system will make a best effort to sample deterministically, such that repeated requests with the same `seed` and parameters should return the same result."#)]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub seed: Option<i64>,

  #[arg(
    long,
    number_of_values = 1,
    help = r#"Up to 4 sequences where the API will stop generating further tokens."#
  )]
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub stop: Vec<String>,

  #[arg(long, value_parser = validate_range_0_to_2, help=r#"Number between 0.0 and 2.0.
Higher values like will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
default: 0.0 (disabled)"#)]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub temperature: Option<f32>,

  #[arg(long, value_parser = validate_range_0_to_1, help=r#"Number between 0.0 and 1.0.
An alternative to sampling with temperature, called nucleus sampling.
The model considers the results of the tokens with top_p probability mass. So 0.1 means only the tokens comprising the top 10% probability mass are considered.
Alter this or `temperature` but not both.
default: 1.0 (disabled)"#)]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub top_p: Option<f32>,

  #[arg(
    long,
    help = r#"A unique identifier representing your end-user, which can help to monitor and detect abuse."#
  )]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub user: Option<String>,
}

fn validate_range_neg_to_pos_2(s: &str) -> Result<f32, String> {
  let lower = -2.0;
  let upper = 2.0;
  validate_range(s, lower, upper)
}

fn validate_range_0_to_2(s: &str) -> Result<f32, String> {
  validate_range(s, 0.0, 2.0)
}

fn validate_range_0_to_1(s: &str) -> Result<f32, String> {
  validate_range(s, 0.0, 1.0)
}

fn validate_range<T: PartialOrd + FromStr + std::fmt::Debug + std::fmt::Display>(
  s: &str,
  lower: T,
  upper: T,
) -> Result<T, String> {
  match s.parse::<T>() {
    Ok(val) if lower <= val && val <= upper => Ok(val),
    Ok(val) => Err(format!(
      "The value {} is out of range. It must be between {:?} and {:?} inclusive.",
      val, lower, upper
    )),
    Err(_) => Err(format!(
      "'{}' is not a valid number. Please enter a number between {:?} and {:?}.",
      s, lower, upper
    )),
  }
}

impl OAIRequestParams {
  /// Apply request parameters directly to a JSON Value without deserializing.
  /// This preserves any non-standard fields that may be present in the request.
  pub fn apply_to_value(&self, request: &mut serde_json::Value) {
    if let Some(obj) = request.as_object_mut() {
      // Only set if not already present in request
      if let Some(val) = &self.frequency_penalty {
        if !obj.contains_key("frequency_penalty") {
          obj.insert("frequency_penalty".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.max_tokens {
        if !obj.contains_key("max_completion_tokens") && !obj.contains_key("max_tokens") {
          obj.insert("max_completion_tokens".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.presence_penalty {
        if !obj.contains_key("presence_penalty") {
          obj.insert("presence_penalty".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.seed {
        if !obj.contains_key("seed") {
          obj.insert("seed".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.temperature {
        if !obj.contains_key("temperature") {
          obj.insert("temperature".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.top_p {
        if !obj.contains_key("top_p") {
          obj.insert("top_p".to_string(), serde_json::json!(val));
        }
      }
      if let Some(val) = &self.user {
        if !obj.contains_key("user") {
          obj.insert("user".to_string(), serde_json::json!(val));
        }
      }
      if !self.stop.is_empty() && !obj.contains_key("stop") {
        obj.insert("stop".to_string(), serde_json::json!(self.stop));
      }
    }
  }
}

// =============================================================================
// AliasSource + UserAlias
// =============================================================================

#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  ToSchema,
  strum::Display,
  strum::EnumString,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AliasSource {
  #[default]
  User,
  Model,
  Api,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, derive_builder::Builder)]
#[builder(setter(into, strip_option), build_fn(error = BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct UserAlias {
  #[builder(setter(skip))]
  pub id: String,
  pub alias: String,
  #[schema(value_type = String, format = "string")]
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  #[serde(default, skip_serializing_if = "is_default")]
  #[builder(default)]
  pub request_params: OAIRequestParams,
  #[serde(default, skip_serializing_if = "JsonVec::is_empty")]
  #[builder(default)]
  pub context_params: JsonVec,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub created_at: chrono::DateTime<chrono::Utc>,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserAliasBuilder {
  pub fn build_with_time(
    &self,
    now: chrono::DateTime<chrono::Utc>,
  ) -> Result<UserAlias, BuilderError> {
    Ok(UserAlias {
      id: ulid::Ulid::new().to_string(),
      alias: self
        .alias
        .clone()
        .ok_or(BuilderError::UninitializedField("alias"))?,
      repo: self
        .repo
        .clone()
        .ok_or(BuilderError::UninitializedField("repo"))?,
      filename: self
        .filename
        .clone()
        .ok_or(BuilderError::UninitializedField("filename"))?,
      snapshot: self
        .snapshot
        .clone()
        .ok_or(BuilderError::UninitializedField("snapshot"))?,
      request_params: self.request_params.clone().unwrap_or_default(),
      context_params: self.context_params.clone().unwrap_or_default(),
      created_at: now,
      updated_at: now,
    })
  }
}

impl std::fmt::Display for UserAlias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Alias {{ alias: {}, repo: {}, filename: {}, snapshot: {} }}",
      self.alias, self.repo, self.filename, self.snapshot
    )
  }
}

// =============================================================================
// ModelAlias
// =============================================================================

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  ToSchema,
  PartialEq,
  derive_builder::Builder,
  derive_new::new,
)]
#[builder(
  setter(into, strip_option),
  build_fn(error = BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct ModelAlias {
  pub alias: String,
  #[schema(value_type = String, format = "string")]
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
}

impl ModelAlias {
  pub fn config_filename(&self) -> String {
    let filename = self.alias.replace(':', "--");
    to_safe_filename(&filename)
  }
}

impl std::fmt::Display for ModelAlias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ModelAlias {{ alias: {}, repo: {}, filename: {}, snapshot: {} }}",
      self.alias, self.repo, self.filename, self.snapshot
    )
  }
}

// =============================================================================
// ApiFormat + ApiAlias
// =============================================================================

/// TTL for API model cache (24 hours)
pub const CACHE_TTL_HOURS: i64 = 24;

/// API format/protocol specification
#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  ToSchema,
  PartialEq,
  strum::Display,
  strum::EnumString,
  sea_orm::DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ApiFormat {
  #[serde(rename = "openai")]
  #[strum(serialize = "openai")]
  #[cfg_attr(any(test, feature = "test-utils"), default)]
  OpenAI,
  #[serde(rename = "placeholder")]
  #[strum(serialize = "placeholder")]
  Placeholder,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, derive_builder::Builder)]
#[builder(setter(into, strip_option), build_fn(error = BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct ApiAlias {
  pub id: String,
  pub api_format: ApiFormat,
  pub base_url: String,
  #[builder(default)]
  pub models: JsonVec,
  #[builder(default)]
  pub prefix: Option<String>,
  #[builder(default)]
  pub forward_all_with_prefix: bool,
  #[builder(default)]
  pub models_cache: JsonVec,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub cache_fetched_at: chrono::DateTime<chrono::Utc>,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub created_at: chrono::DateTime<chrono::Utc>,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

fn epoch_sentinel() -> chrono::DateTime<chrono::Utc> {
  chrono::DateTime::UNIX_EPOCH
}

impl ApiAlias {
  pub fn new(
    id: impl Into<String>,
    api_format: ApiFormat,
    base_url: impl Into<String>,
    models: impl Into<JsonVec>,
    prefix: Option<String>,
    forward_all_with_prefix: bool,
    created_at: chrono::DateTime<chrono::Utc>,
  ) -> Self {
    Self {
      id: id.into(),
      api_format,
      base_url: base_url.into(),
      models: models.into(),
      prefix,
      forward_all_with_prefix,
      models_cache: JsonVec::default(),
      cache_fetched_at: epoch_sentinel(),
      created_at,
      updated_at: created_at,
    }
  }

  pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
    self.prefix = Some(prefix.into());
    self
  }

  /// Returns the appropriate models list based on the forward_all_with_prefix flag.
  /// - If forward_all_with_prefix=true: returns models_cache (unprefixed cached models)
  /// - If forward_all_with_prefix=false: returns models (user-specified models)
  pub fn get_models(&self) -> &[String] {
    if self.forward_all_with_prefix {
      &self.models_cache
    } else {
      &self.models
    }
  }

  pub fn matchable_models(&self) -> Vec<String> {
    let prefix = self.prefix.as_deref().unwrap_or("");

    let source: &[String] = if self.forward_all_with_prefix {
      &self.models_cache
    } else {
      &self.models
    };

    source
      .iter()
      .map(|model| format!("{}{}", prefix, model))
      .collect()
  }

  /// Check if this API alias supports a given model name.
  ///
  /// If `forward_all_with_prefix` is true, checks if the model starts with the prefix.
  /// If `forward_all_with_prefix` is false, checks if the model is in matchable_models list.
  pub fn supports_model(&self, model: &str) -> bool {
    if self.forward_all_with_prefix {
      self.prefix.as_ref().is_some_and(|p| model.starts_with(p))
    } else {
      self.matchable_models().contains(&model.to_string())
    }
  }

  /// Check if the cache is stale (older than TTL).
  pub fn is_cache_stale(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
    now - self.cache_fetched_at > chrono::Duration::hours(CACHE_TTL_HOURS)
  }

  /// Check if the cache is empty.
  pub fn is_cache_empty(&self) -> bool {
    self.models_cache.is_empty()
  }
}

impl ApiAliasBuilder {
  /// Build an ApiAlias with the provided timestamp for created_at and updated_at.
  ///
  /// This is the primary build method for ApiAlias since timestamps cannot be set through
  /// the builder's field setters (they are marked with `#[builder(setter(skip))]`).
  pub fn build_with_time(
    &self,
    timestamp: chrono::DateTime<chrono::Utc>,
  ) -> Result<ApiAlias, BuilderError> {
    Ok(ApiAlias {
      id: self
        .id
        .clone()
        .ok_or(BuilderError::UninitializedField("id"))?,
      api_format: self
        .api_format
        .clone()
        .ok_or(BuilderError::UninitializedField("api_format"))?,
      base_url: self
        .base_url
        .clone()
        .ok_or(BuilderError::UninitializedField("base_url"))?,
      models: self.models.clone().unwrap_or_default(),
      prefix: self.prefix.clone().unwrap_or_default(),
      forward_all_with_prefix: self.forward_all_with_prefix.unwrap_or_default(),
      models_cache: self.models_cache.clone().unwrap_or_default(),
      cache_fetched_at: epoch_sentinel(),
      created_at: timestamp,
      updated_at: timestamp,
    })
  }
}

impl std::fmt::Display for ApiAlias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ApiAlias {{ id: {}, api_format: {}, prefix: {:?}, models: {:?} }}",
      self.id, self.api_format, self.prefix, &*self.models
    )
  }
}

// =============================================================================
// Alias + AliasSource enum
// =============================================================================

/// Flat enum representing all types of model aliases
/// Each variant is identified by the source field
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(tag = "source", rename_all = "kebab-case")]
pub enum Alias {
  /// User-defined local model (source: "user")
  #[serde(rename = "user")]
  User(UserAlias),
  /// Auto-discovered local model (source: "model")
  #[serde(rename = "model")]
  Model(ModelAlias),
  /// Remote API model (source: "api")
  #[serde(rename = "api")]
  Api(ApiAlias),
}

impl Alias {
  /// Check if this alias can serve the requested model
  pub fn can_serve(&self, model: &str) -> bool {
    match self {
      Alias::User(alias) => alias.alias == model,
      Alias::Model(alias) => alias.alias == model,
      Alias::Api(api_alias) => api_alias.supports_model(model),
    }
  }

  /// Get the alias name for this model
  pub fn alias_name(&self) -> &str {
    match self {
      Alias::User(alias) => &alias.alias,
      Alias::Model(alias) => &alias.alias,
      Alias::Api(api_alias) => &api_alias.id,
    }
  }

  /// Get the source of this alias
  pub fn source(&self) -> AliasSource {
    match self {
      Alias::User(_) => AliasSource::User,
      Alias::Model(_) => AliasSource::Model,
      Alias::Api(_) => AliasSource::Api,
    }
  }
}

// =============================================================================
// ModelMetadata and related types
// =============================================================================

/// Model metadata for API responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ModelMetadata {
  pub capabilities: ModelCapabilities,
  pub context: ContextLimits,
  pub architecture: ModelArchitecture,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub chat_template: Option<String>,
}

#[derive(
  Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, sea_orm::FromJsonQueryResult,
)]
pub struct ModelCapabilities {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub vision: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub audio: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub thinking: Option<bool>,
  pub tools: ToolCapabilities,
}

#[derive(
  Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, sea_orm::FromJsonQueryResult,
)]
pub struct ToolCapabilities {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub function_calling: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub structured_output: Option<bool>,
}

#[derive(
  Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, sea_orm::FromJsonQueryResult,
)]
pub struct ContextLimits {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_input_tokens: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_output_tokens: Option<u64>,
}

#[derive(
  Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, sea_orm::FromJsonQueryResult,
)]
pub struct ModelArchitecture {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub family: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parameter_count: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quantization: Option<String>,
  pub format: String,
}

#[cfg(test)]
#[path = "test_model_objs.rs"]
mod test_model_objs;
