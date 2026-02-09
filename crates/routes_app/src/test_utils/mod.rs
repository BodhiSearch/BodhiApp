mod alias_response;
mod router;

#[allow(unused_imports)]
pub use alias_response::*;
pub use router::*;

#[macro_export]
macro_rules! wait_for_event {
  ($rx:expr, $event_name:expr, $timeout:expr) => {{
    loop {
      tokio::select! {
          event = $rx.recv() => {
              match event {
                  Ok(e) if e == $event_name => break true,
                  _ => continue
              }
          }
          _ = tokio::time::sleep($timeout) => break false
      }
    }
  }};
}

pub const TEST_ENDPOINT_APP_INFO: &str = "/test/app/info";
