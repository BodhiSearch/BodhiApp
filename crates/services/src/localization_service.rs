use fluent::{concurrent::FluentBundle, FluentArgs, FluentResource};
use include_dir::Dir;
use objs::{AppError, ErrorType};
use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};
use thiserror::Error;
use unic_langid::LanguageIdentifier;

pub trait LocalizationService: std::fmt::Debug + Send + Sync {
  fn get_message(
    &self,
    locale: &LanguageIdentifier,
    code: &str,
    args: HashMap<String, String>,
  ) -> Result<Option<String>, LocalizationError>;
}

#[derive(Debug, Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LocalizationError {
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "localization_error-fluent_error", args_delegate = false)]
  FluentError(#[from] fluent::FluentError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "localization_error-unic_langid_error", args_delegate = false)]
  UnicLangidError(#[from] unic_langid::LanguageIdentifierError),
  #[error("rwlock_write")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "localization_error-rwlock_write")]
  RwLockWrite(String),
  #[error("rwlock_read")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "localization_error-rwlock_read")]
  RwLockRead(String),
  #[error("message_not_found")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "localization_error-message_not_found")]
  MessageNotFound(String),
  #[error("format_pattern")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "localization_error-format_pattern")]
  FormatPattern(String),
  #[error("locale_not_supported")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "localization_error-locale_not_supported")]
  LocaleNotSupported(String),
}

pub struct FluentLocalizationService {
  bundles: RwLock<HashMap<LanguageIdentifier, Arc<FluentBundle<FluentResource>>>>,
}

impl FluentLocalizationService {
  fn new() -> Self {
    Self {
      bundles: RwLock::new(HashMap::new()),
    }
  }

  pub fn with_locale_resources(embedded_dir: &Dir) -> Result<Self, LocalizationError> {
    let slf = Self::new();
    for locale_dir in embedded_dir.entries() {
      match locale_dir.as_dir() {
        Some(locale_dir) => {
          let locale_name = locale_dir
            .path()
            .display()
            .to_string()
            .trim_end_matches('/')
            .to_string();
          let lang_id = match locale_name.parse::<LanguageIdentifier>() {
            Ok(lang_id) => lang_id,
            Err(err) => {
              tracing::warn!(?err, "invalid locale, skipping: {}", locale_name);
              continue;
            }
          };
          let resources = locale_dir
            .files()
            .filter_map(|file| {
              if file
                .path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("ftl"))
                .unwrap_or(false)
              {
                if let Some(content) = file.contents_utf8() {
                  Some(content.to_string())
                } else {
                  tracing::warn!("failed to read file, skipping: {:?}", file.path());
                  None
                }
              } else {
                tracing::warn!("skipping non-ftl file: {:?}", file.path());
                None
              }
            })
            .collect::<Vec<String>>();
          // Load the locale resources into the service
          slf.load_locale(lang_id, resources)?;
        }
        None => {
          tracing::warn!("skipping non-directory: {:?}", locale_dir.path());
          continue;
        }
      }
    }
    Ok(slf)
  }

  /// Loads Fluent resources for the given locale.
  pub fn load_locale(
    &self,
    locale: LanguageIdentifier,
    resources: Vec<String>,
  ) -> Result<(), LocalizationError> {
    let mut bundle = FluentBundle::new_concurrent(vec![locale.clone()]);
    for resource_str in resources {
      let ftl_resource = match FluentResource::try_new(resource_str) {
        Ok(ftl_resource) => ftl_resource,
        Err((source, errors)) => {
          tracing::warn!(?source, ?errors, "failed to parse resource, skipping");
          continue;
        }
      };
      if let Err(err) = bundle.add_resource(ftl_resource) {
        tracing::warn!(?err, "failed to add resource, skipping");
      };
    }

    let mut bundles = self
      .bundles
      .write()
      .map_err(|err| LocalizationError::RwLockWrite(err.to_string()))?;
    bundles.insert(locale, Arc::new(bundle));
    Ok(())
  }

  /// Retrieves a localized message based on locale and message code.
  pub fn get_message(
    &self,
    locale: &LanguageIdentifier,
    code: &str,
    args: Option<HashMap<String, String>>,
  ) -> Result<Option<String>, LocalizationError> {
    let args: Option<FluentArgs> = args.map(|args| {
      let mut fluent_args = FluentArgs::new();
      for (key, value) in args {
        fluent_args.set(key, value);
      }
      fluent_args
    });
    let bundles = self
      .bundles
      .read()
      .map_err(|err| LocalizationError::RwLockRead(err.to_string()))?;
    let bundle = bundles
      .get(locale)
      .ok_or_else(|| LocalizationError::MessageNotFound(code.to_string()))?;
    let message = bundle
      .get_message(code)
      .ok_or_else(|| LocalizationError::MessageNotFound(code.to_string()))?;
    let pattern = message
      .value()
      .ok_or_else(|| LocalizationError::FormatPattern(code.to_string()))?;
    let mut errors = vec![];
    let result = bundle.format_pattern(pattern, args.as_ref(), &mut errors);
    if errors.is_empty() {
      Ok(Some(result.to_string()))
    } else {
      Err(LocalizationError::FormatPattern(
        errors
          .iter()
          .map(|err| err.to_string())
          .collect::<Vec<String>>()
          .join(", "),
      ))
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::localization_service::FluentLocalizationService;
  use include_dir::{include_dir, Dir};
  use rstest::*;
  use unic_langid::LanguageIdentifier;
  use std::collections::HashMap;

  static RESOURCES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/resources");

  #[fixture]
  fn localization_service() -> FluentLocalizationService {
    FluentLocalizationService::with_locale_resources(&RESOURCES_DIR).unwrap()
  }

  #[rstest]
  #[case("en-US", "hello-world", None, "Hello, World!")]
  #[case(
    "en-US",
    "hello-world-args",
    Some(HashMap::from([("name".to_string(), "Alice".to_string())])),
    "Hello, \u{2068}Alice\u{2069}!"
  )]
  #[case("fr-FR", "hello-world", None, "Bonjour, le monde !")]
  #[case(
    "fr-FR",
    "hello-world-args",
    Some(HashMap::from([("name".to_string(), "Alice".to_string())])),
    "Bonjour, \u{2068}Alice\u{2069} !"
  )]
  fn test_localization_service(
    localization_service: FluentLocalizationService,
    #[case] locale: &str,
    #[case] message_key: &str,
    #[case] args: Option<HashMap<String, String>>,
    #[case] expected: &str,
  ) -> Result<(), anyhow::Error> {
    let lang_id: LanguageIdentifier = locale.parse()?;
    let result = localization_service.get_message(&lang_id, message_key, args)?;
    assert_eq!(result, Some(expected.to_string()));

    Ok(())
  }
}
