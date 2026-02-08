# Response Assertions

Import: `use server_core::test_utils::ResponseTestExt;`

## Status Code

Always assert status first:

```rust
assert_eq!(StatusCode::OK, response.status());
assert_eq!(StatusCode::CREATED, response.status());
assert_eq!(StatusCode::BAD_REQUEST, response.status());
```

## Typed Deserialization

```rust
let result: CreateChatCompletionResponse = response.json().await?;
assert_eq!("test-model", result.model);
```

## Flexible JSON Assertions

```rust
let body = response.json::<Value>().await?;
assert_eq!("expected_value", body["field"].as_str().unwrap());
```

## Pagination Response

```rust
let list = response.json::<PaginatedApiTokenResponse>().await?;
assert_eq!(10, list.data.len());
assert_eq!(15, list.total);
assert_eq!(1, list.page);
assert_eq!(10, list.page_size);
```

## Error Code Assertions

Error responses follow OpenAI format: `{"error": {"code": "...", "message": "...", "type": "..."}}`.

**Always check the `code` field (stable), never the `message` (brittle).**

Error codes are auto-generated: `EnumName::VariantName` becomes `enum_name-variant_name`.

```rust
assert_eq!(StatusCode::BAD_REQUEST, response.status());
let body = response.json::<Value>().await?;
assert_eq!(
  "pull_error-file_already_exists",
  body["error"]["code"].as_str().unwrap()
);
```

### Common error codes

| Enum::Variant | Error Code |
|---|---|
| `PullError::FileAlreadyExists` | `pull_error-file_already_exists` |
| `DataServiceError::AliasNotFound` | `data_service_error-alias_not_found` |
| `HubServiceError::FileNotFound` | `hub_service_error-file_not_found` |
| `EntityError::NotFound` | `entity_error-not_found` |
| `UserRouteError::EmptyToken` | `user_route_error-empty_token` |
| `TokenError::InvalidToken` | `token_error-invalid_token` |
| `LoginError::SessionInfoNotFound` | `login_error-session_info_not_found` |
| `AccessRequestError::AlreadyPending` | `access_request_error-already_pending` |
| `SettingsError::NotFound` | `settings_error-not_found` |
| `JsonRejectionError` (no variant) | `json_rejection_error` |
| `ObjValidationError::ValidationErrors` | `obj_validation_error-validation_errors` |

## SSE Stream Assertions

```rust
let chunks: Vec<CreateChatCompletionStreamResponse> = response.sse().await?;
let content = chunks.into_iter().fold(String::new(), |mut acc, chunk| {
  let text = chunk.choices.first()
    .expect("expected choice")
    .delta.content.as_deref().unwrap_or_default();
  acc.push_str(text);
  acc
});
assert_eq!("expected streamed text", content);
```

## DB State Verification

After an HTTP request, verify the database reflects the change:

```rust
// HTTP response assertion
assert_eq!(StatusCode::OK, response.status());
let updated = response.json::<ApiToken>().await?;
assert_eq!("Updated Name", updated.name);

// Database assertion
let db_record = db_service
  .get_api_token_by_id(&user_id, &token.id)
  .await?
  .expect("Token should exist");
assert_eq!("Updated Name", db_record.name);
assert_eq!(TokenStatus::Inactive, db_record.status);
```

## Anti-Patterns

```rust
// BAD: asserting exact message text (brittle, changes with wording)
assert_eq!("Application is already set up.", body["error"]["message"]);

// GOOD: asserting error code (stable, derived from enum)
assert_eq!("app_service_error-already_setup", body["error"]["code"].as_str().unwrap());
```

```rust
// BAD: assert_eq!(actual, expected) -- wrong order
assert_eq!(response.status(), StatusCode::OK);

// GOOD: assert_eq!(expected, actual) -- JUnit convention
assert_eq!(StatusCode::OK, response.status());
```

```rust
// BAD: asserting full JSON object (fragile, breaks on any field change)
assert_eq!(json!({"message": "...", "type": "...", "code": "..."}), body["error"]);

// GOOD: asserting only the stable field
assert_eq!("error_code-variant", body["error"]["code"].as_str().unwrap());
```
