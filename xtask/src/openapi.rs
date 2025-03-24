use anyhow::Result;
use routes_app::BodhiOpenAPIDoc;
use std::{fs::File, io::Write};
use utoipa::OpenApi;

pub fn generate() -> Result<()> {
  let openai = BodhiOpenAPIDoc::openapi();
  let mut file = File::create("openapi.json")?;
  file.write_all(openai.to_pretty_json()?.as_bytes())?;
  println!("OpenAPI spec written to openapi.json");
  Ok(())
}
