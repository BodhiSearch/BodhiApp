use anyhow::Result;
use routes_app::{BodhiOAIOpenAPIDoc, GlobalErrorResponses, SecurityModifier};
use std::{fs::File, io::Write};
use utoipa::{Modify, OpenApi};

pub fn generate() -> Result<()> {
  let mut openapi = BodhiOAIOpenAPIDoc::openapi();
  SecurityModifier.modify(&mut openapi);
  GlobalErrorResponses::oai().modify(&mut openapi);
  let mut file = File::create("openapi-oai.json")?;
  file.write_all(openapi.to_pretty_json()?.as_bytes())?;
  println!("OpenAI-compat spec written to openapi-oai.json");
  Ok(())
}
