use include_dir::{include_dir, Dir};

pub static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../out");
