use anyhow::Result;
use routes_app::{BodhiOpenAPIDoc, GlobalErrorResponses, SecurityModifier};
use std::{fs::File, io::Write};
use utoipa::{Modify, OpenApi};

pub fn generate() -> Result<()> {
  let mut openapi = BodhiOpenAPIDoc::openapi();
  SecurityModifier.modify(&mut openapi);
  GlobalErrorResponses.modify(&mut openapi);
  let mut file = File::create("openapi.json")?;
  file.write_all(openapi.to_pretty_json()?.as_bytes())?;
  println!("OpenAPI spec written to openapi.json");
  Ok(())
}
