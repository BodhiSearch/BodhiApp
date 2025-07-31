# Error Handling & Troubleshooting

BodhiApp implements a comprehensive error handling system that provides consistent, localized error messages across all API endpoints. This guide covers error response formats, common error scenarios, troubleshooting steps, and best practices for handling errors in your applications.

## Overview

BodhiApp's error system provides:

- **Consistent Format**: All errors follow the same OpenAI-compatible structure
- **Localized Messages**: Human-readable error messages in multiple languages
- **Error Codes**: Machine-readable error codes for programmatic handling
- **Rich Context**: Detailed error information for debugging
- **HTTP Status Mapping**: Proper HTTP status codes for different error types

## Error Response Format

### Standard Error Structure

All BodhiApp API errors follow the OpenAI-compatible error format:

```json
{
  "error": {
    "message": "Human-readable error description",
    "type": "error_category",
    "code": "specific_error_code",
    "param": "field_name_if_applicable"
  }
}
```

**Response Fields**:
- `message`: Localized, human-readable error description
- `type`: Error category (see Error Types section)
- `code`: Specific error code for programmatic handling
- `param`: Field name that caused the error (optional)

### Example Error Response

```json
{
  "error": {
    "message": "invalid request, reason: missing required field 'model'",
    "type": "invalid_request_error",
    "code": "bad_request_error",
    "param": "model"
  }
}
```

## Error Types and HTTP Status Codes

### Error Type Mapping

| Error Type | HTTP Status | Description | Common Causes |
|------------|-------------|-------------|---------------|
| `validation_error` | 400 | Request validation failed | Invalid field values, missing required fields |
| `invalid_request_error` | 400 | Malformed or invalid request | Invalid JSON, wrong content type, malformed data |
| `authentication_error` | 401 | Authentication required or failed | Missing/invalid API token, expired session |
| `forbidden_error` | 403 | Insufficient permissions | User lacks required role/scope for operation |
| `not_found_error` | 404 | Resource not found | Model alias not found, endpoint doesn't exist |
| `internal_server_error` | 500 | Server-side error | Database errors, file system issues, service failures |
| `service_unavailable` | 503 | Service temporarily unavailable | Model loading, system maintenance, resource exhaustion |
| `invalid_app_state` | 500 | Application in invalid state | App not properly configured, missing dependencies |
| `unknown_error` | 500 | Unclassified error | Unexpected errors, fallback category |

## Common Error Scenarios

### Authentication Errors (401)

#### Missing API Token
```json
{
  "error": {
    "message": "Authentication required",
    "type": "authentication_error",
    "code": "unauthorized"
  }
}
```

**Causes**:
- No `Authorization` header provided
- Empty or malformed `Authorization` header

**Solutions**:
```typescript
// Correct authentication
const headers = {
  'Authorization': `Bearer ${apiToken}`,
  'Content-Type': 'application/json'
};
```

#### Invalid Token
```json
{
  "error": {
    "message": "token is invalid: malformed token format",
    "type": "authentication_error", 
    "code": "token_error-invalid_token"
  }
}
```

**Causes**:
- Expired API token
- Corrupted or modified token
- Token from different BodhiApp instance

**Solutions**:
- Generate a new API token
- Verify token is copied correctly
- Check token expiration

### Authorization Errors (403)

#### Insufficient Permissions
```json
{
  "error": {
    "message": "Insufficient permissions for this operation",
    "type": "forbidden_error",
    "code": "forbidden"
  }
}
```

**Causes**:
- User role lacks required permissions
- Token scope insufficient for operation
- Session-only endpoint accessed with API token

**Solutions**:
- Use token with higher scope (`scope_token_power_user` vs `scope_token_user`)
- Use session authentication for admin operations
- Contact admin to upgrade user role

### Validation Errors (400)

#### Invalid Request Data
```json
{
  "error": {
    "message": "invalid request, reason: missing required field 'model'",
    "type": "invalid_request_error",
    "code": "bad_request_error",
    "param": "model"
  }
}
```

**Causes**:
- Missing required fields
- Invalid field values
- Wrong data types

