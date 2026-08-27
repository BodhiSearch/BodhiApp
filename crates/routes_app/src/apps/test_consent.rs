use crate::apps::{
  build_error_redirect, evaluate_consent_query, ConsentQuery, OAUTH_ERROR_INVALID_REQUEST,
  OAUTH_ERROR_INVALID_SCOPE, OAUTH_ERROR_SERVER_ERROR, OAUTH_ERROR_UNAUTHORIZED_CLIENT,
};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use services::{AppClientInfo, AuthServiceError, MockAuthService, UserScope};

const USER_TOKEN: &str = "user-token";
const REDIRECT_URI: &str = "https://acme.dev/cb";
const REDIRECT_URI_ENC: &str = "https%3A%2F%2Facme.dev%2Fcb";

fn mock_auth_with(redirect_uris: Option<Vec<String>>) -> MockAuthService {
  let mut mock = MockAuthService::default();
  mock.expect_get_app_client_info().returning(move |_, _| {
    Ok(AppClientInfo {
      name: "Acme App".to_string(),
      description: "Acme test app".to_string(),
      redirect_uris: redirect_uris.clone(),
    })
  });
  mock
}

fn mock_auth_registered() -> MockAuthService {
  mock_auth_with(Some(vec![REDIRECT_URI.to_string()]))
}

