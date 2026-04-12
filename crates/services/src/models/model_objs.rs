use crate::models::anthropic_model::AnthropicModel;
use crate::shared_objs::{is_default, to_safe_filename};
use async_openai::types::models::Model as OpenAIModel;
use chrono::{DateTime, Utc};
use derive_builder::UninitializedFieldError;
use errmeta::{AppError, ErrorType};
use once_cell::sync::Lazy;
use regex::Regex;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{
  collections::HashMap,
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
  #[error_meta(error_type = ErrorType::BadRequest)]
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
// ApiModel + ApiModelVec
// =============================================================================

/// Discriminated union of provider-specific model metadata.
/// Stored as JSON in `api_model_aliases.models`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, FromJsonQueryResult)]
#[serde(tag = "provider")]
pub enum ApiModel {
  #[serde(rename = "openai")]
  OpenAI(OpenAIModel),
  #[serde(rename = "anthropic")]
  Anthropic(AnthropicModel),
}

impl ApiModel {
  pub fn id(&self) -> &str {
    match self {
      ApiModel::OpenAI(m) => &m.id,
      ApiModel::Anthropic(m) => &m.id,
    }
  }
}

/// DB-storable `Vec<ApiModel>` — stored as JSON binary in SeaORM columns.
#[derive(
  Clone, Debug, PartialEq, Default, Serialize, Deserialize, FromJsonQueryResult, ToSchema,
)]
pub struct ApiModelVec(Vec<ApiModel>);

impl ApiModelVec {
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

impl Deref for ApiModelVec {
  type Target = Vec<ApiModel>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ApiModelVec {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<Vec<ApiModel>> for ApiModelVec {
  fn from(v: Vec<ApiModel>) -> Self {
    Self(v)
  }
}

impl From<ApiModelVec> for Vec<ApiModel> {
  fn from(v: ApiModelVec) -> Self {
    v.0
  }
}

impl FromIterator<ApiModel> for ApiModelVec {
  fn from_iter<I: IntoIterator<Item = ApiModel>>(iter: I) -> Self {
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
      id: crate::new_ulid(),
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
// UserAliasRequest (input for create and update)
// =============================================================================

/// Input request for creating or updating a user model alias.
// Used as `ValidatedJson<UserAliasRequest>` in handlers for both create and update (PUT).
// `alias` is required — unique per (tenant_id, user_id) scope, and updatable.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, derive_builder::Builder)]
#[schema(example = json!({
    "alias": "llama3:instruct",
    "repo": "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
    "filename": "Meta-Llama-3-8B-Instruct.Q4_K_M.gguf",
    "request_params": { "temperature": 0.7 },
    "context_params": ["--ctx-size 4096"]
}))]
#[builder(setter(into, strip_option), build_fn(error = BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct UserAliasRequest {
  /// Alias name — unique per (tenant_id, user_id) scope
  pub alias: String,

  /// Repository in format "user/repo"
  pub repo: String,

  /// Filename of the GGUF model
  pub filename: String,

  /// Snapshot/commit identifier (optional — defaults to latest available)
  #[serde(default)]
  #[builder(default)]
  pub snapshot: Option<String>,

  /// Request parameters for the model
  #[serde(default)]
  #[builder(default)]
  pub request_params: Option<OAIRequestParams>,

  /// Context parameters for the model
  #[serde(default)]
  #[builder(default)]
  pub context_params: Option<Vec<String>>,
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
  #[serde(rename = "openai_responses")]
  #[strum(serialize = "openai_responses")]
  OpenAIResponses,
  #[serde(rename = "anthropic")]
  #[strum(serialize = "anthropic")]
  Anthropic,
  #[serde(rename = "anthropic_oauth")]
  #[strum(serialize = "anthropic_oauth")]
  AnthropicOAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, derive_builder::Builder)]
