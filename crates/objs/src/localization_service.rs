use crate::{AppError, ErrorType};
use fluent::{concurrent::FluentBundle, FluentArgs, FluentResource};
use include_dir::Dir;
use std::{
  collections::HashMap,
  str::FromStr,
  sync::{LazyLock, RwLock},
};
use thiserror::Error;
use unic_langid::LanguageIdentifier;

pub trait LocalizationService: std::fmt::Debug + Send + Sync {
  fn get_message(
    &self,
    locale: &LanguageIdentifier,
    code: &str,
    args: Option<HashMap<String, String>>,
  ) -> Result<String, LocalizationMessageError>;
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("locale_not_supported")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest, status = 400)]
pub struct LocaleNotSupportedError {
  locale: String,
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("l10n_rwlock_read")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest, status = 400)]
pub struct RwLockReadError {
  reason: String,
}

#[derive(Debug, PartialEq, Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LocalizationMessageError {
  #[error(transparent)]
  RwLockRead(#[from] RwLockReadError),
  #[error("message_not_found")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "localization_error-message_not_found")]
  MessageNotFound(String),
  #[error("format_pattern")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "localization_error-format_pattern")]
  FormatPattern(String),
  #[error(transparent)]
  LocaleNotSupported(#[from] LocaleNotSupportedError),
}

#[derive(Debug, PartialEq, Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LocalizationSetupError {
  #[error("rwlock_write")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, code = "localization_error-rwlock_write")]
  RwLockWrite(String),
  #[error(transparent)]
  RwLockRead(#[from] RwLockReadError),
  #[error(transparent)]
  LocaleNotSupported(#[from] LocaleNotSupportedError),
}

// basic support for locales
pub static SUPPORTED_LOCALES: LazyLock<Vec<LanguageIdentifier>> = LazyLock::new(|| {
  vec![
    LanguageIdentifier::from_str("en-US").unwrap(),
    LanguageIdentifier::from_str("fr-FR").unwrap(),
  ]
});

pub struct FluentLocalizationService {
  bundles: RwLock<HashMap<LanguageIdentifier, FluentBundle<FluentResource>>>,
}

impl std::fmt::Debug for FluentLocalizationService {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let bundles = self.bundles.read().unwrap();
    let locales = bundles.keys().collect::<Vec<_>>();
    write!(f, "FluentLocalizationService {{ locales: {:?} }}", locales)
  }
}

#[cfg(all(not(test), not(feature = "test-utils")))]
mod impl_localization_service {
  use super::*;
  use std::sync::{Arc, Once};

  impl FluentLocalizationService {
    pub fn get_instance() -> Arc<FluentLocalizationService> {
      static INSTANCE: Once = Once::new();
      static mut SERVICE: Option<Arc<FluentLocalizationService>> = None;

      INSTANCE.call_once(|| {
        let service = Arc::new(FluentLocalizationService::new());
        unsafe {
          SERVICE = Some(Arc::clone(&service));
        }
      });

      unsafe { Arc::clone(SERVICE.as_ref().unwrap()) }
    }
  }
}

impl FluentLocalizationService {
  pub(super) fn new() -> Self {
    Self {
      bundles: RwLock::new(HashMap::new()),
    }
  }

  pub fn load_resource(&self, embedded_dir: &Dir) -> Result<&Self, LocalizationSetupError> {
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
          let mut resources = locale_dir.files().collect::<Vec<_>>();
          resources.sort_by(|file1, file2| {
            file1
              .path()
              .display()
              .to_string()
              .cmp(&file2.path().display().to_string())
          });
          let resources = resources
            .into_iter()
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
          self.load_locale(lang_id, resources)?;
        }
        None => {
          tracing::warn!("skipping non-directory: {:?}", locale_dir.path());
          continue;
        }
      }
    }
    Ok(self)
  }

  pub fn load_locale(
    &self,
    locale: LanguageIdentifier,
    resources: Vec<String>,
  ) -> Result<(), LocalizationSetupError> {
    if !SUPPORTED_LOCALES.contains(&locale) {
      return Err(LocaleNotSupportedError::new(locale.to_string()))?;
    }
    let mut bundles = self
      .bundles
      .write()
      .map_err(|err| RwLockReadError::new(err.to_string()))?;
    let bundle = match bundles.get_mut(&locale) {
      Some(bundle) => bundle,
      None => {
        let bundle = FluentBundle::new_concurrent(vec![locale.clone()]);
        bundles.insert(locale.clone(), bundle);
        bundles.get_mut(&locale).unwrap()
      }
    };
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
    Ok(())
  }
}

