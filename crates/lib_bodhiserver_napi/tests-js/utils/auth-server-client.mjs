/**
 * Get auth server configuration (no fallbacks - fail if not provided)
 */
export function getAuthServerConfig() {
  const config = {
    authUrl: process.env.INTEG_TEST_MAIN_AUTH_URL,
    authRealm: process.env.INTEG_TEST_AUTH_REALM,
    devConsoleClientSecret: process.env.INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET,
  };

  // Validate required environment variables
  const requiredVars = ['authUrl', 'authRealm', 'devConsoleClientSecret'];
  for (const varName of requiredVars) {
    if (!config[varName]) {
      throw new Error(`Required environment variable missing: ${varName.toUpperCase()}`);
    }
  }

  return config;
}

/**
 * Get test user credentials
 */
export function getTestCredentials() {
  const config = {
    username: process.env.INTEG_TEST_USERNAME,
    password: process.env.INTEG_TEST_PASSWORD,
    userId: process.env.INTEG_TEST_USERNAME_ID,
  };

  // Validate required environment variables
  const requiredVars = ['username', 'password', 'userId'];
  for (const varName of requiredVars) {
    if (!config[varName]) {
      throw new Error(`Required environment variable missing: ${varName.toUpperCase()}`);
    }
  }

  return config;
}

/**
 * Get realm admin credentials for admin API operations
 */
export function getRealmAdminCredentials() {
  const config = {
    username: process.env.INTEG_TEST_REALM_ADMIN,
    password: process.env.INTEG_TEST_REALM_ADMIN_PASS,
  };

  // Validate required environment variables
  const requiredVars = ['username', 'password'];
  for (const varName of requiredVars) {
    if (!config[varName]) {
      throw new Error(
        `Required environment variable missing for realm admin: INTEG_TEST_REALM_${varName.toUpperCase()}`
      );
    }
  }

  return config;
}

/**
 * AuthServerTestClient - Handles OAuth2 Token Exchange v2 operations
 */
export class AuthServerTestClient {
  constructor(config) {
    this.authUrl = config.authUrl;
    this.authRealm = config.authRealm;
    this.devConsoleClientId = 'client-bodhi-dev-console';
    this.devConsoleClientSecret = config.devConsoleClientSecret;
  }

  /**
   * Get dev console user token using direct access grant
   * @param {string} username - Username for authentication
   * @param {string} password - Password for authentication
   * @returns {Promise<string>} Access token for dev console operations
   */
  async getDevConsoleToken(username, password) {
    const tokenUrl = `${this.authUrl}/realms/${this.authRealm}/protocol/openid-connect/token`;

    const params = new URLSearchParams({
      grant_type: 'password',
      client_id: this.devConsoleClientId,
      client_secret: this.devConsoleClientSecret,
      username: username,
      password: password,
      scope: 'openid email profile roles',
    });

    const response = await fetch(tokenUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: params.toString(),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Token request failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to get dev console token: ${response.status} ${response.statusText} - ${errorText}`
      );
    }

    const data = await response.json();
    return data.access_token;
  }

  /**
   * Get admin token using admin-cli client with password grant
   * @param {string} username - Realm admin username
   * @param {string} password - Realm admin password
   * @returns {Promise<string>} Admin access token with realm-management permissions
   */
  async getRealmAdminToken(username, password) {
    const tokenUrl = `${this.authUrl}/realms/master/protocol/openid-connect/token`;

    const params = new URLSearchParams({
      client_id: 'admin-cli',
      grant_type: 'password',
      username: username,
      password: password,
    });

    const response = await fetch(tokenUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: params.toString(),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Admin token request failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to get realm admin token: ${response.status} ${response.statusText} - ${errorText}`
      );
    }

