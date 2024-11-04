use dircpy::CopyBuilder;
use fs_extra::dir::{copy, CopyOptions};
use std::path::Path;

static COPY_OPTIONS: CopyOptions = CopyOptions {
  overwrite: true,
  skip_exist: false,
  copy_inside: true,
  content_only: false,
  buffer_size: 64000,
  depth: 0,
};

pub fn copy_test_dir(src: &str, dst_path: &Path) {
  let src_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(src);
  copy(src_path, dst_path, &COPY_OPTIONS).unwrap();
}
