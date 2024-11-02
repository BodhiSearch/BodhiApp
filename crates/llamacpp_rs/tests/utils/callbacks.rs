use std::ffi::{c_char, c_void};
use std::slice;

pub extern "C" fn test_callback(
  contents: *const c_char,
  size: usize,
  userdata: *mut c_void,
) -> usize {
  let slice = unsafe { slice::from_raw_parts(contents as *const u8, size) };
  let input_str = match std::str::from_utf8(slice) {
    Ok(s) => s,
    Err(_) => return 0,
  };

  let user_data_str = unsafe { &mut *(userdata as *mut String) };
  user_data_str.push_str(input_str);
  size
}