#[builder(setter(into, strip_option), build_fn(error = BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct ApiAlias {
  pub id: String,
  pub api_format: ApiFormat,
  pub base_url: String,
  #[builder(default)]
  pub models: ApiModelVec,
  #[builder(default)]
  pub prefix: Option<String>,
  #[builder(default)]
  pub forward_all_with_prefix: bool,
  #[builder(default)]
  pub extra_headers: Option<serde_json::Value>,
  #[builder(default)]
  pub extra_body: Option<serde_json::Value>,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub created_at: chrono::DateTime<chrono::Utc>,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ApiAlias {
  pub fn new(
    id: impl Into<String>,
    api_format: ApiFormat,
    base_url: impl Into<String>,
    models: impl Into<ApiModelVec>,
    prefix: Option<String>,
    forward_all_with_prefix: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    extra_headers: Option<serde_json::Value>,
    extra_body: Option<serde_json::Value>,
  ) -> Self {
    Self {
      id: id.into(),
      api_format,
      base_url: base_url.into(),
      models: models.into(),
      prefix,
      forward_all_with_prefix,
      extra_headers,
      extra_body,
      created_at,
      updated_at: created_at,
    }
  }

  pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
    self.prefix = Some(prefix.into());
    self
  }

  /// Returns the models list. All models (whether user-selected or all provider models
  /// for forward_all) are stored in the `models` field.
  pub fn get_models(&self) -> &[ApiModel] {
    &self.models
  }

  pub fn matchable_models(&self) -> Vec<String> {
    let prefix = self.prefix.as_deref().unwrap_or("");
    self
      .models
      .iter()
      .map(|model| format!("{}{}", prefix, model.id()))
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
      extra_headers: self.extra_headers.clone().unwrap_or_default(),
      extra_body: self.extra_body.clone().unwrap_or_default(),
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

// =============================================================================
// ApiKey (validated wrapper)
// =============================================================================

/// Validated API key wrapper - validates length when Some, allows None for public APIs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
#[serde(transparent)]
pub struct ApiKey(Option<String>);

impl ApiKey {
  /// Create ApiKey with no authentication
  pub fn none() -> Self {
    ApiKey(None)
  }

  /// Create ApiKey with validation
  pub fn some(key: String) -> Result<Self, validator::ValidationError> {
    if key.is_empty() {
      let mut err = validator::ValidationError::new("api_key_empty");
      err.message = Some("API key must not be empty".into());
      return Err(err);
    }
    if key.len() > 4096 {
      let mut err = validator::ValidationError::new("api_key_too_long");
      err.message =
        Some(format!("API key must not exceed 4096 characters, got {}", key.len()).into());
      return Err(err);
    }
    Ok(ApiKey(Some(key)))
  }

  /// Get as Option<&str>
  pub fn as_option(&self) -> Option<&str> {
    self.0.as_deref()
  }

  /// Into inner Option<String>
  pub fn into_inner(self) -> Option<String> {
    self.0
  }

  /// Check if None
  pub fn is_none(&self) -> bool {
    self.0.is_none()
  }

  /// Check if Some
  pub fn is_some(&self) -> bool {
    self.0.is_some()
  }
}

/// Custom deserializer to validate on deserialization
impl<'de> Deserialize<'de> for ApiKey {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
      None => Ok(ApiKey::none()),
      Some(key) => ApiKey::some(key).map_err(serde::de::Error::custom),
    }
  }
}

// =============================================================================
// ApiKeyUpdate (with serde support for both services and routes)
// =============================================================================

/// Represents an API key update operation for API model aliases.
// Uses tagged enum for JSON: `{"action": "keep"}` or `{"action": "set", "value": "sk-..."}`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
#[serde(tag = "action", content = "value", rename_all = "lowercase")]
pub enum ApiKeyUpdate {
  /// Keep the existing API key unchanged
  #[cfg_attr(any(test, feature = "test-utils"), default)]
  Keep,
  /// Set a new API key (or add one if none exists) - ApiKey validates length
  Set(ApiKey),
}

impl ApiKeyUpdate {
  /// Convert to the raw Option<Option<String>> for DB operations.
  /// Returns None for Keep (don't change), Some(None) for Set(None) (clear), Some(Some(key)) for Set(Some(key)).
  pub fn into_raw(self) -> Option<Option<String>> {
    match self {
      ApiKeyUpdate::Keep => None,
      ApiKeyUpdate::Set(key) => Some(key.into_inner()),
    }
  }

  /// Convert for repository layer that takes raw ApiKeyUpdate
  pub fn into_raw_update(self) -> RawApiKeyUpdate {
    match self {
      ApiKeyUpdate::Keep => RawApiKeyUpdate::Keep,
      ApiKeyUpdate::Set(key) => RawApiKeyUpdate::Set(key.into_inner()),
    }
  }
}

/// Raw API key update for repository layer (no ApiKey validation wrapper)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawApiKeyUpdate {
  /// Keep the existing API key unchanged
  Keep,
  /// Set a new API key (or clear if None)
  Set(Option<String>),
}

