/v1/chat/completions

# No Auth Header
401
```
{
    "error": {
        "message": "You didn't provide an API key. You need to provide your API key in an Authorization header using Bearer auth (i.e. Authorization: Bearer YOUR_KEY), or as the password field (with blank username) if you're accessing the API from your browser and are prompted for a username and password. You can obtain an API key from https://platform.openai.com/account/api-keys.",
        "type": "invalid_request_error",
        "param": null,
        "code": null
    }
}
```

# Invalid Auth Header
401
```
{
    "error": {
        "message": "Incorrect API key provided: foobar. You can find your API key at https://platform.openai.com/account/api-keys.",
        "type": "invalid_request_error",
        "param": null,
        "code": "invalid_api_key"
    }
}
```

# Malformed JSON request
400
```
{
    "error": {
        "message": "Invalid value: 'fool'. Supported values are: 'system', 'assistant', 'user', 'function', and 'tool'.",
        "type": "invalid_request_error",
        "param": "messages[0].role",
        "code": "invalid_value"
    }
}
```