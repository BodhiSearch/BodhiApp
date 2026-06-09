use crate::NetworkService;

#[derive(Debug)]
pub struct StubNetworkService {
  pub ip: Option<String>,
}

impl NetworkService for StubNetworkService {
  fn get_server_ip(&self) -> Option<String> {
    self.ip.clone()
  }
}