    const data = await response.json();
    return data.access_token;
  }

  /**
   * Create an app client (public, no secrets)
   * @param {string} devConsoleToken - Dev console access token
   * @param {number} appUrl - App URL for redirect URI
   * @param {string} name - Client name
   * @param {string} description - Client description
   * @param {string[]} customRedirectUris - Custom redirect URIs (optional)
   * @returns {Promise<Object>} Created app client info
   */
  async createAppClient(
    devConsoleToken,
    appUrl,
    name = 'Test App Client',
    description = 'Test app client for Playwright tests',
    customRedirectUris = null
  ) {
    const appsUrl = `${this.authUrl}/realms/${this.authRealm}/bodhi/apps`;

    // Use custom redirect URIs if provided, otherwise use default ones
    const redirectUris = customRedirectUris || [`${appUrl}/ui/auth/callback`];

    const response = await fetch(appsUrl, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${devConsoleToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name,
        description,
        redirect_uris: redirectUris,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Create app client failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(`Failed to create app client: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();
    return {
      clientId: data.client_id,
    };
  }

  /**
   * Create a resource client (confidential, with secrets)
   * @param {number} serverUrl - Server URL for redirect URI
   * @param {string} name - Client name
   * @param {string} description - Client description
   * @param {boolean} liveTest - Whether to enable liveTest mode for direct grants
   * @returns {Promise<Object>} Created resource client info
   */
  async createResourceClient(
    serverUrl,
    name = 'Test Resource Client',
    description = 'Test resource client for Playwright tests',
    liveTest = true
  ) {
    let resourcesUrl = `${this.authUrl}/realms/${this.authRealm}/bodhi/resources`;

    // Add live_test query parameter if liveTest is true
    if (liveTest) {
      resourcesUrl += '?live_test=true';
    }

    const response = await fetch(resourcesUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name,
        description,
        redirect_uris: [`${serverUrl}/ui/auth/callback`],
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Create resource client failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to create resource client: ${response.status} ${response.statusText}`
      );
    }

    const data = await response.json();
    return {
      clientId: data.client_id,
      clientSecret: data.client_secret,
      scope: data.scope,
    };
  }

  /**
   * Get resource client service account token
   * @param {string} clientId - Resource client ID
   * @param {string} clientSecret - Resource client secret
   * @returns {Promise<string>} Service account access token
   */
  async getResourceServiceAccountToken(clientId, clientSecret) {
    const tokenUrl = `${this.authUrl}/realms/${this.authRealm}/protocol/openid-connect/token`;

    const params = new URLSearchParams({
      grant_type: 'client_credentials',
      scope: 'service_account',
    });

    const credentials = Buffer.from(`${clientId}:${clientSecret}`).toString('base64');

    const response = await fetch(tokenUrl, {
      method: 'POST',
      headers: {
        Authorization: `Basic ${credentials}`,
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: params.toString(),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Get service account token failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to get service account token: ${response.status} ${response.statusText}`
      );
    }

    const data = await response.json();
    return data.access_token;
  }

  /**
   * Get user token using direct access grant for resource clients (liveTest mode)
   * @param {string} clientId - Resource client ID
   * @param {string} clientSecret - Resource client secret
   * @param {string} username - Username for authentication
   * @param {string} password - Password for authentication
   * @returns {Promise<string>} User access token
   */
  async getResourceUserToken(clientId, clientSecret, username, password) {
    const tokenUrl = `${this.authUrl}/realms/${this.authRealm}/protocol/openid-connect/token`;

    const params = new URLSearchParams({
      grant_type: 'password',
      client_id: clientId,
      client_secret: clientSecret,
      username: username,
      password: password,
      scope: 'openid email profile roles',
    });

    const response = await fetch(tokenUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: params.toString(),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Get resource user token failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to get resource user token: ${response.status} ${response.statusText}`
      );
    }

    const data = await response.json();
    return data.access_token;
  }

  /**
   * Request audience access for resource client (dynamic audience management)
   * @param {string} resourceToken - Resource client service account token
   * @param {string} appClientId - App client ID to request access for
   * @returns {Promise<string>} Resource scope name
   */
  async requestAudienceAccess(resourceToken, appClientId) {
    const requestAccessUrl = `${this.authUrl}/realms/${this.authRealm}/bodhi/resources/request-access`;

    const response = await fetch(requestAccessUrl, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${resourceToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        app_client_id: appClientId,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Request audience access failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to request audience access: ${response.status} ${response.statusText}`
      );
    }

    const data = await response.json();
    return data.scope;
  }

  async makeResourceAdmin(resourceClientId, resourceClientSecret, userId) {
    const resourceAdminUrl = `${this.authUrl}/realms/${this.authRealm}/bodhi/resources/make-resource-admin`;
    const resourceToken = await this.getResourceServiceAccountToken(
      resourceClientId,
      resourceClientSecret
    );
    const response = await fetch(resourceAdminUrl, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${resourceToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: userId,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Make resource admin failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(`Failed to make resource admin: ${response.status} ${response.statusText}`);
    }
  }

  /**
   * Assign a role to a user (for testing with liveTest mode)
   * @param {string} reviewerToken - Admin token for authorization
   * @param {string} userId - User ID to assign role to
   * @param {string} role - Role to assign (resource_user, resource_manager, etc.)
   * @returns {Promise<void>}
   */
  async assignUserRole(reviewerToken, userId, role) {
    const assignRoleUrl = `${this.authUrl}/realms/${this.authRealm}/bodhi/resources/assign-role`;

    const response = await fetch(assignRoleUrl, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${reviewerToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: userId,
        role: role,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Assign user role failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to assign user role: ${response.status} ${response.statusText}, url: ${assignRoleUrl}, user_id: ${userId}, role: ${role}`
      );
    }
  }

  /**
   * Remove a user from all roles (for testing with liveTest mode)
   * @param {string} reviewerToken - Admin token for authorization
   * @param {string} userId - User ID to remove from all roles
   * @returns {Promise<void>}
   */
  async removeUser(reviewerToken, userId) {
    const removeUserUrl = `${this.authUrl}/realms/${this.authRealm}/bodhi/resources/remove-user`;

    const response = await fetch(removeUserUrl, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${reviewerToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: userId,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Remove user failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(`Failed to remove user: ${response.status} ${response.statusText}`);
    }
  }

  /**
   * Perform OAuth2 Token Exchange v2
   * @param {string} resourceClientId - Resource client ID
   * @param {string} resourceClientSecret - Resource client secret
   * @param {string} subjectToken - App user token to exchange
   * @returns {Promise<string>} Exchanged access token
   */
  async exchangeToken(resourceClientId, resourceClientSecret, subjectToken) {
    const tokenUrl = `${this.authUrl}/realms/${this.authRealm}/protocol/openid-connect/token`;

    const params = new URLSearchParams({
      subject_token: subjectToken,
      scope: 'openid email profile roles scope_user_user',
      grant_type: 'urn:ietf:params:oauth:grant-type:token-exchange',
      subject_token_type: 'urn:ietf:params:oauth:token-type:access_token',
      requested_token_type: 'urn:ietf:params:oauth:token-type:access_token',
    });

    const credentials = Buffer.from(`${resourceClientId}:${resourceClientSecret}`).toString(
      'base64'
    );

    const response = await fetch(tokenUrl, {
      method: 'POST',
      headers: {
        Authorization: `Basic ${credentials}`,
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: params.toString(),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.log('Token exchange failed:', response.status, response.statusText);
      console.log('Error response body:', errorText);
      throw new Error(`Failed to exchange token: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();
    return data.access_token;
  }

  /**
   * Configure client access token lifespan using Keycloak Admin API
   * @param {string} adminToken - Admin access token for authorization
   * @param {string} clientId - Client ID (not UUID)
   * @param {number} accessTokenLifespan - Access token lifespan in seconds (default 5)
   * @returns {Promise<void>}
   */
  async configureClientTokenLifespan(adminToken, clientId, accessTokenLifespan = 5) {
    const getClientUrl = `${this.authUrl}/admin/realms/${this.authRealm}/clients?clientId=${clientId}`;

    const getResponse = await fetch(getClientUrl, {
      method: 'GET',
      headers: {
        Authorization: `Bearer ${adminToken}`,
        'Content-Type': 'application/json',
      },
    });

    if (!getResponse.ok) {
      const errorText = await getResponse.text();
      console.log('Get client failed:', getResponse.status, getResponse.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to get client: ${getResponse.status} ${getResponse.statusText} - ${errorText}`
      );
    }

    const clients = await getResponse.json();
    if (!clients || clients.length === 0) {
      throw new Error(`Client with clientId '${clientId}' not found`);
    }

    const client = clients[0];
    const clientUuid = client.id;

    const updateClientUrl = `${this.authUrl}/admin/realms/${this.authRealm}/clients/${clientUuid}`;

    const updatedClient = {
      ...client,
      attributes: {
        ...client.attributes,
        'access.token.lifespan': accessTokenLifespan.toString(),
      },
    };

    const updateResponse = await fetch(updateClientUrl, {
      method: 'PUT',
      headers: {
        Authorization: `Bearer ${adminToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(updatedClient),
    });

    if (!updateResponse.ok) {
      const errorText = await updateResponse.text();
      console.log('Update client failed:', updateResponse.status, updateResponse.statusText);
      console.log('Error response body:', errorText);
      throw new Error(
        `Failed to update client: ${updateResponse.status} ${updateResponse.statusText} - ${errorText}`
      );
    }
  }
}

/**
 * Create an auth server test client
 * @param {Object} config - Configuration object with authUrl, authRealm, devConsoleClientSecret
 * @returns {AuthServerTestClient} New auth server test client instance
 */
export function createAuthServerTestClient(config) {
  return new AuthServerTestClient(config);
}