**Solutions**:
```typescript
// Ensure all required fields are present
const request = {
  model: 'llama3:instruct',  // Required field
  messages: [...],           // Required field
  temperature: 0.7           // Optional field with valid range
};
```

#### JSON Parsing Error
```json
{
  "error": {
    "message": "failed to parse the request body as JSON, error: expected ',' at line 5 column 10",
    "type": "invalid_request_error",
    "code": "json_rejection_error"
  }
}
```

**Causes**:
- Malformed JSON in request body
- Wrong `Content-Type` header
- Encoding issues

**Solutions**:
```typescript
// Proper JSON request
const response = await fetch(url, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'  // Required for JSON requests
  },
  body: JSON.stringify(data)  // Properly serialize JSON
});
```

### Resource Not Found Errors (404)

#### Model Not Found
```json
{
  "error": {
    "message": "Alias 'unknown:model' not found",
    "type": "not_found_error",
    "code": "alias_not_found"
  }
}
```

**Causes**:
- Model alias doesn't exist
- Typo in model name
- Model not downloaded yet

**Solutions**:
```typescript
// List available models first
const modelsResponse = await fetch('/bodhi/v1/models', {
  headers: { 'Authorization': `Bearer ${token}` }
});
const models = await modelsResponse.json();
console.log('Available models:', models.data.map(m => m.alias));

// Use correct model alias
const chatResponse = await fetch('/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: 'llama3:instruct',  // Use exact alias name
    messages: [...]
  })
});
```

### Server Errors (500)

#### Internal Server Error
```json
{
  "error": {
    "message": "internal_server_error: database connection failed",
    "type": "internal_server_error",
    "code": "internal_server_error"
  }
}
```

**Causes**:
- Database connectivity issues
- File system permissions
- Memory exhaustion
- Configuration errors

**Solutions**:
- Check server logs for detailed error information
- Verify system resources (disk space, memory)
- Restart BodhiApp service
- Check file permissions for BodhiApp data directory

### Service Unavailable Errors (503)

#### Model Loading
```json
{
  "error": {
    "message": "service unavailable, reason: model is currently loading",
    "type": "service_unavailable",
    "code": "service_unavailable_error"
  }
}
```

**Causes**:
- Model is being loaded into memory
- System resources exhausted
- Model file corruption

**Solutions**:
- Wait for model loading to complete
- Implement retry logic with exponential backoff
- Check available system memory

## Error Handling Patterns

### Basic Error Handling

```typescript
async function handleBodhiAPICall<T>(
  apiCall: () => Promise<Response>
): Promise<T> {
  try {
    const response = await apiCall();
    
    if (!response.ok) {
      const errorData = await response.json();
      throw new BodhiAPIError(errorData.error, response.status);
    }
    
    return response.json();
  } catch (error) {
    if (error instanceof BodhiAPIError) {
      throw error;
    }
    
    // Handle network errors
    if (error instanceof TypeError && error.message.includes('fetch')) {
      throw new Error('Unable to connect to BodhiApp server');
    }
    
    throw new Error(`Unexpected error: ${error.message}`);
  }
}

class BodhiAPIError extends Error {
  public readonly type: string;
  public readonly code: string;
  public readonly status: number;
  public readonly param?: string;

  constructor(error: any, status: number) {
    super(error.message);
    this.name = 'BodhiAPIError';
    this.type = error.type;
    this.code = error.code;
    this.status = status;
    this.param = error.param;
  }
}
```

### Retry Logic with Exponential Backoff

```typescript
async function withRetry<T>(
  operation: () => Promise<T>,
  maxRetries: number = 3,
  baseDelay: number = 1000
): Promise<T> {
  let lastError: Error;
  
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error as Error;
      
      // Don't retry on client errors (4xx)
      if (error instanceof BodhiAPIError && error.status < 500) {
        throw error;
      }
      
      // Don't retry on last attempt
      if (attempt === maxRetries) {
        break;
      }
      
      // Exponential backoff with jitter
      const delay = baseDelay * Math.pow(2, attempt) + Math.random() * 1000;
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  
  throw lastError!;
}

// Usage
const result = await withRetry(() => 
  handleBodhiAPICall(() => 
    fetch('/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(chatRequest)
    })
  )
);
```

