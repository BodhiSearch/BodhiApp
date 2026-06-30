use serde::Deserialize;

/// Shared `Deserialize` body for the versioned grant envelopes (`TokenGrants`,
/// `RequestedResources`, `ApprovedResources`). Each is serialized as
/// `{"version":"1", ...inner V1 fields...}`; this reads the mandatory `version`
/// tag and deserializes the rest into the inner `V1` type.
///
/// `label` names the envelope in the unsupported-version error (e.g. `"token
/// grants"`, `"resources"`). Centralizing this keeps the supported-version list
/// and error message in one place as new versions are added.
pub fn deserialize_versioned<'de, D, T>(deserializer: D, label: &str) -> Result<T, D::Error>
where
  D: serde::Deserializer<'de>,
  T: serde::de::DeserializeOwned,
{
  let value = serde_json::Value::deserialize(deserializer)?;
  let version = value
    .get("version")
    .and_then(|v| v.as_str())
    .ok_or_else(|| serde::de::Error::missing_field("version"))?;
  match version {
    "1" => serde_json::from_value(value).map_err(serde::de::Error::custom),
    unknown => Err(serde::de::Error::custom(format!(
      "Unsupported {label} version '{unknown}'. Supported versions: [1]"
    ))),
  }
}
