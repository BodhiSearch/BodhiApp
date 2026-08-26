#!/bin/bash

changed_files=$(git diff --name-only HEAD | grep '\.rs$')

success_count=0
failure_count=0
test_error_detected=false

for file in $changed_files; do
  if [[ $file == crates/*/src/* ]]; then
    crate=$(echo $file | awk -F'/' '{print $2}')
    module=$(basename "${file%.*}")
    
    echo "Running tests for crate: $crate, module: $module"

    cargo test -p "$crate" "$module"
    status=$?

    if [[ $status -eq 0 ]]; then
      ((success_count++))
    elif [[ $status -ne 0 ]]; then
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

if [[ $success_count -gt 0 || $failure_count -gt 0 ]]; then
  message="Tests completed. Success: $success_count, Failures: $failure_count."
  echo "$message"

  if [[ $test_error_detected == true || $failure_count -eq 0 ]]; then
    osascript -e "display notification \"$message\" with title \"Test Execution Results\""
  fi
else
  echo "No tests run."
fi