### Specific Error Type Handling

```typescript
async function robustChatCompletion(
  model: string,
  messages: any[],
  options: any = {}
) {
  try {
    return await handleBodhiAPICall(() => 
      fetch('/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${apiToken}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ model, messages, ...options })
      })
    );
  } catch (error) {
    if (error instanceof BodhiAPIError) {
      switch (error.code) {
        case 'alias_not_found':
          throw new Error(`Model '${model}' not found. Available models can be listed via /bodhi/v1/models`);
          
        case 'token_error-invalid_token':
          throw new Error('API token is invalid or expired. Please generate a new token.');
          
        case 'bad_request_error':
          if (error.param) {
            throw new Error(`Invalid value for field '${error.param}': ${error.message}`);
          }
          throw new Error(`Invalid request: ${error.message}`);
          
        case 'service_unavailable_error':
          // Implement retry for service unavailable
          console.log('Service unavailable, retrying...');
          await new Promise(resolve => setTimeout(resolve, 5000));
          return robustChatCompletion(model, messages, options);
          
        default:
          throw new Error(`API Error (${error.code}): ${error.message}`);
      }
    }
    throw error;
  }
}
```

## Troubleshooting Guide

### Connection Issues

#### Cannot Connect to Server

**Symptoms**:
- `fetch` throws `TypeError: Failed to fetch`
- Connection timeout errors
- DNS resolution errors

**Troubleshooting Steps**:
1. Verify BodhiApp is running: `curl http://localhost:1135/ping`
2. Check if port 1135 is accessible
3. Verify firewall settings
4. Check BodhiApp logs for startup errors
5. Confirm correct server URL and port

```bash
# Check if BodhiApp is running
curl -v http://localhost:1135/ping

# Check BodhiApp logs
tail -f ~/.cache/bodhi/logs/bodhi.log
```

#### Slow Response Times

**Symptoms**:
- API calls taking longer than expected
- Timeouts on large requests

**Troubleshooting Steps**:
1. Check system resource usage (CPU, memory, disk)
2. Monitor model loading status
3. Verify model file integrity
4. Check for concurrent request limits
5. Review context window size vs available memory

### Authentication Issues

#### Token Not Working

**Symptoms**:
- 401 authentication errors with valid-looking token
- Token worked before but now fails

**Troubleshooting Steps**:
1. Verify jwt token format with the resource client as azp
2. Check token expiration in BodhiApp UI
3. Ensure token has correct scope for operation
4. Verify token wasn't accidentally modified
5. Generate new token if needed

```typescript
// Debug token issues
async function debugToken(token: string) {
  try {
    const response = await fetch('/bodhi/v1/user', {
      headers: { 'Authorization': `Bearer ${token}` }
    });
    
    if (response.ok) {
      const userInfo = await response.json();
      console.log('Token is valid:', userInfo);
      return userInfo;
    } else {
      const error = await response.json();
      console.error('Token validation failed:', error);
    }
  } catch (error) {
    console.error('Token debug failed:', error);
  }
}
```

### Model Issues

#### Model Not Found

**Symptoms**:
- `alias_not_found` errors
- Model worked before but now fails

**Troubleshooting Steps**:
1. List available models: `GET /bodhi/v1/models`
2. Check if model alias exists and is spelled correctly
3. Verify model file exists in HuggingFace cache
4. Check model download status
5. Try re-downloading model if corrupted

```typescript
// Debug model availability
async function debugModel(modelAlias: string) {
  try {
    // Check if model exists
    const modelsResponse = await fetch('/bodhi/v1/models', {
      headers: { 'Authorization': `Bearer ${token}` }
    });
    const models = await modelsResponse.json();
    
    const model = models.data.find(m => m.alias === modelAlias);
    if (!model) {
      console.error(`Model '${modelAlias}' not found. Available models:`, 
        models.data.map(m => m.alias));
      return;
    }
    
    console.log('Model found:', model);
    
    // Check model files
    const filesResponse = await fetch('/bodhi/v1/modelfiles', {
      headers: { 'Authorization': `Bearer ${token}` }
    });
    const files = await filesResponse.json();
    
    const modelFile = files.data.find(f => 
      f.repo === model.repo && f.filename === model.filename
    );
    
    if (!modelFile) {
      console.error('Model file not found, may need to download');
    } else {
      console.log('Model file found:', modelFile);
    }
    
  } catch (error) {
    console.error('Model debug failed:', error);
  }
}
```

