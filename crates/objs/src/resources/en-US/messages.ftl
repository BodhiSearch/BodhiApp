bad_request_error = invalid request, reason: {$reason}
builder_error-uninitialized_field = builder_error: uninitialized field: {$var_0}
builder_error-validation_error = builder_error: validation error: {$var_0}
internal_server_error = internal_server_error: {$reason}
io_error = io_error: {$source}
io_with_path_error = io_error: path: {$path}, {$source}
io_dir_create_error = io_error: failed to create directory $BODHI_HOME/{$path}, error: {$source}
io_file_read_error = io_error: failed to read file $BODHI_HOME/{$path}, error: {$source}
io_file_write_error = io_error: failed to update file $BODHI_HOME/{$path}, error: {$source}
io_file_delete_error = io_error: failed to delete file $BODHI_HOME/{$path}, error: {$source}
serde_json_error = error serializing/deserializing json: {$source}
serde_json_with_path_error = error serializing/deserializing json: path: {$path}, {$source}
serde_yaml_error = error serializing/deserializing yaml: {$source}
serde_yaml_with_path_error = error serializing/deserializing yaml: path: {$path}, {$source}
obj_validation_error-file_pattern_mismatch = file pattern does not match huggingface repo pattern, path: {$var_0}
obj_validation_error-validation_errors = validation_error: {$var_0}
reqwest_error = error connecting to internal service: {$error}
json_rejection_error = failed to parse the request body as JSON, error: {$source}