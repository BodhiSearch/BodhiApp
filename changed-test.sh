#!/bin/bash

# Get the changed files
changed_files=$(git diff --name-only HEAD | grep '\.rs$')

# Initialize counters
success_count=0
failure_count=0
test_error_detected=false

for file in $changed_files; do
  # Match files in the `crates/<crate-name>/src/` structure
  if [[ $file == crates/*/src/* ]]; then
    # Extract the crate name from the path
    crate=$(echo $file | awk -F'/' '{print $2}')
    # Extract the module name (file without extension)
    module=$(basename "${file%.*}")
    
    echo "Running tests for crate: $crate, module: $module"
    
    # Run tests for the crate and module, capture status
    cargo test -p "$crate" "$module"
    status=$?
    
    if [[ $status -eq 0 ]]; then
      ((success_count++))
    elif [[ $status -ne 0 ]]; then
      # Check if it's a compilation error
      if [[ $(cargo test -p "$crate" "$module" 2>&1 | grep -q "error: could not compile") ]]; then
        echo "Compilation failed for crate: $crate, module: $module."
      else
        ((failure_count++))
        test_error_detected=true
        echo "Test failure detected in crate: $crate, module: $module"
      fi
    fi
  fi
done

# Create a summary message
if [[ $success_count -gt 0 || $failure_count -gt 0 ]]; then
  message="Tests completed. Success: $success_count, Failures: $failure_count."
  echo "$message"

  # Display macOS notification only for test failures or success
  if [[ $test_error_detected == true || $failure_count -eq 0 ]]; then
    osascript -e "display notification \"$message\" with title \"Test Execution Results\""
  fi
else
  echo "No tests run."
fi