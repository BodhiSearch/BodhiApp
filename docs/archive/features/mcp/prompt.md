
# Bodhi App as MCP Client

We are implementing Bodhi App as MCP client. The salient features are:
- The feature will be similar to toolsets. 
  * We have admin enabling toolset type app-wide.
  * Then users able to create these types of toolset.
  * Then these toolsets invokable by user using their id, and only by user creating these toolsets.
  * Also external 3rd party apps requesting access to these toolsets using /apps/request-access
  * Once user consents to these access, the apps being able to invoke these toolset methods using their token which have scope claims authorizing the toolset invocation

## Authorization Flow for MCP access

The authorization to invoke MCP methods will be be different from current toolset one. For MCP, we are going to use dynamic client scopes feature of Keycloak. We will have optional scope `scope_mcp:*` at realm level.

We will have scope `scope_mcp:https-//mcp.deepwiki.com/mcp`
here, we are replaced the mcp endpoint url https://mcp.deepwiki.com/mcp so that it does not conflict with scope naming of dynamic scopes, and replaced ':' with '/'

`scope_mcp:*` is also going to be a user consented scope
- in login request, multiple `scope_mcp:...` can be requested
- all the requested `scope_mcp:...` are going to be parsed, and user requested to provide consent to mcp endpoint urls in the scope
- E.g., on User Consent screen
```md
* Requested access to MCP: https://mcp.deepwiki.com/mcp
* Requested access to MCP: https://api.githubcopilot.com/mcp
...
```

We will have dynamic client mappers, which will take this dynamic scope scope_mcp:* and parse the dynamic part, and add claims to access token as 
{
  ...,
  "mcps": [
    {"url": "https://mcp.deepwiki.com/mcp"}
  ]
}

We are keeping the claim as object for forward compatability, in future if we want to add more fields to the claim, we can do so without breaking existing working app.

And if these access requested by and consented to app client request, the app client token will have these claims.
And during token exchange, these claims will be transferrable. So when resource client is requesting exchanged token, these claims will be transferred to resource client token.

