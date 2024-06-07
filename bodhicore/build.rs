use std::{env, fs::File, path::PathBuf};

fn main() {
  let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
  let manifest_dir = PathBuf::from(manifest_dir);
  let db_path = manifest_dir.join("tests/data/db/dev.db");
  let db_url = format!(
    "sqlite:{}",
    db_path.to_str().expect("Failed to convert path to str")
  );
  _ = File::create_new(&db_path);
  env::set_var("DATABASE_URL", db_url);
  println!("cargo:rerun-if-changed=migrations");
}
