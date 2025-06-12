use std::io;
use crate::error::{AppSetupError};
use objs::{ErrorMessage, ErrorType};

#[test]
fn test_app_setup_error_async_runtime_to_error_message() {
    // Simulate an io::Error
    let io_err = io::Error::new(io::ErrorKind::Other, "simulated async runtime failure");
    // Convert to AppSetupError
    let app_setup_err = AppSetupError::AsyncRuntime(io_err);
    // Convert to ErrorMessage
    let err_msg: ErrorMessage = app_setup_err.into();
    // Check the error message fields
    assert_eq!(err_msg.error_type(), ErrorType::InternalServer);
    assert_eq!(err_msg.code(), "async_runtime");
    assert!(err_msg.message().contains("simulated async runtime failure"));
}
