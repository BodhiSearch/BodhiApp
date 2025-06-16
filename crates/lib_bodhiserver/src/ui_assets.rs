use include_dir::{include_dir, Dir};

/// Embedded UI assets built from the Next.js frontend
///
/// This static directory contains all the compiled frontend assets
/// including HTML, CSS, JavaScript, and other static files.
///
/// The assets are built during the lib_bodhiserver compilation process
/// and embedded at compile time for consistent serving across all
/// deployment scenarios (Tauri, NAPI, etc.).
pub static EMBEDDED_UI_ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../bodhi/out");
