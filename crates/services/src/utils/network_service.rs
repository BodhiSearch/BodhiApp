use std::fmt::Debug;

/// Service for network-related operations.
pub trait NetworkService: Send + Sync + Debug {
  /// Get the server's local IP address for future-proofing redirect URIs.
  /// Returns None if unable to determine a non-loopback IP.
  fn get_server_ip(&self) -> Option<String>;
}

/// Default implementation that determines the local IP via UDP socket connection.
#[derive(Debug, Default)]
pub struct DefaultNetworkService;

impl NetworkService for DefaultNetworkService {
  fn get_server_ip(&self) -> Option<String> {
    use std::net::UdpSocket;

    // Try to get local IP by connecting to a remote address
    // This doesn't actually send data, just determines which local interface would be used
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
      if socket.connect("8.8.8.8:80").is_ok() {
        if let Ok(local_addr) = socket.local_addr() {
          let ip = local_addr.ip();
          // Only return if it's not a loopback address
          if !ip.is_loopback() {
            return Some(ip.to_string());
          }
        }
      }
    }

    None
  }
}

#[cfg(any(test, feature = "test-utils"))]
/// Stub implementation for testing that returns a configurable IP.
#[derive(Debug)]
pub struct StubNetworkService {
  pub ip: Option<String>,
}

#[cfg(any(test, feature = "test-utils"))]
impl NetworkService for StubNetworkService {
  fn get_server_ip(&self) -> Option<String> {
    self.ip.clone()
  }
}