fn valid_query() -> String {
  format!(
    "client_id=app-acme&redirect_uri={REDIRECT_URI_ENC}&response_type=code&state=st-123\
     &code_challenge=ch-456&code_challenge_method=S256"
  )
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_evaluate_consent_query_missing_client_id() -> anyhow::Result<()> {
  let mock = MockAuthService::default();
  let failure = evaluate_consent_query(
    &mock,
    USER_TOKEN,
    &format!("redirect_uri={REDIRECT_URI_ENC}"),
  )
  .await
  .expect_err("missing client_id must fail");
  assert_eq!(OAUTH_ERROR_INVALID_REQUEST, failure.error);
  assert_eq!(None, failure.redirect_url);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_evaluate_consent_query_unknown_client_unauthorized_in_app() -> anyhow::Result<()> {
  let mut mock = MockAuthService::default();
  mock.expect_get_app_client_info().returning(|_, _| {
    Err(AuthServiceError::AuthServiceApiError {
      status: 404,
      body: "client not found".to_string(),
    })
  });
  let failure = evaluate_consent_query(&mock, USER_TOKEN, &valid_query())
    .await
    .expect_err("unknown client must fail");
  assert_eq!(OAUTH_ERROR_UNAUTHORIZED_CLIENT, failure.error);
  assert_eq!(None, failure.redirect_url);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_evaluate_consent_query_auth_server_500_server_error_in_app() -> anyhow::Result<()> {
  let mut mock = MockAuthService::default();
  mock.expect_get_app_client_info().returning(|_, _| {
    Err(AuthServiceError::AuthServiceApiError {
      status: 500,
      body: "boom".to_string(),
    })
  });
  let failure = evaluate_consent_query(&mock, USER_TOKEN, &valid_query())
    .await
    .expect_err("KC 500 must fail");
  assert_eq!(OAUTH_ERROR_SERVER_ERROR, failure.error);
  assert_eq!(None, failure.redirect_url);
  Ok(())
}

#[rstest]
#[case::missing_redirect_uri("client_id=app-acme&response_type=code&state=st-123")]
#[case::unparsable_redirect_uri("client_id=app-acme&redirect_uri=not-a-url&response_type=code")]
#[case::mismatch_vs_registered(
  "client_id=app-acme&redirect_uri=https%3A%2F%2Fevil.dev%2Fcb&response_type=code&state=st-123\
   &code_challenge=ch-456&code_challenge_method=S256"
)]
#[tokio::test]
#[anyhow_trace]
async fn test_evaluate_consent_query_redirect_uri_failures_render_in_app(
  #[case] query: &str,
) -> anyhow::Result<()> {
  let mock = mock_auth_registered();
  let failure = evaluate_consent_query(&mock, USER_TOKEN, query)
    .await
    .expect_err("redirect_uri failure expected");
  assert_eq!(OAUTH_ERROR_INVALID_REQUEST, failure.error);
  assert_eq!(None, failure.redirect_url, "must never redirect: {query}");
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_evaluate_consent_query_absent_redirect_uris_skips_validation() -> anyhow::Result<()> {
  // Older auth-server extension: no registered list → Unvalidated, flow proceeds.
  let mock = mock_auth_with(None);
  let ready = evaluate_consent_query(&mock, USER_TOKEN, &valid_query())
    .await
    .map_err(|f| anyhow::anyhow!("unexpected failure: {}", f.error_description))?;
  assert_eq!(REDIRECT_URI, ready.redirect_uri);
  Ok(())
}

#[rstest]
#[case::reserved_scope_token(
  format!("client_id=app-acme&redirect_uri={REDIRECT_URI_ENC}&response_type=code&state=st-123&code_challenge=ch-456&code_challenge_method=S256&scope=scope_access_request%3Ainjected"),
  OAUTH_ERROR_INVALID_SCOPE,
  true
)]
#[case::malformed_scope_apps_token(
  format!("client_id=app-acme&redirect_uri={REDIRECT_URI_ENC}&response_type=code&state=st-123&code_challenge=ch-456&code_challenge_method=S256&scope=scope_apps%3Agarbage"),
  OAUTH_ERROR_INVALID_SCOPE,
  true
)]
#[case::unknown_scope_user_token(
  format!("client_id=app-acme&redirect_uri={REDIRECT_URI_ENC}&response_type=code&state=st-123&code_challenge=ch-456&code_challenge_method=S256&scope=scope_user_admin"),
  OAUTH_ERROR_INVALID_SCOPE,
  true
)]
#[case::scope_error_without_state_omits_state(
  format!("client_id=app-acme&redirect_uri={REDIRECT_URI_ENC}&scope=scope_apps%3Agarbage"),
  OAUTH_ERROR_INVALID_SCOPE,
  false
)]
#[tokio::test]
#[anyhow_trace]
async fn test_evaluate_consent_query_scope_failures_redirect(
  #[case] query: String,
  #[case] expected_error: &str,
  #[case] state_expected: bool,
) -> anyhow::Result<()> {
  let mock = mock_auth_registered();
  let failure = evaluate_consent_query(&mock, USER_TOKEN, &query)
    .await
    .expect_err("scope failure expected");
  assert_eq!(expected_error, failure.error);
  let redirect_url = failure
    .redirect_url
    .expect("validated target must redirect");
  let url = url::Url::parse(&redirect_url)?;
  let pairs: Vec<(String, String)> = url
    .query_pairs()
    .map(|(k, v)| (k.into_owned(), v.into_owned()))
    .collect();
  assert!(redirect_url.starts_with(REDIRECT_URI));
  assert!(pairs.contains(&("error".to_string(), expected_error.to_string())));
  assert!(pairs.contains(&("error_source".to_string(), "bodhi".to_string())));
  assert_eq!(
    state_expected,
    pairs.iter().any(|(k, _)| k == "state"),
    "state presence mismatch in {redirect_url}"
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_evaluate_consent_query_happy_path() -> anyhow::Result<()> {
  let mock = mock_auth_registered();
  let query = format!(
    "{}&scope=scope_user_power_user%20scope_apps%3Amcps%3Afalse%20offline_access\
     &source_access_request_id=ar-prior-1",
    valid_query()
  );
  let ready = evaluate_consent_query(&mock, USER_TOKEN, &query)
    .await
    .map_err(|f| anyhow::anyhow!("unexpected failure: {}", f.error_description))?;
  assert_eq!("app-acme", ready.client_id);
  assert_eq!(REDIRECT_URI, ready.redirect_uri);
  assert_eq!(Some("code".to_string()), ready.response_type);
  assert_eq!(Some("st-123".to_string()), ready.state);
  assert_eq!(Some("ch-456".to_string()), ready.code_challenge);
  assert_eq!(Some("S256".to_string()), ready.code_challenge_method);
  assert_eq!("Acme App", ready.app_info.name);
  assert_eq!(UserScope::PowerUser, ready.scope.role);
  assert_eq!(true, ready.scope.llms);
  assert_eq!(false, ready.scope.mcps);
  assert_eq!(vec!["offline_access".to_string()], ready.scope.passthrough);
  assert_eq!(
    Some("ar-prior-1".to_string()),
    ready.source_access_request_id
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_evaluate_consent_query_empty_scope_defaults() -> anyhow::Result<()> {
  let mock = mock_auth_registered();
  let ready = evaluate_consent_query(&mock, USER_TOKEN, &format!("{}&scope=", valid_query()))
    .await
    .map_err(|f| anyhow::anyhow!("unexpected failure: {}", f.error_description))?;
  assert_eq!(UserScope::User, ready.scope.role);
  assert_eq!(true, ready.scope.llms);
  assert_eq!(true, ready.scope.mcps);
  assert_eq!(Vec::<String>::new(), ready.scope.passthrough);
  assert_eq!(None, ready.source_access_request_id);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_evaluate_consent_query_forwards_oauth_params_without_enforcing() -> anyhow::Result<()>
{
  // Keycloak is the authority on response_type/state/PKCE — their absence is not our error.
  let mock = mock_auth_registered();
  let ready = evaluate_consent_query(
    &mock,
    USER_TOKEN,
    &format!("client_id=app-acme&redirect_uri={REDIRECT_URI_ENC}"),
  )
  .await
  .map_err(|f| anyhow::anyhow!("unexpected failure: {}", f.error_description))?;
  assert_eq!(None, ready.response_type);
  assert_eq!(None, ready.state);
  assert_eq!(None, ready.code_challenge);
  assert_eq!(None, ready.code_challenge_method);
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn test_consent_query_parse_percent_decodes_and_ignores_unknown() -> anyhow::Result<()> {
  let query = ConsentQuery::parse(
    "client_id=app%2Bacme&redirect_uri=https%3A%2F%2Facme.dev%2Fcb%3Ffoo%3Dbar\
     &state=st+1&unknown_param=zzz&scope=a%20b",
  )
  .expect("no duplicates");
  assert_eq!(Some("app+acme".to_string()), query.client_id);
  assert_eq!(
    Some("https://acme.dev/cb?foo=bar".to_string()),
    query.redirect_uri
  );
  // form-urlencoded: '+' decodes to a space.
  assert_eq!(Some("st 1".to_string()), query.state);
  assert_eq!(Some("a b".to_string()), query.scope);
  assert_eq!(None, query.response_type);
  assert_eq!(None, query.code_challenge);
  assert_eq!(None, query.code_challenge_method);
  assert_eq!(None, query.source_access_request_id);
  Ok(())
}

#[rstest]
#[case::duplicate_state("state=a&state=b", "state")]
#[case::duplicate_redirect_uri(
  "redirect_uri=https%3A%2F%2Fa.dev&redirect_uri=https%3A%2F%2Fb.dev",
  "redirect_uri"
)]
#[case::duplicate_scope("scope=a&scope=b", "scope")]
fn test_consent_query_parse_rejects_duplicates(
  #[case] raw_query: &str,
  #[case] expected_param: &str,
) {
  assert_eq!(
    Err(expected_param.to_string()),
    ConsentQuery::parse(raw_query)
  );
}

#[rstest]
#[anyhow_trace]
fn test_build_error_redirect_preserves_existing_query() -> anyhow::Result<()> {
  let redirect_url = build_error_redirect(
    "https://acme.dev/cb?keep=1",
    "access_denied",
    "user denied the access request",
    Some("st-123"),
  )
  .expect("valid URL must compose");
  let url = url::Url::parse(&redirect_url)?;
  let pairs: Vec<(String, String)> = url
    .query_pairs()
    .map(|(k, v)| (k.into_owned(), v.into_owned()))
    .collect();
  assert!(pairs.contains(&("keep".to_string(), "1".to_string())));
  assert!(pairs.contains(&("error".to_string(), "access_denied".to_string())));
  assert!(pairs.contains(&(
    "error_description".to_string(),
    "user denied the access request".to_string()
  )));
  assert!(pairs.contains(&("error_source".to_string(), "bodhi".to_string())));
  assert!(pairs.contains(&("state".to_string(), "st-123".to_string())));
  Ok(())
}

#[rstest]
#[anyhow_trace]
fn test_build_error_redirect_unparsable_target_is_none() -> anyhow::Result<()> {
  assert!(build_error_redirect("not-a-url", "access_denied", "desc", None).is_none());
  Ok(())
}