// =============================================================================
// ApiModelRequest (input for create and update)
// =============================================================================

/// Input request for creating or updating an API model configuration.
// Used as `ValidatedJson<ApiModelRequest>` in handlers for both create and update (PUT).
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, derive_builder::Builder)]
#[schema(example = json!({
    "api_format": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": {"action": "set", "value": "sk-..."},
    "models": ["gpt-4", "gpt-3.5-turbo"],
    "prefix": "openai"
}))]
#[builder(setter(into, strip_option), build_fn(error = BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct ApiModelRequest {
  /// API format/protocol (e.g., "openai")
  pub api_format: ApiFormat,

  /// API base URL
  #[validate(custom(function = "crate::validate_http_url"))]
  pub base_url: String,

  /// API key update action (Keep/Set with Some or None)
  #[serde(default = "default_api_key_keep")]
  #[builder(default = "default_api_key_keep()")]
  pub api_key: ApiKeyUpdate,

  /// List of available models
  #[builder(default)]
  pub models: Vec<String>,

  /// Optional prefix for model namespacing (e.g., "azure/" for "azure/gpt-4")
  #[builder(default)]
  pub prefix: Option<String>,

  /// Whether to forward all requests with this prefix (true) or only selected models (false)
  #[serde(default)]
  #[builder(default)]
  pub forward_all_with_prefix: bool,

  /// Optional extra HTTP headers to send upstream. Cannot include `Authorization`
  /// or `x-api-key` — those are owned by provider clients.
  #[serde(default)]
  #[builder(default)]
  #[validate(custom(function = "crate::validate_extra_headers_no_auth"))]
  pub extra_headers: Option<serde_json::Value>,

  /// Optional extra fields to merge into the request body sent upstream
  #[serde(default)]
  #[builder(default)]
  pub extra_body: Option<serde_json::Value>,
}

fn default_api_key_keep() -> ApiKeyUpdate {
  ApiKeyUpdate::Keep
}

// =============================================================================
// TestCreds (credential specification for test/fetch operations)
// =============================================================================

/// Credentials for test/fetch operations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum TestCreds {
  /// Look up credentials from stored API model
  #[schema(example = json!({"type": "id", "value": "openai-gpt4"}))]
  Id(String),

  /// Use direct API key (null for no authentication)
  #[schema(example = json!({"type": "api_key", "value": "sk-1234567890abcdef"}))]
  ApiKey(ApiKey),
}

impl Default for TestCreds {
  fn default() -> Self {
    TestCreds::ApiKey(ApiKey::none())
  }
}

fn default_api_format_openai() -> ApiFormat {
  ApiFormat::OpenAI
}

// =============================================================================
// TestPromptRequest / TestPromptResponse
// =============================================================================

/// Request to test API connectivity with a prompt
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "creds": {"type": "api_key", "value": "sk-..."},
    "base_url": "https://api.openai.com/v1",
    "model": "gpt-4",
    "prompt": "Hello, how are you?"
}))]
pub struct TestPromptRequest {
  /// Credentials to use for testing
  #[serde(default)]
  pub creds: TestCreds,

  /// API base URL
  #[validate(custom(function = "crate::validate_http_url"))]
  pub base_url: String,

  /// Model to use for testing
  #[validate(length(min = 1))]
  pub model: String,

  /// Test prompt (max 30 characters for cost control)
  #[validate(length(min = 1, max = 30))]
  pub prompt: String,

  /// API format to use for the test request (defaults to OpenAI Chat Completions)
  #[serde(default = "default_api_format_openai")]
  pub api_format: ApiFormat,

  /// Optional extra HTTP headers. `Authorization` / `x-api-key` are forbidden.
  #[serde(default)]
  #[validate(custom(function = "crate::validate_extra_headers_no_auth"))]
  pub extra_headers: Option<serde_json::Value>,

  /// Optional extra fields to merge into the request body
  #[serde(default)]
  pub extra_body: Option<serde_json::Value>,
}

/// Response from testing API connectivity
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "success": true,
    "response": "Hello! I'm doing well, thank you.",
    "error": null
}))]
pub struct TestPromptResponse {
  pub success: bool,
  pub response: Option<String>,
  pub error: Option<String>,
}

impl TestPromptResponse {
  pub fn success(response: String) -> Self {
    Self {
      success: true,
      response: Some(response),
      error: None,
    }
  }

