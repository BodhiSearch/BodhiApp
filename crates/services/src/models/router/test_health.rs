use super::{cooldown_for, order_by_health, target_key, DefaultHealthRegistry, HealthRegistry};
use crate::models::{FallbackConfig, RouterTarget};
use axum::http::HeaderMap;
use chrono::{Duration, TimeZone, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;

fn now() -> chrono::DateTime<Utc> {
  Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()
}

fn t(alias: &str) -> RouterTarget {
  RouterTarget {
    alias: alias.to_string(),
    model: alias.to_string(),
    enabled: true,
    weight: None,
  }
}

#[rstest]
fn test_cooldown_set_and_is_cooled() {
  let reg = DefaultHealthRegistry::default();
  let key = target_key("t1", "a", "m");
  assert!(!reg.is_cooled(&key, now()));
  reg.cooldown(&key, now() + Duration::seconds(30));
  assert!(reg.is_cooled(&key, now()));
  // Exactly at expiry it is no longer cooled (strict >).
  assert!(!reg.is_cooled(&key, now() + Duration::seconds(30)));
  assert!(!reg.is_cooled(&key, now() + Duration::seconds(31)));
}

#[rstest]
fn test_record_success_clears() {
  let reg = DefaultHealthRegistry::default();
  let key = target_key("t1", "a", "m");
  reg.cooldown(&key, now() + Duration::seconds(30));
  assert!(reg.is_cooled(&key, now()));
  reg.record_success(&key);
  assert!(!reg.is_cooled(&key, now()));
}

#[rstest]
fn test_key_isolates_tenant_but_shares_target() {
  let reg = DefaultHealthRegistry::default();
  let t1 = target_key("tenant-1", "alias", "model");
  let t2 = target_key("tenant-2", "alias", "model");
  reg.cooldown(&t1, now() + Duration::seconds(30));
  assert!(reg.is_cooled(&t1, now()));
  assert!(!reg.is_cooled(&t2, now())); // other tenant unaffected
                                       // Same (tenant, alias, model) is the same key regardless of which router set it.
  assert_eq!(t1, target_key("tenant-1", "alias", "model"));
}

#[rstest]
fn test_order_by_health_not_cooled_first_declared_order() {
  let reg = DefaultHealthRegistry::default();
  let a = t("a");
  let b = t("b");
  let c = t("c");
  let targets = vec![&a, &b, &c];
  // Cool b → order becomes a, c, then b.
  reg.cooldown(&target_key("t1", "b", "b"), now() + Duration::seconds(30));
  let ordered = order_by_health(&targets, &reg, "t1", now());
  let names: Vec<&str> = ordered.iter().map(|t| t.alias.as_str()).collect();
  assert_eq!(vec!["a", "c", "b"], names);
}

#[rstest]
fn test_order_by_health_cooled_sorted_by_soonest_recovery() {
  let reg = DefaultHealthRegistry::default();
  let a = t("a");
  let b = t("b");
  let targets = vec![&a, &b];
  // Both cooled; b recovers sooner than a → b before a.
  reg.cooldown(&target_key("t1", "a", "a"), now() + Duration::seconds(60));
  reg.cooldown(&target_key("t1", "b", "b"), now() + Duration::seconds(30));
  let ordered = order_by_health(&targets, &reg, "t1", now());
  let names: Vec<&str> = ordered.iter().map(|t| t.alias.as_str()).collect();
  assert_eq!(vec!["b", "a"], names);
}

#[rstest]
fn test_order_by_health_all_cooled_yields_all() {
  let reg = DefaultHealthRegistry::default();
  let a = t("a");
  let b = t("b");
  let targets = vec![&a, &b];
  reg.cooldown(&target_key("t1", "a", "a"), now() + Duration::seconds(30));
  reg.cooldown(&target_key("t1", "b", "b"), now() + Duration::seconds(30));
  let ordered = order_by_health(&targets, &reg, "t1", now());
  assert_eq!(2, ordered.len()); // never starve
}

#[rstest]
fn test_cooldown_for_uses_configured_when_no_header() {
  let cfg = FallbackConfig::default(); // cooldown_secs 30, honor true
  let until = cooldown_for(&HeaderMap::new(), &cfg, now());
  assert_eq!(now() + Duration::seconds(30), until);
}

#[rstest]
fn test_cooldown_for_honors_larger_retry_after() {
  let cfg = FallbackConfig::default();
  let mut headers = HeaderMap::new();
  headers.insert("retry-after", "120".parse().unwrap());
  let until = cooldown_for(&headers, &cfg, now());
  assert_eq!(now() + Duration::seconds(120), until);
}

#[rstest]
fn test_cooldown_for_keeps_configured_when_retry_after_smaller() {
  let cfg = FallbackConfig::default(); // 30
  let mut headers = HeaderMap::new();
  headers.insert("retry-after", "5".parse().unwrap());
  let until = cooldown_for(&headers, &cfg, now());
  assert_eq!(now() + Duration::seconds(30), until);
}

#[rstest]
fn test_cooldown_for_ignores_retry_after_when_disabled() {
  let cfg = FallbackConfig {
    honor_retry_after: false,
    ..FallbackConfig::default()
  };
  let mut headers = HeaderMap::new();
  headers.insert("retry-after", "120".parse().unwrap());
  let until = cooldown_for(&headers, &cfg, now());
  assert_eq!(now() + Duration::seconds(30), until);
}
