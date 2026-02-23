#[derive(Debug, Clone)]
pub enum AppCommand {
  Serve {
    host: Option<String>,
    port: Option<u16>,
  },
  Default,
}
