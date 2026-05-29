use crate::models::{FallbackConfig, RouterTarget};
use axum::http::HeaderMap;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::RwLock;

/// Identity under which a target's health is remembered. Keyed by the underlying
/// target — `(tenant, referenced-alias identity, pinned model)` — NOT the router
/// that referenced it. `target.alias` is already the `alias_name()` identity (id
/// for api aliases, name for local), so two routers sharing the same provider
/// share its cooldown, while `tenant_id` keeps tenants isolated.
pub fn target_key(tenant_id: &str, alias: &str, model: &str) -> String {
  format!("{tenant_id}:{alias}:{model}")
}

#[derive(Debug, Clone, Default)]
struct EndpointHealth {
  /// When set and in the future, the target is cooled (skipped during selection).
  cooldown_until: Option<DateTime<Utc>>,
}

/// Process-global, in-memory record of which targets are cooling down. No DB, no
/// cross-replica sharing — resets on restart (accepted limitation). Every routing
/// strategy reads/writes it; each applies its own `cooldown_secs`.
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait HealthRegistry: Send + Sync + std::fmt::Debug {
  /// True if `key` is currently cooled (has a `cooldown_until` strictly after `now`).
  fn is_cooled(&self, key: &str, now: DateTime<Utc>) -> bool;

  /// When that cooldown expires, or `None` if not cooled.
  fn cooled_until(&self, key: &str) -> Option<DateTime<Utc>>;

  /// Put `key` into cooldown until `until`.
  fn cooldown(&self, key: &str, until: DateTime<Utc>);

  /// Clear `key`'s health — a successful (possibly half-open) request recovered it.
  fn record_success(&self, key: &str);
}

#[derive(Debug, Default)]
pub struct DefaultHealthRegistry {
  inner: RwLock<HashMap<String, EndpointHealth>>,
}

impl HealthRegistry for DefaultHealthRegistry {
  fn is_cooled(&self, key: &str, now: DateTime<Utc>) -> bool {
    self
      .inner
      .read()
      .unwrap()
      .get(key)
      .and_then(|h| h.cooldown_until)
      .map(|until| until > now)
      .unwrap_or(false)
  }

  fn cooled_until(&self, key: &str) -> Option<DateTime<Utc>> {
    self
      .inner
      .read()
      .unwrap()
      .get(key)
      .and_then(|h| h.cooldown_until)
  }

  fn cooldown(&self, key: &str, until: DateTime<Utc>) {
    self
      .inner
      .write()
      .unwrap()
      .entry(key.to_string())
      .or_default()
      .cooldown_until = Some(until);
  }

  fn record_success(&self, key: &str) {
    self.inner.write().unwrap().remove(key);
  }
}

/// Order targets for selection: enabled non-cooled in declared order first, then
/// cooled targets by soonest recovery. Callers pass the already-enabled targets;
/// this never drops a target (never-starve): when all are cooled, all are returned
/// (soonest-recovery first) so the caller still attempts them.
pub fn order_by_health<'a>(
  targets: &[&'a RouterTarget],
  registry: &dyn HealthRegistry,
  tenant_id: &str,
  now: DateTime<Utc>,
) -> Vec<&'a RouterTarget> {
  let mut not_cooled: Vec<&RouterTarget> = Vec::new();
  let mut cooled: Vec<(&RouterTarget, DateTime<Utc>)> = Vec::new();
  for t in targets {
    let key = target_key(tenant_id, &t.alias, &t.model);
    match registry.cooled_until(&key) {
      Some(until) if until > now => cooled.push((t, until)),
      _ => not_cooled.push(t),
    }
  }
  cooled.sort_by_key(|(_, until)| *until);
  not_cooled
    .into_iter()
    .chain(cooled.into_iter().map(|(t, _)| t))
    .collect()
}

/// How long to cool a target that just failed: the configured `cooldown_secs`,
/// extended to the upstream `Retry-After` when `honor_retry_after` is set and the
/// header is present and parseable (delta-seconds; HTTP-date is best-effort).
pub fn cooldown_for(
  headers: &HeaderMap,
  cfg: &FallbackConfig,
  now: DateTime<Utc>,
) -> DateTime<Utc> {
  let mut secs = cfg.cooldown_secs as i64;
  if cfg.honor_retry_after {
    if let Some(retry_after) = parse_retry_after(headers, now) {
      secs = secs.max(retry_after);
    }
  }
  now + Duration::seconds(secs)
}

/// Parse `Retry-After` to a non-negative number of seconds from `now`. Supports
/// the delta-seconds form (`Retry-After: 120`) and best-effort HTTP-date.
fn parse_retry_after(headers: &HeaderMap, now: DateTime<Utc>) -> Option<i64> {
  let raw = headers
    .get(axum::http::header::RETRY_AFTER)?
    .to_str()
    .ok()?;
  let raw = raw.trim();
  if let Ok(secs) = raw.parse::<i64>() {
    return Some(secs.max(0));
  }
  let when = DateTime::parse_from_rfc2822(raw).ok()?.with_timezone(&Utc);
  Some((when - now).num_seconds().max(0))
}

#[cfg(test)]
#[path = "test_health.rs"]
mod test_health;
