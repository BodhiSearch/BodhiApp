# Auth Middleware Test Scenarios

## Feature: Authentication Middleware
As a user of the Bodhi application
I want to ensure proper authentication handling
So that only authorized users can access protected resources

### Scenario: Authorization Skip When App Status Ready and Auth Disabled
```gherkin
Given the application status is "ready"
And authorization is disabled
When I make a request to a protected endpoint
Then the request should pass through without authentication
And the response should not contain any resource token
```

### Scenario: Redirect to Setup Page
```gherkin
Given the application is not set up
When I make a request to a protected endpoint
Then I should be redirected to "/ui/setup"
And optional auth endpoints should still be accessible
```

### Scenario: Authorization Header Validation
```gherkin
Scenario Outline: Invalid Authorization Headers
  Given I make a request to a protected endpoint
  When I provide an authorization header "<header>"
  Then I should receive a "<status>" status code
  And I should receive an error message "<error>"

Examples:
  | header        | status      | error                                    |
  | none         | 401         | authorization header not found           |
  | bearer foobar| 400         | authorization header is malformed        |
  | Bearer       | 400         | token not found in authorization header  |
```

### Scenario: Token Algorithm Validation
```gherkin
Given I have a token signed with "RS256" algorithm
And the system expects "HS256" algorithm
When I make a request with this token
Then I should receive a 401 status code
And I should receive an algorithm mismatch error
```

### Scenario: Token KID Validation
```gherkin
Given I have a token with KID "test-kid"
And the system expects KID "other-kid"
When I make a request with this token
Then I should receive a 401 status code
And I should receive a KID mismatch error
```

### Scenario: Public Key Validation
```gherkin
Given I have a valid token
But the application registration info is missing
When I make a request with this token
Then I should receive a 500 status code
And I should receive a missing app registration info error
```

### Scenario: Token Issuer Validation
```gherkin
Given I have a token from issuer "test-issuer"
And the system expects issuer "other-issuer"
When I make a request with this token
Then I should receive a 401 status code
And I should receive an invalid issuer error
```

### Scenario: Token Cache Usage
```gherkin
Given I have a valid token
And the token is already cached
When I make a request with this token
Then the token should not be exchanged again
And I should receive the cached token in response
```

### Scenario: Token Exchange When Not Cached
```gherkin
Given I have a valid token
And the token is not cached
When I make a request with this token
Then the token should be exchanged
And the new token should be cached
And I should receive the exchanged token in response
```

### Scenario: Session Token Validation
```gherkin
Given I have a valid session
And the session contains a valid token
When I make a request with this session
Then I should be authenticated
And I should receive the session token in response
```

### Scenario: Expired Session Token Handling
```gherkin
Given I have a valid session
And the session contains an expired token
And a valid refresh token
When I make a request with this session
Then the token should be refreshed
And the session should be updated with new tokens
And I should receive the new token in response
```

### Scenario: Failed Token Refresh Handling
```gherkin
Given I have a valid session
And the session contains an expired token
But the token refresh fails
When I make a request with this session
Then I should receive a 401 status code
And the session tokens should remain unchanged
And I should receive a refresh token error
```

## Technical Notes

1. **Test Setup**:
   - Uses mock services for auth, secrets, and cache
   - Temporary SQLite database for session storage
   - Test tokens with configurable expiration

2. **Validation Checks**:
   - Algorithm matching (RS256 vs HS256)
   - Key ID (KID) matching
   - Issuer validation
   - Token expiration
   - Token signature

3. **Cache Behavior**:
   - Checks cache before token exchange
   - Stores both access and refresh tokens
   - Validates token hash for cache hits

4. **Session Management**:
   - Stores tokens in session
   - Handles token refresh
   - Maintains session state
   - Validates session cookies
