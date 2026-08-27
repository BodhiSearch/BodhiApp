use services::{
  match_redirect_uri, parse_app_scope, AppClientInfo, AuthService, AuthServiceError,
  ParsedAppScope, RedirectUriValidation,
};

pub const OAUTH_ERROR_INVALID_REQUEST: &str = "invalid_request";
pub const OAUTH_ERROR_INVALID_SCOPE: &str = "invalid_scope";
pub const OAUTH_ERROR_UNAUTHORIZED_CLIENT: &str = "unauthorized_client";
pub const OAUTH_ERROR_UNSUPPORTED_RESPONSE_TYPE: &str = "unsupported_response_type";
pub const OAUTH_ERROR_ACCESS_DENIED: &str = "access_denied";
pub const OAUTH_ERROR_SERVER_ERROR: &str = "server_error";

/// The OAuth authorize request's query string as sent to `/ui/apps/auth/`, parsed.
/// Unknown parameters are ignored; only the composed authorize URL reaches Keycloak.
#[derive(Debug, Default, PartialEq)]
pub struct ConsentQuery {
  pub client_id: Option<String>,
  pub redirect_uri: Option<String>,
  pub response_type: Option<String>,
  pub state: Option<String>,
  pub code_challenge: Option<String>,
  pub code_challenge_method: Option<String>,
  pub scope: Option<String>,
  pub source_access_request_id: Option<String>,
}

impl ConsentQuery {
  /// Repeated occurrences of a recognized parameter are rejected per RFC 6749 §3.1
  /// (a duplicated `redirect_uri`/`state` would make the request ambiguous).
  pub fn parse(raw_query: &str) -> Result<Self, String> {
    let mut query = ConsentQuery::default();
    for (key, value) in url::form_urlencoded::parse(raw_query.as_bytes()) {
      let value = value.into_owned();
      let slot = match key.as_ref() {
        "client_id" => &mut query.client_id,
        "redirect_uri" => &mut query.redirect_uri,
        "response_type" => &mut query.response_type,
        "state" => &mut query.state,
        "code_challenge" => &mut query.code_challenge,
        "code_challenge_method" => &mut query.code_challenge_method,
        "scope" => &mut query.scope,
        "source_access_request_id" => &mut query.source_access_request_id,
        _ => continue,
      };
      if slot.is_some() {
        return Err(key.into_owned());
      }
      *slot = Some(value);
    }
    Ok(query)
  }
}

/// A consent request that passed validation and can be rendered or decided.
#[derive(Debug)]
pub struct ConsentReady {
  pub client_id: String,
  pub redirect_uri: String,
  pub state: String,
  pub code_challenge: String,
  pub scope: ParsedAppScope,
  pub app_info: AppClientInfo,
  pub source_access_request_id: Option<String>,
}

/// OAuth-level rejection. `redirect_url` is set only once the redirect target has been
/// validated (or validation is unavailable on an older auth-server extension) — otherwise
/// the page renders in-app and navigates nowhere (RFC 6749 §4.1.2.1).
#[derive(Debug)]
pub struct ConsentFailure {
  pub error: &'static str,
  pub error_description: String,
  pub redirect_url: Option<String>,
}

impl ConsentFailure {
  fn in_app(error: &'static str, error_description: String) -> Self {
    Self {
      error,
      error_description,
      redirect_url: None,
    }
  }

  fn redirecting(
    redirect_uri: &str,
    state: Option<&str>,
    error: &'static str,
    error_description: String,
  ) -> Self {
    Self {
      redirect_url: build_error_redirect(redirect_uri, error, &error_description, state),
      error,
      error_description,
    }
  }
}

/// OAuth error params appended to a validated redirect target. `error_source=bodhi`
/// distinguishes BodhiApp-originated errors from the auth server's own (existing contract
/// pinned by the E2E suite). `None` when the target does not parse as a URL.
pub fn build_error_redirect(
  redirect_uri: &str,
  error: &str,
  error_description: &str,
  state: Option<&str>,
) -> Option<String> {
  let mut url = url::Url::parse(redirect_uri).ok()?;
  {
    let mut pairs = url.query_pairs_mut();
    pairs.append_pair("error", error);
    pairs.append_pair("error_description", error_description);
    pairs.append_pair("error_source", "bodhi");
    if let Some(state) = state {
      pairs.append_pair("state", state);
    }
  }
  Some(url.to_string())
}