impl LocalizationService for FluentLocalizationService {
  fn get_message(
    &self,
    locale: &LanguageIdentifier,
    code: &str,
    args: Option<HashMap<String, String>>,
  ) -> Result<String, LocalizationMessageError> {
    if !SUPPORTED_LOCALES.contains(locale) {
      return Err(LocaleNotSupportedError::new(locale.to_string()))?;
    }
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
      .map_err(|err| RwLockReadError::new(err.to_string()))?;
    let bundle = bundles
      .get(locale)
      .ok_or_else(|| LocaleNotSupportedError::new(code.to_string()))?;
    let message = bundle
      .get_message(code)
      .ok_or_else(|| LocalizationMessageError::MessageNotFound(code.to_string()))?;
    let pattern = message
      .value()
      .ok_or_else(|| LocalizationMessageError::FormatPattern(code.to_string()))?;
    let mut errors = vec![];
    let result = bundle.format_pattern(pattern, args.as_ref(), &mut errors);
    if errors.is_empty() {
      Ok(result.to_string())
    } else {
      Err(LocalizationMessageError::FormatPattern(
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
  use super::FluentLocalizationService;
  use crate::test_utils::{localization_service, setup_l10n};
  use crate::{
    LocaleNotSupportedError, LocalizationMessageError, LocalizationService, LocalizationSetupError,
  };
  use include_dir::{include_dir, Dir};
  use rstest::*;
  use std::str::FromStr;
  use std::{collections::HashMap, sync::Arc};
  use unic_langid::LanguageIdentifier;

  static RESOURCES_DIR_0: Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/resources-0");
  static RESOURCES_DIR_1: Dir = include_dir!("$CARGO_MANIFEST_DIR/tests/resources-1");
  static RESOURCES_UNSUPPORTED: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/tests/resources-unsupported");

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
  fn test_localization_service_loads_resource_dir(
    localization_service: Arc<FluentLocalizationService>,
    #[case] locale: &str,
    #[case] message_key: &str,
    #[case] args: Option<HashMap<String, String>>,
    #[case] expected: &str,
  ) -> Result<(), anyhow::Error> {
    localization_service
      .load_resource(&RESOURCES_DIR_0)
      .unwrap();
    let lang_id: LanguageIdentifier = locale.parse()?;
    let result = localization_service.get_message(&lang_id, message_key, args)?;
    assert_eq!(result, expected.to_string());
    Ok(())
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
  fn test_localization_multiple_resource_dirs(
    localization_service: Arc<FluentLocalizationService>,
    #[case] locale: &str,
    #[case] message_key: &str,
    #[case] args: Option<HashMap<String, String>>,
    #[case] expected: &str,
  ) -> anyhow::Result<()> {
    localization_service
      .load_resource(&RESOURCES_DIR_0)
      .unwrap();
    localization_service
      .load_resource(&RESOURCES_DIR_1)
      .unwrap();
    let lang_id: LanguageIdentifier = locale.parse()?;
    let result = localization_service.get_message(&lang_id, message_key, args)?;
    assert_eq!(result, expected.to_string());
    Ok(())
  }

  #[rstest]
  #[case("fr-FR", "hello-world", None, "Hello, World!")]
  #[case(
    "fr-FR",
    "hello-world-args",
    Some(HashMap::from([("name".to_string(), "Alice".to_string())])),
    "Hello, \u{2068}Alice\u{2069}!"
  )]
  #[case("en-US", "hello-world", None, "Bonjour, le monde !")]
  #[case(
    "en-US",
    "hello-world-args",
    Some(HashMap::from([("name".to_string(), "Alice".to_string())])),
    "Bonjour, \u{2068}Alice\u{2069} !"
  )]
  fn test_localization_service_load_locale(
    localization_service: Arc<FluentLocalizationService>,
    #[case] locale: &str,
    #[case] message_key: &str,
    #[case] args: Option<HashMap<String, String>>,
    #[case] expected: &str,
  ) -> Result<(), anyhow::Error> {
    let en_us = LanguageIdentifier::from_str("en-US").unwrap();
    let fr_fr = LanguageIdentifier::from_str("fr-FR").unwrap();
    let en_us_resource = include_str!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/tests/resources-0/en-US/test-messages-1.ftl"
    ));
    let fr_fr_resource = include_str!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/tests/resources-0/fr-FR/test-messages-1.ftl"
    ));
    localization_service
      .load_locale(en_us, vec![fr_fr_resource.to_string()])
      .unwrap();
    localization_service
      .load_locale(fr_fr, vec![en_us_resource.to_string()])
      .unwrap();

    let lang_id: LanguageIdentifier = locale.parse()?;
    let result = localization_service.get_message(&lang_id, message_key, args)?;
    assert_eq!(result, expected.to_string());

    Ok(())
  }

  #[rstest]
  #[case::invalid_ftl_resource("en-US", vec!["invalid-ftl-resource".to_string()])]
  #[case::empty_vec("en-US", vec![])]
  fn test_load_locale_invalid_resource(
    localization_service: Arc<FluentLocalizationService>,
    #[case] locale: &str,
    #[case] resources: Vec<String>,
  ) {
    let lang_id = LanguageIdentifier::from_str(locale).unwrap();
    let result = localization_service.load_locale(lang_id, resources);
    assert!(result.is_ok());
  }

  #[rstest]
  fn test_localization_service_unsupported_locales_not_loaded(
    localization_service: Arc<FluentLocalizationService>,
  ) {
    let lang_id = LanguageIdentifier::from_str("en-UK").unwrap();
    let resources = vec![];
    let result = localization_service.load_locale(lang_id, resources);
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err(),
      LocalizationSetupError::LocaleNotSupported(LocaleNotSupportedError::new("en-UK".to_string()))
    );
  }

  #[rstest]
  fn test_localization_service_returns_error_for_unsupported_locale(
    localization_service: Arc<FluentLocalizationService>,
  ) {
    let result = localization_service.load_resource(&RESOURCES_UNSUPPORTED);
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err(),
      LocalizationSetupError::LocaleNotSupported(LocaleNotSupportedError::new("en-UK".to_string()))
    );
  }

  #[rstest]
  fn test_load_resource_non_directory(localization_service: Arc<FluentLocalizationService>) {
    static NON_DIR_RESOURCE: Dir =
      include_dir!("$CARGO_MANIFEST_DIR/tests/resources-non-directory");
    let result = localization_service.load_resource(&NON_DIR_RESOURCE);
    assert!(result.is_ok());
  }

  #[rstest]
  fn test_get_message_missing_bundle(localization_service: Arc<FluentLocalizationService>) {
    let lang_id = LanguageIdentifier::from_str("ja-JP").unwrap();
    let result = localization_service.get_message(&lang_id, "test-key", None);
    assert!(result.is_err());
    match result.unwrap_err() {
      LocalizationMessageError::LocaleNotSupported(LocaleNotSupportedError { locale }) => {
        assert_eq!(locale, "ja-JP");
      }
      err => {
        panic!("expected LocaleNotSupported error, got {:?}", err);
      }
    }
  }

  #[rstest]
  fn test_get_message_missing_message(localization_service: Arc<FluentLocalizationService>) {
    localization_service
      .load_resource(&RESOURCES_DIR_0)
      .unwrap();
    let lang_id = LanguageIdentifier::from_str("en-US").unwrap();
    let result = localization_service.get_message(&lang_id, "non-existent-key", None);
    assert!(matches!(
      result,
      Err(LocalizationMessageError::MessageNotFound(key)) if key == "non-existent-key"
    ));
  }

  #[rstest]
  fn test_get_message_missing_pattern_if_message_not_properly_formed(
    localization_service: Arc<FluentLocalizationService>,
  ) {
    static RESOURCES_MISSING_PATTERN: Dir =
      include_dir!("$CARGO_MANIFEST_DIR/tests/resources-missing-pattern");
    localization_service
      .load_resource(&RESOURCES_MISSING_PATTERN)
      .unwrap();
    let lang_id = LanguageIdentifier::from_str("en-US").unwrap();
    let result = localization_service.get_message(&lang_id, "missing-pattern", None);
    assert!(result.is_err());
    match result.unwrap_err() {
      LocalizationMessageError::MessageNotFound(message) => {
        assert_eq!(message, "missing-pattern");
      }
      err => {
        panic!("expected MessageNotFound error, got {:?}", err);
      }
    }
  }

  #[rstest]
  fn test_get_message_format_error(localization_service: Arc<FluentLocalizationService>) {
    static RESOURCES_FORMAT_ERROR: Dir =
      include_dir!("$CARGO_MANIFEST_DIR/tests/resources-format-error");
    localization_service
      .load_resource(&RESOURCES_FORMAT_ERROR)
      .unwrap();
    let lang_id = LanguageIdentifier::from_str("en-US").unwrap();
    let result = localization_service.get_message(
      &lang_id,
      "format-error",
      Some(HashMap::from([(
        "invalid".to_string(),
        "value".to_string(),
      )])),
    );
    assert!(result.is_err());
    match result.unwrap_err() {
      LocalizationMessageError::FormatPattern(message) => {
        assert_eq!(message, "Resolver error: Unknown variable: $nonexistent");
      }
      err => {
        panic!("expected FormatPattern error, got {:?}", err);
      }
    }
  }

  #[rstest]
  #[tokio::test]
  async fn test_concurrent_access(localization_service: Arc<FluentLocalizationService>) {
    localization_service
      .load_resource(&RESOURCES_DIR_0)
      .unwrap();
    let lang_id = LanguageIdentifier::from_str("en-US").unwrap();

    let tasks: Vec<_> = (0..100)
      .map(|_| {
        let service = Arc::clone(&localization_service);
        let lang_id = lang_id.clone();
        tokio::spawn(async move { service.get_message(&lang_id, "hello-world", None).unwrap() })
      })
      .collect();

    let results = futures::future::join_all(tasks).await;
    for result in results {
      assert_eq!(result.unwrap(), "Hello, World!".to_string());
    }
  }

  #[rstest]
  fn test_get_instance(#[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>) {
    let instance1 = FluentLocalizationService::get_instance();
    let instance2 = FluentLocalizationService::get_instance();

    assert!(Arc::ptr_eq(&instance1, &instance2));
  }
}
