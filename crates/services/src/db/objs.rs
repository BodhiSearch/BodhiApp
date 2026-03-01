/// Represents an API key update operation for API model aliases
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeyUpdate {
  /// Keep the existing API key unchanged
  Keep,
  /// Set a new API key (or add one if none exists) - Option<String> supports both setting and clearing
  Set(Option<String>),
}
