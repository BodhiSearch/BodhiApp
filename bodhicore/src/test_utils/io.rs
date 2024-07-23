use dircpy::CopyBuilder;
use std::path::Path;

pub fn copy_test_dir(src: &str, dst_path: &Path) {
  let src_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(src);
  CopyBuilder::new(src_path, dst_path)
    .overwrite(true)
    .with_include_filter("")
    .run()
    .unwrap();
}