  pub fn failure(error: String) -> Self {
    Self {
      success: false,
      response: None,
      error: Some(error),
    }
  }
}

// =============================================================================
// FetchModelsRequest / FetchModelsResponse
// =============================================================================

/// Request to fetch available models from provider
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "creds": {"type": "api_key", "value": null},
    "base_url": "http://localhost:8080/v1",
    "api_format": "openai"
}))]
pub struct FetchModelsRequest {
  /// Credentials to use for fetching models
  #[serde(default)]
  pub creds: TestCreds,

  /// API base URL (required - always needed to know where to fetch models from)
  #[validate(custom(function = "crate::validate_http_url"))]
  pub base_url: String,

  /// API format to use for fetching models (defaults to OpenAI Chat Completions)
  #[serde(default = "default_api_format_openai")]
  pub api_format: ApiFormat,

  /// Optional extra HTTP headers. `Authorization` / `x-api-key` are forbidden.
  #[serde(default)]
  #[validate(custom(function = "crate::validate_extra_headers_no_auth"))]
  pub extra_headers: Option<serde_json::Value>,

  /// Optional extra fields to merge into the request body
  #[serde(default)]
  pub extra_body: Option<serde_json::Value>,
}

/// Returns model IDs only (not full metadata) to minimize information exposure —
/// the endpoint accepts an API key parameter. Full metadata is stored on create/update.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "models": ["gpt-4", "gpt-3.5-turbo", "gpt-4-turbo"]
}))]
pub struct FetchModelsResponse {
  pub models: Vec<String>,
}

// =============================================================================
// ApiFormatsResponse
// =============================================================================

/// Response containing available API formats
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "data": ["openai"]
}))]
pub struct ApiFormatsResponse {
  pub data: Vec<ApiFormat>,
}

// =============================================================================
// CopyAliasRequest
// =============================================================================

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CopyAliasRequest {
  pub alias: String,
}

// =============================================================================
// Refresh types
// =============================================================================

/// Source type discriminator for refresh requests
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RefreshSource {
  /// Refresh all local GGUF models (async)
  All,
  /// Refresh specific GGUF model (sync)
  Model,
}

/// Refresh request - discriminated union by source field
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(tag = "source", rename_all = "lowercase")]
pub enum RefreshRequest {
  /// Bulk async refresh for all models - Request: {"source": "all"}
  All {},
  /// Single sync refresh for specific model - Request: {"source": "model", "repo": "...", "filename": "...", "snapshot": "..."}
  Model {
    /// Repository in format "user/repo"
    #[schema(example = "bartowski/Qwen2.5-3B-Instruct-GGUF")]
    repo: String,
    /// Filename of the GGUF model
    #[schema(example = "Qwen2.5-3B-Instruct-Q4_K_M.gguf")]
    filename: String,
    /// Snapshot/commit identifier
    #[schema(example = "8ba1c3c3ee94ba4b86ff92a749ae687dc41fce3f")]
    snapshot: String,
  },
}

/// Response for metadata refresh operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshResponse {
  /// Number of models queued ("all" for bulk refresh, "1" for single)
  pub num_queued: String,
  /// Model alias (only for single model refresh)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub alias: Option<String>,
}

// =============================================================================
// Alias Response types (moved from routes_app)
// =============================================================================

/// User-defined model alias response
#[allow(clippy::too_many_arguments)]
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, derive_new::new, ToSchema)]
#[cfg_attr(any(test, feature = "test-utils"), derive(derive_builder::Builder))]
#[cfg_attr(
  any(test, feature = "test-utils"),
  builder(
    setter(into),
    build_fn(error = BuilderError)))]
pub struct UserAliasResponse {
  pub id: String,
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: String,
  pub source: String,

  pub model_params: HashMap<String, Value>,
  pub request_params: OAIRequestParams,
  pub context_params: Vec<String>,

  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,

  /// Model metadata extracted from GGUF file (optional)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[cfg_attr(any(test, feature = "test-utils"), builder(default))]
  pub metadata: Option<ModelMetadata>,
}

impl From<UserAlias> for UserAliasResponse {
  fn from(alias: UserAlias) -> Self {
    UserAliasResponse {
      id: alias.id,
      repo: alias.repo.to_string(),
      filename: alias.filename,
      snapshot: alias.snapshot,
      alias: alias.alias,
      source: "user".to_string(),

      model_params: HashMap::new(),
      request_params: alias.request_params,
      context_params: alias.context_params.into(),
      created_at: alias.created_at,
      updated_at: alias.updated_at,
      metadata: None,
    }
  }
}

