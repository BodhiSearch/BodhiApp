use axum::body::Body;
use axum::http::Request;
use services::AuthContext;

pub trait RequestAuthContextExt {
  fn with_auth_context(self, ctx: AuthContext) -> Self;
}

impl RequestAuthContextExt for Request<Body> {
  fn with_auth_context(mut self, ctx: AuthContext) -> Self {
    self.extensions_mut().insert(ctx);
    self
  }
}
