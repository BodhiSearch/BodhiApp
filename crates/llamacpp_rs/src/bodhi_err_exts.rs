use crate::error::LlamaCppError;
use llamacpp_sys::bindings::bodhi_error;
use std::ffi::CStr;

pub trait BodhiErrorExt {
  fn default() -> Self;
  fn is_err(&self) -> bool;
  #[allow(unused)]
  fn is_ok(&self) -> bool;
  fn as_ptr(&self) -> *mut bodhi_error;
  fn map_err<O: FnOnce(String) -> LlamaCppError>(self, op: O) -> crate::error::Result<()>;
  fn to_string(&self) -> String;
}

impl BodhiErrorExt for bodhi_error {
  fn is_err(&self) -> bool {
    self.message[0] != 0
  }

  fn is_ok(&self) -> bool {
    !self.is_err()
  }

  fn as_ptr(&self) -> *mut bodhi_error {
    self as *const bodhi_error as *mut bodhi_error
  }

  fn map_err<O: FnOnce(String) -> LlamaCppError>(self, op: O) -> crate::error::Result<()> {
    if self.is_err() {
      Err(op(self.to_string()))
    } else {
      Ok(())
    }
  }
  fn default() -> Self {
    Self { message: [0; 1024] }
  }

  fn to_string(&self) -> String {
    unsafe {
      CStr::from_ptr(self.message.as_ptr())
        .to_string_lossy()
        .into_owned()
        .to_string()
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{bodhi_err_exts::BodhiErrorExt, error::LlamaCppError};
  use llamacpp_sys::bindings::bodhi_error;
  use std::ffi::CString;

  #[test]
  fn test_default() {
    let error = bodhi_error::default();
    assert!(error.is_ok());
    assert!(!error.is_err());
    assert_eq!(error.to_string(), "");
  }

  #[test]
  fn test_is_err_and_is_ok() {
    let mut error = bodhi_error::default();
    assert!(error.is_ok());
    assert!(!error.is_err());

    // Simulate an error by setting a message
    let error_message = CString::new("Test error").unwrap();
    unsafe {
      std::ptr::copy_nonoverlapping(
        error_message.as_ptr(),
        error.message.as_mut_ptr(),
        error_message.as_bytes_with_nul().len(),
      );
    }

    assert!(error.is_err());
    assert!(!error.is_ok());
  }

  #[test]
  fn test_as_ptr() {
    let error = bodhi_error::default();
    let ptr = error.as_ptr();
    assert!(!ptr.is_null());
  }

  #[test]
  fn test_map_err() {
    let ok_error = bodhi_error::default();
    let ok_result = ok_error.map_err(LlamaCppError::BodhiContextInit);
    assert!(ok_result.is_ok());
  }

  #[test]
  fn test_map_err_from_cstr() {
    let mut err_error = bodhi_error::default();
    let error_message = CString::new("Test error").unwrap();
    unsafe {
      std::ptr::copy_nonoverlapping(
        error_message.as_ptr(),
        err_error.message.as_mut_ptr(),
        error_message.as_bytes_with_nul().len(),
      );
    }
    let err_result = err_error.map_err(LlamaCppError::BodhiContextInit);
    assert!(err_result.is_err());
    assert_eq!(
      err_result.unwrap_err(),
      LlamaCppError::BodhiContextInit("Test error".to_string())
    );
  }

  #[test]
  fn test_to_string() {
    let mut error = bodhi_error::default();
    assert_eq!(error.to_string(), "");

    let error_message = CString::new("Test error message").unwrap();
    unsafe {
      std::ptr::copy_nonoverlapping(
        error_message.as_ptr(),
        error.message.as_mut_ptr(),
        error_message.as_bytes_with_nul().len(),
      );
    }
    assert_eq!(error.to_string(), "Test error message");
  }
}
