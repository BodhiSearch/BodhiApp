use include_dir::{include_dir, Dir};

/// Vite frontend embedded at compile time, built by `build.rs`. Serves the same
/// assets across all deployment scenarios (Tauri, NAPI, etc.).
pub static EMBEDDED_UI_ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../bodhi/out");
