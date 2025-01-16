
our api token story is complete, next we want to start a new story api-authorization

we have 4 roles in our system - user, power_user, manager, admin
- user can use the chat completions API, and see all read only non-sensitive information about the system, like models, modelfiles, etc.
- power_user can do all of above, as well as do certain sensitive but not super sensitive actions, like it can download new models, but it cannot approve users to the system
- manager can do all of above as well as user administratiion
- admin can do all of above, as well as system settings like changing sensitive system settings, making other admins etc.

we want to have a middleware similar to @auth_middleware.rs that will be attached to routes @routes.rs based on the authorization level, we will call it api_auth.rs. it will be in same module as auth_middleware.

once done, the incoming request will first go through auth_middleware which will inject authenticated token in the header, along with any other downstream useful information required by authorization systems

in the api_auth middleware, based on its configured access role, it will check if the user have that role from the injected header information, if it has, then it will allow the request to pass through, if not, it will return 401 with authorization error message.

based on the above requirements, ask any pending questions. once the questions are done, generate a story similar to @story-20250112-api-tokens.md format.


Token Information:
Will the user's role information be included in the access token claims?
Should we validate roles against a database or rely solely on token claims?
there are 2-ways to access the system, using session token that is passed through cookie and use bearer token send in auth header.
auth_middleware will inject active access_token in the header, which will have role information as following claim -
  "resource_access": {
    "resource-71425db3-e706-42d6-b254-81b2e9820346": {
      "roles": [
        "resource_manager",
        "resource_power_user",
        "resource_user",
        "resource_admin"
      ]
    }
  },
we need to check our app instance client_id matches the resource_access client id, and accept the roles defined in claim

for bearer token, when we get active access token, the scope will have the claim -
  "scope": "openid offline_access scope_token_user",

here, scope_token_user is equivalent to user role, similarly scope_token_power_user is to power_user, scope_token_manager is to manager and scope_token_admin is to admin role

we will solely rely on the token claims

Authorization Behavior:
Should we return 401 (Unauthorized) or 403 (Forbidden) when a user has valid authentication but insufficient role?
403

Should we provide detailed error messages indicating which role is required?
No, just a generic message, you dont have sufficient privileges for that action, on those lines

Route Configuration:
Do we want to support multiple roles for a single route (e.g., "manager OR admin")?
Yes, the roles are in heirarchy, so a manager have automatically access to user, power_user and manager routes
A reusable and non-duplicating solution is expected

Should we have a way to bypass role checks in non-authenticated mode (similar to auth_middleware)?
Yes, for non-auth, we will not check for these. Also for non-auth, many of the feature pages will show `app needs to be in authenticated mode for this feature`

Caching:
Should we cache role validation results to improve performance?
If possible without compromising security or complexity of the application

If yes, what should be the cache invalidation strategy?
You havea to analyze

Audit & Logging:
Should we log authorization failures for security monitoring?
Yes, at warn! level

Do we need to track which roles are accessing which endpoints?
No

thanks for. the above questions. here is more context of the application @crates/bodhi/ai-docs 

based on added context, ask any pending questions before generating the story.