impl UserAliasResponse {
  /// Attach model metadata to this response
  pub fn with_metadata(mut self, metadata: Option<ModelMetadata>) -> Self {
    self.metadata = metadata;
    self
  }
}

/// Paginated list of user-defined model aliases
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedUserAliasResponse {
  pub data: Vec<UserAliasResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

/// Response for auto-discovered model aliases
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, ToSchema)]
pub struct ModelAliasResponse {
  pub source: String,
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: String,

  /// Model metadata extracted from GGUF file (optional)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<ModelMetadata>,
}

impl From<ModelAlias> for ModelAliasResponse {
  fn from(alias: ModelAlias) -> Self {
    Self {
      source: "model".to_string(),
      alias: alias.alias,
      repo: alias.repo.to_string(),
      filename: alias.filename,
      snapshot: alias.snapshot,
      metadata: None,
    }
  }
}

impl ModelAliasResponse {
  /// Attach model metadata to this response
  pub fn with_metadata(mut self, metadata: Option<ModelMetadata>) -> Self {
    self.metadata = metadata;
    self
  }
}

/// API response for API model aliases - hides internal cache fields
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, ToSchema)]
pub struct ApiAliasResponse {
  pub source: String,
  pub id: String,
  pub api_format: ApiFormat,
  pub base_url: String,
  pub has_api_key: bool,
  /// Models available through this alias with full provider metadata
  pub models: Vec<ApiModel>,
  pub prefix: Option<String>,
  pub forward_all_with_prefix: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extra_headers: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extra_body: Option<serde_json::Value>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl From<ApiAlias> for ApiAliasResponse {
  fn from(alias: ApiAlias) -> Self {
    let models = alias.get_models().to_vec();
    Self {
      source: "api".to_string(),
      id: alias.id,
      api_format: alias.api_format,
      base_url: alias.base_url,
      has_api_key: false,
      models,
      prefix: alias.prefix,
      forward_all_with_prefix: alias.forward_all_with_prefix,
      extra_headers: alias.extra_headers,
      extra_body: alias.extra_body,
      created_at: alias.created_at,
      updated_at: alias.updated_at,
    }
  }
}

impl ApiAliasResponse {
  pub fn with_has_api_key(mut self, v: bool) -> Self {
    self.has_api_key = v;
    self
  }
}

/// Response envelope for model aliases - hides internal implementation details
/// Uses untagged serialization - each variant has its own "source" field
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum AliasResponse {
  /// User-defined local model (source: "user")
  User(UserAliasResponse),
  /// Auto-discovered local model (source: "model")
  Model(ModelAliasResponse),
  /// Remote API model (source: "api")
  Api(ApiAliasResponse),
}

impl From<Alias> for AliasResponse {
  fn from(alias: Alias) -> Self {
    match alias {
      Alias::User(u) => AliasResponse::User(u.into()),
      Alias::Model(m) => AliasResponse::Model(m.into()),
      Alias::Api(a) => AliasResponse::Api(a.into()),
    }
  }
}

impl AliasResponse {
  /// Attach model metadata to this response (only applies to User and Model variants)
  pub fn with_metadata(self, metadata: Option<ModelMetadata>) -> Self {
    match self {
      AliasResponse::User(r) => AliasResponse::User(r.with_metadata(metadata)),
      AliasResponse::Model(r) => AliasResponse::Model(r.with_metadata(metadata)),
      AliasResponse::Api(r) => AliasResponse::Api(r), // API aliases don't have metadata
    }
  }
}

/// Paginated list of all model aliases (user, model, and API)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct PaginatedAliasResponse {
  pub data: Vec<AliasResponse>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

// =============================================================================
// From<DownloadRequestEntity> for DownloadRequest
// =============================================================================

impl From<super::DownloadRequestEntity> for super::DownloadRequest {
  fn from(e: super::DownloadRequestEntity) -> Self {
    Self {
      id: e.id,
      repo: e.repo,
      filename: e.filename,
      status: e.status,
      error: e.error,
      total_bytes: e.total_bytes,
      downloaded_bytes: e.downloaded_bytes,
      started_at: e.started_at,
      created_at: e.created_at,
      updated_at: e.updated_at,
    }
  }
}

#[cfg(test)]
#[path = "test_model_objs.rs"]
mod test_model_objs;
