data_service_error-dir_missing = directory '{$dirname}' not found in $BODHI_HOME.
  $BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME.
data_service_error-logs_dir_not_exists = BODHI_LOGS does not exists: {$var_0}
alias_not_found_error = alias '{$var_0}' not found in $BODHI_HOME/aliases
alias_exists_error = alias '{$var_0}' already exists in $BODHI_HOME/aliases
auth_service_error-auth_service_api_error = error from auth service: {$var_0}
hub_api_error = error from huggingface api: {$source}
hub_service_error-gated_access = {$source}.
  huggingface repo '{$repo}' is requires requesting for access from website.
  Go to https://huggingface.co/{$repo} to request access to the model and try again.
hub_service_error-may_be_not_exists = {$source}.
  You are not logged in to huggingface using CLI `huggingface-cli login`.
  So either the huggingface repo '{$repo}' does not exists, or is private, or requires request access.
  Go to https://huggingface.co/{$repo} to request access, login via CLI, and then try again.
remote_model_not_found_error = remote model alias '{$alias}' not found, check your alias and try again
data_file_not_found_error = file '{$filename}' not found in $BODHI_HOME/{$dirname}.
  $BODHI_HOME might not have been initialized. Run `bodhi init` to setup $BODHI_HOME.
hub_file_not_found_error = file '{$filename}' not found in huggingface repo '{$repo}', snapshot '{$snapshot}'
hub_api_error-gated_access = {$error}.
  huggingface repo '{$repo}' is requires requesting for access from website.
  Go to https://huggingface.co/{$repo} to request access to the model and try again.
hub_api_error-may_be_not_exists = {$error}.
  You are not logged in to huggingface using CLI `huggingface-cli login`.
  So either the huggingface repo '{$repo}' does not exists, or is private, or requires request access.
  Go to https://huggingface.co/{$repo} to request access, login via CLI, and then try again.
hub_api_error-not_exists = {$error}.
  The huggingface repo '{$repo}' does not exists.
hub_api_error-unknown = {$error}.
  An unknown error occurred accessing huggingface repo '{$repo}'.
hub_api_error-transport = {$error}.
  An error occurred while connecting to huggingface.co. Check your internet connection and try again.
hub_api_error-request = {$error}.
  An error occurred while requesting access to huggingface repo '{$repo}'.
secret_service_error-key_mismatch = passed encryption key and encryption key stored on platform do not match
secret_service_error-key_not_found = encryption key not found on platform secure storage
secret_service_error-encryption_error = {$var_0}
sqlx_error = {$source}
sqlx_migrate_error = {$source}
localization_error-rwlock_write = concurrency error setting up localization resource: {$source}
localization_error-rwlock_read = concurrency error accessing localization resource: {$source}
json_web_token_error-InvalidToken = authentication token is invalid
json_web_token_error-InvalidSignature = authentication token signature does not match
json_web_token_error-InvalidIssuer = authentication token issuer is invalid
json_web_token_error-InvalidAudience = authentication token audience is invalid
json_web_token_error-Unknown = authentication token is invalid, source: {$source}
item_not_found = item '{$id}' of type '{$item_type}' not found in db
env_service_error-bodhi_home_not_exists = BODHI_HOME does not exists: {$var_0}
env_service_error-settings_update_error = failed to update settings: {$var_0}
env_service_error-invalid_setting_key = Setting key is invalid: {$var_0}
setting_error-lock_error = concurrency error accessing settings: {$var_0}
keyring_error-keyring_error = {$var_0}
keyring_error-decode_error = invalid format: {$var_0}
