mod openapi;
mod typescript;

use anyhow::Result;

fn main() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  match args.get(1).map(|s| s.as_str()) {
    Some("openapi") => openapi::generate(),
    Some("types") => typescript::generate_types(),
    _ => xtaskops::tasks::main(),
  }
}