/// Validates one authorize request end to end: app client resolution, redirect_uri
/// exact-match, OAuth parameter checks, and app-facing scope parsing. Both the consent
/// GET and the consent POST run this, so the POST never trusts a client-side reading.
pub async fn evaluate_consent_query(
  auth_service: &dyn AuthService,
  user_token: &str,
  raw_query: &str,
) -> Result<ConsentReady, Box<ConsentFailure>> {
  let query = match ConsentQuery::parse(raw_query) {
    Ok(query) => query,
    Err(param) => {
      return Err(Box::new(ConsentFailure::in_app(
        OAUTH_ERROR_INVALID_REQUEST,
        format!("duplicate parameter '{}'", param),
      )));
    }
  };

  let Some(client_id) = query.client_id.filter(|v| !v.is_empty()) else {
    return Err(Box::new(ConsentFailure::in_app(
      OAUTH_ERROR_INVALID_REQUEST,
      "client_id is required".to_string(),
    )));
  };

  let app_info = match auth_service
    .get_app_client_info(&client_id, user_token)
    .await
  {
    Ok(info) => info,
    Err(AuthServiceError::AuthServiceApiError { status: 404, .. }) => {
      return Err(Box::new(ConsentFailure::in_app(
        OAUTH_ERROR_UNAUTHORIZED_CLIENT,
        format!("unknown app client '{}'", client_id),
      )));
    }
    Err(_) => {
      return Err(Box::new(ConsentFailure::in_app(
        OAUTH_ERROR_SERVER_ERROR,
        "failed to resolve the app client".to_string(),
      )));
    }
  };

  let Some(redirect_uri) = query.redirect_uri.filter(|v| !v.is_empty()) else {
    return Err(Box::new(ConsentFailure::in_app(
      OAUTH_ERROR_INVALID_REQUEST,
      "redirect_uri is required".to_string(),
    )));
  };
  if url::Url::parse(&redirect_uri).is_err() {
    return Err(Box::new(ConsentFailure::in_app(
      OAUTH_ERROR_INVALID_REQUEST,
      format!("redirect_uri '{}' is not a valid URL", redirect_uri),
    )));
  }
  if match_redirect_uri(&redirect_uri, app_info.redirect_uris.as_deref())
    == RedirectUriValidation::Mismatch
  {
    return Err(Box::new(ConsentFailure::in_app(
      OAUTH_ERROR_INVALID_REQUEST,
      format!(
        "redirect_uri '{}' is not registered for this app",
        redirect_uri
      ),
    )));
  }

  // The redirect target is trusted from here on — failures below return to the app.
  let state = query.state.filter(|v| !v.is_empty());

  if query.response_type.as_deref() != Some("code") {
    return Err(Box::new(ConsentFailure::redirecting(
      &redirect_uri,
      state.as_deref(),
      OAUTH_ERROR_UNSUPPORTED_RESPONSE_TYPE,
      "response_type must be 'code'".to_string(),
    )));
  }
  let Some(state) = state else {
    return Err(Box::new(ConsentFailure::redirecting(
      &redirect_uri,
      None,
      OAUTH_ERROR_INVALID_REQUEST,
      "state is required".to_string(),
    )));
  };
  let Some(code_challenge) = query.code_challenge.filter(|v| !v.is_empty()) else {
    return Err(Box::new(ConsentFailure::redirecting(
      &redirect_uri,
      Some(&state),
      OAUTH_ERROR_INVALID_REQUEST,
      "code_challenge is required (PKCE)".to_string(),
    )));
  };
  if query.code_challenge_method.as_deref() != Some("S256") {
    return Err(Box::new(ConsentFailure::redirecting(
      &redirect_uri,
      Some(&state),
      OAUTH_ERROR_INVALID_REQUEST,
      "code_challenge_method must be 'S256'".to_string(),
    )));
  }

  let scope = match parse_app_scope(query.scope.as_deref().unwrap_or("")) {
    Ok(scope) => scope,
    Err(err) => {
      return Err(Box::new(ConsentFailure::redirecting(
        &redirect_uri,
        Some(&state),
        OAUTH_ERROR_INVALID_SCOPE,
        err.to_string(),
      )));
    }
  };

  Ok(ConsentReady {
    client_id,
    redirect_uri,
    state,
    code_challenge,
    scope,
    app_info,
    source_access_request_id: query.source_access_request_id.filter(|v| !v.is_empty()),
  })
}

#[cfg(test)]
#[path = "test_consent.rs"]
mod test_consent;