### Performance Issues

#### Slow Model Loading

**Symptoms**:
- Long delays before first response
- `service_unavailable` errors during model loading

**Troubleshooting Steps**:
1. Check available system memory
2. Monitor disk I/O during model loading
3. Verify model file integrity
4. Consider using smaller quantized models
5. Check for competing processes using resources

#### Memory Issues

**Symptoms**:
- Out of memory errors
- System becoming unresponsive
- Model loading failures

**Troubleshooting Steps**:
1. Check total system memory vs model requirements
2. Monitor memory usage during operation
3. Close other memory-intensive applications
4. Use smaller models or different quantization
5. Adjust context window size

```typescript
// Monitor memory usage patterns
async function monitorMemoryUsage() {
  const startMemory = process.memoryUsage();
  
  try {
    const response = await fetch('/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model: 'llama3:instruct',
        messages: [{ role: 'user', content: 'Hello' }]
      })
    });
    
    const result = await response.json();
    const endMemory = process.memoryUsage();
    
    console.log('Memory usage delta:', {
      heapUsed: endMemory.heapUsed - startMemory.heapUsed,
      heapTotal: endMemory.heapTotal - startMemory.heapTotal,
      rss: endMemory.rss - startMemory.rss
    });
    
    return result;
  } catch (error) {
    console.error('Request failed:', error);
    const errorMemory = process.memoryUsage();
    console.log('Memory at error:', errorMemory);
    throw error;
  }
}
```

## Error Code Reference

### Authentication Errors
- `token_error-invalid_token`: Token is malformed, expired, or invalid
- `unauthorized`: No authentication provided
- `forbidden`: Insufficient permissions

### Request Errors
- `bad_request_error`: Invalid request format or parameters
- `json_rejection_error`: JSON parsing failed
- `validation_errors`: Field validation failed

### Resource Errors
- `alias_not_found`: Model alias doesn't exist
- `entity_error-not_found`: Generic resource not found

### System Errors
- `internal_server_error`: Server-side error occurred
- `service_unavailable_error`: Service temporarily unavailable
- `app_reg_info_missing`: Application not properly configured

### Model Errors
- `chat_template_error-*`: Chat template processing errors
- `settings_metadata_error-*`: Settings validation errors

## Best Practices

### Error Handling Strategy

1. **Implement Proper Error Types**: Create specific error classes for different scenarios
2. **Use Appropriate Retry Logic**: Retry on 5xx errors, not on 4xx errors
3. **Log Errors Appropriately**: Log full error context for debugging
4. **Provide User-Friendly Messages**: Transform technical errors into user-understandable messages
5. **Monitor Error Patterns**: Track error rates and types for system health

### Development Guidelines

1. **Test Error Scenarios**: Include error cases in your test suite
2. **Handle Network Failures**: Always account for network connectivity issues
3. **Implement Timeouts**: Set reasonable timeouts for API calls
4. **Validate Input Early**: Validate data before sending to API
5. **Use Circuit Breakers**: Implement circuit breaker pattern for resilience

### Production Considerations

1. **Error Monitoring**: Set up monitoring and alerting for error rates
2. **Graceful Degradation**: Design fallback behaviors for API failures
3. **Rate Limiting**: Respect server limits and implement client-side rate limiting
4. **Security**: Don't expose sensitive error information to end users
5. **Documentation**: Document error scenarios and recovery procedures

## Next Steps

Now that you understand error handling in BodhiApp:

1. **[See Examples](examples.md)** - Complete integration examples with error handling
2. **[API Reference](api-reference.md)** - Quick endpoint reference
3. **[Back to Overview](overview.md)** - System overview and capabilities
4. **[Authentication Guide](authentication.md)** - Review authentication requirements

---

*Proper error handling is crucial for building robust applications with BodhiApp. The comprehensive error system provides all the information needed to diagnose and resolve issues quickly.* 