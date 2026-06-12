# Authorization Tests

## Test Setup
```gherkin
Background:
  Given the Bodhi app is running in development mode
  And authorization is enabled
  And the following test users exist:
    | username | role       | api_token                    |
    | user1    | user      | test-token-user1             |
    | power1   | power_user| test-token-power1            |
    | admin1   | admin     | test-token-admin1            |
```

## API Authorization Tests

### Public API Access
```gherkin
Feature: Public API Access
  As an unauthenticated user
  I want to access public endpoints
  So that I can use basic app features without authentication

  Scenario: Access health check endpoint
    When I send a GET request to "/ping"
    Then the response status should be 200
    And the response should contain "pong"

  Scenario: Access app info endpoint
    When I send a GET request to "/app/info"
    Then the response status should be 200
    And the response should contain app version information
```

### API Token Authorization
```gherkin
Feature: API Token Authorization
  As an API client
  I want to access endpoints with appropriate token
  So that I can use the API programmatically

  Scenario: User token accessing model list
    Given I use the API token "test-token-user1"
    When I send a GET request to "/v1/models"
    Then the response status should be 200

  Scenario: Power user token accessing model pull
    Given I use the API token "test-token-power1"
    When I send a POST request to "/api/ui/modelfiles/pull"
    Then the response status should be 200

  Scenario: User token accessing power user endpoint
    Given I use the API token "test-token-user1"
    When I send a POST request to "/api/ui/modelfiles/pull"
    Then the response status should be 403

  Scenario: Token accessing session-only endpoint
    Given I use the API token "test-token-power1"
    When I send a GET request to "/api/ui/tokens"
    Then the response status should be 403
```

## UI Authorization Tests

### Navigation Authorization
```gherkin
Feature: Navigation Authorization
  As a user with specific role
  I want to see only authorized navigation items
  So that I can access appropriate features

  Scenario: User sees basic navigation
    Given I am logged in as "user1"
    When I view the main navigation
    Then I should see "Chat" in the menu
    And I should see "Models" in the menu
    But I should not see "Pull Models" in the menu
    And I should not see "Tokens" in the menu

  Scenario: Power user sees extended navigation
    Given I am logged in as "power1"
    When I view the main navigation
    Then I should see "Chat" in the menu
    And I should see "Models" in the menu
    And I should see "Pull Models" in the menu
    And I should see "Tokens" in the menu
```

### Feature Access Control
```gherkin
Feature: Feature Access Control
  As a user with specific role
  I want to be prevented from accessing unauthorized features
  So that security is maintained

  Scenario: User accessing basic features
    Given I am logged in as "user1"
    When I navigate to "/chat"
    Then I should see the chat interface
    When I navigate to "/models"
    Then I should see the models list

  Scenario: User blocked from power features
    Given I am logged in as "user1"
    When I navigate to "/models/pull"
    Then I should see "You do not have access to this feature"
    When I navigate to "/tokens"
    Then I should see "You do not have access to this feature"

  Scenario: Power user accessing extended features
    Given I am logged in as "power1"
    When I navigate to "/models/pull"
    Then I should see the model pull interface
    When I navigate to "/tokens"
    Then I should see the token management interface
```

### UI Element Authorization
```gherkin
Feature: UI Element Authorization
  As a user with specific role
  I want to see only authorized UI elements
  So that I can access appropriate actions

  Scenario: User sees basic model actions
    Given I am logged in as "user1"
    When I view the models page
    Then I should see the "View" button
    But I should not see the "Create Alias" button
    And I should not see the "Pull Model" button

  Scenario: Power user sees extended model actions
    Given I am logged in as "power1"
    When I view the models page
    Then I should see the "View" button
    And I should see the "Create Alias" button
    And I should see the "Pull Model" button
```

### Error Handling
```gherkin
Feature: Authorization Error Handling
  As a user
  I want to see clear error messages
  So that I understand why access is denied

  Scenario: Direct URL access to unauthorized page
    Given I am logged in as "user1"
    When I directly navigate to "/tokens"
    Then I should see "You do not have access to this feature"
    And I should be redirected to the home page

  Scenario: Clicking unauthorized action
    Given I am logged in as "user1"
    When I view the models page
    And I attempt to access the "Create Alias" action
    Then I should see a toast message "Insufficient permissions"
```

### Authorization State Changes
```gherkin
Feature: Authorization State Changes
  As a user
  I want authorization changes to be reflected immediately
  So that my access is always current

  Scenario: Session expiry
    Given I am logged in as "power1"
    When my session expires
    Then I should be redirected to the login page
    And I should see "Session expired, please login again"

  Scenario: Role change reflection
    Given I am logged in as "power1"
    When my role is changed to "user"
    Then I should not see "Pull Models" in the menu
    And I should not see "Tokens" in the menu
``` 